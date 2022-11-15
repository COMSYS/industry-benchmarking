//! Implement Server Sent Events as server push
//! 
//! Enable clients to receive automatic updates from a server via HTTP
//! where the communication is only initializesed once. This is used to
//! indicate updates at our clients, i.e. the number of participants 
//! that are required until the benchmark can begin. (Uses JS EventSource)
//! The mimetype for SSE is `text/event-stream`.  

use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use actix_web::{
    rt::time::{interval_at, Instant},
    web::{Bytes, Data},
    Error,
};
use futures_util::Stream;
use parking_lot::Mutex;
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// Broadcaster type which only can send one message at once
pub struct Broadcaster {
    inner: Mutex<BroadcasterInner>,
}

/// Clients that registered for broadcasting service will be contacted (i.e. companies)
struct BroadcasterInner {
    clients: Vec<Sender<Bytes>>,
}

impl Broadcaster {
    pub fn new() -> Data<Self> {
        
        // Create new instance which will be attached to the HTTP server
        let broadcaster_data = Data::new(Broadcaster {
            inner: Mutex::new(BroadcasterInner {
                clients: Vec::new(),
            }),
        });

        // ping clients every 120 seconds to see if they are alive
        Broadcaster::spawn_ping(broadcaster_data.clone(), 120);

        broadcaster_data
    }

    fn spawn_ping(me: Data<Self>, interval: u64) {
        actix_web::rt::spawn(async move {
            let mut interval = interval_at(Instant::now(), Duration::from_secs(interval));

            loop {
                interval.tick().await;
                me.remove_stale_clients();
            }
        });
    }

    fn remove_stale_clients(&self) {
        let mut inner = self.inner.lock();

        let mut ok_clients = Vec::new();
        for client in inner.clients.iter() {
            let result = client.clone().try_send(Bytes::from("data: keep-alive\n\n"));

            if let Ok(()) = result {
                ok_clients.push(client.clone());
            }
        }
        inner.clients = ok_clients;
    }

    pub fn new_client(&self) -> Client {
        let (tx, rx) = channel(100);

        tx.try_send(Bytes::from("data: connected\n\n")).unwrap();

        let mut inner = self.inner.lock();
        inner.clients.push(tx);

        Client(rx)
    }

    pub fn send(&self, msg: &str) {
        // Transform string to bytestring and send it to every receiver (i.e., Client)
        let msg = Bytes::from(["data: ", msg, "\n\n"].concat());

        let inner = self.inner.lock();
        for client in inner.clients.iter() {
            client.clone().try_send(msg.clone()).unwrap_or(());
        }
    }
}

/// Client is a Receiver type
pub struct Client(Receiver<Bytes>);

impl Stream for Client {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_recv(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(Some(Ok(v))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
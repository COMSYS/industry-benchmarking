//! Client certificate and information extractor
//! 
//! On first request from one client, the certificate is 
//! remembered for successive calls. This allows a unique
//! and cryptographically enforced mapping between one
//! participants UUID and his certificate. The certificate
//! is stored for a company and for the analyst extra and
//! enforces authenticity of requests.

use std::{any::Any, net::SocketAddr};
use actix_tls::accept::rustls::TlsStream;
use actix_web::{
    dev::Extensions,
    rt::net::TcpStream
};


#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub bind: SocketAddr,
    pub peer: SocketAddr,
}

pub fn get_client_cert(connection: &dyn Any, data: &mut Extensions) {
    if let Some(tls_socket) = connection.downcast_ref::<TlsStream<TcpStream>>() {
        log::debug!("TLS connection setup!");

        let (socket, tls_session) = tls_socket.get_ref();

        // Add information to request on connection setup
        data.insert(ConnectionInfo {
            bind: socket.local_addr().unwrap(),
            peer: socket.peer_addr().unwrap(),
        });

        // Extract the information from the peer on his certificate
        if let Some(certs) = tls_session.peer_certificates() {
            // Use Rustls certificate type as it supports comparison and is easier to handle
            data.insert(certs.first().unwrap().clone());
        }

        // Get the SNI (Server name indication) for name identification
        if let Some(sni_host) = tls_session.sni_hostname() {
            data.insert(sni_host.to_string());
        }

    } else if let Some(socket) = connection.downcast_ref::<TcpStream>() {
        log::debug!("plaintext connection setup: {:?}", socket);

        // Add information to request on connection setup
        data.insert(ConnectionInfo {
            bind: socket.local_addr().unwrap(),
            peer: socket.peer_addr().unwrap(),
        });

    }
}
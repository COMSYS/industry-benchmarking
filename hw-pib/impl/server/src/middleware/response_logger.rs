use std::{
    marker::PhantomData, 
    pin::Pin, 
    task::{Context, Poll}
};

use actix_web::{
    dev::{ forward_ready, Service, ServiceRequest, ServiceResponse, Transform },
    web::{ BytesMut, Bytes },
    body::{ BodySize, MessageBody },
    Error, 
};

use futures::{
    future::{ready, Ready},
    Future
};

/// Response Logger for reading out server responses
pub struct Logging;

/// Middleware factory is of `RequestLogging` trait
/// `S` - type of the next service
/// `B` - type of response's body
impl<S: 'static ,B> Transform<S, ServiceRequest> for Logging 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static, 
{
    // Responses of the middleware will be a BodyLogger
    type Response = ServiceResponse<BodyLogger<B>>;
    // Errors produced by the service.
    type Error = Error;
    // Errors produced while building a transform service.
    type InitError = ();
    // The `TransformService` value created by this factory, which is the RequestLogging"Service"
    type Transform = LoggingMiddleware<S>;
    // The future response value.
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new instance of our middleware "service"
    fn new_transform(&self, service: S) -> Self::Future {
        ready(
            Ok(
                LoggingMiddleware {
                    service
                }
            )
        )
    }

}

pub struct LoggingMiddleware<S> {
    // No Rc required
    service: S,
}


impl<S,B> Service<ServiceRequest> for LoggingMiddleware<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    // Response holds a body logger
    type Response = ServiceResponse<BodyLogger<B>>;
    // Errors produced by the service when polling readiness
    type Error = Error;
    // Future Response value → We wait explicitly for our response body which is wrapped in a wrapper stream
    type Future =  WrapperStream<S, B>;

    forward_ready!(service);

    // Process the request and return the response asynchronously.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // We return the Response body when it has been computed → asnchronously wait
        WrapperStream {
            fut: self.service.call(req),
            _t: PhantomData,
        }
    }

}

/// Wrapper for waiting on the response body
#[pin_project::pin_project]
pub struct WrapperStream<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest>,
{
    #[pin]
    fut: S::Future,
    _t: PhantomData<(B,)>,
}

/// Implement Future Trait for WrapperStream
///
/// Allows polling of the service which either
/// returns a `BodyLogger` or a `actix_web::Error`.
impl<S, B> Future for WrapperStream<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<BodyLogger<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res = futures_util::ready!(self.project().fut.poll(cx));

        Poll::Ready(res.map(|res| {
            res.map_body(move |_, body| BodyLogger {
                body,
                body_accum: BytesMut::new(),
            })
        }))
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct BodyLogger<B> {
    #[pin]
    body: B,
    body_accum: BytesMut,
}

#[pin_project::pinned_drop]
impl<B> PinnedDrop for BodyLogger<B> {
    fn drop(self: Pin<&mut Self>) {
        log::debug!("Server Response: {:?}", self.body_accum);
    }
}

/// Implement Concrete MessageBody for generic BodyLogger
/// 
/// Returns body Size and Message if `Ready` or `Pending`
/// in case the future is still kept.
impl<B: MessageBody> MessageBody for BodyLogger<B> {
    type Error = B::Error;

    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        let this = self.project();

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                this.body_accum.extend_from_slice(&chunk);
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
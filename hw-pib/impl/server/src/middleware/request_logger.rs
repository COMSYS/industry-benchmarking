//use std::fmt::Debug;
use std::{rc::Rc};
use actix_web::{
    dev::{
        forward_ready,
        Service,
        ServiceRequest,
        ServiceResponse,
        Transform},
    Error, web::{BytesMut, }, HttpMessage
};
use actix_http::{h1::Payload};
use futures::{
    future::{ready, LocalBoxFuture, Ready}, StreamExt
};

/// Request Logger for extracting information from client requests
pub struct Logging;

/// Middleware factory is of `RequestLogging` trait
/// `S` - type of the next service
/// `B` - type of response's body
impl<S: 'static ,B> Transform<S, ServiceRequest> for Logging 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static, 
{
    // Responses produced by the service.
    type Response = ServiceResponse<B>;
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
                    service: Rc::new(service) 
                }
            )
        )
    }

}

pub struct LoggingMiddleware<S> {
    // Avoid lifetime issues with reference counting
    service: Rc<S>,
}


impl<S: 'static,B> Service<ServiceRequest> for LoggingMiddleware<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    // Responses given by the service.
    type Response = ServiceResponse<B>;
    // Errors produced by the service when polling readiness
    type Error = Error;
    // Future Response value (promise)
    type Future =  LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // An implementation of poll_ready that forwards readiness checks to a named struct field.
    // poll_ready returns "Ready", whenever the service is able to process the request.
    forward_ready!(service);

    // Process the request and return the response asynchronously.
    fn call(&self, mut req: ServiceRequest) -> Self::Future {

        // Measure time for request
        let begin = std::time::SystemTime::now();

        // request information
        let path = req.path().to_string();
        let method = req.method().as_str().to_string();
        let protocol = req.connection_info().scheme().to_string();
        let ip_addr = req.connection_info().realip_remote_addr().unwrap().to_string();
        let queries = req.query_string().to_string();
        
        // Clone the Rc pointers so we can move them into the async block.
        let srv = self.service.clone();

        // Pinning for mitigating self reference errors by stoping their movement
        Box::pin(async move {

            // Extract request body
            // This operation consumes the body
            let mut request_body = BytesMut::new();
            while let Some(chunk) = req.take_payload().next().await {
                request_body.extend_from_slice(&chunk?);
            }

            // Reappend the body to the request
            let (_payload_sender, mut orig_payload) = Payload::create(true);
            orig_payload.unread_data(request_body.clone().freeze());
            req.set_payload(actix_http::Payload::from(orig_payload));

            // Get elapsed time
            let duration = begin.elapsed().unwrap().as_millis();

            // Log info
            log::debug!("Client Request: path: {}, method: {}, scheme: {}, ip: {}, queries: {}, duration: {}ms", path, method, protocol, ip_addr, queries, duration);
            log::debug!("Client Request body: {:?}", request_body);

            // Finally process the response from the body
            let res = srv.call(req).await?;

            log::debug!("Client Response: {:?}", res.headers());

            Ok(res)
        })
    }

}

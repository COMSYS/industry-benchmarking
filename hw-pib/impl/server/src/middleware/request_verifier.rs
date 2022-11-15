use std::{rc::Rc, sync::Arc};
use async_lock::RwLock;

use actix_web::{
    body::EitherBody,
    dev::{
        forward_ready,
        Service,
        ServiceRequest,
        ServiceResponse,
        Transform},
    Error, web::Data
};
use futures::{future::LocalBoxFuture, FutureExt};
use futures_util::future::{ready, Ready};
use rustls::Certificate;
use types::consts::X_APPLICATION_FIELD;
use std::str::FromStr;

use crate::server::BenchmarkingServer;

// Two steps in middleware processing:
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.

/// Signature Verification Middleware to check the authenticity of the requester
pub struct VerifyRequest {
    verify_company: bool,
}

/// Verify either routes for companies or for the analyst
#[allow(unused)]
impl VerifyRequest {
    pub fn verify_company() -> Self {
        VerifyRequest { verify_company: true }
    }

    pub fn verify_analyst() -> Self {
        VerifyRequest { verify_company: false }
    }
}

/// Middleware factory is of "Transform" trait
/// `S` - type of the next service
/// `B` - type of response's body
impl<S,B> Transform<S, ServiceRequest> for VerifyRequest
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static, 
{
    // Responses produced by the service.
    type Response = ServiceResponse<EitherBody<B>>;
    // Errors produced by the service.
    type Error = Error;
    // Errors produced while building a transform service.
    type InitError = ();
    // The `TransformService` value created by this factory, which is the VerifySignature"Service"
    type Transform = VerifySignatureMiddleware<S>;
    // The future response value.
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // Creates and returns a new instance of our middleware "service"
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(VerifySignatureMiddleware { 
            service: Rc::new(service),
            verify_company: self.verify_company,
        }))
    }

}

#[derive(Clone)]
pub struct VerifySignatureMiddleware<S> {
    service: Rc<S>,
    verify_company: bool,
}



impl<S,B> Service<ServiceRequest> for VerifySignatureMiddleware<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    // Responses given by the service.
    type Response = ServiceResponse<EitherBody<B>>;
    // Errors produced by the service when polling readiness
    type Error = Error;
    // Future Response value (promise)
    type Future =  LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // An implementation of poll_ready that forwards readiness checks to a named struct field.
    // poll_ready returns "Ready", whenever the service is able to process the request.
    forward_ready!(service);

    // Process the request and return the response asynchronously.
    fn call(&self, req: ServiceRequest) -> Self::Future {

        // Whether to verify the companies or the analyst
        let verify_company = self.verify_company;

        // Clone the Rc pointers so we can move them into the async block.
        let srv = self.service.clone();


        Box::pin(async move {

            let (analyst, companies) = {
                let srv_rdr = req.app_data::<Data<Arc<RwLock<BenchmarkingServer>>>>().unwrap().read().await;
                let analyst_info = srv_rdr.analyst().unwrap().clone();
                let companies = srv_rdr.companies().clone();
                (analyst_info, companies)
            };

            // Certificate of request
            let certificate = match req.conn_data::<Certificate>() {
                Some(cert) => cert.clone(),
                None => { return Ok(srv.call(req).map(map_body_left).await?); }
            };
            // log::info!("Certificate of request: {:?}", certificate);

            if verify_company {

                //
                // COMPANY VERIFICATION
                //

                // Extract the company ID from the header
                let company_id = req.headers().get(X_APPLICATION_FIELD).map_or("", |result| result.to_str().unwrap());
                let company_id_num = match u128::from_str(company_id) {
                    Ok(uuid) => uuid,
                    _ => { return Ok(req.into_response("Malformed UUID!").map_into_boxed_body().map_into_right_body()); }
                };
                log::debug!("UUID of Request for company {}", company_id_num);
                
                if let Some(company) = companies.get(&company_id_num) {
                    if let Some(company_certificate) = company.certificate() {
                        if company_certificate.eq(&certificate) {
                            Ok(srv.call(req).map(map_body_left).await?)
                        }else{
                            Ok(req.into_response("Certificate missmatch - no access granted!").map_into_boxed_body().map_into_right_body())
                        }
                    } else {
                        Ok(req.into_response("Please register your certificate first!").map_into_boxed_body().map_into_right_body())
                    }
                } else {
                    Ok(req.into_response("Company was not found!").map_into_boxed_body().map_into_right_body())
                }

            } else {

                //
                // ANALYST VERIFICATION
                //

                if analyst.certificate().eq(&certificate) {
                    Ok(srv.call(req).map(map_body_left).await?)
                }else{
                    Ok(req.into_response("Certificate missmatch - no access granted!").map_into_boxed_body().map_into_right_body())   
                }
            }
        })
    }
}

// Helper function for body mapping (left = OK)
fn map_body_left<B, E>(res: Result<ServiceResponse<B>, E>,) -> Result<ServiceResponse<EitherBody<B>>, E> {
    res.map(|res| res.map_into_left_body())
}
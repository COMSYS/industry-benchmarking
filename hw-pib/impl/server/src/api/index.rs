//! Basic information for HTTP server i.e. static pages and favicons (actually not very important)

use std::path::Path;

use actix_http::Method;
use actix_web::{Responder, web::Json, HttpRequest};
use actix_files::NamedFile;

use server_util::error::ApiError;
use types::message::response::RspMsg;
use types::consts::{SERVER_STATIC_PATH, SERVER_FAVICON, SERVER_EXT_FAVICON};
use rustls::Certificate;
use server_util::client_cert_extractor::ConnectionInfo;


/// Index handler
pub async fn index(req: HttpRequest) -> Result<impl Responder, ApiError> {

    match req.method() {
        &Method::GET => {
            Ok(Json(RspMsg::new(true, "TEEBench Benchmarking Application".to_string(), ())))
        },
        _ => {
            Err(ApiError::from(&("You might be using an incorrect method!".to_string())))
        }
    }
}

/// favicon handler
pub async fn favicon(req: HttpRequest) -> Result<impl Responder, ApiError> {
    match req.method() {
        &Method::GET => {
            let favicon_path = Path::new(SERVER_STATIC_PATH).join(SERVER_FAVICON).with_extension(SERVER_EXT_FAVICON);
            match NamedFile::open(favicon_path) {
                Ok(favicon) => Ok(favicon),
                _ => Err(ApiError::from(&("No favicon provided!".to_string())))
            }
        },
        _ => {
            Err(ApiError::from(&("You might be using an incorrect method!".to_string())))
        }
    }
}

/// Default handler
pub async fn default_handler() -> actix_web::HttpResponse{
    actix_web::HttpResponse::NotFound().json(
        RspMsg::new(false, "TEEBench Benchmarking Application".to_string(),"You might not be looking for this.".to_string())
    )
}

// Get connection information on the connecting peer (debug)
pub async fn whoami(req: HttpRequest) -> impl Responder {
    let conn_info = req.conn_data::<ConnectionInfo>().unwrap();
    let client_cert = req.conn_data::<Certificate>();
    let sni_hostname = req.conn_data::<String>();

    if let Some(cert) = client_cert {
        let cert_str = format!("{:?}", Some(&cert));
        Json(RspMsg::new(true,format!("Connection: [Server: {:?}, Peer: {:?}, SNI: {:?}]", conn_info.bind, conn_info.peer, &sni_hostname), cert_str))
    } else {
        Json(RspMsg::new(true,format!("Connection: [Server: {:?}, Peer: {:?}, SNI: {:?}]", conn_info.bind, conn_info.peer, &sni_hostname), "No certificate!".to_string()))
    }
}
//! **ROUTE HANDLERS**
//! 
//! - Setup benchmarking config with Certificates, YAML and Analyst HMAC
//! - The server `check_config` is for attestation purposes: Here the server
//!   shows his public key and attestation data.
//! - Shutdown kills the server and therefore cleans up all the memory.

use std::{sync::{mpsc, Arc}, fs};

use actix_multipart::Multipart;
use actix_web::{Result, Responder, web::{Json, Data}, web};
use rustls::Certificate;
use server_util::{error::ApiError, files::save_multipart_files, crypto_decode::parse_tls_certificate_from_path};
use async_lock::RwLock;

use types::{message::response::{RspMsg, ServerStatus}, entity::{BenchmarkingConfig, Analyst}, consts::{FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_NAME, FORM_DATA_FIELD_01_CONFIGURATION_NAME, FORM_DATA_FIELD_01_CONFIGURATION_MIME, FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_NAME, FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_MIME, FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_MIME}};

use crate::server::BenchmarkingServer;


 /// We receive in intial configuration:
 /// 
 /// - The server Root CA certificate for doing TLS authentication
 /// - The server configuration
 /// - The analysts certificate is given for authentication
pub async fn setup_config(srv: Data<Arc<RwLock<BenchmarkingServer>>>, payload: Multipart, stopper: web::Data<mpsc::Sender<()>>) -> Result<impl Responder, ApiError> {
    
    // Get instance for config
    let mut mut_srv = srv.write().await;

    // Check if server was previously configured and reject
    if mut_srv.crypto_config().root_ca_certificate().is_some() {
        log::info!("Server is already configured - Skipping reconfiguration..");
        return Err(ApiError::from(&("Configuration is locked after initial setting. You need to reset the server for reconfiguration!".to_string())));
    } 

    // Parse the received message (multipart)

    let required_multiparts = vec![
        (FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_NAME,FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_MIME),
        (FORM_DATA_FIELD_01_CONFIGURATION_NAME, FORM_DATA_FIELD_01_CONFIGURATION_MIME),
        (FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_NAME,FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_MIME)];
    let req_multipart_names: Vec<&str> = required_multiparts.iter().map(|x| x.0).collect();
    let required_files = 3;
    let files = save_multipart_files(payload,required_multiparts, required_files).await?;
    
    // Check the received root ca certificate (for rustls' client certification)
    let ca_certificate: Certificate = match parse_tls_certificate_from_path(files.get(req_multipart_names[0]).ok_or( ApiError::from("Did not find the root certificate to be correctly uploaded (incorrect name)!"))?) {
        Ok(ca_certificate) => { ca_certificate},
        _ => { return Err(ApiError::from(&("CA-Certificate parsing failed!".to_string()))); }
    };

    // Parse benchmarking server configuration information
    let data = fs::read_to_string(files.get(req_multipart_names[1]).ok_or(ApiError::from("Did not find yaml to be correctly uploaded (incorrect name)!"))?);
    let server_config: BenchmarkingConfig = serde_yaml::from_str::<BenchmarkingConfig>(&data.unwrap()).map_err(|e| ApiError::from(&e.to_string()))?;
    
    // Check the received root ca certificate (for rustls' client certification)
    let analyst_certificate: Certificate = match parse_tls_certificate_from_path(files.get(req_multipart_names[2]).ok_or( ApiError::from("Did not find the admin certificate to be correctly uploaded (incorrect name)!"))?) {
        Ok(ca_certificate) => { ca_certificate},
        _ => { return Err(ApiError::from(&("CA-Certificate parsing failed!".to_string()))); }
    };

    // Create config and set it in the Application
    mut_srv.set_analyst(Analyst::new(analyst_certificate));
    mut_srv.set_benchmarking_config_all(server_config.clone());
    mut_srv.set_crypto_config().set_root_ca_certificate(ca_certificate);

    // Free lock before killing server
    drop(mut_srv);

    // make request that sends message through the Sender
    // This request targets the main thread and invokes an
    // graceful shutdown on the HTTP Server.
    stopper.send(()).unwrap();

    // Respond with success
    Ok(
        Json(RspMsg::new(
            true,
            "Server setup successful! Connect now over Port 8443!".into(),
            server_config
        ))
    )
}

/// Return configuration in case there is already one
pub async fn check_config(srv: Data<Arc<RwLock<BenchmarkingServer>>>,) -> Result<impl Responder, ApiError> {
    
    let srv_rdr = srv.read().await;
    Ok(Json(
        RspMsg::new(
            true, 
            "Current Server Configuration".into(), 
            ServerStatus::new(srv_rdr.crypto_config().root_ca_certificate().is_some(), srv_rdr.crypto_config().enclave_certificate().encode_pem(), srv_rdr.benchmarking_config().clone()))
        )
    )
}

/// Shutdown of server and clearing of state
pub async fn shutdown(stopper: web::Data<mpsc::Sender<()>>) -> Result<impl Responder, ApiError> {
    
    // graceful shutdown on the HTTP Server
    stopper.send(()).unwrap();
    
    Ok(Json(RspMsg::new(true, "Server Configuration Cleared -- Shutting down!".into(), ())))
}
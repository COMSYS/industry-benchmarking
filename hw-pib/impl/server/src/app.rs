//! Application servers
//! 
//! There are 2 servers, that are provided by this application: one HTTP server and one HTTPs server
//! The HTTP server provides only the possibility to register and configure the server initially by 
//! uploading a configuration and CA_root file.
//! Then the HTTP server shuts down and the HTTPs server with the provided configuration starts up.
//! This ulimately enables a secure connection among all participants.

use rustls::{ServerConfig, server::AllowAnyAuthenticatedClient};
use server_util::{
    broadcast_event::Broadcaster,
    client_cert_extractor::get_client_cert
};


use actix_web::{App, HttpServer, web::{self}, dev::ServerHandle};

use std::sync::{mpsc, Arc};
use async_lock::RwLock;
use std::thread;

use crate::{
    server::BenchmarkingServer, crypto::Crypto, routes::{http_routes, https_routes}, api::index::default_handler,
};

/// HTTP configuration server startup
pub async fn http_server(benchmarking_server: Arc<RwLock<BenchmarkingServer>>) -> std::io::Result<()> {

        // create a channel for server shutdown
        let (tx, rx) = mpsc::channel::<()>();

        let (socket, shutdown_timeout, workers);
        {
            // HTTP Server configs
            let srv_rdr = benchmarking_server.read().await;
            socket = format!("{}:{}", srv_rdr.server_config().host(), srv_rdr.server_config().port());
            shutdown_timeout = srv_rdr.server_config().shutdown_timeout();
            workers = srv_rdr.server_config().http_workers();
        }

        let http_server= HttpServer::new(move|| {
            App::new()
                .app_data(web::Data::new(tx.clone()))       // For halting the server through IPC
                .app_data(web::Data::new(benchmarking_server.clone()))     // For modifying the config at runtime
                //.wrap(ReqLogging)                                                // Logging information on incomming msgs 
                //.wrap(RspLogging)                                                // Logging information on outgoing msgs 
                .configure(http_routes)                               // Configure routes for server
            .   default_service(web::route().to(default_handler))             // Default request handling
        })
        .on_connect(get_client_cert)
        .shutdown_timeout(shutdown_timeout)
        .bind(socket)?
        .workers(workers);
        
        log::info!("{:?} listening for {}-traffic", http_server.addrs_with_scheme()[0].0, http_server.addrs_with_scheme()[0].1);

        let server = http_server.run();
        let server_hanlde: ServerHandle = server.handle();

        // clone the server handle
        thread::spawn(move || {
            // wait for shutdown signal
            rx.recv().ok();

            log::warn!("Induced shutdown starting!");

            // send stop server gracefully command
            server_hanlde.stop(true)
        });

        server.await
}

/// HTTPs application server startup
pub async fn https_server(benchmarking_server: Arc<RwLock<BenchmarkingServer>>) -> std::io::Result<()> {

    // create a channel for server shutdown
    let (tx, rx) = mpsc::channel::<()>();

    // Create a broadcaster for server sent events
    let broadcaster = Broadcaster::new();

    // HTTP Server configs
    let (tls_socket, shutdown_timeout, crypto_config, workers);
    {
        // HTTP Server configs
        let srv_rdr = benchmarking_server.read().await;
        tls_socket = format!("{}:{}", srv_rdr.server_config().host(), srv_rdr.server_config().tls_port());
        shutdown_timeout = srv_rdr.server_config().shutdown_timeout();
        crypto_config = srv_rdr.crypto_config().clone();
        workers = srv_rdr.server_config().https_workers();
    }

    // Configure HTTPs server and bind it with rustls on the preconfigured port
    let https_server= HttpServer::new(move|| {
        App::new()
            .app_data(broadcaster.clone())
            .app_data(web::Data::new(tx.clone()))
            .app_data(web::Data::new(benchmarking_server.clone()))
            //.wrap(ReqLogging)
            //.wrap(RspLogging)
            .configure(https_routes)
            .default_service(web::route().to(default_handler))
    })
    .on_connect(get_client_cert)
    .shutdown_timeout(shutdown_timeout)
    .bind_rustls(
        tls_socket, 
        tls_setup(crypto_config)
    )?
    .workers(workers);

    log::info!("{:?} listening for {}-traffic", https_server.addrs_with_scheme()[0].0, https_server.addrs_with_scheme()[0].1);

    let server = https_server.run();
    let server_hanlde: ServerHandle = server.handle();

    // clone the server handle
    thread::spawn(move || {
        // wait for shutdown signal
        rx.recv().ok();

        // send stop server gracefully command
        server_hanlde.stop(true)
    });

    server.await
}

/// TLS configuration
/// 
/// Use RSA keys and Certificate for authentication
/// Use a provided CA_root Certificate for client Authentication
pub fn tls_setup(crypto_cfg: Crypto) -> rustls::ServerConfig {
    
    // This cert store has to be extended with a add method for DER-encoded certs
    let mut client_auth_roots = rustls::RootCertStore::empty();
    let root_ca = crypto_cfg.root_ca_certificate().clone().expect("[FATAL] Root CA Certificate is not provided!");

    // Extend cert store with analyst provided certificate
    // All signed certificates are accepted
    client_auth_roots.add(&root_ca).expect("[FATAL] Could not register root CA certificate!");

    log::info!("Root CA Certificate configured!");

    // Configure server with client certificate verifier
    let verifier = AllowAnyAuthenticatedClient::new(client_auth_roots);

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_client_cert_verifier(verifier);

    // load TLS key/cert files
    let server_certificate = crypto_cfg.server_certificate().clone();
    let server_private_key = crypto_cfg.server_private_key().clone();

    // log::info!("Server cert: {:?} , server pk {:?}", server_certificate, server_private_key);

    config.with_single_cert(vec![server_certificate.last().unwrap().clone()], server_private_key).expect("Could not configure TLS")    
}
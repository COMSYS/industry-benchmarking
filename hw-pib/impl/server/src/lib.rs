//! MAIN SERVER ENTRY POINT

use std::sync::Arc;
use async_lock::RwLock;

use crate::{app::{http_server, https_server}, server::BenchmarkingServer};

mod config;
mod server;
mod crypto;
mod app;
mod routes;
mod api;
mod benchmark;
mod middleware;

/// The entry point of the server
/// 
/// Since the server starts in an enclave, it cannot be
/// configured by anyone in the begining. Its interface
/// is thus very limited - only the entry point exists.
pub fn execute_server() -> std::io::Result<()> {
    
    // Initialize server
    let benchmarking_server = BenchmarkingServer::load();
    let shutdown_timeout = benchmarking_server.server_config().shutdown_timeout();
    let arc_benchmarking_server = Arc::new(RwLock::new(benchmarking_server));
    
    // Start HTTP setup server
    actix::System::new().block_on(async {
        http_server(arc_benchmarking_server.clone()).await
    }).expect("HTTP Server did not terminate successfully!");

    // In case the server got interrupted (i.e. no config provided) avoid crashing it
    if arc_benchmarking_server.try_read().expect("Could not acquire lock for shutdown check!").crypto_config().root_ca_certificate().is_none() {
        log::warn!("[WARN ADMIN] Server was not configured! Waiting for graceful server shutdown!");
        std::thread::sleep(core::time::Duration::from_secs(shutdown_timeout));
        std::process::exit(0);
    }

    log::info!("Server was configured! HTTP Server finally shut down -- Starting HTTPs Server!");

    // Start application server with rich functionality
    actix::System::new().block_on(async {
        https_server(arc_benchmarking_server.clone()).await
    }).expect("HTTPs Server did not terminate successfully!");


    Ok(())
}
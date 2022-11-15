use std::{collections::HashMap, path::PathBuf};

use benchmark::operation::OperationType;
use client::client_setup;
use config::{ClientConfiguration, EvalMode};
use types::consts::{
    CC_ANALYST_ALGORITHMS_KEY, CC_ANALYST_BENCHMARK_CONFIG_KEY, CC_ANALYST_CA_CERTIFICATE_KEY,
    CC_ANALYST_CERTIFICATE_KEY, CC_CLIENT_PKCS12_KEY, CC_CLIENT_SERVER_CA_CERTIFICATE,
    CC_COMPANY_INPUT_DATA_PATH_KEY, CC_SPECTATOR_EVAL_OUTPUT_KEY,
};

mod api;
mod client;
pub mod config;
mod connection;
mod error;

pub async fn execute_client(config: ClientConfiguration) -> Result<(), Box<dyn std::error::Error>> {
    // Start Client
    log::debug!("CLI Client starting!");
    client_setup(config)
        .await
        .expect("Could not perform client");

    Ok(())
}

/// Use same arguments as in CLI to create an analyst which is executed from within a program
pub async fn execute_client_analyst(
    server_host: String,
    server_http: u16,
    server_https: u16,
    server_cert_path: PathBuf,
    client_pkcs12_path: PathBuf,
    analyst_ca_cert_path: PathBuf,
    analyst_certificate_path: PathBuf,
    algorithm_path: PathBuf,
    benchmarking_config_path: PathBuf,
    offload: Option<Vec<OperationType>>,
) {
    // Verify that all paths indeed exist
    if !server_cert_path.exists() {
        panic!("Server certificate does not exist!");
    }
    if !client_pkcs12_path.exists() {
        panic!("Client PFX does not exist!");
    }
    if !analyst_ca_cert_path.exists() {
        panic!("Analyst CA certificate does not exist!");
    }
    if !analyst_certificate_path.exists() {
        panic!("Analyst certificate does not exist!");
    }
    if !algorithm_path.exists() {
        panic!("Algorithms file does not exist!");
    }
    if !benchmarking_config_path.exists() {
        panic!("Benchmarking configuration file does not exist!");
    }

    // Push them into the map
    let mut paths: HashMap<String, PathBuf> = HashMap::new();
    paths.insert(
        CC_CLIENT_SERVER_CA_CERTIFICATE.to_string(),
        server_cert_path,
    );
    paths.insert(CC_CLIENT_PKCS12_KEY.to_string(), client_pkcs12_path);
    paths.insert(
        CC_ANALYST_CA_CERTIFICATE_KEY.to_string(),
        analyst_ca_cert_path,
    );
    paths.insert(
        CC_ANALYST_CERTIFICATE_KEY.to_string(),
        analyst_certificate_path,
    );
    paths.insert(CC_ANALYST_ALGORITHMS_KEY.to_string(), algorithm_path);
    paths.insert(
        CC_ANALYST_BENCHMARK_CONFIG_KEY.to_string(),
        benchmarking_config_path,
    );

    // Create client configuration and execute
    let cc = ClientConfiguration::new(
        server_host,
        server_http,
        server_https,
        paths,
        (false, EvalMode::Unencrypted),
        offload,
        None,
    );
    client_setup(cc)
        .await
        .expect("Could not start lib-analyst-client");
}

/// Use same arguments as in CLI to create a spectator for eval logging which is executed from within a program
pub async fn execute_client_spectator(
    server_host: String,
    server_http: u16,
    server_https: u16,
    eval_type: EvalMode,
    server_cert_path: PathBuf,
    client_pkcs12_path: PathBuf,
    eval_output_path: PathBuf,
    offload: Option<Vec<OperationType>>,
) {
    // Verify that all paths indeed exist
    if !server_cert_path.exists() {
        panic!("Server certificate does not exist!");
    }
    if !client_pkcs12_path.exists() {
        panic!("Client PFX does not exist!");
    }

    // Push them into the map
    let mut paths: HashMap<String, PathBuf> = HashMap::new();
    paths.insert(
        CC_CLIENT_SERVER_CA_CERTIFICATE.to_string(),
        server_cert_path,
    );
    paths.insert(CC_CLIENT_PKCS12_KEY.to_string(), client_pkcs12_path);
    paths.insert(CC_SPECTATOR_EVAL_OUTPUT_KEY.to_string(), eval_output_path);

    let eval_mode = (true, eval_type);

    // Create client configuration and execute
    let cc = ClientConfiguration::new(
        server_host,
        server_http,
        server_https,
        paths,
        eval_mode,
        offload,
        None,
    );
    client_setup(cc)
        .await
        .expect("Could not start lib-spectator-client");
}

/// Similarly for the client: Use same arguments as in CLI to execute from within a program
pub async fn execute_client_company(
    server_host: String,
    server_http: u16,
    server_https: u16,
    server_cert_path: PathBuf,
    client_pkcs12_path: PathBuf,
    input_data_path: PathBuf,
    uuid: u128,
    offload: Option<Vec<OperationType>>,
) {
    // Verify that all paths indeed exist
    if !server_cert_path.exists() {
        panic!("Server certificate does not exist!");
    }
    if !client_pkcs12_path.exists() {
        panic!("Client PFX does not exist!");
    }

    // Push them into the map
    let mut paths: HashMap<String, PathBuf> = HashMap::new();
    paths.insert(
        CC_CLIENT_SERVER_CA_CERTIFICATE.to_string(),
        server_cert_path,
    );
    paths.insert(CC_CLIENT_PKCS12_KEY.to_string(), client_pkcs12_path);
    paths.insert(CC_COMPANY_INPUT_DATA_PATH_KEY.to_string(), input_data_path);

    // Create client configuration and execute
    let cc = ClientConfiguration::new(
        server_host,
        server_http,
        server_https,
        paths,
        (false, EvalMode::Unencrypted),
        offload,
        Some(uuid),
    );
    client_setup(cc)
        .await
        .expect("Could not start lib-company-client");
}

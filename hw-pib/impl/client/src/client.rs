//! Client configuration
//!
//! Create a TLS enabled client that can post requests to a Teebench
//! application server. There are two types of clients: analyst and
//! companies, that are possible roles.

use std::fs::File;
use std::io::Read;

use crate::config::{ClientConfiguration, ClientType};
use crate::connection::analyst::AnalystConnection;
use crate::connection::company::CompanyConnection;
use crate::connection::connection::TeebenchClient;
use crate::connection::spectator::SpectatorConnection;
use crate::connection::state::ClientConnection;
use reqwest::{header, Certificate, Client, ClientBuilder, Identity};
use types::consts::{CC_CLIENT_PKCS12_KEY, CC_CLIENT_SERVER_CA_CERTIFICATE, X_APPLICATION_FIELD};

/// HTTP configuration server startup
///
/// We use only one client as the authentication does not interfere
/// with HTTP requests and traffic.
pub(crate) async fn client_setup(config: ClientConfiguration) -> std::io::Result<()> {
    // Create teebench https client
    let https_client = req_client(&config);

    // Start connection
    let mut teebench_client = application_client(&config, https_client);

    // Play state machine
    teebench_client.run().await;

    Ok(())
}

fn req_client(config: &ClientConfiguration) -> Client {
    // Read PKCS12 Key file containing the private key and the client certificate
    let mut pkcs12_buf = Vec::new();
    File::open(
        config
            .paths()
            .get(CC_CLIENT_PKCS12_KEY)
            .expect("No pkcs file found!"),
    )
    .unwrap()
    .read_to_end(&mut pkcs12_buf)
    .expect("Could not read PKCS12");
    let ident =
        Identity::from_pkcs12_der(&pkcs12_buf, "").expect("Could not parse PLCS12 PFX File");

    // Read the server root certificate and create the client
    let mut server_cert_buf = Vec::new();
    File::open(
        config
            .paths()
            .get(CC_CLIENT_SERVER_CA_CERTIFICATE)
            .expect("No server certificate!"),
    )
    .unwrap()
    .read_to_end(&mut server_cert_buf)
    .expect("Could not read servercert");
    let server_root_ca_certificate =
        Certificate::from_pem(&server_cert_buf).expect("Could not parse server certificate!");

    // Create client with TLS authentication and server root CA
    let client_build = ClientBuilder::new()
        .user_agent("teebench-application")
        //.tcp_keepalive(Some(std::time::Duration::new(300, 0)))
        .add_root_certificate(server_root_ca_certificate)
        .pool_max_idle_per_host(0)
        .identity(ident);

    match config.role() {
        ClientType::Analyst => {
            // Create standard client without uuid
            client_build
                .build()
                .expect("Could not build analyst client!")
        }
        ClientType::Company(uuid) => {
            // Put client UUID into all requests
            let mut client_header = header::HeaderMap::new();
            client_header.insert(
                X_APPLICATION_FIELD,
                header::HeaderValue::from_str(uuid.to_string().as_str()).expect("UUID is invalid!"),
            );

            client_build
                .default_headers(client_header)
                .build()
                .expect("Could not build company client!")
        }
        ClientType::Spectator => client_build
            .build()
            .expect("Could not build spectator client!"),
    }
}

fn application_client(config: &ClientConfiguration, client: Client) -> TeebenchClient {
    match config.role() {
        ClientType::Analyst => {
            let analyst_conn = AnalystConnection::new(
                client,
                config.server_host().clone(),
                config.server_http_port().to_string(),
                config.server_https_port().to_string(),
                config.paths(),
                None,
                config.eval_mode(),
            );
            TeebenchClient::Analyst(analyst_conn)
        }
        ClientType::Company(uuid) => {
            let company_conn = CompanyConnection::new(
                client,
                config.server_host().clone(),
                config.server_http_port().to_string(),
                config.server_https_port().to_string(),
                config.paths(),
                Some(*uuid),
                config.eval_mode(),
            );
            TeebenchClient::Company(company_conn)
        }
        ClientType::Spectator => {
            let spec_conn = SpectatorConnection::new(
                client,
                config.server_host().clone(),
                config.server_http_port().to_string(),
                config.server_https_port().to_string(),
                config.paths(),
                None,
                config.eval_mode(),
            );
            TeebenchClient::Spectator(spec_conn)
        }
    }
}

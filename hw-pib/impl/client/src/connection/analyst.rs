//! Analyst state machine and internal behaviour

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest::{multipart, Client};
use serde_json::json;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Write},
    path::PathBuf,
};

use crate::{
    api::{TeebenchHttpAPI, TeebenchHttpsAPI},
    config::EvalMode,
    error::{AbstractClientErrorType, ClientError},
};

use types::{
    consts::{
        CC_ANALYST_ALGORITHMS_KEY, CC_ANALYST_BENCHMARK_CONFIG_KEY, CC_ANALYST_CA_CERTIFICATE_KEY,
        CC_ANALYST_CERTIFICATE_KEY, FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_MIME,
        FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_NAME, FORM_DATA_FIELD_01_CONFIGURATION_MIME,
        FORM_DATA_FIELD_01_CONFIGURATION_NAME, FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_MIME,
        FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_NAME, FORM_DATA_FIELD_02_ALGORIHTMS_NAME,
        FORM_DATA_FIELD_02_ALGORITHMS_MIME,
    },
    entity::BenchmarkingConfig,
    message::{io::CompanyUUIDs, request::AnalystEventMsg, response::RspMsg},
};

use super::state::{ClientConnection, Event, StateMachine};

static COMPANY_UUID_PATH: &str = "../data/client_data/uuids.yaml";
static ANALYST_FINISHED_PATH: &str = "../data/client_data/finished";

/// Analyst Connection has connection Strings
/// and a state which has to be initialized.
pub struct AnalystConnection {
    /// Conncetion information
    conn_info: AnalystConnectionInfo,
    /// The state machine itself
    state: AnalystState,
}

/// The signature for Analyst an Company are very similar and allow creation and
/// running of the connection.
#[async_trait]
impl ClientConnection for AnalystConnection {
    /// Create an analyst connection from analyst configuration information
    /// that is passed internally to create the client connection.
    fn new(
        client: Client,
        host: String,
        http_port: String,
        https_port: String,
        paths: &HashMap<String, PathBuf>,
        _uuid: Option<u128>,
        _eval_mode: EvalMode,
    ) -> Self {
        // Get the paths for the analyst
        let analyst_ca_cert_path = paths.get(CC_ANALYST_CA_CERTIFICATE_KEY).unwrap().clone();
        let benchmark_config_path = paths.get(CC_ANALYST_BENCHMARK_CONFIG_KEY).unwrap().clone();
        let analyst_cert_path = paths.get(CC_ANALYST_CERTIFICATE_KEY).unwrap().clone();
        let algorithms_path = paths.get(CC_ANALYST_ALGORITHMS_KEY).unwrap().clone();

        // Get quantity, i.e. the number of participants from config file
        let f = std::fs::File::open(paths.get(CC_ANALYST_BENCHMARK_CONFIG_KEY).unwrap().clone())
            .expect("Could not read benchmarking configuration!");
        let open_file_reader = BufReader::new(f);
        let benchmarking_config: BenchmarkingConfig = serde_yaml::from_reader(open_file_reader)
            .expect("Invalid Benchmarking Configuration Format! Could not read quantity");

        let conn_info = AnalystConnectionInfo::new(
            client,
            host,
            http_port,
            https_port,
            analyst_ca_cert_path,
            benchmark_config_path,
            analyst_cert_path,
            algorithms_path,
            benchmarking_config.k_anonymity(),
        );

        AnalystConnection {
            conn_info,
            state: AnalystState::init(),
        }
    }

    /// Start the state machine for the analyst
    async fn run(&mut self) {
        while !self.state.eq(&AnalystState::Shutdown) {
            match self.state.run(&self.conn_info).await {
                Ok(_) => {
                    log::debug!("[SUCCESS] Current State: {:?}", self.state);
                    self.state = self.state.next(Event::SuccessfulResponse);
                }
                Err(err) => {
                    log::error!("{:?}", err);
                    self.state = self.state.next(Event::ErrorResponse(err.to_string()));
                    break;
                }
            };
        }
    }
}

#[derive(Clone)]
/// Internal data structure to perform connections with the analyst
struct AnalystConnectionInfo {
    /// Connection Specific
    client: Client,
    /// API Routes
    routes_http: TeebenchHttpAPI,
    routes_https: TeebenchHttpsAPI,
    /// Setup state
    analyst_ca_cert_path: PathBuf,
    benchmark_config_path: PathBuf,
    analyst_cert_path: PathBuf,
    /// Algorithm upload state
    algorithms_path: PathBuf,
    /// Company enroll state
    quantity: u64,
}

impl AnalystConnectionInfo {
    /// Create new analyst connection info
    fn new(
        client: Client,
        host: String,
        http_port: String,
        https_port: String,
        analyst_ca_cert_path: PathBuf,
        benchmark_config_path: PathBuf,
        analyst_cert_path: PathBuf,
        algorithms_path: PathBuf,
        quantity: u64,
    ) -> Self {
        let routes_http = TeebenchHttpAPI::new(host.clone(), http_port);
        let routes_https = TeebenchHttpsAPI::new(host, https_port);

        AnalystConnectionInfo {
            algorithms_path,
            analyst_cert_path,
            benchmark_config_path,
            analyst_ca_cert_path,
            client,
            quantity,
            routes_http,
            routes_https,
        }
    }
}

#[derive(Debug, PartialEq)]
/// This state machine describes the way a connecion
/// is setup for an analyst. First he uploads the
/// configuration for the server, then his algorithms.
/// After that he enrolls the given number of companies.
/// Finally he waits until the server reports to have
/// enough companies enrolled and starts the benchmark.
/// When the benchmarking started, the analysts duty is
/// over.
enum AnalystState {
    ServerUnconfigured,
    ServerConfigured,
    AlgorithmsUploaded,
    CompaniesEnrolled,
    CompaniesReady,
    BenchmarkingStarted,
    BenchmarkingComplete,
    Shutdown,
    Failure(String),
}

/// Implement basic state machine behaviour for analyst connection
/// See [`AnalystState`] for more information on transitions.
#[async_trait]
impl StateMachine<AnalystConnectionInfo> for AnalystState {
    fn init() -> AnalystState {
        AnalystState::ServerUnconfigured
    }

    fn next(&self, event: Event) -> AnalystState {
        match (self, event) {
            (AnalystState::ServerUnconfigured, Event::SuccessfulResponse) => {
                AnalystState::ServerConfigured
            }
            (AnalystState::ServerUnconfigured, Event::ErrorResponse(reason)) => {
                AnalystState::Failure(
                    "Server configuration failed with reason: ".to_owned() + &reason,
                )
            }

            (AnalystState::ServerConfigured, Event::SuccessfulResponse) => {
                AnalystState::AlgorithmsUploaded
            }
            (AnalystState::ServerConfigured, Event::ErrorResponse(reason)) => {
                AnalystState::Failure("Algorithm upload failed with reason: ".to_owned() + &reason)
            }

            (AnalystState::AlgorithmsUploaded, Event::SuccessfulResponse) => {
                AnalystState::CompaniesEnrolled
            }
            (AnalystState::AlgorithmsUploaded, Event::ErrorResponse(reason)) => {
                AnalystState::Failure(
                    "Server configuration failed with reason: ".to_owned() + &reason,
                )
            }

            (AnalystState::CompaniesEnrolled, Event::SuccessfulResponse) => {
                AnalystState::CompaniesReady
            }
            (AnalystState::CompaniesEnrolled, Event::ErrorResponse(reason)) => {
                AnalystState::Failure("Algorithm upload failed with reason: ".to_owned() + &reason)
            }

            (AnalystState::CompaniesReady, Event::SuccessfulResponse) => {
                AnalystState::BenchmarkingStarted
            }
            (AnalystState::CompaniesReady, Event::ErrorResponse(reason)) => AnalystState::Failure(
                "Server configuration failed with reason: ".to_owned() + &reason,
            ),

            (AnalystState::BenchmarkingStarted, Event::SuccessfulResponse) => {
                AnalystState::BenchmarkingComplete
            }
            (AnalystState::BenchmarkingStarted, Event::ErrorResponse(reason)) => {
                AnalystState::Failure("Algorithm upload failed with reason: ".to_owned() + &reason)
            }

            (AnalystState::BenchmarkingComplete, Event::SuccessfulResponse) => {
                AnalystState::Shutdown
            }
            (AnalystState::BenchmarkingComplete, Event::ErrorResponse(reason)) => {
                AnalystState::Failure("Algorithm upload failed with reason: ".to_owned() + &reason)
            }

            (AnalystState::Shutdown, Event::SuccessfulResponse) => AnalystState::Shutdown,
            (AnalystState::Shutdown, Event::ErrorResponse(reason)) => {
                AnalystState::Failure("Algorithm upload failed with reason: ".to_owned() + &reason)
            }

            (AnalystState::Failure(reason), _) => AnalystState::Failure(reason.clone()),
        }
    }

    /// Suplementary operations that get perfromed depending on the state
    /// of the connection the analyst has.
    async fn run(&self, conn_info: &AnalystConnectionInfo) -> Result<(), ClientError> {
        match *self {
            AnalystState::ServerUnconfigured => {
                log::debug!("[ST] ServerUnconfigured - Performing server configuration!");

                // Create multipart
                let form = multipart::Form::new();

                // Byte streams of files to send
                let mut ca_cert_buf = Vec::new();
                let mut config_buf = Vec::new();
                let mut cert_buf = Vec::new();

                // Populate byte streams
                File::open(conn_info.analyst_ca_cert_path.clone())
                    .unwrap()
                    .read_to_end(&mut ca_cert_buf)
                    .expect("Could not read Analyst CA Certificate");
                File::open(conn_info.benchmark_config_path.clone())
                    .unwrap()
                    .read_to_end(&mut config_buf)
                    .expect("Could not read Benchmark Config");
                File::open(conn_info.analyst_cert_path.clone())
                    .unwrap()
                    .read_to_end(&mut cert_buf)
                    .expect("Could not read Analyst Certificate");

                // Add up streams to the form
                let root_ca_cert = multipart::Part::bytes(ca_cert_buf)
                    .file_name(FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_NAME)
                    .mime_str(FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_MIME)
                    .expect("Could not create Root CA Certificate Part!");

                let benchmarking_config = multipart::Part::bytes(config_buf)
                    .file_name(FORM_DATA_FIELD_01_CONFIGURATION_NAME)
                    .mime_str(FORM_DATA_FIELD_01_CONFIGURATION_MIME)
                    .expect("Could not create Root CA Certificate Part!");

                let analyst_certificate = multipart::Part::bytes(cert_buf)
                    .file_name(FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_NAME)
                    .mime_str(FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_MIME)
                    .expect("Could not create Root CA Certificate Part!");

                let form = form
                    .part(FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_NAME, root_ca_cert)
                    .part(FORM_DATA_FIELD_01_CONFIGURATION_NAME, benchmarking_config)
                    .part(
                        FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_NAME,
                        analyst_certificate,
                    );

                // Send post request
                let rsp = conn_info
                    .client
                    .post(conn_info.routes_http.setup())
                    .multipart(form)
                    .send()
                    .await;

                // Parse response
                match rsp {
                    Ok(e) => {
                        log::debug!("Got rsp! {:?}", e);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Error! {:?}", e);
                        return Err(ClientError::from((
                            AbstractClientErrorType::NoConnection,
                            e.to_string(),
                        )));
                    }
                }
            }
            AnalystState::ServerConfigured => {
                log::debug!("[ST] ServerConfigured - Uploading algorithms to server!");

                // Wait for the server to restart
                log::info!("Waiting 1.3 seconds for the server to restart!");
                tokio::time::sleep(tokio::time::Duration::from_millis(1300)).await;

                // Create multipart
                let form = multipart::Form::new();

                let mut algorithms = Vec::new();
                File::open(conn_info.algorithms_path.clone())
                    .unwrap()
                    .read_to_end(&mut algorithms)
                    .expect("Could not read algorithms file!");

                let algorithms = multipart::Part::bytes(algorithms)
                    .file_name(FORM_DATA_FIELD_02_ALGORIHTMS_NAME)
                    .mime_str(FORM_DATA_FIELD_02_ALGORITHMS_MIME)
                    .expect("Could not create algorithms Part!");

                let form = form.part(FORM_DATA_FIELD_02_ALGORIHTMS_NAME, algorithms);

                // Send post request
                let rsp = conn_info
                    .client
                    .post(conn_info.routes_https.analyst_algorithms())
                    .multipart(form)
                    .send()
                    .await;

                // Parse response
                match rsp {
                    Ok(e) => {
                        log::debug!("[SUCCESS] [[ST] ServerConfigured]:  {:?}", e);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Error! {:?}", e);
                        return Err(ClientError::from((
                            AbstractClientErrorType::NoConnection,
                            e.to_string(),
                        )));
                    }
                }
            }
            AnalystState::AlgorithmsUploaded => {
                log::debug!(
                    "[ST] AlgorithmsUploaded - Registering the companies to upload their data!"
                );

                let mut uuids = Vec::<u128>::with_capacity(conn_info.quantity as usize);
                // Enroll companies
                for _ in 0..conn_info.quantity {
                    // Set request and save to vector
                    let enroll_req = conn_info
                        .client
                        .post(conn_info.routes_https.analyst_enroll_company())
                        .send()
                        .await;
                    if let Ok(rsp) = enroll_req {
                        let json_body = rsp.json::<RspMsg<u128>>().await.unwrap();
                        let uuid = json_body.content;
                        uuids.push(uuid);
                    } else {
                        return Err(ClientError::from((
                            AbstractClientErrorType::BadRequest,
                            "Could not create company ID".to_string(),
                        )));
                    }
                }

                // Write uuids to file for the analyst to share them
                let comp_uuids: CompanyUUIDs = CompanyUUIDs { comps: uuids };
                let yaml_out_str = serde_yaml::to_string(&comp_uuids).unwrap();
                log::debug!("Enrolled company UUIDs: {:?}", yaml_out_str);
                let mut uuid_file =
                    File::create(COMPANY_UUID_PATH).expect("Could not create UUID File!");
                uuid_file
                    .write_all(yaml_out_str.as_bytes())
                    .expect("Could not write to UUID file!");

                Ok(())
            }
            AnalystState::CompaniesReady => {
                log::debug!("[ST] CompaniesReady - Waiting for the required companies to enroll!");
                // Wait for SSE to report that all participants are OK
                let event_stream = conn_info
                    .client
                    .get(conn_info.routes_https.get_events())
                    .send()
                    .await;

                match event_stream {
                    Ok(rsp) => {
                        // Get event stream
                        let mut stream = rsp.bytes_stream().eventsource();

                        // Listen for events and exit whenever the event is `Ready`
                        while let Some(server_event) = stream.next().await {
                            match server_event {
                                Ok(event) => {
                                    log::debug!(
                                        "Event received: {:?} Type: {:?}",
                                        event.data,
                                        event.event
                                    );

                                    match event.data.as_str() {
                                        "keep-alive" => {
                                            log::debug!("Server keeps connection alive!");
                                        }
                                        "all-participants-enrolled" => {
                                            log::debug!("All participants are enrolled! - Taking state transition!");
                                            return Ok(());
                                        }
                                        _ => {}
                                    }
                                }
                                Err(e) => {
                                    log::error!(
                                        "Server event error during company enroll wait: {:?}",
                                        e
                                    );
                                    return Err(ClientError::from((
                                        AbstractClientErrorType::NotFound,
                                        "Server did not respond correctly".to_string(),
                                    )));
                                }
                            }
                        }
                        Ok(())
                    }
                    Err(e) => {
                        log::error!("Could not establish connection for event stream: {}", e);
                        Ok(())
                    }
                }
            }
            AnalystState::BenchmarkingStarted => {
                log::debug!("[ST] BenchmarkingStarted - Waiting for the server to complete!");

                let rsp = conn_info
                    .client
                    .post(conn_info.routes_https.analyst_benchmark())
                    .json(&json!({ "selected_kpis": null }))
                    .send()
                    .await;

                // Parse response
                match rsp {
                    Ok(_) => {
                        // Wait for SSE to report that the benchmark is finished - then kill process
                        let event_stream = conn_info
                            .client
                            .get(conn_info.routes_https.get_events())
                            .send()
                            .await;

                        match event_stream {
                            Ok(rsp) => {
                                // Get event stream
                                let mut stream = rsp.bytes_stream().eventsource();

                                while let Some(server_event) = stream.next().await {
                                    match server_event {
                                        Ok(event) => {
                                            log::debug!(
                                                "Event received: {:?} Type: {:?}",
                                                event.data,
                                                event.event
                                            );

                                            match event.data.as_str() {
                                                // Wait for server to finish benchmark
                                                "benchmarking-success" => {
                                                    log::debug!("[SUCCESS] Server has computed all the results!");
                                                    return Ok(());
                                                }
                                                "keep-alive" => {
                                                    log::debug!("Server keeps connection alive!");
                                                }
                                                "connected" => {
                                                    log::debug!(
                                                        "Connection established to SSE Stream."
                                                    );
                                                }
                                                _ => {}
                                            }
                                        }
                                        Err(e) => {
                                            log::error!(
                                                "Server event error during benchmark: {:?}",
                                                e
                                            );
                                            return Err(ClientError::from((
                                                AbstractClientErrorType::NotFound,
                                                "Server did not respond correctly".to_string(),
                                            )));
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Could not establish server event stream connection during benchmark: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error! {:?}", e);
                        return Err(ClientError::from((
                            AbstractClientErrorType::NoConnection,
                            e.to_string(),
                        )));
                    }
                }
                return Ok(());
            }
            AnalystState::BenchmarkingComplete => {
                let event = json!(AnalystEventMsg {
                    event: "Benchmark sucessfully completed! Analyst is happy!".to_string()
                });
                log::debug!("Sending event: {}", event);

                // Send additional message to all participants
                conn_info
                    .client
                    .post(conn_info.routes_https.analyst_send_event())
                    .json(&event)
                    .send()
                    .await
                    .ok();

                // Shutdown the server
                let rsp = conn_info
                    .client
                    .post(conn_info.routes_https.shutdown())
                    .send()
                    .await;
                match rsp {
                    Ok(success) => {
                        log::info!(
                            "Server shutdown successfully: {:?}",
                            success.json::<RspMsg<()>>().await.unwrap().message
                        );
                        log::debug!("[My duty is fulfilled] I guess I die.");
                    }
                    Err(e) => {
                        log::error!("Could not shutdown server: {}", e);
                    }
                }

                // Wait before finishing
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                // Write file to indicate finishing
                let mut finish_file =
                    File::create(ANALYST_FINISHED_PATH).expect("Could not create finished File!");
                finish_file
                    .write_all(b"")
                    .expect("Could not write to finish file!");

                // Client does shutdown now
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

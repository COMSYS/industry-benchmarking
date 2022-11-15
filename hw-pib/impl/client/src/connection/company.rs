//! Company connection state machine and behaviour

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest::{multipart, Client};
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use crate::{
    api::TeebenchHttpsAPI,
    config::EvalMode,
    connection::state::{ClientConnection, Event, StateMachine},
    error::{AbstractClientErrorType, ClientError},
};

use types::consts::{
    CC_COMPANY_INPUT_DATA_PATH_KEY, FORM_DATA_FIELD_04_COMPANY_INPUT_MIME,
    FORM_DATA_FIELD_04_COMPANY_INPUT_NAME,
};

/// Company Connection has connection information
/// and a state machine which runs its "role".
pub struct CompanyConnection {
    /// Conncetion information
    conn_info: CompanyConnectionInfo,
    /// The state machine itself
    state: CompanyState,
}

/// The signature Company are very similar and allow creation and
/// running of the connection.
#[async_trait]
impl ClientConnection for CompanyConnection {
    /// Create an company connection from configuration information
    /// that is passed internally to create the client connection.
    /// The http port is not required for the companies since setup
    /// is already complete by now.
    fn new(
        client: Client,
        host: String,
        _http_port: String,
        https_port: String,
        paths: &HashMap<String, PathBuf>,
        uuid: Option<u128>,
        _eval_mode: EvalMode,
    ) -> Self {
        // Get the paths for the analyst and check for UUID
        let company_input_data_path = paths.get(CC_COMPANY_INPUT_DATA_PATH_KEY).unwrap().clone();
        if uuid.is_none() {
            panic!("No uuid provided!");
        }

        let conn_info = CompanyConnectionInfo::new(
            client,
            host,
            https_port,
            company_input_data_path,
            uuid.unwrap(),
        );

        CompanyConnection {
            conn_info,
            state: CompanyState::init(),
        }
    }

    /// Start the state machine for the company.
    /// For understanding ist see [`CompanyState`].
    async fn run(&mut self) {
        while !self.state.eq(&CompanyState::BenchmarkingComplete) {
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
/// Internal data structure to perform connections with the company
struct CompanyConnectionInfo {
    /// Connection Specific
    client: Client,
    /// API Routes (only https required)
    routes_https: TeebenchHttpsAPI,
    /// Input data for upload
    company_input_data: PathBuf,
    /// Company UUID
    uuid: u128,
}

impl CompanyConnectionInfo {
    /// Create new analyst connection info
    fn new(
        client: Client,
        host: String,
        https_port: String,
        company_input_data: PathBuf,
        uuid: u128,
    ) -> Self {
        CompanyConnectionInfo {
            company_input_data,
            client,
            uuid,
            routes_https: TeebenchHttpsAPI::new(host, https_port),
        }
    }
}

#[derive(Debug, PartialEq)]
/// This state machine describes the way a connecion
/// is setup for a company. First it registers itself
/// to make the certificate known to the server, then
/// the input data is uploaded. The analyst does himself
/// the benchmarking process and thus the result is
/// received in the end, which is also downloaded and
/// displayed.
enum CompanyState {
    Unregistered,
    NotParticipating,
    DataUploaded,
    ResultsReady,
    BenchmarkingComplete,
    Failure(String),
}

/// Implement basic state machine behaviour for analyst connection
#[async_trait]
impl StateMachine<CompanyConnectionInfo> for CompanyState {
    fn init() -> CompanyState {
        CompanyState::Unregistered
    }

    fn next(&self, event: Event) -> CompanyState {
        match (self, event) {
            (CompanyState::Unregistered, Event::SuccessfulResponse) => {
                CompanyState::NotParticipating
            }
            (CompanyState::Unregistered, Event::ErrorResponse(reason)) => {
                CompanyState::Failure("Registration failed with reason: ".to_owned() + &reason)
            }

            (CompanyState::NotParticipating, Event::SuccessfulResponse) => {
                CompanyState::DataUploaded
            }
            (CompanyState::NotParticipating, Event::ErrorResponse(reason)) => {
                CompanyState::Failure("Input data upload failed with reason: ".to_owned() + &reason)
            }

            (CompanyState::DataUploaded, Event::SuccessfulResponse) => CompanyState::ResultsReady,
            (CompanyState::DataUploaded, Event::ErrorResponse(reason)) => {
                CompanyState::Failure("Algorithm upload failed with reason: ".to_owned() + &reason)
            }

            (CompanyState::ResultsReady, Event::SuccessfulResponse) => {
                CompanyState::BenchmarkingComplete
            }
            (CompanyState::ResultsReady, Event::ErrorResponse(reason)) => CompanyState::Failure(
                "Server configuration failed with reason: ".to_owned() + &reason,
            ),

            (CompanyState::BenchmarkingComplete, Event::SuccessfulResponse) => {
                CompanyState::BenchmarkingComplete
            }
            (CompanyState::BenchmarkingComplete, Event::ErrorResponse(reason)) => {
                CompanyState::Failure(
                    "Server configuration failed with reason: ".to_owned() + &reason,
                )
            }

            (CompanyState::Failure(reason), _) => CompanyState::Failure(reason.clone()),
        }
    }

    async fn run(&self, conn_info: &CompanyConnectionInfo) -> Result<(), ClientError> {
        match *self {
            CompanyState::Unregistered => {
                log::debug!("[ST] Unregistered - Registering to the server!");

                // Send registration request
                let rsp = conn_info
                    .client
                    .post(conn_info.routes_https.company_register(conn_info.uuid))
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
            CompanyState::NotParticipating => {
                log::debug!("[ST] NotParticipating - Uploading data to the server!");

                // Create multipart
                let form = multipart::Form::new();

                // Byte streams of files to send and populate byte streams
                let mut input_data_buf = Vec::new();
                File::open(conn_info.company_input_data.clone())
                    .unwrap()
                    .read_to_end(&mut input_data_buf)
                    .expect("Could not read Company input data");

                // Add up streams to the form
                let input_data = multipart::Part::bytes(input_data_buf)
                    .file_name(FORM_DATA_FIELD_04_COMPANY_INPUT_NAME)
                    .mime_str(FORM_DATA_FIELD_04_COMPANY_INPUT_MIME)
                    .expect("Could not create Input data part part!");

                // Append part to multipart request
                let form = form.part(FORM_DATA_FIELD_04_COMPANY_INPUT_NAME, input_data);

                // Send post request
                let rsp = conn_info
                    .client
                    .post(conn_info.routes_https.company_input_data(conn_info.uuid))
                    .multipart(form)
                    .send()
                    .await;

                // Parse response
                match rsp {
                    Ok(e) => {
                        log::debug!("Got response for upload: {:?}", e);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Error on upload: {:?}", e);
                        return Err(ClientError::from((
                            AbstractClientErrorType::NoConnection,
                            e.to_string(),
                        )));
                    }
                }
            }
            CompanyState::DataUploaded => {
                log::debug!("[ST] DataUploaded - Waiting for all the other participants to upload and the analyst to start!");

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

                                    // Wait for finalizing the benchmark on server side
                                    match event.data.as_str() {
                                        "benchmarking-success" => {
                                            log::debug!("Benchmarking complete! - Results are ready for retrieval!");
                                            return Ok(());
                                        }
                                        "keep-alive" => {
                                            log::debug!("Server keeps connection alive!");
                                        }
                                        _ => {}
                                    };
                                }
                                Err(e) => {
                                    log::error!("Server event error: {:?}", e);
                                    return Err(ClientError::from((
                                        AbstractClientErrorType::NotFound,
                                        "Server did not respond correctly".to_string(),
                                    )));
                                }
                            };
                        }
                    }
                    Err(e) => {
                        log::error!("Could not connect to server event stream {:?}!", e);
                    }
                };

                return Ok(());
            }
            CompanyState::ResultsReady => {
                log::debug!("[ST] ResultsReady - Getting the results from the server!");

                // Do not ask for results as they are not important in eval
                return Ok(());

                /*
                let rsp = conn_info.client
                    .get(conn_info.routes_https.company_results(conn_info.uuid))
                    .send()
                    .await;

                // Parse response
                match rsp {
                    Ok(e) => {
                        let result_msg = e.json::<RspMsg<Output>>().await.expect("Message could not be decoded");
                        log::debug!("Results for company benchmark: {:#?}", result_msg.content);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Error! {:?}", e);
                        return Err(ClientError::from((AbstractClientErrorType::NotFound, e.to_string())));
                    }
                };*/
            }
            CompanyState::BenchmarkingComplete => {
                log::debug!("[FINISH]  I guess I die.");
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

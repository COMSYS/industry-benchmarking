use std::{collections::HashMap, path::PathBuf};

use benchmark::operation::OperationType;
use clap::{Parser, Subcommand};

use strum::{Display, EnumString};
use types::consts::{
    CC_ANALYST_ALGORITHMS_KEY, CC_ANALYST_BENCHMARK_CONFIG_KEY, CC_ANALYST_CA_CERTIFICATE_KEY,
    CC_ANALYST_CERTIFICATE_KEY, CC_CLIENT_PKCS12_KEY, CC_CLIENT_SERVER_CA_CERTIFICATE,
    CC_COMPANY_INPUT_DATA_PATH_KEY,
};

/// Server Configuration
///
/// This configuration is preprovided and can only be
/// changed by the application developer. Any participant
/// has the possibility to alter the settings.
///
///
/// Command Line Interface Arguments

#[derive(Debug, Parser)]
#[clap(name = "teebench-client")]
#[clap(about = "Secure benchnarking application client written in Rust.", long_about = None)]
struct CLIArguments {
    /// Decision of whether to be a company or an analyst
    #[clap(subcommand)]
    command: TeeBenchCLISubcommands,
    /// Hostname of server
    #[clap(long, short = 'h')]
    server_host: String,
    /// HTTP port in case of analyst
    #[clap(long, short = 'u')]
    server_http: u16,
    /// HTTPs port for all others
    #[clap(long, short = 's')]
    server_https: u16,
    /// Crypto key for client certificate
    #[clap(long, short = 'c')]
    client_pkcs12_path: std::path::PathBuf,
    /// Crypto key for server CA certificate
    #[clap(long, short = 'p')]
    server_ca_certificate_path: std::path::PathBuf,
    /// Evaluation mode (whether offloading is used)
    #[clap(long, short = 'e')]
    eval_mode: EvalMode,
    /// Evaluation mode (which offloading ops)
    #[clap(long, short = 'o')]
    offload: Option<Vec<OperationType>>,
}

#[derive(Debug, Subcommand, Clone)]
pub(crate) enum TeeBenchCLISubcommands {
    /// Required for an analyst: algorithms and benchmarking config
    #[clap(arg_required_else_help = true)]
    Analyst {
        /// [For analyst] Personal Analyst CA for auth and id
        analyst_ca_cert_path: std::path::PathBuf,
        /// [For analyst] Enrolling over HTTP requires manual upload of certificate
        analyst_certificate_path: std::path::PathBuf,
        /// [For analyst] Data path for input uploads
        algorithm_path: std::path::PathBuf,
        /// [For analyst] Data path for benchmarking config upload
        benchmarking_config_path: std::path::PathBuf,
    },
    /// Required for a company: input data and own UUID
    #[clap(arg_required_else_help = true)]
    Company {
        /// [For companies] Data path for input uploads
        input_data_path: std::path::PathBuf,
        /// [For companies] UUID of specific company known from analyst
        uuid: u128,
    },
    /// For other: keep it free: this is debug
    #[clap(arg_required_else_help = true)]
    Other {},
}

#[derive(Debug, Clone)]
pub struct ClientConfiguration {
    /// Hostname of server
    server_host: String,
    /// HTTP port in case of analyst
    server_http: u16,
    /// HTTPs port for all others
    server_https: u16,
    /// Paths
    paths: HashMap<String, PathBuf>,
    /// Evaluation mode (whether offloading is used)
    eval_mode: EvalMode,
    /// [For companies in eval mode] Which operations to offload
    offload: Option<Vec<OperationType>>,
    /// Client role
    role: ClientType,
}

#[derive(Clone, Debug)]
pub(crate) enum ClientType {
    Analyst,
    Company(u128),
    Spectator,
}

#[derive(Debug, Clone, EnumString, Display)]
pub enum EvalMode {
    Unencrypted,
    Enclave,
    Homomorphic,
}

pub fn configure_client_application() -> ClientConfiguration {
    let arguments = CLIArguments::parse();
    log::debug!("{:?}", arguments);

    ClientConfiguration::load(arguments)
}

impl ClientConfiguration {
    /// For in-program start
    pub(crate) fn new(
        server_host: String,
        server_http: u16,
        server_https: u16,
        paths: HashMap<String, PathBuf>,
        eval_mode: (bool, EvalMode),
        offload: Option<Vec<OperationType>>,
        is_company: Option<u128>,
    ) -> Self {
        // Depending on whether a uuid is given, create a company or an analyst
        let role = match (is_company, eval_mode.clone()) {
            (Some(uuid), (false, _)) => ClientType::Company(uuid),
            (None, (false, _)) => ClientType::Analyst,
            (_, (true, _)) => ClientType::Spectator,
        };

        // Return client configuration
        ClientConfiguration {
            server_host,
            server_http,
            server_https,
            paths,
            offload,
            role,
            eval_mode: eval_mode.1,
        }
    }

    /// For CLI start
    fn load(arguments: CLIArguments) -> Self {
        // Verify the files to exist
        if !arguments.client_pkcs12_path.exists() {
            panic!("PKCS12 (.PFX) path is invalid!");
        }
        if !arguments.server_ca_certificate_path.exists() {
            panic!("Server CA Certificate path is invalid!");
        }

        // Add paths for fileupload
        let mut paths = HashMap::new();
        paths.insert(
            CC_CLIENT_PKCS12_KEY.to_string(),
            arguments.client_pkcs12_path.clone(),
        );
        paths.insert(
            CC_CLIENT_SERVER_CA_CERTIFICATE.to_string(),
            arguments.server_ca_certificate_path,
        );

        // Check which mode is used and check hat all files exist
        let role = match arguments.command {
            TeeBenchCLISubcommands::Company {
                input_data_path,
                uuid,
            } => {
                if !input_data_path.exists() {
                    panic!("Input data path is invalid!");
                }

                paths.insert(
                    CC_COMPANY_INPUT_DATA_PATH_KEY.to_string(),
                    input_data_path.clone(),
                );

                ClientType::Company(uuid)
            }
            TeeBenchCLISubcommands::Analyst {
                analyst_ca_cert_path,
                analyst_certificate_path,
                algorithm_path,
                benchmarking_config_path,
            } => {
                if !analyst_ca_cert_path.exists() {
                    panic!("Analyst CA Certificate path is invalid!");
                }
                if !analyst_certificate_path.exists() {
                    panic!("Analyst Certificate path is invalid!");
                }
                if !algorithm_path.exists() {
                    panic!("Algorithm path is invalid!");
                }
                if !benchmarking_config_path.exists() {
                    panic!("Benchmarking Config path is invalid!");
                }

                paths.insert(
                    CC_ANALYST_CA_CERTIFICATE_KEY.to_string(),
                    analyst_ca_cert_path.clone(),
                );
                paths.insert(
                    CC_ANALYST_CERTIFICATE_KEY.to_string(),
                    analyst_certificate_path.clone(),
                );
                paths.insert(
                    CC_ANALYST_ALGORITHMS_KEY.to_string(),
                    algorithm_path.clone(),
                );
                paths.insert(
                    CC_ANALYST_BENCHMARK_CONFIG_KEY.to_string(),
                    benchmarking_config_path.clone(),
                );

                ClientType::Analyst
            }
            TeeBenchCLISubcommands::Other {} => {
                log::warn!("Starting client with debug option!");
                ClientType::Spectator
            }
        };

        //paths.insert(k, v)

        let cconfig = ClientConfiguration {
            server_host: arguments.server_host,
            server_http: arguments.server_http,
            server_https: arguments.server_https,
            eval_mode: arguments.eval_mode,
            offload: arguments.offload,
            role,
            paths,
        };

        cconfig
    }

    pub fn server_host(&self) -> &String {
        &self.server_host
    }

    pub fn server_http_port(&self) -> u16 {
        self.server_http
    }

    pub fn server_https_port(&self) -> u16 {
        self.server_https
    }

    pub fn paths(&self) -> &HashMap<String, PathBuf> {
        &self.paths
    }

    pub fn eval_mode(&self) -> EvalMode {
        self.eval_mode.clone()
    }

    pub fn offload(&self) -> &Option<Vec<OperationType>> {
        &self.offload
    }

    pub(crate) fn role(&self) -> &ClientType {
        &self.role
    }
}

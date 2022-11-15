//! Constants to configure the server and clients
//! 
//! The server and clients (i.e. the analyst and the companies)
//! use this crate for locating data while executing.

use const_format::concatcp;


///
/// SERVER CONSTANTS
/// 

/// Data path for uploaded data
pub const SERVER_DATA_PATH: &str = "../data/server_data";

/// File for CA certificate by analyst
pub const SERVER_CA_CERTIFICATE: &str = "rootCA";


/// YAML Configuration Path
pub const SERVER_YAML_PATH: &str = "../templates/yaml";
/// File for server config
pub const SERVER_CONFIG_YAML: &str = "server_config";


/// Crypto: Certificates and Keys Path
pub const SERVER_CRYPTO_PATH: &str = "../templates/crypto";

/// File for server certificate (pem) AND private key (key)
pub const SERVER_SERVER_CRYPTO: &str = "server/server";
/// File for enclave certificate (pem)
pub const SERVER_ENCLAVE_CRYPTO: &str = "enclave/enclave";


/// Static client files for uploads (i.e. scripts and sites)
pub const SERVER_STATIC_PATH: &str = "../templates/static";

/// Favicon path
pub const SERVER_FAVICON: &str = "favicon";
/// Favicon file extension
pub const SERVER_EXT_FAVICON: &str = "ico";

/// Shutdown period to wait at most for all threads to finish
pub const SERVER_SHUTDOWN_TIMEOUT: u64 = 10;

///
/// JOINT CONTSTANTS
/// 

///
/// HEADER FIELDS
/// 
pub const X_APPLICATION_FIELD: &str = "X-Application-Teebench";


///
/// FORM-DATA-FIELDS
/// 

/// 01. HTTP-SETUP
pub const FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_NAME: &str = "root-ca-certificate";
pub const FORM_DATA_FIELD_01_ROOT_CA_CERTIFICATE_MIME: &str = "application/x-x509-ca-cert";
pub const FORM_DATA_FIELD_01_CONFIGURATION_NAME: &str = "configuration";
pub const FORM_DATA_FIELD_01_CONFIGURATION_MIME: &str = "text/yaml";
pub const FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_NAME: &str = "analyst-certificate";
pub const FORM_DATA_FIELD_01_ANALYST_CERTIFICATE_MIME: &str = "application/x-x509-ca-cert";

/// 02. ANALYST-ALGORITHM-UPLOAD
pub const FORM_DATA_FIELD_02_ALGORIHTMS_NAME: &str = "algorithms";
pub const FORM_DATA_FIELD_02_ALGORITHMS_MIME: &str = "text/yaml";

/// 03. MODIFY-BENCHMARK-CONFIG
pub const FORM_DATA_FIELD_03_CONFIGURATION_NAME: &str = "configuration";
pub const FORM_DATA_FIELD_03_CONFIGURATION_MIME: &str = "text/yaml";

/// 04. UPLOAD-INPUT-DATA
pub const FORM_DATA_FIELD_04_COMPANY_INPUT_NAME: &str = "input_data";
pub const FORM_DATA_FIELD_04_COMPANY_INPUT_MIME: &str = "text/yaml";

///
/// API ROUTES GENERAL
/// 

/// MISC AND DEBUG
pub const ROUTE_FAVICON: &str = "/favicon.ico";
pub const ROUTE_INDEX: &str = "/";
pub const ROUTE_WHOAMI: &str = "/whoami";

/// API NAME
pub const ROUTE_API: &str = "/api";

/// SETUP SPECIFIC
pub const ROUTE_SETUP: &str = "setup";
pub const ROUTE_ATTEST: &str = "attest";
pub const ROUTE_SHUTDOWN: &str = "shutdown";

/// COMPANY SPECIFIC
pub const ROUTE_COMPANY: &str = "company";
pub const ROUTE_COMPANY_EXT_REGISTER: &str = "register";
pub const ROUTE_COMPANY_EXT_INPUT_DATA: &str = "input_data";
pub const ROUTE_COMPANY_EXT_RESULTS: &str = "results";

/// EVENT SPECIFIC
pub const ROUTE_ENROLL_EVENTS: &str = "events";

/// ANALYST SPECIFIC
pub const ROUTE_ANALYST: &str = "analyst";
pub const ROUTE_ANALYST_EXT_BENCHMARK_CONFIG: &str = "benchmark_config";
pub const ROUTE_ANALYST_EXT_COMPANY_STATUS: &str = "company";
pub const ROUTE_ANALYST_EXT_ENROLL_COMPANY: &str = "enroll_company";
pub const ROUTE_ANALYST_EXT_ALGORITHMS: &str = "algorithms";
pub const ROUTE_ANALYST_EXT_BENCHMARK: &str = "benchmark";
pub const ROUTE_ANALYST_EXT_EVENT: &str = "event";

///
/// MAKE ROUTES EASIER FOR CLIENTS TO USE
///
 
/// SETUP SPECIFIC
pub const C_ROUTE_SETUP: &str = concatcp!(ROUTE_API, "/", ROUTE_SETUP);
pub const C_ROUTE_ATTEST: &str = concatcp!(ROUTE_API, "/", ROUTE_ATTEST);
pub const C_ROUTE_SHUTDOWN: &str = concatcp!(ROUTE_API, "/", ROUTE_SHUTDOWN);

/// COMPANY SPECIFIC [HAS TRAILING "/" for appending UUID]
pub const C_ROUTE_COMPANY_EXT_REGISTER_ID: &str = concatcp!(ROUTE_API, "/", ROUTE_COMPANY, "/", ROUTE_COMPANY_EXT_REGISTER, "/");
pub const C_ROUTE_COMPANY_EXT_INPUT_DATA_ID: &str = concatcp!(ROUTE_API, "/",  ROUTE_COMPANY, "/",ROUTE_COMPANY_EXT_INPUT_DATA, "/");
pub const C_ROUTE_COMPANY_EXT_RESULTS_ID: &str = concatcp!(ROUTE_API, "/",  ROUTE_COMPANY, "/",ROUTE_COMPANY_EXT_RESULTS, "/");

/// EVENT SPECIFIC
pub const C_ROUTE_ENROLL_EVENTS: &str = concatcp!(ROUTE_API, "/", ROUTE_ENROLL_EVENTS);

/// ANALYST SPECIFIC
pub const C_ROUTE_ANALYST_EXT_BENCHMARK_CONFIG: &str = concatcp!(ROUTE_API, "/", ROUTE_ANALYST, "/", ROUTE_ANALYST_EXT_BENCHMARK_CONFIG);
pub const C_ROUTE_ANALYST_EXT_COMPANY_STATUS_ID: &str = concatcp!(ROUTE_API, "/", ROUTE_ANALYST, "/", ROUTE_ANALYST_EXT_COMPANY_STATUS, "/");
pub const C_ROUTE_ANALYST_EXT_ENROLL_COMPANY: &str = concatcp!(ROUTE_API, "/", ROUTE_ANALYST, "/", ROUTE_ANALYST_EXT_ENROLL_COMPANY);
pub const C_ROUTE_ANALYST_EXT_ALGORITHMS: &str = concatcp!(ROUTE_API, "/", ROUTE_ANALYST, "/", ROUTE_ANALYST_EXT_ALGORITHMS);
pub const C_ROUTE_ANALYST_EXT_BENCHMARK: &str = concatcp!(ROUTE_API, "/", ROUTE_ANALYST, "/", ROUTE_ANALYST_EXT_BENCHMARK);
pub const C_ROUTE_ANALYST_EXT_EVENT: &str = concatcp!(ROUTE_API, "/", ROUTE_ANALYST, "/", ROUTE_ANALYST_EXT_EVENT);


///
/// MAKE ROUTES FOR SERVER EASIER TO USE
/// 

pub const S_ROUTE_COMPANY_EXT_REGISTER_ID: &str = concatcp!(ROUTE_COMPANY_EXT_REGISTER, "/{id}");
pub const S_ROUTE_COMPANY_EXT_INPUT_DATA_ID: &str = concatcp!(ROUTE_COMPANY_EXT_INPUT_DATA, "/{id}");
pub const S_ROUTE_COMPANY_EXT_RESULTS_ID: &str = concatcp!(ROUTE_COMPANY_EXT_RESULTS, "/{id}");

pub const S_ROUTE_ANALYST_EXT_COMPANY_STATUS: &str = concatcp!(ROUTE_ANALYST_EXT_COMPANY_STATUS, "/{id}");

/// File extension for certificate
pub const EXT_CERTIFICATE: &str = "pem";
/// File extension for certificate
pub const EXT_PRIVATE_KEY: &str = "key";
/// File extension for YAML
pub const EXT_YAML: &str = "yaml";


///
/// CLIENT CONSTANTS
/// 

/// Each client has to have a PFX key and the certificate
pub const CC_CLIENT_PKCS12_KEY: &str = "pkcs12_path";
pub const CC_CLIENT_SERVER_CA_CERTIFICATE: &str = "server_certificate_path";

/// The companies provide their input data
pub const CC_COMPANY_INPUT_DATA_PATH_KEY: &str = "input_data_path";

/// The analyst provides the ca certificate, his own certificate, the config and his algorithms 
pub const CC_ANALYST_CA_CERTIFICATE_KEY: &str = "analyst_ca_cert_path";
pub const CC_ANALYST_ALGORITHMS_KEY: &str = "algorithm_path";
pub const CC_ANALYST_BENCHMARK_CONFIG_KEY: &str = "benchmarking_config_path";
pub const CC_ANALYST_CERTIFICATE_KEY: &str = "analyst_cert_path";

/// The spectator writes to a specific file which holds the configuration
pub const CC_SPECTATOR_EVAL_OUTPUT_KEY: &str = "spectator_eval_output_path";
//! Response message formats
//! 
//! The message formats are used for the server who responds 
//! to clients. Clients (similar to requests) parse the 
//! same message types.


use serde::{Deserialize, Serialize};
use crate::entity::BenchmarkingConfig;

///
/// RESPONSE MESSAGES
///

#[derive(Serialize, Deserialize, Default, Debug)]
/// General response message format for communication.
pub struct RspMsg<T> { 
    pub success: bool,
    pub message: String,
    pub content: T
}

impl<T> RspMsg<T> {
    pub fn new(success: bool, message: String, content: T) -> Self {
        RspMsg {success, message, content}
    }
}

#[derive(Debug, Serialize)]
/// Server status message
/// This is receivable by any (non) participant
/// and shows that the enclave is trusted!
pub struct ServerStatus {
    is_setup: bool,
    enclave_certificate: String,
    server_info: BenchmarkingConfig,
}

impl ServerStatus {
    pub fn new(is_setup: bool, enclave_certificate: String, server_info: BenchmarkingConfig) -> Self {
        ServerStatus {is_setup,enclave_certificate, server_info}
    }
}

/// Company IDs are UUID
pub type CompanyID = u128;

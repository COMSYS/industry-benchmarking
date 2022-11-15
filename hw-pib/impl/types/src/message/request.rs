//! Request message formats
//! 
//! The message formats are used for clients who communicate 
//! with the server. The server parses the messages with the
//! same format.

use serde::{Deserialize, Serialize};

///
/// On Instance Verification
/// 

/// In order to be assured that the server is running in a trusted
/// environment, all clients can issue a get request to get information
/// on the enclave that hosts the server. This is done using a GET,
/// therefore it is not shown here.


///
/// Analyst Messages
///

/// The message flow is assumed to be as follows:
/// 
/// 1. The analyst sets up initial communication to the
///    server and uploads the following **files** and one text:
///     - The CA root certificate for accepting incomming
///       communication from all clients that have their
///       certificate signed by this root certificate.
///     - The symmetric HMAC key that is encrypted with the
///       servers public key.
///     - The configuratin yaml file that sets up basic config
///       for the server (i.e. the k_anonymity, name,...)
/// 
/// - The analyst has now successfully configured the server.
///   He might still be able to change the configuration 
///   afterwards (i.e. name, k_anonymity, ...)
///     - Especially do the companies now have access to the 
///       server when the analyst signs the companies' public
///       keys with the private key of the root CA.
///     - Keep in mind that all registered companies are now
///       allowed to communicate (one company could be malicious
///       and use the service to register multiple times.
///     - Thus only the analyst performs registration of clients.
/// 
/// 2. The analyst registers companies. He then gets server 
///    generated Company-IDs that get passed along to the 
///    individual companies (by using their public key to encrypt
///    the received ID). Using this scheme each company may only
///    register once. (This is a GET-type request, i.e. empty!) 
/// 
/// 2. The analyst uploads the Algorithms for KPI computation
///     - This includes one yaml file containing all computing
///       schemes used for computation.
/// 
/// 3. The analyst finally starts the benchmarking process by
///    sending a message to the server in which he specifies 
///    which of the provided KPIs should be used for computat 
/// 
/// - The analyst can dispatch messages to all clients by issuing
///   event messages. These messages are broadcasted to all clients
///   that listen on the event channel.

#[derive(Deserialize)]
/// First message from analyst which holds information
/// on the ca_certificate
/// It is accompanied by a yaml and a certificate file 
/// in a form-data request
pub struct AnalystSetupMsg {
    pub encrypted_analyst_hmac: String,
}

/// The second request is a GET-Request.

/// The third message is only a form-data request and thus not
/// seen here. The format of the config is in file is in 
/// `templates/config/benchmark_config.yaml`.

#[derive(Deserialize, Debug)]
/// The third message only holds the information on the selected
/// algorithms. This field is allowed to be null, meaning that all
/// algorithms will be executed.
pub struct AnalystBenchmarkingMsg {
    pub selected_kpis: Option<Vec<String>>
}

#[derive(Deserialize, Serialize)]
/// The message of the analyst that he can dispatch is in string
/// format.
pub struct AnalystEventMsg {
    pub event: String,
}


///
/// Company Messages
///

/// Their message flow is assumed to be as follows:
/// 
/// Before they can start the communication, they have to receive the
/// signed public key from the analysts root CA. Before companies 
/// upload their data anywhere to the server, they verify the instance.
/// Since TLS is used data confidentiality is given, but in order to
/// have integrity the companies need to setup the HMAC key for 
/// communication with the server. This key is for integrity checking.
/// 
/// Since every company has its own ID, the HMAC key (that is again 
/// encrypted with the servers public key) is now bound to its ID. The
/// server allows setting the key only once, thus forging messages is
/// not possible, even if the ID of one company gets public afterwards.
/// If it gets public before, the company cannot register and thus no 
/// data is further shared.
/// 
/// 1. The company sends the encrypted HMAC key.
/// 
/// 2. The company uploads the data to the server (multipated) 
/// 
/// - The company may also register for publicly (meaning only to 
///   all participants) dispatched messages from the analyst (GET).
/// 
/// 3. The company can request the results (again GET).
/// 
/// 
/// TODO: EVAL MESSAGES FOR DIRECT COMPARISON TO OFFLOADING APPROACH.

#[derive(Deserialize)]
pub struct CompanySetupMsg {
    pub encrypted_company_hmac: String,
}

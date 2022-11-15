use std::{fmt, error::Error};
use serde::Serialize;
use serde_json::{Value as JsonValue};

#[derive(Debug)]
pub struct ClientError {
    cause: ClientErrorCause,
}

///
/// Error types the on user
///

#[allow(unused)]
pub enum AbstractClientErrorType {
    NoConnection,
    Unauthorized,
    BadRequest,
    NotFound,
}

#[derive(Debug, Serialize)]
enum ClientErrorCause {
    NoConnection(JsonValue),
    Unauthorized(JsonValue),
    BadRequest(JsonValue),
    NotFound(JsonValue),
}


impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.cause {
            ClientErrorCause::NoConnection(error) => {write!(f, "No connection: {}", error)},
            ClientErrorCause::Unauthorized(error) => {write!(f, "Unauthorized: {}", error)},
            ClientErrorCause::BadRequest(error) => {write!(f, "BadRequest: {}", error)},
            ClientErrorCause::NotFound(error) => {write!(f, "NotFound: {}", error)},
        } 
    }
}

impl Error for ClientError {
    fn description(&self) -> &str {
        "Benchmarking failed: "
    }

    fn cause(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

///
/// ERROR CONVERSION FOR ERROR TYPES
///

impl From<(AbstractClientErrorType, String)> for ClientError {
    fn from(error: (AbstractClientErrorType, String)) -> Self {
        match error.0 {
            AbstractClientErrorType::BadRequest     => { ClientError { cause: ClientErrorCause::BadRequest(serde_json::Value::String(error.1)) }},
            AbstractClientErrorType::NoConnection   => { ClientError { cause: ClientErrorCause::NoConnection(serde_json::Value::String(error.1)) }},
            AbstractClientErrorType::Unauthorized   => { ClientError { cause: ClientErrorCause::Unauthorized(serde_json::Value::String(error.1)) }},
            AbstractClientErrorType::NotFound       => { ClientError { cause: ClientErrorCause::NotFound(serde_json::Value::String(error.1)) }},
        }
    }
}

impl From<ClientErrorCause> for ClientError {
    fn from(error: ClientErrorCause) -> Self {
        ClientError { cause: error }
    }
}
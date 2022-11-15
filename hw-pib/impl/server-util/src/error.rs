use actix_web::{
    error::ResponseError,
    http::{StatusCode},
    HttpResponse,
};
use validator::ValidationErrors;


use std::fmt;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use benchmark::error::BenchmarkingError;

///
/// Error types the API can return on user connection
///

#[derive(Debug, Serialize)]
pub enum ApiError {
    ParseError(JsonValue),
    NotImplemented(JsonValue),
    Unauthorized(JsonValue),
    BadRequest(JsonValue),
    InternalServerError(JsonValue),
    NotFound(JsonValue),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Unauthorized(error) => {write!(f, "Unauthorized: {}", error)},
            ApiError::BadRequest(error) => {write!(f, "BadRequest: {}", error)},
            ApiError::NotImplemented(error) => {write!(f, "(⌒_⌒;) NotImplemented: {}", error)},
            ApiError::InternalServerError(error) => {write!(f, "InternalServerError: {}", error)},
            ApiError::NotFound(error) => {write!(f, "NotFound: {}", error)},
            _ => {write!(f, "Unknown Error!")}
        } 
    }
}

///
/// Return a list of error in case multiple errors occur
///

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiErrorResponse {
    errors: Vec<String>,
}

///
/// ERROR CONVERSION FOR ERROR TYPES
///

/// Utility to make transforming a string reference into an ErrorResponse
impl From<&String> for ApiError {
    fn from(error: &String) -> Self {
        ApiError::BadRequest(
            json!(ApiErrorResponse {errors: vec![format!("{}", error)]})
        )
    }
}

impl From<BenchmarkingError> for ApiError {
    fn from(error: BenchmarkingError) -> Self {
        ApiError::BadRequest(
            json!(ApiErrorResponse {errors: vec![format!("Benchmarking Error: {}", error.to_string())]})
        )
    }
}

/// Utility to make transforming a string reference into an ErrorResponse
impl From<&str> for ApiError {
    fn from(error: &str) -> Self {
        ApiError::BadRequest(
            json!(ApiErrorResponse {errors: vec![error.to_string()]})
        )
    }
}

/// Utility to make transforming a vector of strings into an ErrorResponse
impl From<Vec<String>> for ApiError {
    fn from(errors: Vec<String>) -> Self {
        ApiError::BadRequest(json!(ApiErrorResponse { errors }))
    }
}

/// Allow error Converison from complex error objects to api responses
/// Value is a JsonValue
impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        let mut err_map = JsonMap::new();

        // transforms errors into objects that err_map can take
        for (field, errors) in errors.field_errors().iter() {
            let errors: Vec<JsonValue> = errors
                .iter()
                .map(|error| {
                    //dbg!(error); // Debug information
                    json!(error)
                })
                .collect();
            err_map.insert(field.to_string(), json!(errors));
        }

        ApiError::BadRequest(json!({
            "errors": err_map,
        }))
    }
}

/// Implement Error Handling for Standard Error
impl From<std::fmt::Error> for ApiError {
    fn from(error: std::fmt::Error) -> Self {
        ApiError::from(&error.to_string())
    }
}

/// Implement the `ResponseError`-trait for our error types
/// - Basically a wrapper for converting own errors into actix' errors
impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        // Log error to console with custom formatter
        log::error!("{}", self);

        match self {
            ApiError::BadRequest(error) => {
                HttpResponse::BadRequest().json(error)
            }
            ApiError::NotFound(error) => {
                HttpResponse::NotFound().json(error)
            }
            ApiError::NotImplemented(error) => {
                HttpResponse::NotFound().json(error)
            }
            ApiError::Unauthorized(error) => {
                HttpResponse::Unauthorized().json(error)
            }
            _ => { 
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR) 
            },
        }
    }
}
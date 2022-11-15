//! Benchmarking Error Type
//! 
//! Document all benchmarking specific errors:
//!     - Atomic errors: in case computation is incorrect (dimensionality or similar)
//!     - Operation errors: In case computatoin fails on a specific operation type
//!     - Algorithm: General error: In case Input parsing, algorithm parsing,.. fail

use std::error::Error;
use std::fmt;

use super::atomic::Atomic;
use super::operation::OperationInput;

#[derive(Debug)]
pub struct BenchmarkingError {
    cause: BenchmarkingErrorCause,
}


#[derive(Debug)]
enum BenchmarkingErrorCause {
    Atomic(Atomic, String),
    Operation(OperationInput, String, OperationInput),
    Algorithm(String),
}

impl fmt::Display for BenchmarkingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.cause {
            BenchmarkingErrorCause::Atomic(var, reason) => {
                write!(f, "Atomic var {} with op {} violates {}", var.name(), var.op(), reason)
            }
            BenchmarkingErrorCause::Operation(op_in, op_type,op_out) => {
                write!(f, "Operation {} failed on {} with intermediary result {:?}", op_type.to_string(), op_in, op_out)
            }
            BenchmarkingErrorCause::Algorithm(reason) => {
                write!(f, "Algorithm computation failed with reason: {}", reason)
            }
        }
        
    }
}

impl Error for BenchmarkingError {
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

impl From<(Atomic, String)> for BenchmarkingError {
    fn from(err: (Atomic, String)) -> Self {
        BenchmarkingError {
            cause: BenchmarkingErrorCause::Atomic(err.0, err.1)
        }
    }
}

impl From<(Atomic, &str)> for BenchmarkingError {
    fn from(err: (Atomic, &str)) -> Self {
        BenchmarkingError {
            cause: BenchmarkingErrorCause::Atomic(err.0, err.1.to_string())
        }
    }
}

impl From<(OperationInput, String, OperationInput)> for BenchmarkingError {
    fn from(err: (OperationInput, String, OperationInput)) -> Self {
        BenchmarkingError {
            cause: BenchmarkingErrorCause::Operation(err.0, err.1, err.2)
        }
    }
}

impl From<String> for BenchmarkingError {
    fn from(err: String) -> Self {
        BenchmarkingError {
            cause: BenchmarkingErrorCause::Algorithm(err)
        }
    }
}
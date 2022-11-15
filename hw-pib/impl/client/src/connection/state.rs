use async_trait::async_trait;
use std::{collections::HashMap, path::PathBuf};

use reqwest::Client;

use crate::{config::EvalMode, error::ClientError};

#[derive(Debug, Clone)]
/// Each state transisition can either be successful
/// or eroneous. In case of error go to the FailureState.
pub enum Event {
    SuccessfulResponse,
    ErrorResponse(String),
}

#[async_trait]
pub trait StateMachine<T> {
    /// Initialize state machine
    fn init() -> Self;
    /// Take next state transition with event (alphabet literal)
    fn next(&self, event: Event) -> Self;
    /// Run operation for one specific state
    async fn run(&self, conn_info: &T) -> Result<(), ClientError>;
}

#[async_trait]
pub trait ClientConnection {
    /// Create a new client connection
    fn new(
        client: Client,
        host: String,
        http_port: String,
        https_port: String,
        paths: &HashMap<String, PathBuf>,
        uuid: Option<u128>,
        eval_mode: EvalMode,
    ) -> Self;
    /// Run the client connection which hols a state machine in it
    async fn run(&mut self);
}

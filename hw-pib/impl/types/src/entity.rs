use serde::{Serialize, Deserialize};
use rustls::Certificate;

use crate::output::Output;

use super::input::Input;

#[derive(Debug, Clone, Default)]
pub struct Company {
    certificate: Option<Certificate>,
    input_data: Input,
    results_data: Output,
}

impl Company {
    pub fn new() -> Self {
        Company {..Default::default()}
    }

    pub fn input_data(&self) -> &Input {
        &self.input_data
    }

    pub fn results_data(&self) -> &Output {
        &self.results_data
    }

    pub fn certificate(&self) -> &Option<Certificate> {
        &self.certificate
    }

    pub fn does_participate(&self) -> bool {
        self.input_data.size() != 0 as usize
    }

    pub fn set_input_data(&mut self, input_data: Input){
        self.input_data = input_data;
    }

    pub fn set_results_data(&mut self, results_data: Output){
        self.results_data = results_data;
    }

    pub fn set_certificate(&mut self, cert: Certificate) {
        self.certificate = Some(cert);
    }
}

#[derive(Debug, Clone)]
pub struct Analyst {
    certificate: Certificate,
}

impl Analyst {
    pub fn new(certificate: Certificate) -> Self {
        Analyst { certificate }
    }

    pub fn certificate(&self) -> &Certificate {
        &self.certificate
    }

    pub fn set_certificate(&mut self, certificate: Certificate) {
        self.certificate = certificate;
    }
}

#[derive(Debug, Deserialize, Clone, Serialize, Default)]
pub struct BenchmarkingConfig {
    /// The server name
    name: String,    
    /// The server description
    description: String,
    /// K-anonymity value
    k_anonymity: u64,
    /// Eval mode (only relevant for eval) (true → offloading used)
    eval_mode: bool,
    /// Offloaded Operations (only relevant for eval)
    offload: Vec<String>,
}

impl BenchmarkingConfig {
    pub fn new(name: String, description: String, k_anonymity: u64, eval_mode: bool, offload: Vec<String>) -> Self {
        BenchmarkingConfig { name, description, k_anonymity, eval_mode, offload }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn k_anonymity(&self) -> u64 {
        self.k_anonymity
    }

    pub fn eval_mode(&self) -> bool {
        self.eval_mode
    }

    pub fn offload(&self) -> &Vec<String> {
        &self.offload
    }

    /// Modification

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_k_anonymity(&mut self, k_anonymity: u64) {
        self.k_anonymity = k_anonymity;
    }
    
    pub fn set_eval_mode(&mut self, eval_mode: bool) {
        self.eval_mode = eval_mode;
    }
    
    pub fn set_offload(&mut self, offload: Vec<String>) {
        self.offload = offload;
    }
}

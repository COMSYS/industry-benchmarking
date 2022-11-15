use std::collections::HashMap;

use crate::{
    config::Config,
    crypto::Crypto,
};
use benchmark::Algorithm;
use types::entity::{Analyst, Company, BenchmarkingConfig};



#[derive(Debug, Clone)]
pub struct BenchmarkingServer {
    /// Static Server Configuration
    server_config: Config,
    /// Benchmarking Configuration
    benchmarking_config: BenchmarkingConfig,
    /// Crypto configuration 
    crypto_config: Crypto,
    /// Companies, where  the companyID Value is and the value is the DER key, the certificate and a monotonic counter
    companies: HashMap<u128, Company>,
    /// Analyst information
    analyst: Option<Analyst>, 
    /// Analyst Algorithms
    algorithms: Option<Algorithm>,
    // Number of uploaded files
    active_participants: u64,
}

impl BenchmarkingServer {
    pub fn load() -> Self {
        // Load configuration from standard path
        let server_config = Config::load(); 
        let benchmarking_config = BenchmarkingConfig::default();
        let crypto_config = Crypto::load();

        let companies: HashMap<u128, Company> = HashMap::new();
        let analyst: Option<Analyst> = None;
        let algorithms: Option<Algorithm> = None;

        let active_participants = 0_u64;

        let benchmarking_server = BenchmarkingServer {server_config, benchmarking_config, crypto_config, companies, analyst, algorithms, active_participants  };

        benchmarking_server
    }

    pub fn server_config(&self) -> &Config {
        &self.server_config
    }

    pub fn benchmarking_config(&self) -> &BenchmarkingConfig {
        &self.benchmarking_config
    }

    pub fn crypto_config(&self) -> &Crypto {
        &self.crypto_config
    }

    pub fn companies(&self) -> &HashMap<u128, Company> {
        &self.companies
    }

    pub fn analyst(&self) -> Option<&Analyst> {
        self.analyst.as_ref()
    }

    pub fn algorithms(&self) -> Option<&Algorithm> {
        self.algorithms.as_ref()
    }

    pub fn active_participants(&self) -> u64 {
        self.active_participants
    }

    /// Modification of benchmarking server 

    pub fn set_benchmarking_config_all(&mut self, cfg: BenchmarkingConfig) {
        self.benchmarking_config = cfg;
    }

    pub fn set_crypto_config(&mut self) -> &mut Crypto {
        &mut self.crypto_config
    }

    pub fn set_companies(&mut self) -> &mut HashMap<u128, Company> {
        &mut self.companies
    }

    pub fn set_analyst(&mut self, analyst: Analyst) {
        self.analyst = Some(analyst);
    }

    pub fn set_algorithms(&mut self, algorithms: Option<Algorithm>) {
        self.algorithms = algorithms;
    }

    pub fn increment_active_participants(&mut self) {
        self.active_participants += 1;
    }
}
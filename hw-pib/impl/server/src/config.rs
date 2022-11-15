use std::{path::Path, fs::OpenOptions, io::Read};
use serde::Deserialize;

use types::consts::*;


#[derive(Clone, Deserialize, Debug, Default)]
/// Server Configuration
/// 
/// This configuration is preprovided and can only be
/// changed by the application developer. Any participant
/// has the possibility to alter the settings.
pub struct Config {
    #[serde(default)]
    host: String,
    #[serde(default)]
    port: String,
    #[serde(default)] 
    tls_port: String,
    #[serde(default)] 
    http_workers: usize,
    #[serde(default)] 
    https_workers: usize,
    #[serde(default)]
    shutdown_timeout: u64,
}


impl Config {
    pub fn load() -> Self {
        let config_path = Path::new(SERVER_YAML_PATH).join(SERVER_CONFIG_YAML).with_extension(EXT_YAML);

        let mut file = OpenOptions::new().read(true).open(config_path).expect("[FATAL] Could not open configuration file!");

        let mut buffer = String::new();
        file.read_to_string(&mut buffer).expect("[FATAL] Could not extract configuration file!");

        let config: Self = serde_yaml::from_str(&buffer).expect("[FATAL] Could not parse configuration file!");

        config
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &str {
        &self.port
    }

    pub fn tls_port(&self) -> &str {
        &self.tls_port
    }

    pub fn http_workers(&self) -> usize {
        self.http_workers
    }

    pub fn https_workers(&self) -> usize {
        self.https_workers
    }

    pub fn shutdown_timeout(&self) -> u64 {
        self.shutdown_timeout
    }
}

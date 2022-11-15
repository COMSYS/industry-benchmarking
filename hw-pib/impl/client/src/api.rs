use types::consts::*;
#[derive(Debug, Clone)]
pub struct TeebenchHttpAPI {
    base: String,
}

impl TeebenchHttpAPI {
    /// Host example: `teebench.xyz`
    /// HTTP_port example: `8080`
    pub fn new(host: String, http_port: String) -> Self {
        let base = "http://".to_string() + &host + &":".to_string() + &http_port;
        TeebenchHttpAPI {
            base
        }
    }

    pub fn setup(&self) -> String {
        self.base.clone() + C_ROUTE_SETUP
    }

    #[allow(unused)]
    pub fn attest(&self) -> String {
        self.base.clone() + C_ROUTE_ATTEST
    }
}

#[derive(Debug, Clone)]
pub struct TeebenchHttpsAPI {
    base: String,
}

impl TeebenchHttpsAPI {
    /// Host example: `teebench.xyz`
    /// HTTP_port example: `8443`
    pub fn new(host: String, https_port: String) -> Self {
        let base = "https://".to_string() + &host + &":".to_string() + &https_port;
        TeebenchHttpsAPI {
            base,
        }
    }

    #[allow(unused)]
    pub fn setup(&self) -> String {
        self.base.clone() + C_ROUTE_SETUP
    }

    #[allow(unused)]
    pub fn attest(&self) -> String {
        self.base.clone() + C_ROUTE_ATTEST
    }

    pub fn shutdown(&self) -> String {
        self.base.clone() + C_ROUTE_SHUTDOWN
    }
    pub fn company_register(&self, uuid: u128) -> String {
        self.base.clone() + C_ROUTE_COMPANY_EXT_REGISTER_ID + &uuid.to_string()
    }
    pub fn company_input_data(&self, uuid: u128) -> String {
        self.base.clone() + C_ROUTE_COMPANY_EXT_INPUT_DATA_ID + &uuid.to_string()
    }
    #[allow(unused)]
    pub fn company_results(&self, uuid: u128) -> String {
        self.base.clone() + C_ROUTE_COMPANY_EXT_RESULTS_ID + &uuid.to_string()
    }
    pub fn get_events(&self) -> String {
        self.base.clone() + C_ROUTE_ENROLL_EVENTS
    }

    #[allow(unused)]
    pub fn analyst_benchmark_config(&self) -> String {
        self.base.clone() + C_ROUTE_ANALYST_EXT_BENCHMARK_CONFIG
    }

    #[allow(unused)]
    pub fn analyst_company_status(&self, uuid: u128) -> String {
        self.base.clone() + C_ROUTE_ANALYST_EXT_COMPANY_STATUS_ID + &uuid.to_string()
    }

    pub fn analyst_enroll_company(&self) -> String {
        self.base.clone() + C_ROUTE_ANALYST_EXT_ENROLL_COMPANY
    }
    
    pub fn analyst_algorithms(&self) -> String {
        self.base.clone() + C_ROUTE_ANALYST_EXT_ALGORITHMS
    }
    pub fn analyst_benchmark(&self) -> String {
        self.base.clone() + C_ROUTE_ANALYST_EXT_BENCHMARK
    }
    
    pub fn analyst_send_event(&self) -> String {
        self.base.clone() + C_ROUTE_ANALYST_EXT_EVENT
    }
}
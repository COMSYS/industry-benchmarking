//! Structs for eval messages
//! and analyst to share
use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CompanyUUIDs {
    pub comps: Vec<u128>,
}
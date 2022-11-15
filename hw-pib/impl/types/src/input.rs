//! Load and organize the inputs of `Algorithms`

use serde::{Deserialize, Serialize};
use serde_yaml::Error;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::PathBuf;

/// Defines the field in the Input file
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize, Default)]
pub struct InputVariable {
    /// Name of variable
    name: String,
    /// Min value: Not necessary but for compliance to existing solution
    max_val: f64,
    /// Max value: Not necessary but for compliance to existing solution
    min_val: f64,
    /// Values of this variable
    values: Vec<f64>,
}

impl InputVariable {
    pub fn new(name: String, values: Vec<f64>) -> Self {
        InputVariable {
            name,
            values,
            max_val: 0_f64,
            min_val: 0_f64,
        }
    }

    pub fn new_with_bounds(name: String, values: Vec<f64>, min_val: f64, max_val: f64) -> Self {
        InputVariable {
            name,
            values,
            max_val,
            min_val,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn values(&self) -> &Vec<f64> {
        &self.values
    }
}

/// Struct to hold the input config file
#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct Input {
    vars: HashMap<String, InputVariable>,
}

#[derive(Deserialize, Serialize)]
pub struct InputFmt {
    vars: Vec<InputVariable>,
}

impl Input {
    /// Load the Config file and parse its contents
    pub fn load(path: &PathBuf) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .expect("Could not open INPUTs_FILE");

        let buf = BufReader::new(file);
        let input_vec: InputFmt = serde_yaml::from_reader(buf)?;

        // Insert in map
        let mut var_map: HashMap<String, InputVariable> =
            HashMap::with_capacity(input_vec.vars.len());
        for var in input_vec.vars {
            var_map.insert(var.name().to_string(), var);
        }

        Ok(Self { vars: var_map })
    }

    /// Number of variables
    pub fn size(&self) -> usize {
        self.vars.len()
    }

    pub fn has_input_var(&self, id: &str) -> bool {
        self.vars.contains_key(id)
    }

    pub fn get_input_var(&self, id: &str) -> Option<&InputVariable> {
        self.vars.get(id)
    }
}

impl From<Vec<InputVariable>> for InputFmt {
    fn from(vars: Vec<InputVariable>) -> Self {
        InputFmt { vars }
    }
}

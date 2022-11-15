//! Results for companies to do comparisons on

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Defines the field in the Input file
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize, Default)]
pub struct OutputVariable {
    /// Name of variable
    name: String,
    /// Company result for variable 
    result: Vec<f64>,
    /// Best in class
    best_in_class: Vec<f64>,
    /// Worst in class
    worst_in_class: Vec<f64>,
    /// Averag among all
    average: Vec<f64>,
    /// Median of all
    median: Vec<f64>,
    /// 25% lower quantile
    lower_quantile: Vec<f64>,
    /// 75% upper quantile
    upper_quantile: Vec<f64>,
}

impl OutputVariable {
    /// Create a full variable with all statistical information
    pub fn new(name: String, result: Vec<f64>, best_in_class: Vec<f64>, worst_in_class: Vec<f64>, average: Vec<f64>, median: Vec<f64>, lower_quantile: Vec<f64>, upper_quantile: Vec<f64>) -> Self {
        OutputVariable {name, result, average, best_in_class, lower_quantile, median, upper_quantile, worst_in_class}
    }
    
    /// Create only the name and the specific value without any statistical data
    pub fn new_result_only(name: String, result: Vec<f64>) -> Self {
        OutputVariable {name, result, ..Self::default()}
    }

    /// Append the results of statistical evaluations on the end
    pub fn new_result_with_statistics(result: &Self, statistics: &Self) -> Self {
        
        Self {
            best_in_class: statistics.best_in_class.clone(),
            worst_in_class: statistics.worst_in_class.clone(),
            average: statistics.average.clone(),
            median: statistics.median.clone(),
            lower_quantile: statistics.lower_quantile.clone(),
            upper_quantile: statistics.upper_quantile.clone(),
            name: result.name.clone(),
            result: result.result.clone()
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn result(&self) -> &Vec<f64> {
        &self.result
    }

    pub fn best_in_class(&self) -> &Vec<f64> {
        &self.best_in_class
    }

    pub fn worst_in_class(&self) -> &Vec<f64> {
        &self.worst_in_class
    }

    pub fn average(&self) -> &Vec<f64> {
        &self.average
    }

    pub fn median(&self) -> &Vec<f64> {
        &self.median
    }

    pub fn lower_quantile(&self) -> &Vec<f64> {
        &self.lower_quantile
    }

    pub fn upper_quantile(&self) -> &Vec<f64> {
        &self.upper_quantile
    }
}


/// Struct to hold the input config file
#[derive(Debug, Deserialize, Clone, Default, Serialize)]
pub struct Output {
    vars: HashMap<String, OutputVariable>,
}

impl Output {
    /// Load the Config file and parse its contents
    pub fn from_output_vars(results: HashMap<String, OutputVariable>) -> Self {
        Output { vars: results }
    }

    pub fn new_empty() -> Self {
        Output { vars: HashMap::new() }
    }

    pub fn add_var(&mut self, outvar: OutputVariable) {
        self.vars.insert(outvar.name().to_string(), outvar);
    }

    pub fn get_result_from_var(&self, var_name: &str) -> Option<&Vec<f64>> {
        match self.vars.get(var_name) {
            Some(outvar) => { Some(outvar.result()) },
            None => None
        }
    }

    pub fn vars(&self) -> &HashMap<String, OutputVariable> {
        &self.vars
    }

    /// Number of variables
    pub fn size(&self) -> usize {
        self.vars.len()
    }

}

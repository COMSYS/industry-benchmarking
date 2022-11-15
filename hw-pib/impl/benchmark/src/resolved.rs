//! Resolved or "known" computation values
//! 
//! Resolved values are nothing more than a dynamic data structure that holds information on
//! the computation of all KPI values of one company. It serves as a lookuptable for existing
//! information in which intermediary results are inserted. In the end, the resolved values
//! hold all information on the KPIs of one specific company. 

use super::{variable::{VariableID, Variable}, error::BenchmarkingError};

use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct ResolvedValues {
    resolved: HashMap<VariableID, Variable>,
}

impl ResolvedValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has(&self, name: &String) -> bool {
        self.resolved.contains_key(name)
    }

    pub fn insert(&mut self, name: VariableID, var: Variable) -> Result<(), BenchmarkingError> {
        if self.resolved.contains_key(&name) {
            return Err(BenchmarkingError::from(name.clone()));
        }
        self.resolved.insert(name, var);
        Ok(())
    }

    pub fn get<'a>(&self, name: &'a VariableID) -> Result<&Variable, BenchmarkingError> {
        self.resolved.get(name).ok_or_else(|| BenchmarkingError::from(name.clone()))
    }

    pub fn resolved(&self) -> &HashMap<VariableID, Variable> {
        &self.resolved
    }

    /// Remove a liste of atomics by name 
    pub fn filter_atomics_by_name(&mut self, atomics: &std::collections::HashSet<String>) {
        for atomic in atomics.iter() {
            self.resolved.remove(atomic);
        }
    }

    /// Debug print: Show intermediary results
    pub fn print_plain(&self) {
        let mut values = Vec::new();
        for (name, val) in self.resolved.iter() {
            let s = format!("ID: {:^10}, Val: {:?}", name, val);
            values.push(s);
        }
        values.sort();
        log::debug!("-------------------- values ------------------");
        for s in values {
            log::debug!("{}", s);
        }
        log::debug!("----------------------------------------------");
    }
}

//! Variable Type
//! 
//! This type is used for computations of input data and 
//! holds n-dimensional f64 values for input. This allows
//! arbitrary complex operations. The comparison of the
//! varbiables (as seen in `operations.rs`) can panic when
//! the input is not one dimensional!
//! 
//! Otherwise this acts as a wrapper for f64 vectors. 

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
/// Variable type in general
pub struct Variable {
    vector: Vec<f64>,
}

impl Variable {
    pub fn new(vector: Vec<f64>) -> Self {
        Self { vector }
    }

    /// Return reference to `vector`
    pub fn vector(&self) -> &[f64] {
        &self.vector
    }

    /// Return mutable reference to `vector`
    pub fn vector_mut(&mut self) -> &mut [f64] {
        &mut self.vector
    }

    /// Consume `self` and return `vector`
    pub fn into_vector(self) -> Vec<f64> {
        self.vector
    }

    /// Return the dimension (number of entries) of of the `VectorLWE` in `vector`
    pub fn dim(&self) -> usize {
        self.vector.len()
    }
}

impl Default for Variable {
    fn default() -> Self {
        /* Float(0.0, 0.0..=0.0) */
        Variable::new(vec![0.0])
    }
}

/// Turn a Constant into a variable to be able to compute with other variables
impl From<f64> for Variable {
    fn from(input_val: f64) -> Self {
        Self::new(vec![input_val])
    }
}

impl From<Vec<f64>> for Variable {
    fn from(mut input_val: Vec<f64>) -> Self {
        input_val.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Self::new(input_val)
    }
}

/// Type a const has
pub type Const = f64;

/// Variable identification type
pub type VariableID = String;
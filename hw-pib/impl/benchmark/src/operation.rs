//! Operation Specific Data Structures
//!
//! Three important data structures for processing of
//! data:
//!     1. [`OperationInput`]: Can be unary, binary or nary
//!        information that is processed in a specific operation.
//!     2. [`OperationType`]: The type of predefined operations
//!        an analyst can use to write his algorithm upon. The names
//!        (except for "OverN" which means "Sum" all vector fields
//!         and return a scalar) are self explanatory.
//!     3. [`OperationOutput`]: This is a [`Variable`] Value
//!        representing the result of an operation.
//!
//! Keep in mind that a [`Variable`] is defined as a vector of values
//! (even scalars are one-dimensional!). Whereas operations are
//! using the [`Variable`]s thus we always perform vector operations
//! which we need to define.
//!
//! Furthermore: NAry operations are **always considered to be left
//! associative**!

use std::{
    cmp::PartialEq,
    iter::{Product, Sum},
    ops::{Add, Div, Mul, Sub},
};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, EnumVariantNames};

use super::variable::Variable;

/// Enum to combine all different input types for the calculations for easier handling
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display)]
pub enum OperationInput {
    Unary(Variable),
    Binary(Variable, Variable),
    NAry(Vec<Variable>),
}

/// All types of operations
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    PartialEq,
    Serialize,
    EnumVariantNames,
    Display,
    Hash,
    Eq,
    EnumString,
)]
pub enum OperationType {
    Addition,
    AdditionConst,
    AdditionOverN,
    MinimaOverN,
    MaximaOverN,
    Subtraction,
    SubtractionConstVar,
    SubtractionVarConst,
    Multiplication,
    MultiplicationConst,
    Division,
    DivisionConstVar,
    DivisionVarConst,
    Squareroot,
    Power,
    PowerConst,
    PowerBaseConst,
    Minima,
    Maxima,
    Absolute,
    DefConst,
}

pub type OperationOutput = Variable;

///
/// ON STANDARD CASES: They **cannot** be invoked since the operation caller does the case checking!
///                    We use this approach since Rust does not allow Errors while performing trait
///                    operations. This means that we need to handle them elsewhere!
///

/// Add all values in list
pub fn def_const(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Unary(constant) => constant,
        _ => OperationOutput::default(),
    }
}

/// Add all values in list
pub fn add(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::NAry(vars) => vars.into_iter().sum(),
        _ => OperationOutput::default(),
    }
}

/// Add a constant to a value
pub fn add_const(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, constant) => n0.add(constant),
        _ => OperationOutput::default(),
    }
}

/// Scalarize n-dimensional vector
pub fn add_over_n(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Unary(n0) => {
            let res = n0.into_vector().iter().fold(0_f64, |acc, x| acc + x);
            Variable::new(vec![res])
        }
        _ => OperationOutput::default(),
    }
}

/// Take minimum entry of n-dimensional vector
pub fn min_over_n(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Unary(n0) => {
            let mut sorted_vec = n0.into_vector();
            sorted_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
            Variable::new(vec![sorted_vec[0]])
        }
        _ => OperationOutput::default(),
    }
}

/// Take maximum entry of n-dimensional vector
pub fn max_over_n(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Unary(n0) => {
            let mut sorted_vec = n0.into_vector();
            sorted_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
            Variable::new(vec![sorted_vec[sorted_vec.len() - 1]])
        }
        _ => OperationOutput::default(),
    }
}

/// Subtract all elements from each other in order
pub fn sub(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::NAry(vars) => {
            let mut values = vars.clone();
            let first = values.remove(0); // take first value
            let sum_of_rest: Variable = values.into_iter().sum(); // sum all others
            first.sub(sum_of_rest)
        }
        _ => OperationOutput::default(),
    }
}

/// Subtract a variable from a constant (const - var)
pub fn sub_const_var(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, constant) => constant.sub(n0),
        _ => OperationOutput::default(),
    }
}

/// Subtract a constant from a variable (var - const)
pub fn sub_var_const(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, constant) => n0.sub(constant),
        _ => OperationOutput::default(),
    }
}

/// Take the product off all values
pub fn mul(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::NAry(vars) => vars.into_iter().product(),
        _ => OperationOutput::default(),
    }
}

/// Multiply a constant to a value
pub fn mul_const(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, constant) => n0.mul(constant),
        _ => OperationOutput::default(),
    }
}

/// Divide the first value by the seocond value
pub fn div(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, n1) => n0.div(n1),
        _ => OperationOutput::default(),
    }
}

/// Divide a constant by a variable (const/var)
pub fn div_const_var(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, constant) => constant.div(n0),
        _ => OperationOutput::default(),
    }
}

/// Divide the first value by the seocond value
pub fn div_var_const(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, constant) => n0.div(constant),
        _ => OperationOutput::default(),
    }
}

/// Take the square root of the value
pub fn sqrt(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Unary(n) => n.sqrt(),
        _ => OperationOutput::default(),
    }
}

/// Take the first value to the power of the second value
pub fn power(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, power) => n0.powf(power),
        _ => OperationOutput::default(),
    }
}

/// The same as `power` but with switched parameters
pub fn power_base_const(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Binary(n0, constant) => n0.powf(constant),
        _ => OperationOutput::default(),
    }
}

/// Returns the minimum value of the list
/// Warning: This implementation underlies the IEEE 754 inaccuracies!
pub fn min(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::NAry(values) => {
            let mut vars = values.clone();
            vars.sort_by(|a, b| a.partial_cmp(b).unwrap());
            vars.first().unwrap().clone()
        }
        _ => OperationOutput::default(),
    }
}

/// Returns the maximum value of the list
/// Warning: This implementation underlies the IEEE 754 inaccuracies!
pub fn max(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::NAry(values) => {
            let mut vars = values.clone();
            vars.sort_by(|a, b| a.partial_cmp(b).unwrap());
            vars.last().unwrap().clone()
        }
        _ => OperationOutput::default(),
    }
}

/// Returns the absolute value of the input
pub fn abs(input: OperationInput) -> OperationOutput {
    match input {
        OperationInput::Unary(n0) => n0.abs(),
        _ => OperationOutput::default(),
    }
}

///
/// Trait implementations for `Variable` and standard types
///

/// Warning: This implementation always returns NEQ in case the dimensions do not match
impl PartialEq<f64> for Variable {
    fn eq(&self, other: &f64) -> bool {
        if self.dim() != 1 {
            panic!("Can only compare single value variable with f64")
        };
        self.vector().first().unwrap().eq(other)
    }
}

impl PartialEq<Vec<f64>> for Variable {
    fn eq(&self, other: &Vec<f64>) -> bool {
        self.vector().eq(other)
    }
}

impl Add for Variable {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        map_binary_op(self, rhs, Add::add)
    }
}

impl Sub for Variable {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        map_binary_op(self, rhs, Sub::sub)
    }
}

impl Mul for Variable {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        map_binary_op(self, rhs, Mul::mul)
    }
}

impl Div for Variable {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        map_binary_op(self, rhs, Div::div)
    }
}

impl Sum for Variable {
    fn sum<I>(mut iter: I) -> Self
    where
        I: Iterator<Item = Variable>,
    {
        let first: Variable = iter.next().unwrap_or_default();
        iter.fold(first, |acc, x| acc + x)
    }
}

impl Product for Variable {
    fn product<I>(mut iter: I) -> Self
    where
        I: Iterator<Item = Variable>,
    {
        let first: Variable = iter.next().unwrap_or_default();
        iter.fold(first, |acc, x| acc * x)
    }
}

///
/// Non trait implementations for unary constant ops
///

impl Variable {
    pub fn sqrt(self) -> Self {
        map_unary_op(self, f64::sqrt)
    }

    pub fn powf(self, power: Self) -> Self {
        map_binary_op(self, power, f64::powf)
    }

    pub fn abs(self) -> Self {
        map_unary_op(self, f64::abs)
    }
}

///
/// Helper functions for mapping binary and unary [`OperationType`]s
///

/// Apply binary function to two variables
fn map_binary_op(first: Variable, second: Variable, op: fn(f64, f64) -> f64) -> Variable {
    // Check for proper dimesions
    if first.dim() != second.dim() {
        panic!(
            "Vector dimension mismatch: {}, {}",
            first.dim(),
            second.dim()
        )
    }

    // Perform map calculation
    let first_vector_iter = first.into_vector().into_iter();
    let second_vector_iter = second.into_vector().into_iter();
    let res = first_vector_iter
        .zip(second_vector_iter)
        .map(|a| op(a.0, a.1))
        .collect();

    Variable::new(res)
}

/// Apply unary function to one variable
fn map_unary_op(var: Variable, op: fn(f64) -> f64) -> Variable {
    let res = var.into_vector().into_iter().map(|val| op(val)).collect();
    Variable::new(res)
}

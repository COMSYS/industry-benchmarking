//! An atomic is the base operation of the analyst's uploaded
//! algorithms. It has a `name` field that specifies the necessarily
//! **unique** identifier for the opration.
//! Each atomic holds an operationType which can be one of in
//! [`OperationType`] specified operations, like addition, mult,...
//! There are var fields that hold the "variables" that are
//! involved in a specific atomic operation. The "variables" refer to
//! dependent atomic operations. This can be one (unary or with the
//! optional [`Const`] constant), two (binary) or $n$ variables ($n$-ary).
//! The dependendent variables have to be resolved first before the
//! computation of the current field can be performed.
//!
//! Additionally the boolean `is_kpi` field states wether the result
//! is important to the end result for companies that participate in the
//! benchmark. Others will be simply discarded.
//!
//! A template of one [`Atomic`] field in YAML:
//!
//!     - name: test_op
//!         op: Addition
//!         is_kpi: true
//!         var:
//!             - three
//!             - two_op
//!             - three
//!             - four
//!
//! The implementation of calc performs the computation of the result for
//! one specific input pair, that is provided (for a company).

use crate::{
    error::BenchmarkingError,
    operation::{
        abs, add, add_const, add_over_n, def_const, div, div_const_var, div_var_const, max,
        max_over_n, min, min_over_n, mul, mul_const, power, power_base_const, sqrt, sub,
        sub_const_var, sub_var_const, OperationInput, OperationOutput, OperationType,
        OperationType::*,
    },
    resolved::ResolvedValues,
    variable::{Const, Variable, VariableID},
};
use serde::{Deserialize, Serialize};

/// Atomic calculation unit
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Atomic {
    /// YAML var name
    name: VariableID,
    /// KPI is final result
    is_kpi: bool,
    /// Operation from
    op: OperationType,
    /// input variables for algorithm
    var: Vec<VariableID>,
    #[serde(default)]
    /// Constant operand
    constant: Option<Const>,
}

impl Atomic {
    /// New variable that is required as input but not specified by the analyst as an "atomic"
    pub fn new_required(name: VariableID) -> Self {
        // Required Variables are constant 0 additions
        Atomic {
            name: name.clone(),
            is_kpi: false,
            op: AdditionConst,
            var: Vec::new(),
            constant: Some(0_f64),
        }
    }

    /// make operation valid for computations
    pub fn make_op_valid(&mut self) {
        self.var.push(self.name.clone());
    }

    /// Return name of Atomic
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the `OperationType`
    pub fn op(&self) -> OperationType {
        self.op
    }

    /// Returns `true` if this is an kpi
    pub fn is_kpi(&self) -> bool {
        self.is_kpi
    }

    /// Retuns the names of all dependency variables
    pub fn var_ids(&self) -> &[VariableID] {
        &self.var
    }

    /// Retuns the constant value of operation
    pub fn constant(&self) -> &Option<Const> {
        &self.constant
    }

    // For parsing
    pub fn new(
        name: VariableID,
        is_kpi: bool,
        op: OperationType,
        var: Vec<VariableID>,
        constant: Option<Const>,
    ) -> Self {
        Atomic {
            name,
            is_kpi,
            op,
            var,
            constant,
        }
    }

    /// Calculate atomic
    pub fn calc<'a>(&'a self, resolved: &mut ResolvedValues) -> Result<(), BenchmarkingError> {
        // Error handling for calculation
        let throw_computation_error =
            |op_type: OperationType, expected_op_input: &str, receive_op_input: OperationInput| {
                BenchmarkingError::from(format!(
                    "operation {} expects {}, but received {:?}",
                    op_type.to_string(),
                    expected_op_input.to_string(),
                    receive_op_input.to_string()
                ))
            };

        // Operands and operation for computation of atomic
        let input = self.get_resolved_for_op(resolved, false)?;
        let input_op = self.get_map_op();

        let _input_cpy: OperationInput;

        #[cfg(not(feature = "evaluation"))]
        {
            _input_cpy = input.clone();
        }

        // Exhaustive list of operations and their constraints that mitigate panics on runtime
        // When returning, the result is a variable
        let var_res = match self.op {
            Absolute => match input {
                OperationInput::Unary(_) => input_op(input),
                _ => return Err(throw_computation_error(self.op, "Unary", input)),
            },
            Addition => match input {
                OperationInput::NAry(_) => input_op(input),
                _ => return Err(throw_computation_error(self.op, "NAry", input)),
            },
            AdditionConst => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => return Err(throw_computation_error(self.op, "Binary", input)),
            },
            AdditionOverN => match input {
                OperationInput::Unary(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Unary", input));
                }
            },
            MinimaOverN => match input {
                OperationInput::Unary(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Unary", input));
                }
            },
            MaximaOverN => match input {
                OperationInput::Unary(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Unary", input));
                }
            },
            Subtraction => match input {
                OperationInput::NAry(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "NAry", input));
                }
            },
            SubtractionConstVar => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            SubtractionVarConst => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            Multiplication => match input {
                OperationInput::NAry(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "NAry", input));
                }
            },
            MultiplicationConst => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            Division => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            DivisionVarConst => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            DivisionConstVar => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            Squareroot => match input {
                OperationInput::Unary(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Unary", input));
                }
            },
            Power => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            PowerConst => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            PowerBaseConst => match input {
                OperationInput::Binary(_, _) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "Binary", input));
                }
            },
            Minima => match input {
                OperationInput::NAry(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "NAry", input));
                }
            },
            Maxima => match input {
                OperationInput::NAry(_) => input_op(input),
                _ => {
                    return Err(throw_computation_error(self.op, "NAry", input));
                }
            },
            DefConst => match input {
                OperationInput::Unary(_) => input_op(input),
                _ => return Err(throw_computation_error(self.op, "Unary", input)),
            },
        };

        #[cfg(not(feature = "evaluation"))]
        {
            log::info!(
                "[Atomic: {}] [Type: {}] [Input: {:?}] [Output: {:?}]",
                self.name(),
                self.op.to_string(),
                _input_cpy,
                var_res
            );
        }

        // Insert into resolved
        resolved.insert(self.name().to_string(), var_res)?;

        Ok(())
    }

    // Select input values from `resolved_values`. Does some sanity checks and chooses the right amount of numbers
    fn get_resolved_for_op<'a>(
        &'a self,
        resolved_values: &'a ResolvedValues,
        _is_eval: bool,
    ) -> Result<OperationInput, BenchmarkingError> {
        // Prepares the input with the InputType enum. This makes handling easier
        // For example `Addition` input can have arbitrary length.
        // `Power` on the other hand has always two input vars, and `Absolute` has only one input variable.

        // Eval requires another connection mode to peers to communicate with!
        log::info!("{} uses var {:?}", self.name(), &self.var);

        match self.op {
            //
            // NARY OPERATIONS
            //
            Addition | Subtraction | Multiplication | Minima | Maxima => {
                #[cfg(not(feature = "evaluation"))]
                {
                    // Sanity checks: at least one var and no const required
                    if self.var.len() == 0 {
                        return Err(BenchmarkingError::from((self.clone(), "has no variables")));
                    }
                    if let Some(constant) = self.constant {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("has unused constant {:?}", constant),
                        )));
                    }
                }

                // Extract resolved nary values for further computation
                let mut values = Vec::new();
                for var in &self.var {
                    let val = resolved_values.get(var)?;
                    values.push(val.clone());
                }
                Ok(OperationInput::NAry(values))
            }

            //
            // BINARY OPERATIONS WITH 2 VARIABLES
            //
            Division | Power => {
                #[cfg(not(feature = "evaluation"))]
                {
                    // need exactly 2 operands
                    let len = self.var.len();
                    if len != 2 {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("binary op expected 2 vars but received {}", len),
                        )));
                    }
                    if let Some(constant) = self.constant {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("has unused constant {:?}", constant),
                        )));
                    }
                }

                // Extract resolved binary values for further computation
                let var_names = (&self.var[0], &self.var[1]);
                let operands = (
                    resolved_values.get(var_names.0)?,
                    resolved_values.get(var_names.1)?,
                );

                #[cfg(not(feature = "evaluation"))]
                {
                    // Verify against 0-divisions
                    if operands.1.vector().iter().find(|&&x| x == 0_f64).is_some()
                        && self.op == Division
                    {
                        //operands.1 = Variable::new(vec![1.0]);
                        //log::error!("0-Division for {}", var_names.1);
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("has 0-Division for value {}", var_names.1),
                        )));
                    }
                }

                Ok(OperationInput::Binary(
                    operands.0.clone(),
                    operands.1.clone(),
                ))
            }

            //
            // BINARY OPERATIONS WITH ONE VAR AND ONE CONSTANT ON SECOND POSITION
            //
            AdditionConst | SubtractionVarConst | MultiplicationConst | DivisionVarConst
            | PowerConst | PowerBaseConst => {
                #[cfg(not(feature = "evaluation"))]
                {
                    // sanity check: expect one var and one const
                    let len = self.var.len();
                    if len != 1 {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!(
                                "binary op with constant expected 1 var but received {}",
                                len
                            ),
                        )));
                    }
                    if self.constant.is_none() {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            "is binary op with constant but no constant is provided!",
                        )));
                    }
                }

                // Extract computed values → The constant is always the second operand
                let var_name = &self.var[0];
                let operand0 = resolved_values.get(var_name)?;
                let operand1 = Variable::new(vec![self.constant.unwrap_or_default()]);

                #[cfg(not(feature = "evaluation"))]
                {
                    // Verify against 0-divisions
                    if operand1.vector().iter().find(|&&x| x == 0_f64).is_some()
                        && (self.op == DivisionVarConst || self.op == SubtractionVarConst)
                    {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            "has 0-Division for provided constant",
                        )));
                    }
                }

                Ok(OperationInput::Binary(operand0.clone(), operand1))
            }

            //
            // BINARY OPERATIONS WITH ONE VAR AND ONE CONSTANT ON FIRST POSITION
            //
            SubtractionConstVar | DivisionConstVar => {
                #[cfg(not(feature = "evaluation"))]
                {
                    // sanity check: expect one var and one const
                    let len = self.var.len();
                    if len != 1 {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!(
                                "binary op with constant expected 1 var but received {}",
                                len
                            ),
                        )));
                    }
                    if self.constant.is_none() {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            "is binary op with constant but no constant is provided!",
                        )));
                    }
                }

                // Extract computed values → The constant is always the second operand
                // We do error handling here to check whether the divisor is 0.
                let var_name = &self.var[0];
                let operand0 = resolved_values.get(var_name)?;
                let operand1 = Variable::new(vec![self.constant.unwrap_or_default()]);

                #[cfg(not(feature = "evaluation"))]
                {
                    // Verify against 0-divisions
                    if operand0.vector().iter().find(|&&x| x == 0_f64).is_some()
                        && (self.op == DivisionVarConst || self.op == SubtractionVarConst)
                    {
                        //operand1 = Variable::new(vec![1.0]);
                        //log::error!("0-Division for {:?}", operand0);
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!(
                                "has 0-Division for provided variable operand: {:?}",
                                operand0
                            ),
                        )));
                    }
                }

                Ok(OperationInput::Binary(operand0.clone(), operand1))
            }

            //
            // UNARY OPERATION WITH NO CONSTANT
            //
            Squareroot | Absolute | AdditionOverN | MinimaOverN | MaximaOverN => {
                #[cfg(not(feature = "evaluation"))]
                {
                    // sanity checks: one operand only
                    let len = self.var.len().clone();
                    if len != 1 {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("binary op expected 2 vars but received {}", len).as_str(),
                        )));
                    }
                    if let Some(constant) = self.constant {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("has unused constant {:?}", constant),
                        )));
                    }
                }

                // Extract resolved values
                let var_name = &self.var[0];
                let n = resolved_values.get(var_name)?;
                Ok(OperationInput::Unary(n.clone()))
            }

            //
            // DEFINITION OF CONSTANTS
            //
            DefConst => {
                #[cfg(not(feature = "evaluation"))]
                {
                    // sanity checks: no variable should exist
                    let len = self.var.len().clone();
                    if len != 0 {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("constant definition expected 0 vars but received {}", len)
                                .as_str(),
                        )));
                    }
                    if self.constant.is_none() {
                        return Err(BenchmarkingError::from((
                            self.clone(),
                            format!("has no constant"),
                        )));
                    }
                }

                // Extract resolved values
                let operand0 = Variable::new(vec![self.constant.unwrap_or_default().into()]);
                Ok(OperationInput::Unary(operand0))
            }
        }
    }

    #[allow(unused_parens)]
    /// Map the operation type to the function that performs it
    fn get_map_op(&self) -> (fn(OperationInput) -> OperationOutput) {
        match self.op() {
            Addition => add,
            AdditionConst => add_const,
            AdditionOverN => add_over_n,
            MinimaOverN => min_over_n,
            MaximaOverN => max_over_n,
            Subtraction => sub,
            SubtractionConstVar => sub_const_var,
            SubtractionVarConst => sub_var_const,
            Multiplication => mul,
            MultiplicationConst => mul_const,
            Division => div,
            DivisionConstVar => div_const_var,
            DivisionVarConst => div_var_const,
            Squareroot => sqrt,
            Power | PowerConst => power,
            PowerBaseConst => power_base_const,
            Minima => min,
            Maxima => max,
            Absolute => abs,
            DefConst => def_const,
        }
    }
}

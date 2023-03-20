# Benchmarking

Our benchmarking uses a predefined input format to describe algorithms that compute KPIs.
The input format requires some variables to be provided for computation.
These variables are usually provided by the companies that participate in the form of an input list.

The algorithm computation has *no bounds* for the HW-PIB implementation, i.e., it can be arbitrarily nested and complex.
This is not necessarily the case for the SW-PIB implementation.
Since HW-PIB is compared to SW-PIB using homomorphic encryption (HE), a shared scheme is used to provide compatibility.
The scheme uses `atomic`-subformulas, which consist of constant, unary, binary, and $n$-ary operators. 

## Workflow

We have summarized an example workflow in [workflow](./workflow)

## Atomic type

An atomic formula is a base operation for KPI computation.
The usage of atomic formulas can be nested and thus create complex evaluation trees. 
The scheme of one atomic is explained in the following by using a template of one `Atomic` field in YAML:

```yaml
- name: test_op
    op: Addition
    is_kpi: true
    var:
        - three
        - two_op
        - three
        - four
```

Each atomic holds an operation type `op`, which can be one of in `OperationType` specified operations:

| Operation name | Explanation |
|----------------|-------------|
|`Addition`             | Add two variables. |
|`AdditionConst`        | Add one variable to one constant. |
|`AdditionOverN`        | Sum the variable (which is a vector) to a scalar. |
|`Subtraction`          | Subtract the first input variable from the second input variable.|
|`SubtractionConstVar`  | Subtract a variable from a constant.|
|`SubtractionVarConst`  | Subtract a constant from a variable.|
|`Multiplication`       | Multiply two variables.|
|`MultiplicationConst`  | Multiply a variable with a constant. |
|`Division`             | Divide the first input variable from the second.|
|`DivisionConstVar`     | Divide a constant through a variable.|
|`DivisionVarConst`     | Divide a variable through a constant.|
|`Squareroot`           | Take the square root of a non-negative input variable.| 
|`Power`                | Exponentiation of two variables.|
|`PowerConst`           | Exponentiation with variable base and constant exponent.|
|`PowerBaseConst`       | Exponentiation with constant base and variable exponent.|
|`Minima`               | Take the minimum over $n$ input variables.|
|`Maxima`               | Take the maximum over $n$ input variables.|
|`Absolute`             | Take the absolute value of the computation.|
|`DefConst`             | Define a constant value.|


There is a var list that holds the "variables" that are involved in a specific atomic operation. 
The variables refer to dependent atomic operations, which are either one (unary or with the optional [`Const`] constant), two (binary), or $n$ variables ($n$-ary).
The dependent variables must be resolved first before the computation of the current field can be performed.

## Constructing Custom Examples

To find out more information on defining concrete algorithms and corresponding necessary inputs, we refer to the following guides:

- [Algorithms](./algorithms.md) describes how to create custom algorithms using the presented operations.
- [Inputs](./input.md) delivers more information on creating concrete input files for an existing algorithm.

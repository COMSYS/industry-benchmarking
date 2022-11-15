# Algorithms and Formulae

As described in [the introduction](./main.md), atomic formulas are used as base computations.
We present the way to compute and extract them in the following.

## Definition of the intermediary file format

As a (input) common format for all implementations, we defined an intermediary format on which the respective approaches operate on.
We describe it in the following.
It uses a YAML format to maintain the file format along with all other files.

```yaml
- name: abc
  is_kpi: false
  op:
    Binary:
      op: Multiplication
      lhs:
        Literal:
          var: def
      rhs:
        Literal:
          var: geh
- name: ijk
  is_kpi: false
  op:
    Literal:
      constant: 0.16
- name: lmn
  is_kpi: false
  op:
    NAry:
      op: Maxima
      vars:
        - Literal:
            var: opq
        - Literal:
            var: rst
```

Each variable has (similar to the atomic definition) `name` and `is_kpi` fields.
In contrast to the atomic format, it holds an `op`-field which is an expression of type `Unary, Binary, NAry`, or a `Literal`.

- For `Unary` operations, an `op`-field and a (possibly nested) expression are provided.  
- For `Binary` operations, an `op`-field is given but also `lhs` and `rhs` (left-and-right-hand-side), which can again consist of expressions.
- As for `NAry` operations, an `op`-field is also given but *now only a `vars` field for the used variables*.
- `Literal`-fields are either a numeric literal `constant` or `var` for a variable that is to be used.

The recursion stops as soon as the `Literal` field is reached, as it only can evaluate to a terminal symbol.
Note that no constants have to be introduced as an explanation due to the naming of the binary operator's sides.

# Intermediary Formula Parser


## Using the parser to make nested formulas atomic

To use the parser, you have to provide an input file:

```bash
cargo run -- <Path of YAML file with formulas in intermediary format>
```

## Minimum Working Example

In this repository, you can execute the command below to compute the atomic formulas for the practical example:

```bash
cargo run --release -- ../data/intermediary.yaml
```

Alternatively, you could provision the YAML file with some lines as shown below:

```yaml
---
- name: testformula
  is_kpi: true
  op:
    Binary:
      op: Power
      lhs:
        Binary:
          op: Subtraction
          lhs:
            Literal:
              var: input
          rhs:
            Literal:
              constant: 40.0
      rhs:
        Literal:
          constant: 2.0
```

This formula corresponds to the following natural description. 
The input represents the Abstract syntax tree (AST) for the formula. 
This format is subsequently used to prune the tree and to make the formula atomic.

```
[testformula] := ([input] - 40.0)^2 
```

In general, it is possible to use all kinds of variables as inputs, like in the following case `[input]`.
In this example, it is mandatory to demand it from all participating companies.
Otherwise, the computation will fail / not commence due to missing inputs. 

You may find the given MWE in the `../data/intermediary.yaml` directory, where you can invoke the program accordingly.


### Minimum working example (MWE)

The output of the above input consists of two files: One is a YAML AST-styled format output, whereas the other one is an atomic output that is compatible with homomorphic evaluation. The output for the MWE is shown below:

```YAML
The output of this file has only operations that use/define other variables or constants. For this purpose, helper variables are introduced to allow a straightforward computation.
---
operations:
  - name: he0001
    is_kpi: false
    op: DefConst
    var: []
    constant: 40.0
  - name: he0000
    is_kpi: false
    op: Subtraction
    var:
      - input
      - he0001
    constant: ~
  - name: he0002
    is_kpi: false
    op: DefConst
    var: []
    constant: 2.0
  - name: testformula
    is_kpi: true
    op: Power
    var:
      - he0000
      - he0002
    constant: ~
```

By default, the output is written to `../files/algo_atomic.yaml`.
Still, it is possible to change this behavior by providing a user-defined output path that is relative to the execution directory.
The script expects that the path exists and is writable.

```bash
Usage:   cargo run -- <Intermediary Input YAML> <Atomic Output YAML>
Example: cargo run -- ../data/intermediary.yaml ../data/out.yaml
```

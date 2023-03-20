# Workflow for evaluating SW-PIB and HW-PIB

To evaluate the artifacts, we now present a workflow elaborating on the respective steps. 
Prior to the evaluation, the implementations must be installed according to the instructions described in [`README.md`](./README.md).

The evaluation starts with the input generation using the `formula-parsing` artifact. 
Subsequently, the generated files are used separately for `HW-PIB` and `SW-PIB` to evaluate the individual performance.

To provide guidance, we use a small example to perform the steps outlined below. 
For this purpose, we use the implementation of the well-known midnight formula:

```math
 x_{1,2} = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}
```

## 1. Generating Input Data

We constructed two formulas to generate the input data, each for the $x_1$ and $x_2$ variants. 
Both of them use the "root_part", which we remove for reasons of redundancy. 
The input is available in `abc.yaml`.

To transform the input into an atomic format, apply the transformation rules: 

```
cd formula-parsing/formula-parsing-impl
cargo run --release -- ../../benchmarking-docs/workflow/abc.yaml ../../benchmarking-docs/workflow/atomic_abc.yaml
```

The result of this application is the `atomic_abc.yaml` file.
As input for the sample, we use the file `company0.yaml`, which holds the values `input_a`, `input_b`, and `input_c`.


## 2. Computing the Benchmark with HW-PIB

1. Start the server inside a docker container (with or without working SGX)

```
cd hw-pib/
./run-scone-container-wo-las-sim.sh # Simulation mode
./run-scone-container-wo-las.sh # Hardware mode
```

2. Run the clients via our [orchestra](./hw-pib/impl/orchestra) tool.

This deployment should then result in a successful computation using HW-PIB.


## 3. Computing the Benchmark with SW-PIB

Assuming an installed SW-PIB, the execution is possible by executing the following command:

```
cd sw-pib/src/
python main.py -i ../../benchmarking-docs/workflow/input -a ../../benchmarking-docs/workflow/abc_atomic.yaml -c ./settings/PIB_SEAL.yaml -e ../../benchmarking-docs/workflow/sw_eval
```

# SW-PIB Usage

To set up SW-PIB (if not already done), run `./setup_seal.sh`!


Invoking the `SW-PIB` implementation is as easy as `cd src/` and calling `python main.py`.
To get an overview of possible arguments, use the `-h` option.

```
usage: PIB Execution [-h] [-a ALGORITHMS] [-i INPUTS] [-e EVAL] [-c CONFIG]

options:
  -h, --help            show this help message and exit
  -a ALGORITHMS, --algorithms ALGORITHMS
                        Atomic algorithm file
                        DEFAULT: "../data/Test/Algorithms/atomics.yaml"
  -i INPUTS, --inputs INPUTS
                        Directory where the "comp00.yaml" ... "compN.yaml" are inputs.
                        DEFAULT: "../data/Test/Inputs/"
  -e EVAL, --eval EVAL  File where the eval is written to.
                        DEFAULT: "../data/Results/results.csv"
  -c CONFIG, --config CONFIG
                        Configuration file for the Proxy (relevant or eval)
                        DEFAULT: "../settings/PIB_SEAL.yaml"
  ```

### Minimum Working Example (MWE) for Algorithms and Inputs

  The `inputs` directory holds the inputs for the algorithms that were previously specified. 
  We provide an MWE:

  ```yaml
  ---
  operations:
  - name: test_op
    op: Multiplication
    is_kpi: true
    var:
      - three
      - two

  - name: add_n3
    op: Multiplication
    is_kpi: true
    var:
      - test_op
      - one

  - name: mult_n4
    op: Multiplication
    is_kpi: true
    var:
      - add_n3
      - two
```

Observe that there are variables `three`, `two`, and `one` that are not defined as operations but are used as input values.
In this case, the participants must define these variables; for example:

```yaml
vars:
    - name: one
      min_val: -10
      max_val: 10
      values: [1.0]

    - name: two
      min_val: -10
      max_val: 10
      values: [1.1]

    - name: three
      min_val: -10
      max_val: 10
      values: [1.0]

    - name: four
      min_val: 0
      max_val: 0
      values: [1.1]
```

Please note that it is allowed to provide additional inputs without enforcing errors.
Additionally, the `values` parameter generally holds a list of values, even though only one value is entered in this example.
This implementation decision allows performing computations on multidimensional ciphertexts. 

### Evaluation

The evaluation is written into the `./data/Results/results.csv` file with measurements regarding computations.
These include:

| Parameter | Description |
|-----------|-------------|
|`traffic_bytes` | Bytes that were up- and downloaded between the proxy and the clients. |
|`ciphers_up` | Amount of ciphers that were uploaded from the proxy to the clients. |
|`ciphers_down` | Amount of ciphers that were downloaded from the clients to the proxy. |
|`cipher_size` | Size of the public key, relinearization key and galois key that are used by the proxy. |
|`op_local` | Average duration for an operation that was computed locally. |
|`levels` | CKKS Level that specifies the depth of successive multiplications. |
|`op_offload` | Duration of an offloaded operation (without latencies). |
|`benchmarking_clients` | Average benchmarking duration for one company. |
|`client_agg`| Aggregation duration on the statistics server. |
|`keygen`| Duration for key generation, which depends on the `level` and `poly_modulus_size`. |
|`keygen_size`| Size of the public key, relinearization key and galois key that are used by the proxy. |
|`sample`| Path of the utilized algorithm (contains the name) |
|`benchmarking`| Overall benchmarking duration.|
|`proxy_agg`| Aggregation duration on the proxy.| 
|`server_agg`| Aggregation duration on the statistics server.|
|`accuracy`| Average percentage of deviation from the plaintext result value of all participants' KPIs.|
|`offloaded_pct`| Percentage of operations that were required to be offloaded.|

You may provide an alternative path as long as the directory structure and permissions exist for writing to the file.


### Configuration

The file `settings/PIB-SEAL.yaml` holds an exemplary and annotated configuration for using `SW-PIB`.
To change configurations, please refer to the explanations and change the `-c` parameter in case of multiple configurations.

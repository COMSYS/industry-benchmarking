# Designing Secure and Privacy-Preserving Information Systems for Industry Benchmarking


## About

This repository contains our fully-tested prototypes of HW-PIB & SW-PIB, our implementations that offer Privacy-Preserving Industry Benchmarking using Trusted Execution Environments (TEEs) and Fully Homomorphic Encryption (FHE), respectively.

> Benchmarking is an essential tool for industrial organizations to identify potentials that allows them to improve their competitive position through operational and strategic means. However, the handling of sensitive information, in terms of (i) internal company data and (ii) the underlying algorithm to compute the benchmark, demands strict (technical) confidentiality guarantees—an aspect that existing approaches fail to address adequately. Still, advances in private computing provide us with building blocks to reliably secure even complex computations and their inputs, as present in industry benchmarks. In this paper, we thus compare two promising and fundamentally different concepts (hardware- and software-based) to realize privacy-preserving benchmarks. Thereby, we provide detailed insights into the concept-specific benefits. Our evaluation of two real-world use cases from different industries underlines that realizing and deploying secure information systems for industry benchmarking is possible with today's building blocks from private computing.


## Publication

- Jan Pennekamp, Johannes Lohmöller, Eduard Vlad, Joscha Loos, Niklas Rodemann, Patrick Sapel, Ina Berenice Fink, Seth Schmitz, Christian Hopmann, Matthias Jarke, Günther Schuh, Klaus Wehrle, Martin Henze: *Designing Secure and Privacy-Preserving Information Systems for Industry Benchmarking*. Proceedings of the 35th International Conference on Advanced Information Systems Engineering (CAiSE '23), LNCS - Volume 13901, Springer, 2023.

If you use any portion of our work, please cite our publication.


```bibtex
@inproceedings{pennekamp2023designing,
    author = {Pennekamp, Jan and Lohmöller, Johannes and Vlad, Eduard and Loos, Joscha and Rodemann, Niklas and Sapel, Patrick and Fink, Ina Berenice and Schmitz, Seth and Jarke, Matthias and Schuh, Günther and Wehrle, Klaus and Henze, Martin},
    title = {{Designing Secure and Privacy-Preserving Information Systems for Industry Benchmarking}},
    booktitle = {Proceedings of the 35th International Conference on Advanced Information Systems Engineering (CAiSE '23)}
    year = {2023},
    month = {06},
    doi = {10.1007/978-3-031-34560-9_29},
    publisher = {Springer}
}
```


## License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.


This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see http://www.gnu.org/licenses/.

If you are planning to integrate parts of our work into a commercial product and do not want to disclose your source code, please contact us for other licensing options via email at pennekamp (at) comsys (dot) rwth-aachen (dot) de.


## Acknowledgments

Funded by the Deutsche Forschungsgemeinschaft (DFG, German Research Foundation) under Germany's Excellence Strategy — EXC-2023 Internet of Production — 390621612.
We thank Jan-Gustav Michnia for his initial exploration of the FHE library CONCRETE.


## Workflow

We have summarized an example workflow in [workflow](./benchmarking-docs/workflow).


## Artifacts

We provide two implementations for conducting privacy-preserving industry benchmarking.

1. **Trusted Execution Environment**-based benchmarking using SCONE and Docker.
2. **Fully Homomorphic Encryption**-based benchmarking using Microsoft SEAL.

The setup has been tested by the way on Arch Linux and Ubuntu 22.04.
We do not guarantee that this setup works for any other operating system.
After the setup, we describe how to execute the benchmarking applications.

### Trusted Execution Environment Implementation using Rust and SCONE

Running the TEE implementation requires an Intel SGX-capable platform for running secured computations.
Nonetheless, it is still possible to run the following examples in "Simulation" mode.
The implementation of "HW-PIB" (sometimes "TEEBench") uses Scone to function correctly.

0. Before running the implementation, it is necessary to install Scone (with or without SGX support enabled).
Please refer to their documentation for [installing Scone on your system](https://sconedocs.github.io/installation/).
When using Intel SGX, please ensure that the driver is installed and working successfully.
    - Additionally, you need to [get access to Scone's Docker registry](https://sconedocs.github.io/workshop/setup/).
    - Furthermore, it is mandatory to [define a `scone` alias](https://sconedocs.github.io/Exercise-Docu/exercise0/).
    - To retrieve the SGX device, you can execute the following function with bash, which is given [by Scone](https://sconedocs.github.io/sgxinstall/#determine-sgx-device):
    ```
    function determine_sgx_device {
        export SGXDEVICE="/dev/sgx_enclave"
        export MOUNT_SGXDEVICE="--device=/dev/sgx_enclave"
        if [[ ! -e "$SGXDEVICE" ]] ; then
            export SGXDEVICE="/dev/sgx"
            export MOUNT_SGXDEVICE="--device=/dev/sgx"
            if [[ ! -e "$SGXDEVICE" ]] ; then
                export SGXDEVICE="/dev/isgx"
                export MOUNT_SGXDEVICE="--device=/dev/isgx"
                if [[ ! -c "$SGXDEVICE" ]] ; then
                    echo "Warning: No SGX device found! Will run in SIM mode." > /dev/stderr
                    export MOUNT_SGXDEVICE=""
                    export SGXDEVICE=""
                fi
            fi
        fi
    }
    ```
1. Clone the repository with `git clone --recursive git@github.com:COMSYS/industry-benchmarking.git`.
2. `cd industry-benchmarking/hw-pib/scone/`
3. Follow the instructions in [`README.md`](./hw-pib/scone/README.md) for creating and executing the container.
    - You may start the Docker container on localhost. Please make sure to add an entry into your systems `/etc/hosts` file:
    ```
    127.0.0.1	teebench.xyz
     ```
    - To attach to the running container, execute `docker attach server_encrypted`.
4. To interact with the Docker container, refer to the usage of the [evaluation orchestra](./hw-pib/impl/orchestra/README.md).

#### Dependencies

The installation in SGX hardware mode was only tested on Ubuntu 22.04.
The simulation mode was tested on Arch Linux but is not officially supported.

```
- Scone and Docker (+ Docker-compose) are required to run the containers.
- Rust is required

- Ubuntu 22.04:
    - sudo apt install apt-transport-https ca-certificates curl software-properties-common gnupg-agent # Docker deps
    - curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add - # Docker signing key
    - sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" # Docker stable repository
    - sudo apt update # Update package information
    - sudo apt install docker-ce docker-ce-cli containerd.io docker-compose -y # Finally, install docker and docker-compose

    - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # Install rust with their rustup utility
```

We always used Rust version 1.57, but we assume that newer stable versions should also operate as intended.


### Fully Homomorphic Encryption Implementation using Python and Microsoft SEAL

1. Clone the repository with `git clone --recursive git@github.com:COMSYS/industry-benchmarking.git` to also retrieve the required submodules/libraries.
This step automatically clones [Microsoft SEAL](https://github.com/microsoft/SEAL) and [SEAL-Python](https://github.com/Huelse/SEAL-Python).

1. Run `cd industry-benchmarking/sw-pib/`.

2. Install dependencies.

3. Run the `setup_seal.sh` script to get the latest version of SEAL-Python.

4. Run `cd src/`.

6. Run `python3 main.py`.


#### Dependencies

```
- Recommended for SEAL-Python and SEAL: Clang++ (>= 10.0) or GNU G++ (>= 9.4), CMake (>= 3.16)

- Ubuntu 22.04:
   - sudo apt install git build-essential cmake python3 python3-dev python3-pip
- Arch Linux:
    - yay -S git ** cmake base-devel python python-pip python-devtools
```

### Formula Parsing

To translate intermediary formulas into atomic ones, the formula-parsing utility may be used.
For Ubuntu 22.04, the following Rust dependencies are required:

```
sudo apt install curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # Install rust with their rustup utility
```

To use this utility, please refer to its specific [README](./formula-parsing/formula-parsing-impl/README.md).

### Benchmarking Documentation

A detailed explanation of the definition of the formulas and inputs is found in the [benchmarking documentation](./benchmarking-docs/main.md).
This information is especially helpful when designing individual and custom algorithms.


## Usage of all Implementations

The usage of all implementations can be looked up separately in the matching subdirectories: `formula-parsing/formula-parsing-impl`, `hw-pib`, and `sw-pib`.

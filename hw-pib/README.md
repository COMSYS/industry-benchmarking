# HW-PIB Overview

This README file provides an overview of all relevant components for executing HW-PIB. 
We refer to further information in the respective README files.

## Executing Scone Containers inside TEE

To execute the Scone container, hosting the HW-PIB server inside a TEE, you need to run `./run-scone-container-wo-las.sh` (after building the container).

In case you want to run the Scone container in Simulation mode, run `.run-scone-container-wo-las-sim.sh`. 
Please make sure that in this case, you need to modify the `hw-pib/scone/teebench-template.yml` file's `services::environment` section to have `SCONE_MODE: sim` set.

## Executing the benchmarking application outside TEE

To run the benchmarking application outside the TEE, we refer to the [server subdirectory](./impl/server/README.md), holding more information on concrete command line interface arguments and configurations. 

## Executing the Companies and the Analyst outside the TEE

Automated operation of the analyst and clients from outside the enclave is possible using the [Orchestra tool](./impl/orchestra/README.md).
Of course, it is also possible to use it manually with the clients; however, the execution is more complex, which is why we do not recommend it.

# Application Deployment


## Server Setup

First, we start the server using one of the following two alternatives:

1. Directly _without_ the use of security features, which allows the execution of a plain binary, outside of SCONE.
2. The use of Scone, which deploys the application inside a Docker container.

### 01 — Without security features

The first option is easy to achieve:
We tested this with rust version `1.66`.

```bash
cd ../impl/server
cargo install --path . --features=evaluation # Evaluation features are enabled
cargo install --path . # Evaluation features are disabled
```

This installs the `server` binary. Make sure that `~./.cargo/bin` is in your `PATH`
%% Then, the application can be executed with the server command on the CLI.
Please be sure to have a `templates/crypto/server/` in the parent directory of your current directory in which you execute the binary (relative path).
There the program expects a `server.key` and a `server.pem` file for running the HTTPS server.

The server uses the ports `8080` and `8443`.

### 02 — With security features

To set up Scone, perform the following steps:

1. Find out the `MRENCLAVE` of the container by building it and running it for testing.
    - Copy from the line:
    ```
    teebench-scone-rust-1  | Enclave hash: fe1639f15e46fff40656ac580729c09235a905b88d057f083f1ee627eae9c4f4
    ```
    - This is the required hash for the Container
2. Create a real SCONE policy by pasting the `MRENCLAVE` from above in the `create-scone-container.sh` file
3. Build again but this time without simulation and debug options

## Scone Setup Modes

Before executing the `hw-pib/scone/create-scone-container.sh` file, you have to make sure that your system runs Intel SGX with matching drivers.
Furthermore, having all prerequisites fulfilled, it is still mandatory to export an environment variable before executing the setup script.

```bash
cd hw-pib/scone/

export SCONE_HW=1

# Create a session with the provided session template
sudo ./create-scone-container.sh
```

This environment variable forces using the `/dev/isgx` device (which is usually the standard with the Intel driver).
In case this does not match your use case, change the `$DEVICE` variable in the script.

> If the `SCONE_HW` variable is not set, the execution mode defaults to simulation mode! This results in errors when executing the `docker-compose` command or the other scripts!
> ```
> [SCONE|FATAL] tools/starter/main.c:555:enclave_create(): Could not create enclave builder: Error opening SGX device
> [SCONE|WARN] tools/libsgx/src/platform.c:207:platform_detect_sgx_driver(): No SGX driver found, won't be able to run in SGX HW mode
> ```

Executing this script creates a container that is tagged with `teebench_server_image:latest`.


## Verifying the created container

To verify the created container, you may execute the `./verify-scone-container.sh` script, which writes the CA-certificate of the container to `/tmp/teebench-ca.crt`.
To establish a truely secure connection, we recommend to add this CA-certificate to your system's trusted store.

## Executing the scone container with LAS

Executing the container with [LAS](https://sconedocs.github.io/helm_las/) is possible by running `./run-scone-container.sh`. Remember that for successful attestation, it is mandatory to use the commercial version of Scone. We quote the [documentation of Scone](https://sconedocs.github.io/public-CAS/):

> We maintain public CAS instances at domain `scone-cas.cf`.
> **These instances run in debug mode and should be used only for evaluation purposes.**
> **They are also regularly restarted with a clean database. Don't use them in production!**

For the execution *without* LAS, you can simply run `../run-scone-container-wo-las.sh`, which omits this attestation.
Still, it requires the container to run with Intel SGX enabled.
All security guarantees of SGX still hold in this setup and do not differ from the LAS mode!

## Executing the Scone container in simulation mode

Before being able to execute the container in simulation mode, it is required to change the `../container-scripts/execute.sh`.
In this file, you should replace `SCONE_HW=HW` with `SCONE_HW=SIM`.
This adjustment circumvents errors when not having an Intel SGX-capable device at hand.
To execute this mode, run `../run-scone-container-wo-las-sim.sh`.

## Output when starting HW-PIB

An exemplary output (here for SIM-mode) is given below.
It should look similar when invoking `docker attach server_encrypted`:

```
export SCONE_QUEUES=8
export SCONE_SLOTS=512
export SCONE_SIGPIPE=1
export SCONE_MMAP32BIT=0
export SCONE_SSPINS=100
export SCONE_SSLEEP=4000
export SCONE_TCS=8
export SCONE_LOG=WARNING
export SCONE_HEAP=6442450944
export SCONE_STACK=4194304
export SCONE_CONFIG=/etc/sgx-musl.conf
export SCONE_ESPINS=10000
export SCONE_MODE=sim
export SCONE_ALLOW_DLOPEN=no
export SCONE_MPROTECT=no
export SCONE_FORK=no
export SCONE_FORK_OS=0
musl version: 1.1.24
SCONE version: 5.7.0 (Tue Jan 18 08:45:24 2022 +0100)
Enclave hash: d6603c7d60ed5fd140ce62446154759f13449853411012baa378b45ab5353f04
[SCONE|WARN] src/enclave/dispatch.c:203:print_version(): Application runs in SGX simulation mode.
        Its memory can be read by the untrusted system! This is not secure!
2022-10-21T08:24:42.437543714+00:00
== Evaluation Enabled! ==
```

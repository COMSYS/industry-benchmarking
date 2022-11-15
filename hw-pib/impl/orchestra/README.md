# TEEBench Orchestra

An orchestra (/ˈɔːrkɪstrə/) is a large ensemble typical of classical subprograms, which combines instruments from different families, including

- The Server that can be executed on the host
- The `Analyst` client who uploads CA certificates, algorithms, and benchmarking configuration files
- The `Company` clients who upload their **secret**__ data along with each other to get their results

each grouped in sections.

## File Structure

To use the orchestra, create a `.zip` file containing the following files with this directory structure:

```
teebench.zip
├── analyst
│    ├── algorithms.yaml
│    └── benchmarking_config.yaml
├── crypto
│   ├── analyst
│   │   ├── analyst.key
│   │   ├── analyst.pem
│   │   └── analyst.pfx
│   ├── analyst_ca
│   │   ├── analyst_ca.key
│   │   └── analyst_ca.pem
│   ├── comp00
│   │   ├── comp00.key
│   │   ├── comp00.pem
│   │   └── comp00.pfx
│   ├── ...
│   │   ├── ... .key
│   │   ├── ... .pem
│   │   └── ... .pfx
│   ├── compNN
│   │   ├── compNN.key
│   │   ├── compNN.pem
│   │   └── compNN.pfx
│   ├── server
│   │   ├── server.key
│   │   ├── server.pem
│   │   └── server.pfx
│   └── server_ca
│       ├── server_ca.key
│       └── server_ca.pem
├── inputs
│   ├── comp00.yaml
│   ├── comp ... .yaml
│   └── compNN.yaml
└── orchestra.yaml
```

**The companies have the prefix "comp" — for the cryptographic keys and the inputs**!**

**The analyst has to have the name "analyst"! The name of the files has to match as well!**

`orchestra.yaml` has the following format:

"`yaml
server_host: teebench.xyz   # Host of server - make sure to have your /etc/hosts set correctly!
server_http: 8080           # Http port of server
server_https: 8443          # Https port of server

exec_server: true           # Whether to start the server (for local deployment) or for evaluation: only the clients live on the system while the server is isolated in the enclave.

rounds: 10                  # Number of rounds to run the eval
```

## Start orchestra 🎵

An example is in this directory.
Please use the following command(s) to execute it:

"`bash
cargo run inputs/ToEval/test_orchestra.zip Enclave # In hardware mode using Intel SGX
cargo run inputs/ToEval/test_orchestra.zip Unencrypted # In case simulation mode is used or the server is executed without docker 
```

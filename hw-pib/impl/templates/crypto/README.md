# Cryptography

Since the project heavily relies on cryptography, the generation and usage of the techniques should be explained in the following.
All clients (including the analyst) have an asymmetric key pair to identify themselves and guarantee confidentiality and integrity.
In practice, it is to be assumed, that these keys already exist however, signing is still required to perform access control.

## Scheme of keys and their use

We will show how to use the key generation script to set up the keys and deploy them in the orchestra or for any other scenario.
The general structure for the key generation in short detail:
- The analyst has/creates a Certificate Authority (CA).
- The analyst has his access key (self-generated) and signs it with *his own* CA-certificate.
- The analyst uses the CA-certificate to sign the company certificates of the companies that wish to participate in the benchmark.
    - This requires an information exchange on the PKI which is rather trivial and thus abstracted here.
- The analyst *uploads his CA-certificate* to the server in the setup phase and allows all participants with signed keys to connect through client authentication.
    - **Note**: the analyst can also sign the companies' keys later on as long as the final CA-certificate matches the signing key which is uploaded to the server.
- Companies connect to the server with their *CA-signed certificates* and get access to the server.
    - **Note**: the server stores the certificate fingerprint of each participant and does not allow resetting a connection!
- Since the server uses a self-signed and generated key, *all clients* require to put the certificate into their **system** or user trust store**.
    - They can verify the genuineness of the server to verify that the key generation was indeed secure, and no party can decrypt the traffic.

## Usage of `TEEBench Cert Gen`

To generate keys use the `TEEBench Cert Gen` utility found in this template subdirectory.
The `generate.sh` script generates either CA certificates or CA-signed certificates where the CA is provided.
Generating TLS Certificates for the server when setting it up manually is nothing more than a self-signed CA-certificate and key.

```
[Example CA]:                   ./generate_certs.sh --ca=true --keylength=2048 --root_ca=test_ca --dns_name=teebench.xyz --validity=365 --verbose # generates a root CA
[Example Client w/ CFGs]:       ./generate_certs.sh --client=true --keylength=2048 --client_name=comp05 --root_ca=test_ca --client_cert_ext=client_cert_ext.cnf --csr_config=csr.conf --validity=365 --dns_name=teebench.xyz --verbose
[Example Client w/o CFG]:       ./generate_certs.sh --client=true --keylength=8192 --client_name=test_client --root_ca=test_ca --validity=365 --dns_name=teebench.xyz --verbose
```

### Attention!

To use the certificates you need to add them to your system trust store (as root) or your local trust store (i.e., in Chromium & Evolution / Firefox)!

### CLI Arguments

Several options are provided:


|`Argument`|Explanation|
|----------|-----------|
|`--ca=true`                        | Generate a root certificate.|
|`--client=true`                    | Generate a client certificate.|
|`--keylength=Y`                    | The key length for `openssl` to create the certificate. Standard is 2048 bits.|
|`--root-ca=root_ca_name`           | Generate a root CA certificate which has a predefined standard length. The certificate is automatically self-signed and creates `root_ca_name.key` as private key and `root_ca_name.pem` as certificate file. |
|`--client_name=client_cert_name`   | This creates a client private key and certificate and a `.pfx` file for importing the keys to HTTPS capable clients. |
|`--client_cert_ext=cfg_path`       | Give a config path to create a CSR. Standard is a file similar to `client_cert_ext.cnf`.|
|`--csr_config=csr.conf`            | Signing request format. This is used in case additional options are set. In this case the `--dns_name` option is not used.|
|`--dns_name=teebench.xyz`          | Specify the Subject alternative name (SAN). *This is particularly important for secure connections that use DNS!*|
|`--validity`                       | Specify the validity of the certificate in days.|
|`--verbose`                        | Output all set parameters.|
|`--help`                           | Display help text.|

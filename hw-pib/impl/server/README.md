# Server for Benchmarking application

## Key generation for own setup

To quickly generate a set of keys for a testing run `make` certs`. You get the following certificates.
- Enclave certificate (self-signed)
- Root CA certificate (self-signed)  
- Server CA certificate (self-signed)
- Analyst and Company Certificates (signed by Root CA certificate)

Additionally, you can use the `generate_certs.sh` script to generate certificates at your own will. You can see the options with `generate_certs.sh --help`.

[See here for more information on key generation](./file_templates/key_files/generate.md)

## Executing the `run-inf.sh` script

To execute the server infinitely (restarts after shutdown) you can execute `./run-inf.sh`.
If you want to limit the number of server restarts, you can set an environment variable:

```bash
export ITERATIONS=10; # 10 restarts -- after that the server does not restart
./run-inf.sh
```


### Server Functionality

The server is built up on many modules that allow easy maintainability of the server.
It also shares some common parts with the client for easier benchmarking. 
Especially for evaluation purposes where the native performance is compared to the TEE and to the HE, this is especially interesting.

#### Protocols

The server uses HTTP to allow configuration and maintainability.
It makes use of HTTP and HTTPS but in two different time-relevant steps.

#### HTTP -- configuration

When the server starts up, it is initially **not configured**.
This state is used to boot it up in the enclave without the necessity to provide parameters.

We assume the DNS domain of the server to be `teebench.xyz.` and the HTTP port to be `8443` (standard).

The server provides 3 basic routes that are described below:


|Route endpoint | HTTP-Method | Parameters | Description |
|---------------|-------------|------------|-------------|
| `/whoami`     | `GET`       | `None`     | The server returns information on the client that tries to connect i.e., his certificate, the SNI hostname and connection information. |
| `/api/attest` | `GET`       | `None`     | The server returns his current configuration (empty if any) and his certificate that is extracted from the enclave internally. The returned certificate can be used to verify that the server is indeed genuine. |
| `/api/setup`  | `POST`      | `AnalystCARootCert, Configuration, AnalystCertificate` | The analyst (who usually starts the server on his own) uploads his CA root Certificate, the configuration and his own certificate to the server. He is required to do so for his own certificate, since the server cannot check in HTTP mode, which certificate is used. This enables easier access mechanism, since the initial configuration is crucial for the security of the server. |

When the setup procedure is performed, the HTTP server automatically shuts down and starts an HTTPS server which is configured with the `AnalystCARootCert` that got previously uploaded.
Additionally, the server is configured to enforce client authentication, which means that clients have to configure a client certificate and private key to contact the server.
Only when the certificate is signed by the `AnalystCARootCert` the connection is even possible.
From there on the server is further configurable to receive confidential information, since TLS allows for it.


#### Server configuration format

The server configuration format is in YAML syntax and requires all the fields described below:

```yaml
name: TEEBench Server           # Name of the server for clients to identify it.
description: Fancy Server       # Some message of the day.
k_anonymity: 1                  # The number of participants that are required 
                                # before the benchmark can start.
eval_mode: false                # Whether evaluation is used (only for testing)
offload: [ ]                    # Which operations are offloaded during 
                                # evaluation (no effect - only for testing)
```

#### HTTPS -- Benchmarking application

The server is now able to create TLS connections that are trustworthy.
Thus, the analyst and the clients can upload their data to it in this state.

#### Roles and Request verification

It is important to point out that by using TLS, the server verifies the requests which reach it and checks for roles, where at least one must be satisfied:

- `Analyst`: Only the analyst can access this route.
- `Company`: Only a _registered_ company can access this route.
- `Any`: Any company, that is *enrolled but not registered* (no `UUID`) and the analyst can access this route.

The routes for the HTTPS application server in short:

|Route endpoint | HTTP-Method | Parameters | Access Role | Description |
|---------------|-------------|------------|-------------|-------------|
| **CONFIG**||||
| `/whoami`     | `GET`       | `None`     | `Any`       | Exactly the same as for HTTP. |
| `/api/attest` | `GET`       | `None`     | `Any`       | Exactly the same as for HTTP. |
| `/api/setup`  | `POST`      | `AnalystCARootCert, Configuration, AnalystCertificate` | `Analyst` | This endpoint exists but disallows modification. It has no use other than reporting, that the server is configured. |
| `/api/events`| `GET`     | `None`     | `Any`   | Functionality to enroll in the server event stream. Here the server posts information on the progress of the benchmark, and how many participants are ready. |
| **COMPANIES**||||
| `/api/company/register/{id}`      | `POST`     | `id`     | `Any`   | One company registers with a provided UUID. This stores the used certificate which is mandatory for succeeding requests. |
| `/api/company/input_data/{id}`     | `POST`     | `Company_data`     | `Company`   | The company uploads its data. The server checks for all required variables from the analysts algorithms are present. Otherwise, the upload is rejected.  |
| `/api/company/input_data/{id}`     | `GET`     | `None`     | `Company`   | The company can verify that the uploaded data is *correct*. |
| `/api/company/input_data/{id}`     | `PUT`     | `Company_data`     | `Company`   | The company can modify that the uploaded data in case changes are required. |
| `/api/company/results/{id}`| `POST`     | `None`     | `Company`   | After a benchmarking process is complete the companies can retrieve their results. |
| **ANALYST**||||
| `/api/analyst/benchmark_config`| `PUT`     | `Configuartion`     | `Analyst`   | Functionality to modify the configuration of the server from [above](#server-configuration-format) afterwards. |
| `/api/analyst/benchmark_config`| `GET`     | `None`     | `Analyst`   | Functionality to retrieve the configuration (similar to setup but only limited to the config). |
| `/api/analyst/company/{id}`| `GET`     | `None`     | `Analyst`   | Functionality to check whether a specific company registered (certificate) and the company data is uploaded. |
| `/api/analyst/enroll_company`| `POST`     | `None`     | `Analyst`   | Functionality to enroll a company. This returns a 128-bit `UUID` for a company. |
| `/api/analyst/algorithms`| `GET`     | `None`     | `Analyst`   | Functionality to get the uploaded algorithms if they are already uploaded. |
| `/api/analyst/algorithms`| `POST`     | `Algorithms`     | `Analyst`   | Functionality to upload algorithms. Invalid uploads are rejected (i.e., circular dependencies or malformed input). |
| `/api/analyst/algorithms`| `PUT`     | `None`     | `Analyst`   | Functionality to modify the algorithms. Again the checks for integrity are performed. |
| `/api/analyst/benchmark`| `POST`     | `None`     | `Analyst`   | Functionality start benchmarking of companies. This process computes all KPIs that the analyst has provided in his algorithms. Events on the progress are shared over the event stream. |
| `/api/analyst/event`| `POST`     | `Message`     | `Analyst`   | Functionality to broadcast a message over the server's event stream. |


This enables the analyst to…

1. Upload his algorithms which are required before any other operation is possible.
2. Register the number of participants that were previously defined by $k$-anonymity.
3. Wait for the companies to upload their data.
4. Finally, start the benchmarking process when all participants are ready.

The registration of the companies is initiated by the analyst and the resulting `UUID` is transmitted to the individual participants.
In order for the companies to verify that all other participants are indeed genuine and not created by the analyst with the intent to gain information on the individual companies, the list of participants requires to be set out before.
Otherwise, it is **not** possible to rule Sybil attacks out, other than guessing from the configuration which is not guaranteed to provide meaningful data.

For the first connection of the client, the `UUID` which is generated from the server is placed in the HTTP header of the client.
The registration stores the used certificate of the request which is used from thereon to authenticate the client.
This mapping is **permanent** for the server instance and **cannot be modified**.
# Rust webservice example

## Setup
0) Run `make certs`
0) Run `make docker-build`
0) Run `docker-compose -f docker/docker-compose.yml up -d`
0) Run `curl -kX GET https://localhost:8080/data`
0) Profit.

## Environment variables

|Variable|Usage|
|-- |-- |
|SVC_KEY_FILE|Path to private key file to use for SSL.|
|SVC_CERT_FILE|Path to certificate file to use for SSL.|
|SVC_BIND_ADDRESS|The address to bind the service onto. Looks like "`0.0.0.0:8080`"|
|SVC_NUM_WORKERS|Number of ACTIX workers.|
|SVC_SHUTDOWN_TIMEOUT|Shutdown timeout of the ACTIX server.|

#!/bin/bash

echo "########################";
echo "###     TEEBench     ###";
echo "########################";
echo "";
echo "> Benchmarking on Trusted Execution Environments";
echo "";
echo "=== Starting REST API ===";

# Debug print keys
# cat /usr/src/teebench/templates/crypto/server/server.pem;
# cat /usr/src/teebench/templates/crypto/server/server.key;

# Set env vars for scone
SCONE_STACK=4M SCONE_HEAP=6G SCONE_QUEUES=8 SCONE_SLOTS=512 SCONE_SIGPIPE=1 SCONE_ALLOW_DLOPEN=yes SCONE_MODE=HW SCONE_VERSION=1 SCONE_ALLOW_DLOPEN=0

# Create directory
mkdir -p ../data/server_data

while true; do
   SCONE_STACK=4M SCONE_HEAP=6G SCONE_QUEUES=8 SCONE_SLOTS=512 SCONE_SIGPIPE=1 SCONE_ALLOW_DLOPEN=yes SCONE_MODE=HW SCONE_VERSION=1 SCONE_ALLOW_DLOPEN=0 server && wait;
   rm -rf ../data/server_data/*;
done
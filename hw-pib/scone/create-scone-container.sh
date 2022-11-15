#!/bin/bash

#####################################
# TEEBench scone container creation #
#####################################

# > This file provides the scriping to
# > enable automated Docker containers
# > and attestation to SCONE CAS.

#####################################


#####################
# 0. What this does #
#####################

# 1. Create a sconified image which is based on a vanilla Dockerfile, i.e., creates a new docker container similar to the vanilla one with encrypted input files and program
# 2. The defined policy from "teebench-policy.yaml" is pushed as a new session to the CAS instance
# 3. The created session is given as a result file: "teebench-session.yaml"


#######################
# 1. Preconfiguration #
#######################

# show what we do (-x), export all varialbes (-a), and abort of first error (-e)
set -x -a -e
trap "echo Unexpected error! Could not set output mode; exit 1" ERR

# CONFIG Parameters
# - The Image name: teebench
# - The public SCONE_CAS address for the policy: scone-cas.cf
# - The CAS MRENCLAVE is provided by SCONE to authenticate their CAS service
# - The used device that provides SGX functionalities: /dev/(i)sgx

export IMAGE=${IMAGE:-teebench_server_image}
export SCONE_CAS_ADDR="scone-cas.cf"
export CAS_MRENCLAVE="3061b9feb7fa67f3815336a085f629a13f04b0a1667c93b14ff35581dc8271e4"
export DEVICE="/dev/isgx"

# IMAGES for Container Creation
# - CLI image for uploading the session file to CAS
# - Create random and unique session number

export CLI_IMAGE="registry.scontain.com:5050/sconecuratedimages/kubernetes:hello-k8s-scone0.1"
export TEEBENCH_MRENCLAVE="4f1717be9b801834df82b8f9f483050c862d620b28e2d2e0bd727d23abc3c77d"
TEEBENCH_SESSION="TEEBench-$RANDOM-$RANDOM-$RANDOM"


######################
# 2. CAS Attestation #
######################

# Ensure that we have an up-to-date image for uploading to CAS
docker pull $CLI_IMAGE

# Check if SGX device exists for CAS uploading
# Only if "SCONE_HW environment variable is set"
if [[ -t "$SCONE_HW" ]]; then
    if [[ ! -c "$DEVICE" ]] ; then
        export DEVICE_O="DEVICE"
        export DEVICE="/dev/isgx"
        if [[ ! -c "$DEVICE" ]] ; then
            echo "Neither $DEVICE_O nor $DEVICE exist"
            exit 1
        fi
    fi
fi

# Attest CAS before uploading the session file
# > This script uses the public CAS:
#   - It is running in debug mode [-d] with an outdated TCB [-G]
if [[ -t "$SCONE_HW" ]]; then 
    docker run --device=$DEVICE -it $CLI_IMAGE sh -c "
    scone cas attest -G --only_for_testing-debug  $SCONE_CAS_ADDR $CAS_MRENCLAVE >/dev/null \
    &&  scone cas show-certificate" > cas-ca.pem
else 
    docker run -e "SCONE_MODE=SIM" -it $CLI_IMAGE sh -c "
    scone cas attest -G --only_for_testing-ignore-signer --only_for_testing-debug  $SCONE_CAS_ADDR $CAS_MRENCLAVE >/dev/null \
    &&  scone cas show-certificate" > cas-ca.pem
fi

######################
# 3. Build Container #
######################

# create a image with encrypted service
docker build --pull -t $IMAGE ../

# ensure that we have self-signed client certificate
if [[ ! -f ./keys/client.pem || ! -f ./keys/client-key.pem  ]] ; then
    openssl req -newkey rsa:4096 -days 365 -nodes -x509 -out ./keys/client.pem -keyout ./keys/client-key.pem -config ./keys/clientcertreq.conf
fi

# Create session file from template ("teebench-template.yml")
# We substitute environment vars and fill out the template
# Finally we upload the created session to the SCONE CAS
MRENCLAVE=$TEEBENCH_MRENCLAVE envsubst '$MRENCLAVE $TEEBENCH_SESSION' < teebench-template.yml > teebench-session.yml
# note: this is insecure - use scone session create instead
curl -v -k -s --cert ./keys/client.pem  --key ./keys/client-key.pem  --data-binary @teebench-session.yml -X POST https://$SCONE_CAS_ADDR:8081/session


# Create file with environment variables
# This is used by the script which calls docker-compose with the template config
cat > build_env_vars << EOF
export TEEBENCH_SESSION="$TEEBENCH_SESSION"
export SCONE_CAS_ADDR="$SCONE_CAS_ADDR"
export IMAGE="$IMAGE"
export DEVICE="$DEVICE"
EOF

echo "Build completed - Session uploaded to SCONE CAS."

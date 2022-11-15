source ./build_env_vars;

echo "Get public CACERT from CAS REST API"
echo "When this key is possible to be used to connect - this serves as attestation. See https://sconedocs.github.io/node-example/!";
echo -e $(curl --cacert cas-ca.pem https://$SCONE_CAS_ADDR:8081/v1/values/session=$TEEBENCH_SESSION,secret=teebench_ca_certificate) | head -c -2 | tail -c +2 > /tmp/teebench-ca.crt

# https://gitlab.scontain.com/community/secure-doc-management

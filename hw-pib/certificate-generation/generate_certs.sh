#! /bin/bash

#############
#  GREETER  #
#############

greeter(){
echo  "______  ____  ____ ____   ____ __  __   ___ __  __      ___  ____ ____  ______      ___   ____ __  __"
echo  "| || | ||    ||    || )) ||    ||\ ||  //   ||  ||     //   ||    || \\\\ | || |     // \\\\ ||    ||\ ||"
echo  "  ||   ||==  ||==  ||=)  ||==  ||\\\\|| ((    ||==||    ((    ||==  ||_//   ||      (( ___ ||==  ||\\\\||"
echo  "  ||   ||___ ||___ ||_)) ||___ || \||  \\\\__ ||  ||     \\\\__ ||___ || \\\\   ||       \\\\_|| ||___ || \||"
echo  ""
}

#############
#    MAIN   #
#############
# - Start of greeter
# - eval of input
# - generating certificates
# - running the pipeline

main() {
    greeter;
    evaluate_input "$@";
    generate_certificate;
}

##########################################
#   GENERATION INPUT FORMAT AND OPTIONS  #
##########################################
# - Several options are provided:
#   - [--ca] → Generate a root cert.
#   - [--client] → Generate a client cert.
#   - [--keylength=Y] → The keylength for
#     openssl to create the cert. Standard
#     is 2048 bits.
#   - [--ca_name=root_ca_name] → Generate a 
#     root CA cert which has a predefined 
#     standard length. The certificate is
#     automatically self-signed and creates
#     root_ca_name.key as private key and
#     root_ca_name.pem as certificate file.
#     Furthermore is there a 
#     root_ca_name_esc.pem which has the 
#     newlines escaped.
#   - [--client_name=client_cert_name] → 
#     This creates a client private key and 
#     certificate and a pfx file for
#     importing the keys to a HTTP capable
#     client. 
#   - [--client_cert_ext=cfg_path] → Give
#     a config path to the create a CSR. 
#     Standard is a file similar to 
#     `client_cert_ext.cnf`.
#   - [--verbose] → Output all set params.
#   - [--help] → Display help text.

show_help(){
    echo -e "\nThis script generates certificates.\n - Either a CA certificate or CA-signed certificates:\n - CA-signed certificates require an existing CA certificate!\n - TLS Certificates as they are nothing more then a self-signed CA-certificate and key.\n\n [Example CA]:              ./generate_certs.sh --ca=true --keylength=2048 --root_ca=test_ca --dns_name=teebench.xyz --validity=365 --verbose # generates a root CA\n [Example Client w/ CFGs]:  ./generate_certs.sh --client=true --keylength=2048 --client_name=comp05 --root_ca=test_ca --client_cert_ext=client_cert_ext.cnf --csr_config=csr.conf --validity=365 --dns_name=teebench.xyz --verbose\n [Example Client w/o CFG]: ./generate_certs.sh --client=true --keylength=8192 --client_name=test_client --root_ca=test_ca --validity=365 --dns_name=teebench.xyz --verbose\n[WARN] To use the certificates you need to add them to your system trust store (as root) or to your local trust store (i.e. in Chromium & Evolution / Firefox)!";
    exit 0;
}

#############
#   INPUT   #
#############
# - We check all input parameters and
#   set them accordingly.
# - We also set default params and check
#   for verbose output. 

evaluate_input() {
    # Switch case over the input params
    while [[ $# -gt 0 ]] && [[ "$1" == "--"* ]] ;
    do
        opt="$1";
        shift;              #Shift (get next argument) after each arg 
        case "$opt" in
            "--" ) break 2 # Terminate check
            ;;
            "--ca="* )
            GEN_CA=true;
            ;;
            "--client="* )
            GEN_CA=false;
            ;;
            "--keylength="* )
            KEYLENGTH="${opt#*=}"
            ;;
            "--root_ca="* )
            ROOT_CA_NAME="${opt#*=}"
            ;;
            "--client_name="* )
            CLIENT_NAME="${opt#*=}"
            ;;
            "--dns_name="* )
            DNS_NAME="${opt#*=}"
            ;;
            "--csr_config="* )
            CLIENT_CSR_PATH="${opt#*=}"
            ;; 
            "--validity="* )
            VALIDITY="${opt#*=}"
            ;;
            "--client_cert_ext="* )
            CLIENT_CERT_EXT_PATH="${opt#*=}"
            ;;
            "--verbose" )
            VERBOSE=true;
            shift;;
            "--help" )
            show_help;
            break 2;;
            *) 
            echo >&2 "Invalid option: $@. See --help for usage."; exit 1;;
        esac
    done

    # Set unset arguments
    set_default_parameters;
    # Verbose output if verbose is set and true
    if [[ $VERBOSE == true ]]; then print_verbose; fi
}

#############
#  DEFAULTS #
#############
# - KEYLENGTH = 2048
# - VERBOSE = false

set_default_parameters(){
    
    # Exit when no cert type was configured
    if [[ -z $GEN_CA ]]; then 
        echo "Use either --ca or --client to generate certificates"
        exit -1;
    fi

    # Exit when no cert type was configured
    if [[ -z $DNS_NAME ]]; then 
        echo "[WARNING] Not using DNS hinders use of Certificate's Subject Alternative Name! You probably don't want that, as you will run into 'SSL_ERROR_BAD_CERT_DOMAIN'. Aborting..";
        exit -1;
    fi

    # Keylength
    if [[ -z $KEYLENGTH ]]; then KEYLENGTH=2048; fi

    # Validty
    if [[ -z $VALIDITY ]]; then VALIDITY=365; fi
    
    # Verbose Mode toggle 
    if [[ -z $VERBOSE ]]; then VERBOSE=false; fi
}

print_verbose() {
    echo -e "\n##############\n Selected options:\n##############\n";
    echo "Generate CA       : $GEN_CA";
    echo "Keylength         : $KEYLENGTH";
    echo "Root CA Name      : $ROOT_CA_NAME";
    echo "Client Name       : $CLIENT_NAME";
    echo "DNS Name          : $DNS_NAME";
    echo "CSR Cofig Path    : $CLIENT_CSR_PATH";
    echo "Client Config Path: $CLIENT_CERT_EXT_PATH";
    echo "Validity          : $VALIDITY";
    echo "Verbose Output    : $VERBOSE";
    echo -e "\n##############";
}

##########################################
#         CERTIFICATE GENERATION         #
##########################################
# - Check first whether CA or client cert is used
# - On  GEN_CA: Generate only certificate where
#   the name is the only necessity.
# - On ~GEN_CA: Generate client certificates from
#   an existing CA certificate for signing. If
#   the CA private key is not given error out. 

generate_certificate() {
    # Generation 
    if [[ $GEN_CA == true ]]; then
        generate_ca_cert;
    else
        generate_client_cert;
    fi
}

generate_ca_cert() {
    
    echo "[1/2] Create Directory for root CA";
    # Create a separate directory
    mkdir $ROOT_CA_NAME;
    cd $ROOT_CA_NAME;
    
    echo "[2/2] Create private key of root CA and self sign it";
    # Generate the keypair and self sign it at once
    openssl req -x509 -sha256 -days $VALIDITY -nodes -newkey rsa:$KEYLENGTH -subj "/CN=Teebench Dev Certificate/C=DE/L=NRW" -keyout $ROOT_CA_NAME.key -out $ROOT_CA_NAME.pem

    # If build fails we abort run
    if [[ $? -ne 0 ]]; then 
        echo "Generation failed!";
        exit -1; 
    fi
    cd ..;
}

generate_client_cert() {
    
    echo "[1/7] Creating Directory for client certificate";
    # Create a separate directory
    mkdir $CLIENT_NAME;
    cd $CLIENT_NAME;


    echo "[2/7] Generating RSA key pair...";
    # Generate the private key
    openssl genrsa -out $CLIENT_NAME.key $KEYLENGTH


    echo "[3/7] Creating certificate signing request if not provided";
    if [[ -z $CLIENT_CSR_PATH ]]; then
        # Create std CSR
        cat > csr.conf <<EOF

[ req ]
default_bits = 2048
prompt = no
default_md = sha256
req_extensions = req_ext
distinguished_name = dn

[ dn ]
C = DE
ST = Aachen
L = NRW
O = Teebench
OU = Teebench Dev
CN = teebench.xyz

[ req_ext ]
subjectAltName = @alt_names

[ alt_names ]
DNS.1 = $DNS_NAME
IP.1 = 127.0.0.1

EOF

        CLIENT_CSR_PATH=csr.conf;
    else
        CLIENT_CSR_PATH = ../$CLIENT_CSR_PATH;
    fi

    echo "[4/7] Create signing request for root CA";
    openssl req -new -key $CLIENT_NAME.key -out $CLIENT_NAME.csr -config csr.conf;


    echo "[5/7] Creating certificate extension constraints if not provided";
    if [[ -z $CLIENT_CERT_EXT_PATH ]]; then 
        # Create file and put it in current dir
        cat >client_cert_ext.cnf <<EOF

basicConstraints = CA:FALSE
nsCertType = server, client, email
nsComment = "OpenSSL Generated Client Certificate"
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer
keyUsage = critical, nonRepudiation, digitalSignature, keyEncipherment
extendedKeyUsage = clientAuth, emailProtection, serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = $DNS_NAME

EOF
        CLIENT_CERT_EXT_PATH=client_cert_ext.cnf; 
    else
        CLIENT_CERT_EXT_PATH=../$CLIENT_CERT_EXT_PATH;
    fi 

    echo "[6/7] Generate client private key";

    openssl x509 -req -in $CLIENT_NAME.csr -CA ../${ROOT_CA_NAME}/${ROOT_CA_NAME}.pem -CAkey ../${ROOT_CA_NAME}/${ROOT_CA_NAME}.key -CAcreateserial -out $CLIENT_NAME.pem -days $VALIDITY -sha256 -extfile $CLIENT_CERT_EXT_PATH

    echo "[7/7] Create PFX file for browser usage"
    # Create pfx for client
    openssl pkcs12 -export -in ${CLIENT_NAME}.pem -inkey ${CLIENT_NAME}.key -out ${CLIENT_NAME}.pfx -keypbe NONE -passout pass: &> /dev/null

    # If build fails we abort run
    if [[ $? -ne 0 ]]; then 
        echo "Generation failed!";
        exit -1; 
    fi
    cd ..;

    # Warn myself because I am stupid
    echo "[INFO] Client Certificate created and signed! DO NOT FORGET TO ADD THE ROOT_CA TO YOUR TRUSTED STORES!!!";
}

# Entry point of program
main "$@";

use std::path::Path;

use rustls::{Certificate, PrivateKey};
use types::consts::{SERVER_CRYPTO_PATH, SERVER_SERVER_CRYPTO, EXT_CERTIFICATE, EXT_PRIVATE_KEY, SERVER_ENCLAVE_CRYPTO};
use x509_certificate::CapturedX509Certificate;
use server_util::crypto_decode::{parse_tls_private_key_from_path, parse_x509_certificate_from_path, parse_tls_certificates_from_path};

#[derive(Debug, Clone)]
pub struct Crypto {
    /// TLS server certificate
    server_certificate: Vec<Certificate>,
    /// TLS private key 
    server_private_key: PrivateKey,
    /// Analysts's CA root certificate for accepting 
    /// companies' signed certificates
    root_ca_certificate: Option<Certificate>,
    /// Enclave certificate for attestation
    enclave_certificate: CapturedX509Certificate
}

impl Crypto {
    pub fn load() -> Self {
        let server_certificate_path = Path::new(SERVER_CRYPTO_PATH).join(SERVER_SERVER_CRYPTO).with_extension(EXT_CERTIFICATE);
        let server_private_key_path = Path::new(SERVER_CRYPTO_PATH).join(SERVER_SERVER_CRYPTO).with_extension(EXT_PRIVATE_KEY);
        let enclave_certificate_path = Path::new(SERVER_CRYPTO_PATH).join(SERVER_ENCLAVE_CRYPTO).with_extension(EXT_CERTIFICATE);

        let server_certificate = parse_tls_certificates_from_path(&server_certificate_path).expect("[FATAL] Could not extract TLS Certificate");
        let server_private_key = parse_tls_private_key_from_path(&server_private_key_path).expect("[FATAL] Could not extract TLS Private Key");
        let enclave_certificate = parse_x509_certificate_from_path(&enclave_certificate_path).expect("[FATAL] Could not extract Enclave Certificate");

        let crypto =  Crypto { server_certificate, server_private_key, root_ca_certificate: None, enclave_certificate };

        crypto
    }

    pub fn server_certificate(&self) -> &Vec<Certificate> {
        &self.server_certificate
    } 

    pub fn server_private_key(&self) -> &PrivateKey {
        &self.server_private_key
    } 

    pub fn enclave_certificate(&self) -> &CapturedX509Certificate {
        &self.enclave_certificate
    } 

    pub fn root_ca_certificate(&self) -> &Option<Certificate> {
        &self.root_ca_certificate
    } 

    /// Only the root_ca_certificate is modifiable

    pub fn set_root_ca_certificate(&mut self, root_ca: Certificate) {
        self.root_ca_certificate = Some(root_ca);
    } 
}
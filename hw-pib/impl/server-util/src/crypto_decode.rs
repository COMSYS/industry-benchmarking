//! TLS and X509 Certificate parsers
//! 
//! These functions do parsing of PEM (base64 encoded) Certificates
//! and private keys and return an internal representation (DER: 
//! Distinguished Encoding Rules) that allows further processing with it.

use std::{path::{Path, PathBuf}, io::{BufReader, Read}, fs::File};
use rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, rsa_private_keys};
use stringreader::StringReader;
use x509_certificate::CapturedX509Certificate;

//TODO: REMOVE THE X509 dependency

/// Parse PEM encoded certificate file and return a DER encoded result (Bytecode)
/// This function gets a `&str` slice and returns 
/// **the first certificate** in the chain it decoded
/// even if a whole cert chain is in the file. 
pub fn parse_tls_certificate_from_path(path: &PathBuf) -> Result<Certificate, Box<dyn std::error::Error>> {   
    // We received at least one certificate and use it
    Ok(parse_tls_certificates_from_path(path).unwrap()[0].clone())
}

pub fn parse_tls_certificates_from_path(path: &PathBuf) -> Result<Vec<Certificate>, Box<dyn std::error::Error>> {   
    // Check whether file exists
    if !Path::new(path).is_file() {
        return Err("Error: path given is not a regular file, please update to point to a certificate.".into());
    }

    // Read provided file if exists
    let cert_reader = &mut BufReader::new(File::open(path)?);

    let cert_chain: Vec<Certificate> = certs(cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    // Check the number of received certificates and add them
    match cert_chain.len() {
        0 => {
            // The certificate could not be parsed successfully
            Err("Malformed Certificate file".into())
        },
        _ => {
            Ok(cert_chain)
        }
    }
}

/// Parse PEM private Key to DER format
/// Similar to the certificate, only the first entry is returned
pub fn parse_tls_private_key_from_path(path: &PathBuf) -> Result<PrivateKey, Box<dyn std::error::Error>> {   
    // Check whether file exists
    if !Path::new(path).is_file() {
        return Err("Error: path given is not a regular file, please update to point to a private key.".into());
    }

    // Read provided file if exists
    let priv_key_reader = &mut BufReader::new(File::open(path)?);

    let key_chain: Vec<PrivateKey> = rsa_private_keys(priv_key_reader)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // Check the number of received certificates and add them
    match key_chain.len() {
        0 => { Err("Malformed Certificate file".into()) },
        _ => { Ok(key_chain[0].clone()) }
    }
}

/// Same as above only that you have a string instead of a file
pub fn parse_x509_certificate_from_path(path: &PathBuf) -> Result<CapturedX509Certificate, Box<dyn std::error::Error>> {   
    // Check whether file exists
    if !Path::new(path).is_file() {
        return Err("Error: path given is not a regular file, please update to point to a certificate.".into());
    }

    // Read provided file if exists
    let cert_reader = &mut BufReader::new(File::open(path)?);
    let mut der = Vec::new();
    cert_reader.read_to_end(&mut der)?;

    // NEW PUBLIC KEY
    Ok(CapturedX509Certificate::from_pem(der.to_vec())?)
}

/// Same as above only that you have a string instead of a file
pub fn parse_tls_certificate_from_string(input: &str) -> Result<Certificate, Box<dyn std::error::Error>> {   

    // Decompose string to buffered reader s.t. it can be parsed like a file
    let streader = StringReader::new(input);
    let mut cert_reader = BufReader::new(streader);

    let cert_chain: Vec<Certificate> = certs(&mut cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    // Check the number of received certificates and add them
    match cert_chain.len() {
        0 => {
            // The certificate could not be parsed successfully
            Err("Malformed Certificate file".into())
        },
        _ => {
            // We received at least one certificate and use it
            Ok(cert_chain[0].clone())
        }
    }
}


#[cfg(test)]
mod test {
    use x509_certificate::CapturedX509Certificate;

    use super::*;
    
    /// Tests for private keys

    #[test]
    #[should_panic]
    pub fn decode_missing_private_key() {
        let pb = &Path::new("").to_path_buf();
        parse_tls_private_key_from_path(pb).unwrap();
    }

    #[test]
    #[should_panic]
    pub fn decode_malformed_private_key() {
        // Use certificate to force error
        let pb = &Path::new("test/files/keys/rsa.pem").to_path_buf();
        parse_tls_private_key_from_path(pb).unwrap();
    }

    #[test]
    pub fn decode_ok_private_key() {
        let pb = &Path::new("test/files/keys/rsa.key").to_path_buf();
        assert!(parse_tls_private_key_from_path(pb).is_ok());
    }

    /// Tests for certs 

    #[test]
    #[should_panic]
    pub fn decode_missing_certificate() {
        let pb = &Path::new("").to_path_buf();
        parse_tls_certificate_from_path(pb).unwrap();
    }

    #[test]
    #[should_panic]
    pub fn decode_malformed_certificate() {
        let pb = &Path::new("test/files/keys/rsa.key").to_path_buf();
        parse_tls_certificate_from_path(pb).unwrap();
    }

    #[test]
    pub fn decode_ok_certificate() {
        // Use certificate to force error
        let pb = &Path::new("test/files/keys/rsa.pem").to_path_buf();
        assert!(parse_tls_certificate_from_path(pb).is_ok());
    }

    #[test]
    pub fn decode_x509_certificate_from_string_with_escapes() {
        assert!(CapturedX509Certificate::from_pem("-----BEGIN CERTIFICATE-----MIIFETCCAvkCFDj3NWKaJfO/WSPAgtdnU48lQJOyMA0GCSqGSIb3DQEBCwUAMEUxCzAJBgNVBAYTAkFVMRMwEQYDVQQIDApTb21lLVN0YXRlMSEwHwYDVQQKDBhJbnRlcm5ldCBXaWRnaXRzIFB0eSBMdGQwHhcNMjIwNTIyMDc0ODE5WhcNMjMwNTIyMDc0ODE5WjBFMQswCQYDVQQGEwJBVTETMBEGA1UECAwKU29tZS1TdGF0ZTEhMB8GA1UECgwYSW50ZXJuZXQgV2lkZ2l0cyBQdHkgTHRkMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA1WJhk83ZieOPR2To4AcFUr4hgnKieJcma/s9j6MJOt6AUeuNngbq0Dy7R6F0EZwZ+6d83BitZGYVYnEIBI6EVvTXP9a0n+JXBNEoxCxtffLj6l4y4y73L6bBHzwhpfbQTBZgbSm1IPH7g8XmYW694GXUWvIG1ulrjNHYRMDybkBSwVwH1Kqt6y6xaZyIg43MysWa8oUVXu0Ng54Ha+iLTb56GYaEe/1yV9f7EyBsX/RrBIubFYxt7Zw+LSbolThbAmBUhBbjZdlHZVR7Qn9c5iKIAheIH1pAdq9+BGLzsNmTosIGIcCpyttE1WycNvStR1d7tLX1FriJPTOkTkVox9jZiNCrjeuVJjQA4x3IzCfuYBENA+OzoVQLfqsP3MmMsCgQwvSH3QDCWBWOBjs4UGAaMBFyQFiyS822QebEbgP+HpEg5iLO597C84AHnqFKqE2PiwKkr8wMhLvMXx8Uzcwl4AkUSXV0fX2pZcFmTH6Uy6vK87iyvNsi6Uudnb3ZQbJ9Ru6Vu50fnodR32QWIzuYpRI+ucbrHX20XcouNRDaFFJO7vADmBd/Dw7ez0hH/755hV3s1LtOZ/bHRPwDVKhzrdQSOsSWATUIj5mMIdC/KY0Q1vz6QHFLeTZgJt0rBH7/0sCM3o+9O16sAfi7FqEO4q/IUhNHUTyJA5LrMrUCAwEAATANBgkqhkiG9w0BAQsFAAOCAgEAjjbB1VZOmARTOfH2m5fkHvbyVVikh2MXvh0M9HoO4UImXSo6B1YFWnrEygBY5LCCAttf0UuiRPH4b/ah1slMRgAbUSwiVh61ql4CEgukq6/dHNRo/jjF62BHSuWZ2jql1sLzRfMmL2jW/TdWEMULvMAgNfx3DK5qtzSudReyU9mRxiscadfoUgJVBfDmiKfHFUN4faFpxxho3+AoPMOK2jsoK/ekS9JM0B7Ngtzk7mPfvnCC9C4kd39Pk+MwLcwOk7grxofFbkymJyCi1NYQCG7igWYjSazckrWdusMegAkFnw0Qt+UZ2JPsAkPJI0dWiGAxbPXHoyi8Ll2uSjk9EKlSYAsXhGcGvp0whPDa8RLw6i8Iz4CBtr6LMJZH0Q+lqCnh8vbRGgAoB4mETBydJTPibt9YvwMU+dN+ui43O6uXIJoFfQ6WKu0638xmw715B6T5XA0FhehBt16Hc91QGtC/bKhgWsNlEpi1tyfAVzQSftVSQ1xZkhcywVS2BYUH7fuFVxBw91YN+V7crCLJyPcr/2k5jrP2fTNv3qY0cIeEnWFCZpLff8LIANgBduKO9VdLp9GEQlE/z+ubpQdbx2IwBHQVpnsu/z8pKsVmxUPKeGK/cQuqyxB7shp+079Zvph6moXQ4z6kgf2hgaHvR2+5rWZlkZ7WyujGsaP8Xd0=-----END CERTIFICATE-----").unwrap().eq(&CapturedX509Certificate::from_pem("-----BEGIN CERTIFICATE-----\r\nMIIFETCCAvkCFDj3NWKaJfO/WSPAgtdnU48lQJOyMA0GCSqGSIb3DQEBCwUAMEUx\r\nCzAJBgNVBAYTAkFVMRMwEQYDVQQIDApTb21lLVN0YXRlMSEwHwYDVQQKDBhJbnRl\r\ncm5ldCBXaWRnaXRzIFB0eSBMdGQwHhcNMjIwNTIyMDc0ODE5WhcNMjMwNTIyMDc0\r\nODE5WjBFMQswCQYDVQQGEwJBVTETMBEGA1UECAwKU29tZS1TdGF0ZTEhMB8GA1UE\r\nCgwYSW50ZXJuZXQgV2lkZ2l0cyBQdHkgTHRkMIICIjANBgkqhkiG9w0BAQEFAAOC\r\nAg8AMIICCgKCAgEA1WJhk83ZieOPR2To4AcFUr4hgnKieJcma/s9j6MJOt6AUeuN\r\nngbq0Dy7R6F0EZwZ+6d83BitZGYVYnEIBI6EVvTXP9a0n+JXBNEoxCxtffLj6l4y\r\n4y73L6bBHzwhpfbQTBZgbSm1IPH7g8XmYW694GXUWvIG1ulrjNHYRMDybkBSwVwH\r\n1Kqt6y6xaZyIg43MysWa8oUVXu0Ng54Ha+iLTb56GYaEe/1yV9f7EyBsX/RrBIub\r\nFYxt7Zw+LSbolThbAmBUhBbjZdlHZVR7Qn9c5iKIAheIH1pAdq9+BGLzsNmTosIG\r\nIcCpyttE1WycNvStR1d7tLX1FriJPTOkTkVox9jZiNCrjeuVJjQA4x3IzCfuYBEN\r\nA+OzoVQLfqsP3MmMsCgQwvSH3QDCWBWOBjs4UGAaMBFyQFiyS822QebEbgP+HpEg\r\n5iLO597C84AHnqFKqE2PiwKkr8wMhLvMXx8Uzcwl4AkUSXV0fX2pZcFmTH6Uy6vK\r\n87iyvNsi6Uudnb3ZQbJ9Ru6Vu50fnodR32QWIzuYpRI+ucbrHX20XcouNRDaFFJO\r\n7vADmBd/Dw7ez0hH/755hV3s1LtOZ/bHRPwDVKhzrdQSOsSWATUIj5mMIdC/KY0Q\r\n1vz6QHFLeTZgJt0rBH7/0sCM3o+9O16sAfi7FqEO4q/IUhNHUTyJA5LrMrUCAwEA\r\nATANBgkqhkiG9w0BAQsFAAOCAgEAjjbB1VZOmARTOfH2m5fkHvbyVVikh2MXvh0M\r\n9HoO4UImXSo6B1YFWnrEygBY5LCCAttf0UuiRPH4b/ah1slMRgAbUSwiVh61ql4C\r\nEgukq6/dHNRo/jjF62BHSuWZ2jql1sLzRfMmL2jW/TdWEMULvMAgNfx3DK5qtzSu\r\ndReyU9mRxiscadfoUgJVBfDmiKfHFUN4faFpxxho3+AoPMOK2jsoK/ekS9JM0B7N\r\ngtzk7mPfvnCC9C4kd39Pk+MwLcwOk7grxofFbkymJyCi1NYQCG7igWYjSazckrWd\r\nusMegAkFnw0Qt+UZ2JPsAkPJI0dWiGAxbPXHoyi8Ll2uSjk9EKlSYAsXhGcGvp0w\r\nhPDa8RLw6i8Iz4CBtr6LMJZH0Q+lqCnh8vbRGgAoB4mETBydJTPibt9YvwMU+dN+\r\nui43O6uXIJoFfQ6WKu0638xmw715B6T5XA0FhehBt16Hc91QGtC/bKhgWsNlEpi1\r\ntyfAVzQSftVSQ1xZkhcywVS2BYUH7fuFVxBw91YN+V7crCLJyPcr/2k5jrP2fTNv\r\n3qY0cIeEnWFCZpLff8LIANgBduKO9VdLp9GEQlE/z+ubpQdbx2IwBHQVpnsu/z8p\r\nKsVmxUPKeGK/cQuqyxB7shp+079Zvph6moXQ4z6kgf2hgaHvR2+5rWZlkZ7WyujG\r\nsaP8Xd0=\r\n-----END CERTIFICATE-----\r\n").unwrap()));
    }
}
use tokio_rustls::rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ServerConfig,
};

use std::fs::File;
use std::io::{BufReader, Read};


fn load_tls_config() -> ServerConfig {
    let cert_file = &mut BufReader::new(
        File::open("cert/server.crt").unwrap()
    );

    let key_file = &mut BufReader::new(
        File::open("cert/server.key").unwrap()
    );

    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(cert_file)
            .collect::<Result<_, _>>()
            .unwrap();

    let key: PrivateKeyDer<'static> =
        rustls_pemfile::private_key(key_file)
            .unwrap()
            .unwrap();

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap()
}
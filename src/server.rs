use std::fs::File;
use std::io::{BufReader, Read};

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;
use tokio_rustls::rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ServerConfig,
};


pub async fn start_server() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:443").await?;

    let cert = load_tls_config();
    let acceptor = TlsAcceptor::from(Arc::new(cert));

    println!("TLS server listening on 0.0.0.0:443");

    loop {
        let (stream, addr) = listener.accept().await?;

        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(_tls_stream) => {
                    println!("Client connected: {}", addr);

                    // proces client
                }

                Err(e) => {
                    eprintln!("TLS error from {}: {}", addr, e);
                }
            }
        });
    }
}


pub fn load_tls_config() -> ServerConfig {
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
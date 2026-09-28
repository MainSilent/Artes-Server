use std::fs::File;
use std::io::{BufReader, Read};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_rustls::rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ServerConfig,
};


pub async fn start_server(tun_fd: i32) -> tokio::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:443").await?;

    let cert = load_tls_config();
    let acceptor = TlsAcceptor::from(Arc::new(cert));

    println!("TLS server listening on 0.0.0.0:443");

    // for this current version I'm only allowing one user
    let is_already_connected = Arc::new(Semaphore::new(1));

    loop {
        let (stream, addr) = listener.accept().await?;

        let acceptor = acceptor.clone();

        let is_already_connected = is_already_connected.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(mut client) => {
                    // Check if a client already is connected
                    let permit = is_already_connected.try_acquire_owned();
                    if permit.is_err() {
                        println!("Another client is already connected: {}", addr);
                        let _ = client.shutdown().await;
                        return;
                    }
                    let _permit = permit.unwrap();

                    println!("Client connected: {}", addr);

                    let mut buf = [0u8; 4096];

                    loop {
                        match client.read(&mut buf).await {
                            Ok(0) => {
                                println!("Client disconnected: {}", addr);
                                break;
                            }
                            Ok(_) => {
                                // process client data
                            }
                            Err(e) => {
                                eprintln!("Client error {}: {}", addr, e);
                                break;
                            }
                        }
                    }
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
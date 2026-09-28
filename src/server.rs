use std::fs::File;
use std::io::{BufReader, Read};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::{Mutex, Semaphore};
use tokio_rustls::rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    ServerConfig,
};

use crate::auth::authenticate;
use crate::tun::{ tun_write, tun_read };


const BUFFER_SIZE : usize = 4096;


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
                    let is_auth = Arc::new(AtomicBool::new(false));
                    let is_auth_tun = is_auth.clone();
                    
                    // Ack to send username and password
                    client.write_all(&[1u8]).await;

                    println!("Client connected: {}", addr);

                    // Split client into reader and writer
                    let (mut reader, writer) = tokio::io::split(client);
                    let writer = Arc::new(Mutex::new(writer));

                    // TUN to Client
                    let writer_tun = writer.clone();
                    let tun_to_client = tokio::spawn(async move {
                        loop {
                            // Don't read from TUN until client is authenticated
                            if !is_auth_tun.load(Ordering::Acquire) {
                                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                                continue;
                            }

                            // Start Reading from tun and write to client
                            let result = tokio::task::spawn_blocking(move || {
                                let mut buf = [0u8; BUFFER_SIZE];

                                let n = tun_read(tun_fd, &mut buf)?;

                                Ok::<_, std::io::Error>((buf, n))
                            })
                            .await;

                            let (buf, n) = match result {
                                Ok(Ok(data)) => data,

                                Ok(Err(e)) => {
                                    eprintln!("TUN read error: {}", e);
                                    break;
                                }

                                Err(e) => {
                                    eprintln!("TUN task error: {}", e);
                                    break;
                                }
                            };

                            let mut writer = writer_tun.lock().await;
                            if let Err(e) = writer.write_all(&buf[..n]).await {
                                eprintln!("Client disconnected: {}", e);
                                break;
                            }
                        }
                    });

                    // Client to TUN
                    let mut buf = [0u8; BUFFER_SIZE];

                    loop {
                        match reader.read(&mut buf).await {
                            Ok(0) => {
                                println!("Client disconnected: {}", addr);
                                tun_to_client.abort();
                                break;
                            }

                            Ok(n) => {
                                if !is_auth.load(Ordering::Acquire) {
                                    if authenticate(&buf[..n]) {
                                        is_auth.store(true, Ordering::Release);

                                        println!("Client authenticated: {}", addr);

                                        // Send authentication success ip
                                        let mut writer = writer.lock().await;
                                        writer.write_all(&[100, 10, 31, 0, 2]).await;
                                    } else {
                                        // Send authentication failure
                                        let mut writer = writer.lock().await;

                                        writer.write_all(&[99]).await;
                                        writer.shutdown().await;

                                        tun_to_client.abort();
                                        break;
                                    }
                                } else {
                                    let data = buf[..n].to_vec();

                                    tokio::task::spawn_blocking(move || {
                                        if let Err(e) = tun_write(tun_fd, &data) {
                                            eprintln!("TUN write error: {}", e);
                                        }
                                    });
                                }
                            }

                            Err(e) => {
                                eprintln!("Client error {}: {}", addr, e);
                                tun_to_client.abort();
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
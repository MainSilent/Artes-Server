use std::fs;
use std::io::{self, Write};
use std::path::Path;


pub fn check_certificate() -> io::Result<()> {
    let cert_path = Path::new("cert/certificate.pem");
    let key_path = Path::new("cert/private_key.pem");

    // Both already exist
    if cert_path.exists() && key_path.exists() {
        return Ok(());
    }

    println!("You don't have a certificate.");
    print!("Press Enter to create one... ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Create the cert directory if it doesn't exist
    fs::create_dir_all("cert")?;

    // Generate a real self-signed certificate
    let params = rcgen::CertificateParams::new(vec![])
        .map_err(|e| io::Error::other(e.to_string()))?;

    let key_pair = rcgen::KeyPair::generate()
        .map_err(|e| io::Error::other(e.to_string()))?;

    let cert = params
        .self_signed(&key_pair)
        .map_err(|e| io::Error::other(e.to_string()))?;

    fs::write(cert_path, cert.pem())?;
    fs::write(key_path, key_pair.serialize_pem())?;

    println!("Certificate created.");
    println!("Certificate: {}", cert_path.display());
    println!("Private key: {}", key_path.display());

    Ok(())
}
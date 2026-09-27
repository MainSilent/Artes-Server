mod auth;
mod cert;
mod tun;
mod utils;


use std::process;
use cert::{ check_certificate };
use auth::{
    Auth,
    read_auth
};
use tun::{ create_tunnel };


fn main() {
    check_certificate().unwrap();
    let auth : Auth = read_auth().unwrap();

    match create_tunnel("Artes", "10.31.0.1") {
        Ok(_) => println!("Tunnel started successfully"),
        Err(e) => {
            eprintln!("Tunnel failed: {}", e);
            process::exit(1);
        }
    }
    
    println!("Hello, world!");
}

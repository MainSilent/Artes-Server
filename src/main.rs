mod auth;
mod cert;
mod tun;
mod utils;
mod server;


use std::process;
use cert::{ check_certificate };
use auth::{
    Auth,
    read_auth
};
use tun::{ get_tunnel_fd, create_tunnel_interface };
use server::{ start_server };


const INTERFACE_NAME : &str = "Artes";


#[tokio::main]
async fn main() {
    check_certificate().unwrap();
    let auth : Auth = read_auth().unwrap();

    match create_tunnel_interface(INTERFACE_NAME, "10.31.0.1") {
        Ok(_) => println!("Tunnel started successfully"),
        Err(e) => {
            eprintln!("Tunnel failed: {}", e);
            process::exit(1);
        }
    }

    let tun_fd = tun::get_tunnel_fd(INTERFACE_NAME).unwrap();

    println!("id: {}", tun_fd);
    
    start_server().await.unwrap();
}

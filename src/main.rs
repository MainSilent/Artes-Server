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
const IP_RANGE : &str = "10.31.0.1";


#[tokio::main]
async fn main() {
    check_certificate().unwrap();
    read_auth().unwrap();

    match create_tunnel_interface(INTERFACE_NAME, IP_RANGE) {
        Ok(_) => println!("Tunnel started successfully"),
        Err(e) => {
            eprintln!("Tunnel failed: {}", e);
            process::exit(1);
        }
    }

    let tun_fd = tun::get_tunnel_fd(INTERFACE_NAME).unwrap();
    
    start_server(tun_fd).await.unwrap();
}

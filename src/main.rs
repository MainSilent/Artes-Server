mod auth;
mod cert;


use cert::{check_certificate};
use auth::{
    Auth,
    read_auth
};


fn main() {
    check_certificate().unwrap();
    let auth : Auth = read_auth().unwrap();
    
    println!("Hello, world!");
}

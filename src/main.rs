mod auth;


use auth::{
    Auth,
    read_auth
};


fn main() {
    let auth : Auth = read_auth().unwrap();
    
    println!("Hello, world!");
}

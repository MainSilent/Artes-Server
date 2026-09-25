use std::{
    env,
    io,
};

pub struct Auth {
    pub username: String,
    pub password: String,
}

pub fn read_auth() -> io::Result<Auth> {
    dotenvy::dotenv().unwrap();

    let username = env::var("CLIENT_USERNAME").map_err(|_| io::Error::other("USERNAME is not set"))?;

    let password = env::var("CLIENT_PASSWORD").map_err(|_| io::Error::other("PASSWORD is not set"))?;

    Ok(Auth {
        username,
        password,
    })
}
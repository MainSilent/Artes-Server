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

pub fn authenticate(data: &[u8]) -> bool {
    let auth = match read_auth() {
        Ok(auth) => auth,
        Err(_) => return false,
    };

    let credentials = String::from_utf8_lossy(data);
    let mut parts = credentials.split('\0');

    let username = parts.next().unwrap_or("");
    let password = parts.next().unwrap_or("");

    username == auth.username && password == auth.password
}
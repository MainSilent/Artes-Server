use std::process::Command;
use std::io;

pub fn sh(cmd: &str) -> io::Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("set -e; {}", cmd))
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);

        return Err(io::Error::new(
            io::ErrorKind::Other,
            err.trim().to_string(),
        ));
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}
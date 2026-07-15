use std::io;
use std::process::Command;

pub fn launch_package(package: &str) -> io::Result<()> {
    let status = Command::new("waydroid")
        .args(["app", "launch", package])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("failed to launch {}", package),
        ));
    }

    Ok(())
}


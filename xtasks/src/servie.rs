use xtasks::*;

use std::io;
use std::process::Command;

pub fn build() -> io::Result<()> {
    println!("[xtask]: Building servie");
    Command::new("cargo")
        .arg("build")
        .args(["--package", "servie"])
        .status()?
        .early_ret()?;

    Ok(())
}

pub fn serve() -> io::Result<()> {
    println!("[xtask]: Serving servie");
    Command::new("cargo")
        .arg("run")
        .args(["--package", "servie"])
        .status()?
        .early_ret()?;

    Ok(())
}
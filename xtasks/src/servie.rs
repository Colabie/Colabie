use xtasks::*;

use std::process::Command;
use std::{env, io, path};

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

    let cargo_root = path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("Failed to find cargo root"))?;

    env::set_current_dir(cargo_root.join("servie"))?;

    Command::new("cargo")
        .arg("run")
        .args(["--package", "servie"])
        .status()?
        .early_ret()?;

    Ok(())
}

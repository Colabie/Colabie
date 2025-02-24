use xtasks::*;

use std::io;
use std::process::Command;

const WEB_OUTPUT_DIR: &str = "target/web";

pub fn build() -> io::Result<()> {
    // Install wasm-bindgen if it's not already installed
    if Command::new("wasm-bindgen")
        .arg("--version")
        .status()
        .is_err()
    {
        println!("[xtask]: Installing wasm-bindgen");
        Command::new("cargo")
            .args(["install", "wasm-bindgen-cli"])
            .status()?
            .early_ret()?;
    }

    // Build Clientie
    println!("[xtask]: Building clientie");
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--package")
        .arg("clientie")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .status()?
        .early_ret()?;

    _ = std::fs::remove_dir_all(WEB_OUTPUT_DIR);

    println!("[xtasks]: Building wasm files");
    Command::new("wasm-bindgen")
        .arg("target/wasm32-unknown-unknown/release/clientie.wasm")
        .arg("--no-typescript")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg(format!("{WEB_OUTPUT_DIR}/wasm"))
        .status()?
        .early_ret()?;

    println!("[xtask]: Moving clientie web files to {WEB_OUTPUT_DIR}");
    copy_dir_all("clientie/web", WEB_OUTPUT_DIR)?;

    Ok(())
}

pub fn serve() -> io::Result<()> {
    // Install static-web-server if it's not already installed
    if Command::new("static-web-server")
        .arg("--version")
        .status()
        .is_err()
    {
        println!("[xtask]: Installing static-web-server");
        Command::new("cargo")
            .args(["install", "static-web-server"])
            .status()?
            .early_ret()?;
    }

    println!("[xtask]: Starting static-web-server at http://localhost:8080");
    Command::new("static-web-server")
        .args(["--host", "0.0.0.0"])
        .args(["--port", "8080"])
        .args(["--root", "./target/web"])
        .status()?;

    Ok(())
}

use xtasks::*;

use std::process::{exit, Command};
use std::{env, io};

mod clientie;
mod registrie;
mod servie;

fn print_usage() {
    eprintln!("[xtask]: Usage: cargo x <command>");
    eprintln!("[xtask]: Available commands are:");
    eprintln!("[xtask]:     make");
    eprintln!("[xtask]:     serve-clientie");
    eprintln!("[xtask]:     watch-clientie");
    eprintln!("[xtask]:     serve-registrie");
    eprintln!("[xtask]:     watch-registrie");
    eprintln!("[xtask]:     serve-servie");
    eprintln!("[xtask]:     watch-servie");
}

fn main() -> io::Result<()> {
    let mut args = env::args();
    _ = args.next().expect("Program name");

    let Some(command) = args.next() else {
        eprintln!("[xtask]: Error: No command provided");
        print_usage();
        exit(-1);
    };

    match command.as_str() {
        "make" => {
            println!("[xtask]: Making the Colabie project, finally ;)");
            clientie::build()?;
            registrie::build()
        }

        "serve-clientie" => {
            clientie::build()?;
            clientie::serve()
        }
        "watch-clientie" => watch("clientie/", "run --package xtasks serve-clientie"),

        "serve-registrie" => {
            registrie::build()?;
            registrie::serve()
        }
        "watch-registrie" => watch("registrie/src", "run --package xtasks serve-registrie"),

        "serve-servie" => {
            servie::build()?;
            servie::serve()
        }
        "watch-servie" => watch("servie/src", "run --package xtasks serve-servie"),

        command => {
            eprintln!("[xtask]: Error: Invalid command `{command}`");
            print_usage();
            exit(-1);
        }
    }
}

fn watch(dir: &str, run_command: &str) -> io::Result<()> {
    // Install cargo-watch if it's not already installed
    if Command::new("cargo")
        .args(["watch", "--version"])
        .status()?
        .early_ret()
        .is_err()
    {
        println!("[xtask]: Installing cargo-watch");
        Command::new("cargo")
            .args(["install", "cargo-watch"])
            .status()?
            .early_ret()?;
    }

    Command::new("cargo")
        .arg("watch")
        .arg("-qc")
        .args(["-w", dir])
        .args(["-x", run_command])
        .status()?;

    Ok(())
}

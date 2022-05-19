use once_cell::sync::OnceCell;
use std::env;
use std::error::Error;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::exit;

mod delete;
mod fs;
mod install;
mod messages;

const COMMANDS: [&str; 4] = ["install", "cleanup", "autocleanup", "implode"];

#[derive(Debug)]
pub struct This {
    platform: String,
    machine: String,
}

pub static HOME: OnceCell<PathBuf> = OnceCell::new();
pub static REPO_LOCATION: OnceCell<PathBuf> = OnceCell::new();
pub static THIS: OnceCell<This> = OnceCell::new();

pub fn run(args: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let name = &args[0];

    if args.len() < 2 {
        usage(&name);
    }

    let home = canonicalize(env::var("HOME").unwrap_or_else(|err| {
        eprintln!("Error reading HOME environment variable: {:?}", err);
        exit(1);
    }))
    .unwrap();

    HOME.set(home).unwrap();

    let repo_location = canonicalize(env::var("REPO_LOCATION").unwrap_or_else(|err| {
        eprintln!(
            "Error reading REPO_LOCATION environment variable: {:?}",
            err
        );
        exit(1);
    }))
    .unwrap();

    REPO_LOCATION.set(repo_location).unwrap();

    let host42 = env::var("HOST42").unwrap_or_else(|err| {
        eprintln!("Error reading HOST42 environment variable: {:?}", err);
        exit(1);
    });

    THIS.set(This {
        platform: match std::env::consts::OS {
            "linux" => String::from("linux"),
            "macos" => String::from("mac"),
            _ => String::from("unknown"),
        },
        machine: host42,
    })
    .unwrap_or_else(|err| {
        eprintln!("Problem getting hostname or OS: {:?}", err);
        exit(1);
    });

    let input = &args[1].to_lowercase();

    let command = COMMANDS.iter().find(|&command| command == &input.as_str());

    match command {
        Some(selection) => match selection {
            &"install" => install::run(),
            &"cleanup" => delete::run(false, false),
            &"autocleanup" => delete::run(false, true),
            &"implode" => delete::run(true, false),
            _ => Ok(usage(&name)),
        },
        None => Ok(usage(&name)),
    }
}

fn usage(cmd: &str) {
    println!(
        "Usage: {} <command>

Commands: {}",
        cmd,
        COMMANDS.join(" ")
    );
    exit(1);
}

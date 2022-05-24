use once_cell::sync::OnceCell;
use std::env;
use std::error::Error;
use std::fs::canonicalize;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Mutex;

mod delete;
mod fs;
mod install;
mod messages;

use messages::Messenger;

const COMMANDS: [&str; 4] = ["install", "cleanup", "autocleanup", "implode"];

#[derive(Debug)]
pub struct This {
    platform: String,
    machine: String,
}

pub static HOME: OnceCell<PathBuf> = OnceCell::new();
pub static REPO_LOCATION: OnceCell<PathBuf> = OnceCell::new();
pub static THIS: OnceCell<This> = OnceCell::new();

pub static DELETE_ALL: OnceCell<Mutex<bool>> = OnceCell::new();

pub fn run(args: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let name = &args[0];

    if args.len() < 2 {
        usage(&name);
    }

    let home = canonicalize(env::var("HOME").unwrap_or_else(|err| {
        Messenger::new().error(Some(format!(
            "reading HOME environment variable: {:?}",
            err
        )))
    }))
    .unwrap();

    HOME.set(home).unwrap();

    let repo_location = canonicalize(env::var("REPO_LOCATION").unwrap_or_else(|err| {
        Messenger::new().error(Some(format!(
            "reading REPO_LOCATION environment variable: {:?}",
            err
        )))
    }))
    .unwrap();

    REPO_LOCATION.set(repo_location).unwrap();

    let host42 = env::var("HOST42").unwrap_or_else(|err| {
        Messenger::new().error(Some(format!(
            "reading HOST42 environment variable: {:?}",
            err
        )))
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
        Messenger::new().error(Some(format!("Problem getting hostname or OS: {:?}", err)));
    });

    let input = &args[1].to_lowercase();

    let command = COMMANDS.iter().find(|&command| command == &input.as_str());

    DELETE_ALL.set(Mutex::new(false)).unwrap();

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

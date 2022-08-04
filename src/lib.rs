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

const COMMANDS: [&str; 5] = ["install", "cleanup", "sync", "autocleanup", "implode"];

pub fn run(args: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let name = &args[0];

    if args.len() < 2 {
        usage(&name);
    }

    let input = &args[1].to_lowercase();

    let command = COMMANDS.iter().find(|&command| command == &input.as_str());

    DELETE_ALL.set(Mutex::new(false)).unwrap();

    match command {
        Some(selection) => match selection {
            &"install" => install::run(),
            &"cleanup" => delete::run(false, false),
            &"autocleanup" => delete::run(false, true),
            &"implode" => delete::run(true, false),
            &"sync" => {
                install::run()?;
                delete::run(false, false)
            }
            _ => Ok(usage(&name)),
        },
        None => Ok(usage(&name)),
    }
}

static HOME: OnceCell<PathBuf> = OnceCell::new();
pub fn get_home() -> &'static PathBuf {
    HOME.get_or_init(|| canonicalize_or_bust(&get_env_or_bust("HOME")))
}

static REPO_LOCATION: OnceCell<PathBuf> = OnceCell::new();
pub fn get_repo() -> &'static PathBuf {
    REPO_LOCATION.get_or_init(|| canonicalize_or_bust(&get_env_or_bust("REPO_LOCATION")))
}

static DELETE_ALL: OnceCell<Mutex<bool>> = OnceCell::new();
pub fn get_delete_all() -> &'static Mutex<bool> {
    DELETE_ALL.get().unwrap()
}

#[derive(Debug)]
pub struct This {
    platform: String,
    machine: String,
}
static THIS: OnceCell<This> = OnceCell::new();
pub fn get_this() -> &'static This {
    THIS.get_or_init(|| {
        let machine = get_env_or_bust("HOST42");

        let platform = match std::env::consts::OS {
            "linux" => "linux",
            "macos" => "mac",
            _ => "unknown",
        }
        .into();

        This { platform, machine }
    })
}

fn get_env_or_bust(name: &str) -> String {
    env::var(name).unwrap_or_else(|err| {
        Messenger::new().with_verb("Error").error(Some(format!(
            "reading {} environment variable: {:?}",
            name, err
        )));
        exit(1);
    })
}

fn canonicalize_or_bust(name: &String) -> PathBuf {
    canonicalize(name).unwrap_or_else(|err| {
        Messenger::new().with_verb("Error").error(Some(format!(
            "canonicalizing {} environment variable: {:?}",
            name, err
        )));
        exit(1);
    })
}

fn usage(cmd: &str) {
    println!(
        "
link_o_matic v{}

Usage: {} <command>

Commands: {}
",
        env!("CARGO_PKG_VERSION"),
        cmd,
        COMMANDS.join(" ")
    );
    exit(1);
}

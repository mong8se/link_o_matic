use once_cell::sync::OnceCell;
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;

mod delete;
mod fs;
mod install;
mod messages;

const COMMANDS: [&str; 5] = ["install", "delete", "cleanup", "autocleanup", "implode"];

#[derive(Debug)]
pub struct This {
    platform: String,
    machine: String,
}

pub static HOME: OnceCell<PathBuf> = OnceCell::new();
pub static THIS: OnceCell<This> = OnceCell::new();

pub fn run(args: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let name = &args[0];

    if args.len() < 2 {
        usage(&name);
    }

    let home = PathBuf::from(env::var("HOME").unwrap_or_else(|err| {
        eprintln!("Error reading HOME environment variable: {:?}", err);
        exit(1);
    }));

    HOME.set(home).unwrap();

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
            &"delete" => delete(),
            &"cleanup" => delete(),
            &"autocleanup" => delete(),
            &"implode" => delete(),
            _ => Ok(usage(&name)),
        },
        None => Ok(usage(&name)),
    }
}

fn delete() -> Result<(), Box<dyn Error>> {
    println!("deleting...");
    Ok(())
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

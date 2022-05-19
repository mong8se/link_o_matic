use std::path::PathBuf;

use std::io::{stdin, stdout, Write};
use std::process::exit;

use crate::fs::get_dot_path;

use crate::delete::DeleteOptions;

fn relative_dot_file(entry: &PathBuf) -> String {
    entry
        .to_path_buf()
        .strip_prefix(&get_dot_path(None))
        .unwrap()
        .display()
        .to_string()
}

pub fn log_message_for_path(verb: &str, path: &PathBuf) {
    log_message(verb, &[relative_dot_file(&path)])
}

pub fn log_message(verb: &str, rest: &[String]) {
    println!("{:>9} {}", verb, rest.join(" "))
}

pub fn delete_prompt_help() {
    println!("y - yes , n - no, a - all, q - quit");
}

pub fn display_delete_prompt(name: &PathBuf, options: &DeleteOptions) -> char {
    let template = &options.verb_template;
    let parts: Vec<&str> = template.split('%').collect();

    let conjugate_with = |ending| format!("{}{}{}", parts[0], ending, parts[1]);

    let mut input = String::new();
    let result = if options.without_prompting.get() {
        "y"
    } else {
        print!(
            "{:>9} {} ? [y, n, a, q] ",
            &conjugate_with(&"e"),
            relative_dot_file(name)
        );

        stdout().flush().unwrap_or_else(|err| {
            eprintln!("Problem flushing stdout: {:?}", err);
            exit(1);
        });
        stdin().read_line(&mut input).unwrap();
        input.trim()
    };

    if result == "q" {
        log_message("quitting", &[String::from("per user")]);
        exit(0);
    } else if result == "y" || result == "a" {
        log_message_for_path(&conjugate_with(&"ing"), name);
        return result.chars().nth(0).unwrap();
    } else if result == "?" {
        delete_prompt_help();
        return display_delete_prompt(name, options);
    }

    log_message_for_path(&"skipping", name);
    return 'n';
}

use std::path::PathBuf;

use std::io;
use std::io::Write;

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
    println!("y - yes , n - no");
}

pub fn display_delete_prompt(name: &PathBuf, options: DeleteOptions) -> char {
    let template = &options.verb_template.as_ref().unwrap();
    let parts: Vec<&str> = template.split('%').collect();

    let conjugate_with = |ending| format!("{}{}{}", parts[0], ending, parts[1]);

    print!(
        "{:>9} {} ? [y, n] ",
        &conjugate_with(&"e"),
        relative_dot_file(name)
    );

    let mut input = String::new();
    io::stdout().flush();
    io::stdin().read_line(&mut input).unwrap();

    let result = input.trim();

    if result == "y" {
        log_message_for_path(&conjugate_with(&"ing"), name);
        return 'y';
    } else if result == "a" {
        log_message_for_path(&conjugate_with(&"ing"), name);
        return 'a';
    } else if result == "?" {
        delete_prompt_help();
        return display_delete_prompt(name, options);
    }

    log_message_for_path(&"skipping", name);
    return 'n';
}

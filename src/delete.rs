use std::cell::Cell;
use std::error::Error;
use std::fs::metadata;
use std::path::PathBuf;
// use std::fs::remove_file;

use crate::fs::{find_dot_links, get_dot_path, is_invalid_to_target, DotEntry};
use crate::messages::display_delete_prompt;

#[derive(Debug)]
pub struct DeleteOptions {
    pub implode: bool,
    pub without_prompting: Cell<bool>,
    pub verb_template: String,
}

impl DeleteOptions {
    pub fn new(implode: bool, without_prompting: bool, verb_template: Option<String>) -> Self {
        Self {
            implode,
            without_prompting: Cell::new(without_prompting),
            verb_template: match verb_template {
                Some(template) => template,
                None => String::from("delet%"),
            },
        }
    }
}

pub fn run(implode: bool, without_prompting: bool) -> Result<(), Box<dyn Error>> {
    let delete_options = &DeleteOptions::new(implode, without_prompting, None);

    let handle_delete = |entry: DotEntry| {
        if decide_delete(&entry, delete_options) {
            println!("Deleting: {}", entry.link.to_str().unwrap());
        }
    };

    find_dot_links(&get_dot_path(None), false, None, &handle_delete)?;
    find_dot_links(&get_dot_path(Some(".config")), true, None, &handle_delete)?;

    Ok(())
}

pub fn decide_delete(entry: &DotEntry, delete_options: &DeleteOptions) -> bool {
    if delete_options.implode
        || is_invalid_to_target(&entry.target)
        || metadata(&entry.target).is_err()
    {
        return delete_prompt(&entry.link, delete_options);
    }
    false
}

pub fn delete_prompt(path: &PathBuf, options: &DeleteOptions) -> bool {
    let result = display_delete_prompt(path, options);

    if result == 'a' {
        options.without_prompting.set(true);
    }

    if result == 'y' || result == 'a' {
        // remove_file(path);

        // return true;
        return false;
    }
    return false;
}

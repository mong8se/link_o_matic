use crate::messages::display_delete_prompt;
// use std::fs::remove_file;
use std::cell::Cell;
use std::path::PathBuf;

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
                None => String::from("delete"),
            },
        }
    }
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

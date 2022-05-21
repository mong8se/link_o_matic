use std::cell::Cell;
use std::error::Error;
use std::fs::{metadata, remove_dir, remove_file};
use std::path::PathBuf;

use crate::fs::{
    find_dot_links, get_dot_path, has_bad_underscore, has_no_matching_target, is_empty,
    is_invalid_to_target, DotEntry,
};

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
    let dir_delete_options =
        &DeleteOptions::new(false, false, Some(String::from("remov% empty directory")));

    let handle_delete = |entry: DotEntry| {
        decide_delete(&entry, delete_options);
    };

    find_dot_links(&get_dot_path(None), false, None, &handle_delete)?;

    let handle_delete_with_directories = |entry: DotEntry| {
        if decide_delete(&entry, delete_options) {
            let parent = &entry.link.parent().unwrap();
            let parent_buf = parent.to_path_buf();

            if is_empty(&parent_buf) {
                dir_delete_options
                    .without_prompting
                    .set(delete_options.without_prompting.get());
                delete_prompt(&parent_buf, dir_delete_options);
            }
        }
    };

    find_dot_links(
        &get_dot_path(Some(".config")),
        true,
        None,
        &handle_delete_with_directories,
    )?;

    Ok(())
}

pub fn decide_delete(entry: &DotEntry, delete_options: &DeleteOptions) -> bool {
    if delete_options.implode
        || is_invalid_to_target(&entry.target)
        || has_bad_underscore(&entry.link)
        || metadata(&entry.target).is_err()
        || has_no_matching_target(&entry.link)
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
        if path.is_symlink() {
            remove_file(path).unwrap();
        } else if path.is_dir() {
            // hope it's empty
            remove_dir(path).unwrap();
        } else {
            // what's left?
            return false;
        }

        return true;
    }
    return false;
}

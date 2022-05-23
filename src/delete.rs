use std::error::Error;
use std::fs::{metadata, remove_dir, remove_file};
use std::path::PathBuf;

use crate::fs::{
    find_links_to_targets, get_dot_path, has_bad_underscore, has_no_matching_target, is_empty,
    is_invalid_to_target, DotEntry,
};

use crate::messages::{display_delete_prompt, Messenger};
use crate::DELETE_ALL;

#[derive(Debug, Default)]
pub struct DeleteOptions<'a> {
    pub implode: bool,
    pub without_prompting: bool,
    pub verb_template: &'a str,
}

pub fn run(implode: bool, without_prompting: bool) -> Result<(), Box<dyn Error>> {
    if without_prompting {
        let mut delete_all = DELETE_ALL.get().unwrap().lock().unwrap();
        *delete_all = true;
    };

    let delete_options = &DeleteOptions {
        implode,
        without_prompting,
        verb_template: &"delet%",
    };

    let handle_delete = |entry: DotEntry| {
        decide_delete(&entry, delete_options);
    };

    find_links_to_targets(
        &get_dot_path(None),
        false,
        Some(&|x: &PathBuf| x.file_name().unwrap().to_str().unwrap().starts_with(".")),
        &handle_delete,
    )?;

    let dir_delete_options = &DeleteOptions {
        implode: false,
        without_prompting,
        verb_template: &"remov% empty directory",
    };

    let handle_delete_with_directories = |entry: DotEntry| {
        if decide_delete(&entry, delete_options) {
            let parent = &entry.link.parent().unwrap();
            let parent_buf = parent.to_path_buf();

            if is_empty(&parent_buf) {
                delete_prompt(&parent_buf, dir_delete_options);
            }
        }
    };

    find_links_to_targets(
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
        if delete_prompt(&entry.link, delete_options) {
            if entry.link.is_symlink() {
                remove_file(&entry.link).unwrap();
            } else if entry.link.is_dir() {
                // hope it's empty
                remove_dir(&entry.link).unwrap();
            } else {
                eprintln!("WHAT THE");
                // what's left?
                return false;
            }
            return true;
        }
    }

    false
}

pub fn delete_prompt(path: &PathBuf, options: &DeleteOptions) -> bool {
    let mut delete_all = DELETE_ALL.get().unwrap().lock().unwrap();

    let result = if *delete_all || options.without_prompting {
        'y'
    } else {
        display_delete_prompt(path, options)
    };

    if result == 'a' {
        *delete_all = true;
    }

    if result == 'y' || result == 'a' {
        Messenger::new()
            .with_verb(&options.verb_template)
            .conjugate_with(&"ing")
            .with_path(path)
            .success(None);

        return true;
    }

    Messenger::new()
        .with_verb(&"skipping")
        .with_path(path)
        .warning(None);

    return false;
}

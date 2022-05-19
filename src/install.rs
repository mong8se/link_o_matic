use std::env;
use std::error::Error;
use std::fs::{canonicalize, create_dir_all, metadata, read_link, symlink_metadata};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use crate::delete::{delete_prompt, DeleteOptions};
use crate::fs::*;
use crate::messages::*;

pub fn run() -> Result<(), Box<dyn Error>> {
    env::set_current_dir(get_dot_path(Some(&".dotfiles")))?;

    println!("installing...");

    let overwrite_options = &DeleteOptions::new(false, false, Some(String::from("replac%")));

    let process_link = |entry: DotEntry| {
        match decide_link(entry, overwrite_options) {
            Some(decided_link) => create_link(decided_link),
            None => Ok(()),
        }
        .unwrap();
    };

    let home = &"home";

    get_dot_links(&PathBuf::from(home), false, Some(home), &process_link)?;
    get_dot_links(&PathBuf::from(&"config"), true, None, &process_link)?;

    Ok(())
}

fn create_link(entry: DotEntry) -> Result<(), Box<dyn Error>> {
    create_dir_all(entry.link.parent().unwrap())?;
    symlink(entry.target, entry.link)?;
    Ok(())
}

fn decide_link(entry: DotEntry, overwrite_options: &DeleteOptions) -> Option<DotEntry> {
    if is_invalid_to_target(&entry.target) {
        log_message_for_path("ignoring", &entry.link);
        return None;
    }

    let target_resolved = canonicalize(&entry.target);
    if target_resolved.is_err() {
        log_message_for_path("skipping", &entry.link);
        return None;
    }

    let target = target_resolved.unwrap();

    let entry = DotEntry {
        link: entry.link,
        target,
    };

    let new_target_stat_lookup = metadata(&entry.target);
    if new_target_stat_lookup.is_err() {
        log_message_for_path("skipping", &entry.link);
        return None;
    };

    let link_stat_lookup = symlink_metadata(&entry.link);
    if link_stat_lookup.is_err() {
        log_message_for_path("linking", &entry.link);
        return Some(entry);
    }

    let mut current_target_exists = false;
    let link_target_stat_lookup = metadata(&entry.link);
    if link_target_stat_lookup.is_ok() {
        current_target_exists = true;

        if is_identical(
            &new_target_stat_lookup.unwrap(),
            &link_target_stat_lookup.unwrap(),
        ) {
            log_message_for_path("", &entry.link);
            return None;
        }
    }

    let mut old_link: Option<PathBuf> = None;
    if link_stat_lookup.unwrap().is_symlink() {
        old_link = Some(read_link(&entry.link).unwrap());
    }

    log_message_for_path("found", &entry.link);

    match old_link {
        Some(link) => log_message(
            "",
            &[format!(
                "Link already exists and points elsewhere: {} {}",
                link.to_str().unwrap(),
                if current_target_exists {
                    &""
                } else {
                    &"(dead)"
                }
            )],
        ),
        None => println!("File exists and is not a link"),
    }

    if delete_prompt(&entry.link, overwrite_options) {
        return Some(entry);
    }

    return None;
}

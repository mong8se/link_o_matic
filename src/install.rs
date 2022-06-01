use std::env;
use std::error::Error;
use std::fs::{create_dir_all, metadata, read_link, symlink_metadata};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use crate::{
    delete::{decide_delete, DeleteOptions},
    fs::{find_targets_for_linking, is_identical, is_invalid_to_target, DotEntry},
    get_repo,
    messages::Messenger,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    env::set_current_dir(get_repo())?;

    let replace_options = &DeleteOptions {
        implode: true,
        without_prompting: false,
        verb_template: &"replac%",
    };

    let auto_replace_options = &DeleteOptions {
        implode: true,
        without_prompting: true,
        verb_template: &"autoreplac%",
    };

    let process_link = move |entry: DotEntry| -> Result<(), Box<dyn Error>> {
        if decide_link(&entry, replace_options, auto_replace_options) {
            create_link(entry)
        } else {
            Ok(())
        }
    };

    let home = &"home";

    find_targets_for_linking(&PathBuf::from(home), false, Some(home), &process_link)?;
    find_targets_for_linking(&PathBuf::from(&"config"), true, None, &process_link)?;

    Ok(())
}

fn create_link(entry: DotEntry) -> Result<(), Box<dyn Error>> {
    create_dir_all(entry.link.parent().unwrap())?;
    symlink(entry.target, entry.link)?;
    Ok(())
}

fn decide_link(
    entry: &DotEntry,
    replace_options: &DeleteOptions,
    auto_replace_options: &DeleteOptions,
) -> bool {
    if is_invalid_to_target(&entry.target) {
        Messenger::new()
            .with_path(&entry.link)
            .with_verb("ignoring")
            .log(None);
        return false;
    }

    let new_target_stat_lookup = metadata(&entry.target);
    if new_target_stat_lookup.is_err() {
        Messenger::new()
            .with_verb("skipping")
            .with_path(&entry.link)
            .log(None);
        return false;
    };

    let link_stat_lookup = symlink_metadata(&entry.link);
    if link_stat_lookup.is_err() {
        Messenger::new()
            .with_verb("linking")
            .with_path(&entry.link)
            .success(None);
        return true;
    }

    let mut current_target_exists = false;
    let link_target_stat_lookup = metadata(&entry.link);
    if link_target_stat_lookup.is_ok() {
        current_target_exists = true;

        if is_identical(
            &new_target_stat_lookup.unwrap(),
            &link_target_stat_lookup.unwrap(),
        ) {
            Messenger::new().with_path(&entry.link).log(None);
            return false;
        }
    }

    let mut old_link: Option<PathBuf> = None;
    if link_stat_lookup.unwrap().is_symlink() {
        old_link = Some(read_link(&entry.link).unwrap());
    }

    Messenger::new()
        .with_verb("found")
        .with_path(&entry.link)
        .warning(Some(match old_link {
            Some(link) => format!(
                "Link already exists and points elsewhere: {} {}",
                link.to_str().unwrap(),
                if current_target_exists {
                    &""
                } else {
                    &"(dead: auto-replacing)"
                }
            ),
            None => "File exists and is not a link".to_string(),
        }));

    decide_delete(
        &entry,
        if current_target_exists {
            replace_options
        } else {
            auto_replace_options
        },
    )
}

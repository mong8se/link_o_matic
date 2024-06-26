use std::env;
use std::error::Error;
use std::fs::{create_dir_all, metadata, read_link, symlink_metadata};
use std::os::unix::fs::symlink;

use crate::{
    delete::{decide_delete, DeleteOptions},
    fs::{find_targets_for_linking, is_identical, is_invalid_to_target, DotEntry},
    get_root,
    messages::Messenger,
};

pub fn run() -> Result<(), Box<dyn Error>> {
    env::set_current_dir(get_root())?;

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

    let process_link = |entry: DotEntry| -> Result<(), Box<dyn Error>> {
        if decide_link(&entry, replace_options, auto_replace_options) {
            create_link(entry)?
        }
        Ok(())
    };

    find_targets_for_linking(&"home", &process_link)?;

    Ok(())
}

fn create_link(entry: DotEntry) -> Result<(), Box<dyn Error>> {
    create_dir_all(entry.link.parent().expect("What is this at the root?"))?;
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

    let new_target_stat = match metadata(&entry.target) {
        Ok(s) => s,
        Err(e) => {
            Messenger::new()
                .with_verb("skipping")
                .with_path(&entry.link)
                .warning(Some(format!("Broken link: {}", e)));
            return false;
        }
    };

    let link_stat = match symlink_metadata(&entry.link) {
        Ok(s) => s,
        Err(_) => {
            Messenger::new()
                .with_verb("linking")
                .with_path(&entry.link)
                .success(None);
            return true;
        }
    };

    let current_target_exists = match metadata(&entry.link) {
        Ok(current_target) if is_identical(&new_target_stat, &current_target) => {
            Messenger::new().with_path(&entry.link).log(None);
            return false;
        }
        Ok(_) => true,
        Err(_) => false,
    };

    let old_target_link = link_stat
        .is_symlink()
        .then(|| read_link(&entry.link))
        .and_then(|p| p.ok());
    let old_target = old_target_link.as_ref().and_then(|p| p.to_str());

    Messenger::new()
        .with_verb("found")
        .with_path(&entry.link)
        .warning(Some(match old_target {
            Some(link) => format!(
                "Link already exists and points elsewhere: {} {}",
                link,
                if current_target_exists {
                    &""
                } else {
                    &"(dead: auto-replacing)"
                }
            ),
            None => "File exists and is not a link, a .bak will be made".to_string(),
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

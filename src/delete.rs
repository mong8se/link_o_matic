use std::error::Error;
use std::fs::{metadata, remove_dir, remove_file, rename};
use std::path::PathBuf;

use crate::{
    fs::{
        find_links_to_targets, has_bad_underscore, has_no_matching_target,
        home_path_starts_with_dot, is_empty, is_invalid_to_target, name_with_bak, DotEntry,
    },
    get_delete_all, get_root,
    messages::display_delete_prompt,
    Messenger,
};

#[derive(Debug, Default)]
pub struct DeleteOptions<'a> {
    pub implode: bool,
    pub without_prompting: bool,
    pub verb_template: &'a str,
}

pub fn run(implode: bool, without_prompting: bool) -> Result<(), Box<dyn Error>> {
    if without_prompting {
        let mut delete_all = get_delete_all().lock().expect("How did I break mutex");
        *delete_all = true;
    };

    let delete_options = &DeleteOptions {
        implode,
        without_prompting,
        verb_template: &"delet%",
    };

    let dir_delete_options = &DeleteOptions {
        implode: false,
        without_prompting,
        verb_template: &"remov% empty directory",
    };

    let root = get_root();

    let handle_delete = &|link: PathBuf| -> Result<(), Box<dyn Error>> {
        if link.is_symlink() && home_path_starts_with_dot(&link) {
            let target = link.read_link().expect("is_symlink, what gives?");

            if target.starts_with(root) {
                let entry = DotEntry {
                    link: link.to_path_buf(),
                    target,
                };

                if decide_delete(&entry, delete_options) {
                    let parent = &entry
                        .link
                        .parent()
                        .map(|p| p.to_path_buf())
                        .expect("why is there no parent?");

                    if is_empty(&parent) {
                        delete_prompt(&parent, dir_delete_options);
                    }
                }
            }
        }
        Ok(())
    };

    find_links_to_targets(&handle_delete)?;

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
            let link = &entry.link;

            if link.is_symlink() {
                remove_file(link).expect(format!("Couldn't delete {:?}", link).as_str());
            } else if link.is_dir() {
                // hope it's empty
                remove_dir(link).expect(format!("Couldn't delete dir {:?}", link).as_str());
            } else if link.is_file() {
                rename(link, name_with_bak(link))
                    .expect(format!("Couldn't rename file {:?}", link).as_str());
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
    let mut delete_all = get_delete_all().lock().expect("How did I break mutex");

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

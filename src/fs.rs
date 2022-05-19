use std::error::Error;
use std::fs::metadata;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

pub struct DotEntry {
    pub link: PathBuf,
    pub target: PathBuf,
}

pub fn get_dot_path(prefix: Option<&str>) -> PathBuf {
    let base = crate::HOME.get().unwrap();

    match prefix {
        Some(val) => [base, &PathBuf::from(val)].iter().collect(),
        None => base.clone(),
    }
}

fn link_from_dot_path(dot_path: &PathBuf, prefix_to_strip: Option<&str>) -> PathBuf {
    let base = match prefix_to_strip {
        Some(prefix) => dot_path.strip_prefix(prefix).unwrap(),
        None => dot_path,
    };

    [
        get_dot_path(None),
        PathBuf::from(format!(".{}", base.to_str().unwrap())),
    ]
    .iter()
    .collect()
}

fn replace_this_labels(entry: PathBuf) -> PathBuf {
    let file_name = entry.file_name().unwrap().to_str().unwrap();

    if file_name.starts_with('_') {
        let this = crate::THIS.get().unwrap();

        let result = PathBuf::from(
            entry
                .to_str()
                .unwrap()
                .replace(&(String::from("_") + &this.platform), &"_platform")
                .replace(&(String::from("_") + &this.machine), &"_machine"),
        );

        return result;
    }

    return entry;
}

fn final_link_name(path: &PathBuf, prefix_to_strip: Option<&str>) -> PathBuf {
    replace_this_labels(link_from_dot_path(path, prefix_to_strip))
}

fn final_target_name(path: &PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path.to_owned())
}

pub fn has_bad_underscore(path: &PathBuf) -> bool {
    let file_name = path.file_name().unwrap().to_str().unwrap();

    if file_name.starts_with('_') {
        !(file_name.starts_with(&"_machine") || file_name.starts_with(&"_platform"))
    } else {
        false
    }
}

pub fn has_no_matching_target(path: &PathBuf) -> bool {
    let home = crate::HOME.get().unwrap().to_str().unwrap();
    let repo_location = crate::REPO_LOCATION.get().unwrap().to_str().unwrap();

    let base_path = path
        .to_str()
        .unwrap()
        .replace(&(String::from("") + &home + "."), &repo_location);
    metadata(base_path).is_err()
}

pub fn get_dot_links(
    dir: &PathBuf,
    recurse: bool,
    prefix_to_strip: Option<&str>,
    cb: &dyn Fn(DotEntry),
) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && recurse {
                get_dot_links(&path, recurse, prefix_to_strip, cb)?;
            } else {
                cb(DotEntry {
                    link: final_link_name(&entry.path(), prefix_to_strip),
                    target: final_target_name(&entry.path()),
                })
            }
        }
    }
    Ok(())
}

pub fn find_dot_links(
    dir: &PathBuf,
    recurse: bool,
    filter: Option<&str>,
    cb: &dyn Fn(DotEntry),
) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        let repo = crate::REPO_LOCATION.get().unwrap().to_str().unwrap();

        for entry in dir.read_dir()? {
            let entry = entry?;
            let link = entry.path();
            if link.is_dir() && recurse {
                find_dot_links(&link, recurse, filter, cb)?;
            } else if link.is_symlink() {
                let target = link.read_link().unwrap();
                if target.starts_with(repo) {
                    cb(DotEntry { link, target })
                }
            }
        }
    }

    Ok(())
}

pub fn is_identical(a: &dyn MetadataExt, b: &dyn MetadataExt) -> bool {
    a.dev() == b.dev() && a.ino() == b.ino()
}

pub fn is_invalid_to_target(entry: &PathBuf) -> bool {
    let file_name = entry.file_name().unwrap().to_str().unwrap();

    if file_name.starts_with('_') {
        let this = crate::THIS.get().unwrap();

        if file_name.starts_with(&format!("_{}", &this.platform))
            || file_name.starts_with(&format!("_{}", &this.machine))
        {
            return false;
        }

        return true;
    }

    return false;
}

pub fn is_empty(path: &PathBuf) -> bool {
    path.is_dir() && path.read_dir().unwrap().next().is_none()
}

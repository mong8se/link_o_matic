use crate::{get_home, get_root, get_this, Messenger};
use std::error::Error;
use std::fmt;
use std::fs::{canonicalize, metadata};
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

pub struct DotEntry {
    pub link: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone)]
struct PathError;

impl Error for PathError {}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Trouble parsing path for links")
    }
}

pub fn get_dot_path(prefix: Option<&str>) -> PathBuf {
    let base = get_home();

    match prefix {
        Some(val) => base.join(val),
        None => base.clone(),
    }
}

fn link_from_dot_path(dot_path: &PathBuf, prefix_to_strip: Option<&str>) -> Option<PathBuf> {
    let base = match prefix_to_strip {
        Some(prefix) => dot_path
            .strip_prefix(prefix)
            .expect("Somehow prefix isn't in path"),
        None => dot_path,
    };

    Some(get_dot_path(None).join(PathBuf::from(format!(".{}", base.to_str()?))))
}

fn replace_this_labels(entry: PathBuf) -> Option<PathBuf> {
    let file_name = file_name_as_str(&entry);

    if file_name.starts_with('_') {
        let this = get_this();

        let result = PathBuf::from(
            entry
                .to_str()?
                .replace(&(String::from("_") + &this.platform), &"_platform")
                .replace(&(String::from("_") + &this.machine), &"_machine"),
        );

        return Some(result);
    }

    return Some(entry);
}

fn final_link_name(path: &PathBuf, prefix_to_strip: Option<&str>) -> Option<PathBuf> {
    replace_this_labels(link_from_dot_path(path, prefix_to_strip)?)
}

fn final_target_name(path: &PathBuf) -> PathBuf {
    let root = get_root();

    if path.is_symlink() {
        return canonicalize(
            root.join(path.parent().expect("Why is there no parent?"))
                .join(path.read_link().expect("Why can't this be read?")),
        )
        .expect("Why can't I canonicalize this?");
    } else {
        return root.join(path);
    }
}

pub fn has_bad_underscore(path: &PathBuf) -> bool {
    let file_name = file_name_as_str(path);

    if file_name.starts_with('_') {
        !(file_name.starts_with(&"_machine") || file_name.starts_with(&"_platform"))
    } else {
        false
    }
}

pub fn has_no_matching_target(path: &PathBuf) -> bool {
    let [name, home, root] =
        [path, get_home(), get_root()].map(|buf| buf.to_str().expect("why no strings?"));

    let base_path = name.replace(&(String::from("") + home + "."), root);

    metadata(base_path).is_err()
}

pub fn walk_dir(
    dir: &PathBuf,
    doit: &dyn Fn(PathBuf) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        dir.read_dir().map_or_else(
            |e| {
                Ok(Messenger::new()
                    .with_verb("skipping")
                    .with_path(&dir)
                    .warning(Some(format!("couldn't read: {}", e))))
            },
            |entries| -> Result<(), Box<dyn Error>> {
                for entry in entries {
                    let path = entry?.path();
                    if !path.is_symlink() && path.is_dir() {
                        walk_dir(&path, doit)?
                    } else {
                        doit(path)?
                    }
                }
                Ok(())
            },
        )
    } else {
        Ok(())
    }
}

pub fn find_targets_for_linking(
    dir_name: &str,
    doit: &dyn Fn(DotEntry) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    walk_dir(&PathBuf::from(dir_name), &|path: PathBuf| -> Result<
        (),
        Box<dyn Error>,
    > {
        final_link_name(&path, Some(dir_name))
            .zip(Some(final_target_name(&path)))
            .map_or(Err(PathError.into()), |(link, target)| {
                doit(DotEntry { link, target })
            })
    })
}

pub fn find_links_to_targets(
    process: &dyn Fn(PathBuf) -> Result<(), Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    let root = get_root();

    // Check at root of ~
    for entry in get_dot_path(None).read_dir()? {
        let path = entry?.path();
        process(path)?;
    }

    // for every directory in root of .dotfiles/home/
    // check correspondingly name directory in ~/
    for entry in root.join("home").read_dir()? {
        let path = entry?.path();
        if path.is_dir() {
            let dirname = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| format!(".{}", name));
            walk_dir(&get_dot_path(dirname.as_deref()), &process)?;
        }
    }
    Ok(())
}

pub fn home_path_starts_with_dot(path: &PathBuf) -> bool {
    let relative_path = &path.strip_prefix(get_dot_path(None));
    relative_path.as_ref().map_or(false, |rp| {
        rp.to_str().map_or(false, |p| p.starts_with("."))
    })
}

pub fn is_identical(a: &dyn MetadataExt, b: &dyn MetadataExt) -> bool {
    [a.dev(), a.ino()] == [b.dev(), b.ino()]
}

fn file_name_as_str(word: &PathBuf) -> &str {
    word.file_name()
        .and_then(|w| w.to_str())
        .expect("Why is there no file name")
}

pub fn is_invalid_to_target(entry: &PathBuf) -> bool {
    let file_name = file_name_as_str(entry);

    if file_name.starts_with('_') {
        let this = get_this();

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
    path.is_dir()
        && path
            .read_dir()
            .and_then(|p| Ok(p.count() == 0))
            .expect("seems you tried to read a dir you cannot read")
}

pub fn name_with_bak(path: &PathBuf) -> PathBuf {
    path.with_extension(match path.extension() {
        Some(e) => format!(
            "{}.bak",
            e.to_str()
                .expect("Why can't this extension be cast as a str?")
        ),
        None => String::from("bak"),
    })
}

use std::fs::read_dir;
// use std::io::Result;
use std::error::Error;
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

pub fn get_dot_links(
    dir: &PathBuf,
    recurse: bool,
    prefix_to_strip: Option<&str>,
    cb: &dyn Fn(DotEntry),
) -> Result<(), Box<dyn Error>> {
    if dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && recurse {
                get_dot_links(&path, recurse, prefix_to_strip, cb)?;
            } else {
                cb(DotEntry {
                    link: link_from_dot_path(&entry.path(), prefix_to_strip),
                    target: entry.path(),
                })
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
    let this = crate::THIS.get().unwrap();

    if file_name.starts_with('_') {
        if file_name.starts_with(&format!("_{}", &this.platform))
            || file_name.starts_with(&format!("_{}", &this.machine))
        {
            return false;
        }

        return true;
    }

    return false;
}

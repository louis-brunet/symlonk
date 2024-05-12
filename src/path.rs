use std::{io, path::{Component, Path, PathBuf}};

pub fn path_exists<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let symlink_metadata =
        std::fs::symlink_metadata(path)
            .map(Some)
            .or_else(|err| match err.kind() {
                io::ErrorKind::NotFound => Ok(None),
                e => Err(e),
            })?;

    Ok(symlink_metadata.is_some())
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                if c == "~" {
                    // if let Some(home_dir) = get_home_dir() {
                    //     ret = home_dir;
                    // }
                    match get_home_dir() {
                        Some(home_dir) => ret = home_dir,
                        None => ret.push(c),
                    }
                } else {
                    ret.push(c);
                }
            }
        }
    }
    ret
}

pub fn join(first: &Path, second: &Path) -> PathBuf {
    crate::path::normalize_path(first.join(second).as_path())
}

fn get_home_dir() -> Option<PathBuf> {
    std::env::var("HOME").map(PathBuf::from).ok()
}

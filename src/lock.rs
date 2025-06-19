use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::log::Logger;

#[derive(Debug)]
pub enum LockFileVerifyError {
    IoError(std::io::Error),
    NotASymlink(PathBuf),
    SymlinkNotFound(PathBuf),
    SymlinkTargetNotFound {
        link_name: PathBuf,
        link_target: PathBuf,
    },
    InvalidSymlinkTarget {
        link_name: PathBuf,
        lock_file_link_target: PathBuf,
        disk_link_target: Option<PathBuf>,
    },
    // SymlinkNotFoundInLockFile(PathBuf),
    SymlinkNotFoundInLockFile {
        symlink_name: PathBuf,
        lock_file_path: PathBuf,
    },
    SymlinkNotFoundInConfig(PathBuf),
    InvalidSymlinkTargetInLockFile {
        link_name: PathBuf,
        lock_file_link_target: Option<PathBuf>,
        config_link_target: PathBuf,
    },
}

impl Display for LockFileVerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockFileVerifyError::NotASymlink(path) => {
                f.write_fmt(format_args!("not a symlink: {}", path.to_string_lossy()))
            }
            LockFileVerifyError::SymlinkNotFound(path) => f.write_fmt(format_args!(
                "symlink not found: {}",
                path.to_string_lossy()
            )),
            LockFileVerifyError::SymlinkTargetNotFound {
                link_name,
                link_target,
            } => f.write_fmt(format_args!(
                "symlink points to nonexistent file: {} -> {}",
                link_name.to_string_lossy(),
                link_target.to_string_lossy()
            )),
            LockFileVerifyError::InvalidSymlinkTarget {
                link_name,
                lock_file_link_target,
                disk_link_target,
            } => {
                let disk_target_str = match disk_link_target {
                    Some(target) => target.to_string_lossy(),
                    None => "<NONE>".into(),
                };
                f.write_fmt(format_args!(
                    "symlink target does not match lock file: {} -> {} (expected {})",
                    link_name.to_string_lossy(),
                    disk_target_str,
                    lock_file_link_target.to_string_lossy()
                ))
            }
            LockFileVerifyError::SymlinkNotFoundInLockFile { symlink_name, lock_file_path } => {
                f.write_fmt(format_args!(
                    "symlink from config not found in lock file: {}, was it generated using the following lock file ? {}",
                    symlink_name.to_string_lossy(),
                    lock_file_path.to_string_lossy(),
                ))
            },
            LockFileVerifyError::SymlinkNotFoundInConfig(path) => f.write_fmt(format_args!(
                "outdated symlink in lock file: {} is not in config, run with --prune to delete outdated symlinks",
                path.to_string_lossy()
            )),
            LockFileVerifyError::InvalidSymlinkTargetInLockFile {
                link_name,
                lock_file_link_target,
                config_link_target,
            } => {
                let lock_file_target_str = match lock_file_link_target {
                    Some(target) => target.to_string_lossy(),
                    None => "<NONE>".into(),
                };
                f.write_fmt(format_args!(
                    "invalid symlink target in lock file: {} -> {} (config expects {})",
                    link_name.to_string_lossy(),
                    lock_file_target_str,
                    config_link_target.to_string_lossy(),
                ))
            },
            LockFileVerifyError::IoError(error)=> f.write_fmt(format_args!("IO error: {:?}", error)),
            // _ => f.write_fmt(format_args!("{:?}", self)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LockFile {
    #[serde(default)]
    symlinks: HashMap<PathBuf, PathBuf>,

    #[serde(skip, default = "LockFile::default_path")]
    file_path: PathBuf,
}

impl LockFile {
    pub const DEFAULT_LOCK_FILE_PATH: &'static str = "symlonk-lock.toml";

    pub fn new(file_path: PathBuf) -> Self {
        Self {
            symlinks: HashMap::new(),
            file_path,
        }
    }

    fn default_path() -> PathBuf {
        PathBuf::from(Self::DEFAULT_LOCK_FILE_PATH)
    }

    pub fn set_symlink(&mut self, name: &Path, target: &Path) -> Option<PathBuf> {
        self.symlinks
            .insert(name.to_path_buf(), target.to_path_buf())
    }

    pub fn remove_symlink(&mut self, name: &Path) -> Option<PathBuf> {
        self.symlinks.remove(name)
    }

    pub fn remove_symlinks(&mut self) {
        self.symlinks.clear()
    }

    pub fn symlinks(&self) -> &HashMap<PathBuf, PathBuf> {
        &self.symlinks
    }

    pub fn symlink_count(&self) -> usize {
        self.symlinks.len()
    }

    pub fn verify_config(
        &self,
        config_symlinks: &HashMap<PathBuf, PathBuf>,
    ) -> Result<(), LockFileVerifyError> {
        let mut lock_file_keys = HashSet::new();
        for link_name in self.symlinks.keys() {
            lock_file_keys.insert(link_name);
        }

        for (config_link_name, config_link_target) in config_symlinks {
            match self.symlinks.get(config_link_name.as_path()) {
                Some(lock_symlink_target) => {
                    if lock_symlink_target.as_path() != config_link_target.as_path() {
                        return Err(LockFileVerifyError::InvalidSymlinkTargetInLockFile {
                            link_name: config_link_name.clone(),
                            lock_file_link_target: Some(lock_symlink_target.clone()),
                            config_link_target: config_link_target.clone(),
                        });
                    }

                    lock_file_keys.remove(config_link_name);
                }

                None => {
                    return Err(LockFileVerifyError::SymlinkNotFoundInLockFile {
                        symlink_name: config_link_name.clone(),
                        lock_file_path: self.file_path.clone(),
                    });
                }
            }
        }

        if let Some(&x) = lock_file_keys.iter().next() {
            return Err(LockFileVerifyError::SymlinkNotFoundInConfig(x.clone()));
        }

        Ok(())
    }

    // pub fn verify_symlinks(&self) -> Result<(), LockFileVerifyError> {
    //     self.verify_symlinks_created()?;
    //     self.verify_symlink_targets_exist()?;
    //     Ok(())
    // }

    pub fn verify_symlinks_created(&self) -> Result<(), LockFileVerifyError> {
        // for (lock_link_name, _lock_link_target) in &self.symlinks {
        for lock_link_name in self.symlinks.keys() {
            let symlink_metadata = std::fs::symlink_metadata(lock_link_name).map_err(|error| {
                if let io::ErrorKind::NotFound = error.kind() {
                    LockFileVerifyError::SymlinkNotFound(lock_link_name.clone())
                } else {
                    LockFileVerifyError::IoError(error)
                }
            })?;

            if !symlink_metadata.is_symlink() {
                return Err(LockFileVerifyError::NotASymlink(lock_link_name.clone()));
            }
        }

        Ok(())
    }

    pub fn verify_symlink_targets_exist(&self) -> Result<(), LockFileVerifyError> {
        for (lock_link_name, lock_link_target) in &self.symlinks {
            let symlink_target = lock_link_name.canonicalize().map_err(|error| {
                if let io::ErrorKind::NotFound = error.kind() {
                    LockFileVerifyError::SymlinkTargetNotFound {
                        link_name: lock_link_name.clone(),
                        link_target: lock_link_target.clone(),
                    }
                } else {
                    LockFileVerifyError::IoError(error)
                }
            })?;

            let invalid_target_error = LockFileVerifyError::InvalidSymlinkTarget {
                link_name: lock_link_name.clone(),
                lock_file_link_target: lock_link_target.clone(),
                disk_link_target: Some(symlink_target.clone()),
            };
            if symlink_target.as_path() != lock_link_target.as_path() {
                return Err(invalid_target_error);
            }
        }

        Ok(())
    }

    pub fn get_symlinks_to_delete(
        &self,
        config_symlinks: &HashMap<PathBuf, PathBuf>,
    ) -> Vec<(PathBuf, PathBuf)> {
        self.symlinks
            .iter()
            .filter(|(lockfile_name, lockfile_target)| {
                config_symlinks
                    .get(*lockfile_name)
                    .is_none_or(|config_target| {
                        config_target.as_path() != lockfile_target.as_path()
                    })
            })
            .map(|(name, target)| (name.clone(), target.clone()))
            .collect()
    }

    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(&self)
    }
}

#[derive(Debug)]
pub enum ParseLockFileError {
    Deserialize(toml::de::Error),
    Io(io::Error),
}

impl From<io::Error> for ParseLockFileError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<toml::de::Error> for ParseLockFileError {
    fn from(value: toml::de::Error) -> Self {
        Self::Deserialize(value)
    }
}

pub fn parse_lock_file(file_path: &Path) -> Result<LockFile, ParseLockFileError> {
    let log = Logger::default();
    let lock_file_contents = std::fs::read_to_string(file_path)?;
    let lock_file: LockFile = toml::from_str(lock_file_contents.as_str())?;

    log.debug(format_args!("parsed lock file: {:#?}", lock_file));

    Ok(lock_file)
}

pub fn verify(lock_file_path: &Path, config_files: Option<Vec<PathBuf>>) {
    let log = Logger::default();
    let lock_file = crate::lock::parse_lock_file(lock_file_path).unwrap_or_else(|error| {
        log.error(format_args!("parse_lock_file: {:?}", error));
        panic!();
    });

    if let Some(config_files) = config_files {
        let config_symlinks = crate::config::parse_symlinks_from_config_files(&config_files)
            .unwrap_or_else(|error| {
                log.error(format_args!(
                    "[parse_symlinks_from_config_files] {:?}",
                    error
                ));
                panic!()
            });

        match lock_file.verify_config(&config_symlinks) {
            Ok(_) => {
                // todo!()
                log.info(format_args!("verify: config matches lock file"));
            }
            Err(error) => {
                log.error(format_args!("verify: {}", error));
            }
        }
    }

    match lock_file.verify_symlinks_created() {
        Ok(_) => {
            log.info(format_args!(
                "verify: {} symlinks from lock file are created",
                lock_file.symlink_count()
            ));
        }
        Err(error) => {
            log.error(format_args!("verify: {}", error));
        }
    }

    match lock_file.verify_symlink_targets_exist() {
        Ok(_) => {
            log.info(format_args!(
                "verify: {} symlinks point to existing files",
                lock_file.symlink_count()
            ));
        }
        Err(error) => {
            log.error(format_args!("verify: {}", error));
        }
    }
}

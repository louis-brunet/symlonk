use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::log::Logger;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub source_dir: PathBuf,
    pub destination_dir: PathBuf,
}

#[derive(Debug)]
pub struct ExtendedConfig {
    pub config: Config,

    /// link_name -> link_target
    pub absolute_symlinks: HashMap<PathBuf, PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ConfigCore {
    RootConfig(Config),
    ChildConfig {
        extends: PathBuf,
        source_dir: Option<PathBuf>,
        destination_dir: Option<PathBuf>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigFile {
    config: ConfigCore,
    symlinks: Option<HashMap<PathBuf, PathBuf>>,
}

#[derive(Debug)]
pub enum ParseConfigFileErrorKind {
    // NotFound { path: PathBuf, io_error: io::Error },
    IoError(io::Error),
    InvalidToml(toml::de::Error),
}

impl From<toml::de::Error> for ParseConfigFileErrorKind {
    fn from(value: toml::de::Error) -> Self {
        Self::InvalidToml(value)
    }
}

impl From<io::Error> for ParseConfigFileErrorKind {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Debug)]
pub struct ParseConfigFileError {
    kind: ParseConfigFileErrorKind,
    config_file: PathBuf,
}

impl ParseConfigFileError {
    fn new<E: Into<ParseConfigFileErrorKind>>(error: E, file: &Path) -> Self {
        Self {
            kind: error.into(),
            config_file: file.to_path_buf(),
        }
    }
}

pub type ParseConfigFileResult<Res> = Result<Res, ParseConfigFileError>;

pub fn parse_symlinks_from_config_files(
    config_files: &Vec<PathBuf>,
) -> ParseConfigFileResult<HashMap<PathBuf, PathBuf>> {
    let mut symlinks = HashMap::new();

    for config_file in config_files {
        let config = parse_config_file(config_file.as_path())?;

        for (name, target) in config.absolute_symlinks {
            symlinks.insert(name, target);
        }
    }

    Ok(symlinks)
}

fn deserialize_config_file(
    file_contents: &str,
    file_path: &Path,
) -> ParseConfigFileResult<ConfigFile> {
    toml::from_str(file_contents).map_err(|error| ParseConfigFileError::new(error, file_path))
}

fn parse_config_file(config_file: &Path) -> ParseConfigFileResult<ExtendedConfig> {
    let log = Logger::default();
    let config_file = config_file
        .canonicalize()
        .map_err(|error| ParseConfigFileError::new(error, config_file))?;
    let config_file_dir = config_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or(PathBuf::from("/"));
    let file_contents = std::fs::read_to_string(config_file.as_path())
        .map_err(|error| ParseConfigFileError::new(error, config_file.as_path()))?;
    let parsed_file = deserialize_config_file(file_contents.as_str(), config_file.as_path())?;

    let config = match parsed_file.config {
        ConfigCore::RootConfig(root_config) => Config {
            source_dir: crate::path::normalize_path(
                config_file_dir
                    .join(root_config.source_dir.as_path())
                    .as_path(),
            ),
            destination_dir: crate::path::normalize_path(
                config_file_dir
                    .join(root_config.destination_dir.as_path())
                    .as_path(),
            ),
        },
        ConfigCore::ChildConfig {
            extends,
            source_dir,
            destination_dir,
        } => {
            let parent_path = config_file_dir.join(extends);
            let parent_config = parse_config_file(parent_path.as_path())?;

            Config {
                source_dir: source_dir
                    .map(|dir| crate::path::join(config_file_dir.as_path(), dir.as_path()))
                    .unwrap_or(parent_config.config.source_dir.clone()),
                destination_dir: destination_dir
                    .map(|dir| crate::path::join(config_file_dir.as_path(), dir.as_path()))
                    .unwrap_or(parent_config.config.destination_dir.clone()),
            }
        }
    };
    let absolute_symlinks = parsed_file
        .symlinks
        .unwrap_or_default()
        .iter()
        .map(|(link_name, link_target)| {
            (
                crate::path::join(config.destination_dir.as_path(), link_name),
                crate::path::join(config.source_dir.as_path(), link_target),
            )
        })
        .collect();

    let extended_config = ExtendedConfig {
        config,
        absolute_symlinks,
    };
    log.debug(
        format_args!(
            "parsed config file {} : {:#?}",
            config_file.to_string_lossy(),
            extended_config
        ),
    );
    Ok(extended_config)
}

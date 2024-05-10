use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigCore {
    pub source_dir: PathBuf,
    pub destination_dir: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub config: ConfigCore,
    pub symlinks: HashMap<PathBuf, PathBuf>,
}

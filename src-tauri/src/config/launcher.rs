use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LauncherConfig {
    #[serde(default)]
    pub install_dir: Option<PathBuf>,
}

pub const LAUNCHER_CONFIG_FILENAME: &'static str = "config.json";

impl LauncherConfig {
    pub fn get_path(dir: &PathBuf) -> PathBuf {
        dir.join(LAUNCHER_CONFIG_FILENAME)
    }
}

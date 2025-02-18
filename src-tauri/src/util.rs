use std::{fs::File, path::PathBuf};

use serde::de::DeserializeOwned;

pub fn load_json_or_default<T>(path: &PathBuf) -> T
where
    T: Default + DeserializeOwned,
{
    File::open(path)
        .ok()
        .and_then(|file| serde_json::from_reader(file).ok())
        .unwrap_or_default()
}

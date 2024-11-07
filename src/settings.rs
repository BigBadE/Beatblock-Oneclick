use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LocalData {
    pub levels_dir: Option<PathBuf>
}

impl LocalData {
    pub fn new() -> Self {
        let Some(settings) = dirs::config_dir() else {
            return Self::default();
        };
        let settings = settings.join("beatblock-oneclick").join("settings.txt");
        if !fs::exists(settings.clone()).unwrap_or(false) {
            return Self::default();
        }
        match fs::read(settings.clone()).map_err(Error::new)
            .and_then(|file| serde_json::from_slice(file.deref()).map_err(Error::new)) {
            Ok(result) => result,
            Err(_) => {
                let _ = fs::remove_file(settings);
                Self::default()
            }
        }
    }
}

impl Default for LocalData {
    fn default() -> Self {
        let levels_dir = match dirs::config_dir().map(|dir| dir.join("beatblock").join("Custom Levels")) {
            Some(levels) => fs::exists(levels.clone()).ok().map(move |_| levels),
            None => None
        };
        Self {
            levels_dir
        }
    }
}
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub volume: f32,
    pub play_mode: PlayMode,
    pub show_parent_folders: bool,
    pub last_folder: Option<PathBuf>,
    pub font_size: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayMode {
    Normal,
    Single,
    Repeat,
    RepeatOne,
    Shuffle,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            play_mode: PlayMode::Normal,
            show_parent_folders: false,
            last_folder: None,
            font_size: 13.0,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("bobby_config.json") {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("bobby_config.json", content);
        }
    }
}

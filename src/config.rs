use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub volume: f32,
    pub play_mode: PlayMode,
    pub show_parent_folders: bool,
    pub last_folder: Option<PathBuf>,
    pub font_size: f32,
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
    pub window_x: Option<f32>,
    pub window_y: Option<f32>,
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
            window_width: Some(680.0),
            window_height: Some(480.0),
            window_x: None,
            window_y: None,
        }
    }
}

impl AppConfig {
    pub fn config_file_path() -> PathBuf {
        let base_dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".config")
        } else {
            PathBuf::from(".")
        };

        let app_dir = base_dir.join("bobby");
        let _ = std::fs::create_dir_all(&app_dir);
        app_dir.join("bobby_config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_file_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_file_path();
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }
}

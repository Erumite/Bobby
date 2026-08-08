use rand::seq::SliceRandom;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub filename: String,
    pub parent_folder: String,
    pub selected: bool,
}

impl Track {
    pub fn new(path: PathBuf) -> Self {
        let filename = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let parent_folder = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        Self {
            path,
            filename,
            parent_folder,
            selected: false,
        }
    }

    pub fn display_name(&self, show_parent: bool) -> String {
        if show_parent && !self.parent_folder.is_empty() {
            format!("{}/{}", self.parent_folder, self.filename)
        } else {
            self.filename.clone()
        }
    }
}

pub struct Playlist {
    pub tracks: Vec<Track>,
    pub current_index: Option<usize>,
    pub loaded_folder: Option<PathBuf>,
    pub filter: String,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            loaded_folder: None,
            filter: String::new(),
        }
    }

    pub fn load_directory(&mut self, dir: &Path) -> usize {
        self.loaded_folder = Some(dir.to_path_buf());
        self.tracks.clear();
        self.current_index = None;

        let valid_extensions = ["mp3", "flac", "ogg", "wav", "m4a", "aac", "opus"];

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if valid_extensions.contains(&ext.to_lowercase().as_str()) {
                        self.tracks.push(Track::new(path.to_path_buf()));
                    }
                }
            }
        }

        // Sort alphabetically by path for clean album ordering
        self.tracks.sort_by(|a, b| a.path.cmp(&b.path));
        self.tracks.len()
    }

    pub fn refresh_directory(&mut self) -> usize {
        if let Some(folder) = self.loaded_folder.clone() {
            self.load_directory(&folder)
        } else {
            0
        }
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        if self.filter.is_empty() {
            (0..self.tracks.len()).collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| {
                    t.filename.to_lowercase().contains(&filter_lower)
                        || t.parent_folder.to_lowercase().contains(&filter_lower)
                })
                .map(|(i, _)| i)
                .collect()
        }
    }

    pub fn selected_indices(&self) -> Vec<usize> {
        self.tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.selected)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn select_all(&mut self, selected: bool) {
        for t in &mut self.tracks {
            t.selected = selected;
        }
    }

    pub fn crop_to_selected(&mut self) {
        let current_path = self.current_track().map(|t| t.path.clone());
        self.tracks.retain(|t| t.selected);
        if let Some(cp) = current_path {
            self.current_index = self.tracks.iter().position(|t| t.path == cp);
        } else {
            self.current_index = None;
        }
    }

    pub fn remove_selected(&mut self) -> usize {
        let current_path = self.current_track().map(|t| t.path.clone());
        let initial_len = self.tracks.len();
        self.tracks.retain(|t| !t.selected);
        let removed_count = initial_len - self.tracks.len();

        if let Some(cp) = current_path {
            self.current_index = self.tracks.iter().position(|t| t.path == cp);
        } else if let Some(curr) = self.current_index {
            if curr >= self.tracks.len() && !self.tracks.is_empty() {
                self.current_index = Some(self.tracks.len() - 1);
            } else if self.tracks.is_empty() {
                self.current_index = None;
            }
        }
        removed_count
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_index.and_then(|i| self.tracks.get(i))
    }

    pub fn next_index(&self, play_mode: crate::config::PlayMode) -> Option<usize> {
        if self.tracks.is_empty() {
            return None;
        }

        match play_mode {
            crate::config::PlayMode::Single => None,
            crate::config::PlayMode::RepeatOne => self.current_index,
            crate::config::PlayMode::Shuffle => {
                let mut rng = rand::thread_rng();
                let indices: Vec<usize> = (0..self.tracks.len()).collect();
                indices.choose(&mut rng).copied()
            }
            crate::config::PlayMode::Normal | crate::config::PlayMode::Repeat => {
                if let Some(curr) = self.current_index {
                    if curr + 1 < self.tracks.len() {
                        Some(curr + 1)
                    } else if play_mode == crate::config::PlayMode::Repeat {
                        Some(0)
                    } else {
                        None
                    }
                } else {
                    Some(0)
                }
            }
        }
    }

    pub fn prev_index(&self) -> Option<usize> {
        if self.tracks.is_empty() {
            return None;
        }
        if let Some(curr) = self.current_index {
            if curr > 0 {
                Some(curr - 1)
            } else {
                Some(self.tracks.len() - 1)
            }
        } else {
            Some(0)
        }
    }

    pub fn batch_rename(&mut self, indices: &[usize], target: &str, replacement: &str) -> usize {
        let mut renamed_count = 0;
        let regex_res = Regex::new(target);

        for &idx in indices {
            if let Some(track) = self.tracks.get_mut(idx) {
                let old_name = track.filename.clone();
                let new_name = if let Ok(ref re) = regex_res {
                    re.replace_all(&old_name, replacement).to_string()
                } else {
                    old_name.replace(target, replacement)
                };

                if new_name != old_name {
                    let parent = track.path.parent().unwrap_or_else(|| Path::new(""));
                    let new_path = parent.join(&new_name);
                    if std::fs::rename(&track.path, &new_path).is_ok() {
                        track.path = new_path;
                        track.filename = new_name;
                        renamed_count += 1;
                    }
                }
            }
        }

        renamed_count
    }
}

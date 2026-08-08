use crate::audio::AudioPlayer;
use crate::config::{AppConfig, PlayMode};
use crate::playlist::Playlist;
use eframe::egui::{
    self, Align, Color32, Context, Key, Layout, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, Vec2,
};
use rfd::FileDialog;

pub struct BobbyApp {
    audio: AudioPlayer,
    playlist: Playlist,
    config: AppConfig,

    // UI Modal States
    show_easy_finder: bool,
    show_batch_rename: bool,
    show_shortcuts: bool,

    rename_target: String,
    rename_replacement: String,
    single_rename_text: String,
    single_rename_idx: Option<usize>,

    status_msg: String,
}

impl BobbyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        let mut audio = AudioPlayer::new();
        audio.set_volume(config.volume);

        let mut app = Self {
            audio,
            playlist: Playlist::new(),
            config,
            show_easy_finder: false,
            show_batch_rename: false,
            show_shortcuts: false,
            rename_target: String::new(),
            rename_replacement: String::new(),
            single_rename_text: String::new(),
            single_rename_idx: None,
            status_msg: "Ready".to_string(),
        };

        if let Some(ref folder) = app.config.last_folder.clone() {
            if folder.exists() {
                let count = app.playlist.load_directory(folder);
                app.set_status(&format!("Loaded {} tracks from last folder", count));
            }
        }

        app
    }

    fn set_status(&mut self, msg: &str) {
        self.status_msg = msg.to_string();
    }

    fn play_track_at(&mut self, index: usize) {
        if let Some(track) = self.playlist.tracks.get(index) {
            let path = track.path.clone();
            match self.audio.play(&path) {
                Ok(_) => {
                    self.playlist.current_index = Some(index);
                    self.set_status(&format!("Playing: {}", track.filename));
                }
                Err(e) => {
                    self.set_status(&format!("Playback error: {}", e));
                }
            }
        }
    }

    fn next_track(&mut self) {
        if let Some(next) = self.playlist.next_index(self.config.play_mode) {
            self.play_track_at(next);
        } else {
            self.audio.stop();
        }
    }

    fn prev_track(&mut self) {
        if let Some(prev) = self.playlist.prev_index() {
            self.play_track_at(prev);
        }
    }

    fn open_folder_dialog(&mut self) {
        if let Some(path) = FileDialog::new().pick_folder() {
            let count = self.playlist.load_directory(&path);
            self.config.last_folder = Some(path.clone());
            self.config.save();
            self.set_status(&format!("Loaded {} tracks", count));
        }
    }

    fn render_led_meter(&self, ui: &mut egui::Ui, left: f32, right: f32) {
        let (rect, _response) = ui.allocate_exact_size(Vec2::new(140.0, 24.0), Sense::hover());
        let painter = ui.painter_at(rect);

        // Dark background box
        painter.rect_filled(rect, 3.0, Color32::from_rgb(15, 20, 25));
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Color32::from_rgb(40, 50, 60)));

        let led_count = 14;
        let pad = 2.0;
        let bar_height = (rect.height() - pad * 3.0) / 2.0;
        let led_width = (rect.width() - pad * (led_count as f32 + 1.0)) / led_count as f32;

        for ch in 0..2 {
            let val = if ch == 0 { left } else { right };
            let active_leds = (val * led_count as f32).round() as usize;
            let y = rect.min.y + pad + (ch as f32) * (bar_height + pad);

            for i in 0..led_count {
                let x = rect.min.x + pad + (i as f32) * (led_width + pad);
                let led_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(led_width, bar_height));

                let color = if i < active_leds {
                    if i < 9 {
                        Color32::from_rgb(50, 220, 90) // Green
                    } else if i < 12 {
                        Color32::from_rgb(240, 210, 40) // Yellow
                    } else {
                        Color32::from_rgb(240, 60, 50) // Red
                    }
                } else {
                    Color32::from_rgb(30, 40, 45) // Off LED
                };

                painter.rect_filled(led_rect, 1.0, color);
            }
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
        if ctx.wants_keyboard_input() {
            return;
        }

        ctx.input(|i| {
            // F4: Open Folder
            if i.key_pressed(Key::F4) {
                self.open_folder_dialog();
            }
            // F5: Refresh Folder
            if i.key_pressed(Key::F5) {
                let count = self.playlist.refresh_directory();
                self.set_status(&format!("Refreshed directory: {} tracks", count));
            }
            // F8: Toggle Parent Subfolder display
            if i.key_pressed(Key::F8) {
                self.config.show_parent_folders = !self.config.show_parent_folders;
                self.config.save();
                let status = if self.config.show_parent_folders { "ON" } else { "OFF" };
                self.set_status(&format!("Parent folder view: {}", status));
            }
            // F2: Rename single or multiple files
            if i.key_pressed(Key::F2) {
                let selected = self.playlist.selected_indices();
                if selected.len() > 1 {
                    self.show_batch_rename = true;
                } else if selected.len() == 1 {
                    let idx = selected[0];
                    if let Some(t) = self.playlist.tracks.get(idx) {
                        self.single_rename_idx = Some(idx);
                        self.single_rename_text = t.filename.clone();
                    }
                } else if let Some(idx) = self.playlist.current_index {
                    if let Some(t) = self.playlist.tracks.get(idx) {
                        self.single_rename_idx = Some(idx);
                        self.single_rename_text = t.filename.clone();
                    }
                }
            }
            // Space: Toggle Play / Pause / Stop
            if i.key_pressed(Key::Space) {
                if i.modifiers.ctrl {
                    // Ctrl + Space: Reset audio mixer volume
                    self.audio.set_volume(1.0);
                    self.config.volume = 1.0;
                    self.config.save();
                    self.set_status("Volume reset to 100%");
                } else if self.audio.is_playing() {
                    self.audio.toggle_pause();
                } else if let Some(idx) = self.playlist.current_index {
                    self.play_track_at(idx);
                } else if !self.playlist.tracks.is_empty() {
                    self.play_track_at(0);
                }
            }
            // Enter: Play selected track
            if i.key_pressed(Key::Enter) {
                let selected = self.playlist.selected_indices();
                if let Some(&first) = selected.first() {
                    self.play_track_at(first);
                }
            }
            // Ctrl + M: Cycle play mode
            if i.key_pressed(Key::M) && i.modifiers.ctrl {
                self.config.play_mode = match self.config.play_mode {
                    PlayMode::Normal => PlayMode::Single,
                    PlayMode::Single => PlayMode::Repeat,
                    PlayMode::Repeat => PlayMode::RepeatOne,
                    PlayMode::RepeatOne => PlayMode::Shuffle,
                    PlayMode::Shuffle => PlayMode::Normal,
                };
                self.config.save();
                self.set_status(&format!("Play mode: {:?}", self.config.play_mode));
            }
            // Ctrl + Delete: Crop playlist to selected tracks
            if i.key_pressed(Key::Delete) && i.modifiers.ctrl {
                self.playlist.crop_to_selected();
                self.set_status("Playlist cropped to selected items");
            }
            // Key V: Temp lower volume by 30% (-30% Mute toggle)
            if i.key_pressed(Key::V) {
                self.audio.toggle_temp_mute();
                let status = if self.audio.is_temp_muted() { "-30% Muted" } else { "Normal" };
                self.set_status(&format!("Quick Volume Attenuation: {}", status));
            }
            // Home / Tilde: Jump to top
            if i.key_pressed(Key::Home) {
                self.set_status("Jumped to top");
            }
            // Slash or F3: Open Easy Finder
            if i.key_pressed(Key::Slash) || i.key_pressed(Key::F3) {
                self.show_easy_finder = true;
            }
        });
    }
}

impl eframe::App for BobbyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Continuous repaint loop while playing for smooth 14-LED meter animation
        if self.audio.is_playing() {
            ctx.request_repaint();
        }

        // Auto gapless track progression
        if self.audio.is_finished() {
            self.next_track();
        }

        self.handle_keyboard_shortcuts(ctx);

        // Retro dark navy theme styling
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(18, 22, 28);
        visuals.window_fill = Color32::from_rgb(24, 28, 36);
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(20, 25, 32);
        ctx.set_visuals(visuals);

        // Top Control & Header Panel
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("❓").on_hover_text("View keyboard shortcuts guide").clicked() {
                    self.show_shortcuts = true;
                }

                ui.separator();

                ui.heading(RichText::new("BOBBY").strong().color(Color32::from_rgb(80, 190, 250)));
                ui.label(RichText::new("v1.0.0 (Linux Native)").small().color(Color32::GRAY));

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (l, r) = self.audio.get_levels();
                    self.render_led_meter(ui, l, r);
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Audio & Playback Toolbar
            ui.horizontal(|ui| {
                // Combined Play/Pause button
                let (play_pause_symbol, play_pause_tooltip) = if self.audio.is_playing() {
                    ("⏸", "Pause playback")
                } else {
                    ("▶", "Play / Resume track")
                };

                if ui.button(play_pause_symbol).on_hover_text(play_pause_tooltip).clicked() {
                    if self.audio.is_playing() {
                        self.audio.toggle_pause();
                    } else if let Some(idx) = self.playlist.current_index {
                        self.play_track_at(idx);
                    } else if !self.playlist.tracks.is_empty() {
                        self.play_track_at(0);
                    }
                }

                if ui.button("⏹").on_hover_text("Stop playback").clicked() {
                    self.audio.stop();
                }
                if ui.button("⏮").on_hover_text("Previous track").clicked() {
                    self.prev_track();
                }
                if ui.button("⏭").on_hover_text("Next track").clicked() {
                    self.next_track();
                }

                ui.separator();

                // PlayMode Compact Square Button
                let (mode_symbol, mode_desc) = match self.config.play_mode {
                    PlayMode::Normal => ("▶", "Play List (Normal)"),
                    PlayMode::Single => ("📄", "Play Single File (Stop when finished)"),
                    PlayMode::Repeat => ("🔁", "Repeat All (Loop List)"),
                    PlayMode::RepeatOne => ("🔂", "Repeat Single Track"),
                    PlayMode::Shuffle => ("🔀", "Shuffle (Random Order)"),
                };

                let btn = egui::Button::new(mode_symbol)
                    .min_size(Vec2::new(24.0, ui.spacing().interact_size.y));

                if ui.add(btn).on_hover_text(format!("Mode: {} (Click or Ctrl+M to change)", mode_desc)).clicked() {
                    self.config.play_mode = match self.config.play_mode {
                        PlayMode::Normal => PlayMode::Single,
                        PlayMode::Single => PlayMode::Repeat,
                        PlayMode::Repeat => PlayMode::RepeatOne,
                        PlayMode::RepeatOne => PlayMode::Shuffle,
                        PlayMode::Shuffle => PlayMode::Normal,
                    };
                    self.config.save();
                    self.set_status(&format!("Play mode set to: {}", mode_desc));
                }

                ui.separator();

                // Volume slider dynamically stretching across remaining width
                ui.label("🔊");
                ui.scope(|ui| {
                    let available_slider_w = (ui.available_width() - 48.0).max(40.0);
                    ui.spacing_mut().slider_width = available_slider_w;
                    let mut vol = self.config.volume;
                    if ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false)).changed() {
                        self.config.volume = vol;
                        self.audio.set_volume(vol);
                        self.config.save();
                    }
                });
                ui.label(format!("{:.0}%", self.config.volume * 100.0));
            });
            ui.add_space(4.0);
        });

        // Bottom Status Bar
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let track_count = self.playlist.tracks.len();
                let current_info = if let Some(t) = self.playlist.current_track() {
                    format!("Playing: {}", t.filename)
                } else {
                    "Stopped".to_string()
                };

                ui.label(RichText::new(format!("Tracks: {} | {}", track_count, current_info)).small());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(&self.status_msg).small().color(Color32::from_rgb(180, 220, 250)));
                });
            });
        });

        // Main Playlist Panel
        egui::CentralPanel::default().show(ctx, |ui| {
            let visible_indices = self.playlist.visible_indices();
            let show_parent = self.config.show_parent_folders;

            ScrollArea::vertical().show_rows(ui, 22.0, visible_indices.len(), |ui, row_range| {
                for i in row_range {
                    if let Some(&track_idx) = visible_indices.get(i) {
                        let is_current = self.playlist.current_index == Some(track_idx);
                        let track = &mut self.playlist.tracks[track_idx];
                        let is_selected = track.selected;

                        let text_color = if is_current {
                            Color32::from_rgb(100, 220, 255)
                        } else {
                            Color32::from_rgb(220, 230, 240)
                        };

                        let icon = if is_current && self.audio.is_playing() { "▶ " } else { "   " };
                        let label_text = format!("{}{:03}. {}", icon, track_idx + 1, track.display_name(show_parent));

                        let response = ui.selectable_label(
                            is_selected || is_current,
                            RichText::new(label_text).color(text_color),
                        );

                        if response.double_clicked() {
                            self.play_track_at(track_idx);
                        } else if response.clicked() {
                            if !ui.input(|inp| inp.modifiers.ctrl) {
                                self.playlist.select_all(false);
                            }
                            self.playlist.tracks[track_idx].selected = !is_selected;
                        }
                    }
                }
            });
        });

        // Easy Finder Modal Overlay
        if self.show_easy_finder {
            egui::Window::new("🔍 Bobby Easy Finder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0.0, -50.0))
                .show(ctx, |ui| {
                    ui.label("Type song or folder name:");
                    let response = ui.add(egui::TextEdit::singleline(&mut self.playlist.filter).hint_text("Search playlist..."));
                    response.request_focus();

                    ui.horizontal(|ui| {
                        if ui.button("Clear Search").clicked() {
                            self.playlist.filter.clear();
                        }
                        if ui.button("Close").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
                            self.show_easy_finder = false;
                        }
                    });
                });
        }

        // Batch Rename Modal
        if self.show_batch_rename {
            egui::Window::new("✏ Batch File Replacer (F2)")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    let selected_count = self.playlist.selected_indices().len();
                    ui.label(format!("Renaming {} selected files:", selected_count));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label("Find text:");
                        ui.text_edit_singleline(&mut self.rename_target);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Replace with:");
                        ui.text_edit_singleline(&mut self.rename_replacement);
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Apply Batch Rename").clicked() {
                            let selected = self.playlist.selected_indices();
                            let count = self.playlist.batch_rename(&selected, &self.rename_target, &self.rename_replacement);
                            self.set_status(&format!("Renamed {} files", count));
                            self.show_batch_rename = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_batch_rename = false;
                        }
                    });
                });
        }

        // Single Rename Modal
        if let Some(idx) = self.single_rename_idx {
            egui::Window::new("✏ Rename File")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("New file name:");
                    ui.text_edit_singleline(&mut self.single_rename_text);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let old_name = self.playlist.tracks[idx].filename.clone();
                            let count = self.playlist.batch_rename(&[idx], &old_name, &self.single_rename_text);
                            if count > 0 {
                                self.set_status("File renamed successfully");
                            }
                            self.single_rename_idx = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.single_rename_idx = None;
                        }
                    });
                });
        }

        // Shortcuts Guide Modal
        if self.show_shortcuts {
            egui::Window::new("⌨ Bobby Keyboard Shortcuts Guide")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .show(ctx, |ui| {
                    egui::Grid::new("shortcuts_grid").striped(true).show(ui, |ui| {
                        ui.label(RichText::new("Shortcut").strong());
                        ui.label(RichText::new("Action").strong());
                        ui.end_row();

                        ui.label("F4"); ui.label("Open folder select dialog"); ui.end_row();
                        ui.label("F5"); ui.label("Refresh folder for new / removed files"); ui.end_row();
                        ui.label("F8"); ui.label("Toggle parent folder path view"); ui.end_row();
                        ui.label("F2"); ui.label("Rename file or launch batch replacer"); ui.end_row();
                        ui.label("Space"); ui.label("Play / Pause audio track"); ui.end_row();
                        ui.label("Ctrl + Space"); ui.label("Reset volume to 100%"); ui.end_row();
                        ui.label("Ctrl + M"); ui.label("Cycle playmode (Normal, Single, Repeat All, Repeat 1, Shuffle)"); ui.end_row();
                        ui.label("Ctrl + Del"); ui.label("Crop playlist to selected tracks"); ui.end_row();
                        ui.label("V"); ui.label("Quick lower volume by 30%"); ui.end_row();
                        ui.label("Home"); ui.label("Jump to top of playlist"); ui.end_row();
                        ui.label("/ or F3"); ui.label("Pop up Easy Finder instant search"); ui.end_row();
                    });
                    ui.add_space(8.0);
                    if ui.button("Close").clicked() {
                        self.show_shortcuts = false;
                    }
                });
        }
    }
}

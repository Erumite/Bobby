use crate::audio::AudioPlayer;
use crate::config::{AppConfig, PlayMode};
use crate::playlist::Playlist;
use eframe::egui::{
    self, Align, Color32, Context, Key, Layout, Pos2, Rect, RichText, ScrollArea, Sense, Stroke, Vec2,
};
use rfd::FileDialog;
use std::path::PathBuf;

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

    scrubbing_pos: Option<f32>,
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
            scrubbing_pos: None,
            status_msg: "Ready".to_string(),
        };

        let initial_arg = std::env::args().nth(1).map(PathBuf::from);

        if let Some(arg_path) = initial_arg {
            if arg_path.is_file() {
                if let Some(parent) = arg_path.parent() {
                    app.playlist.load_directory(parent);
                    app.config.last_folder = Some(parent.to_path_buf());
                    app.config.save();
                    if let Some(pos) = app.playlist.tracks.iter().position(|t| t.path == arg_path) {
                        app.play_track_at(pos);
                    }
                }
            } else if arg_path.is_dir() {
                let count = app.playlist.load_directory(&arg_path);
                app.config.last_folder = Some(arg_path.clone());
                app.config.save();
                app.set_status(&format!("Loaded {} tracks", count));
                if count > 0 {
                    app.play_track_at(0);
                }
            }
        } else if let Some(ref folder) = app.config.last_folder.clone() {
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
            // Escape: Close active modals (Shortcuts, Easy Finder, Batch Rename, Single Rename)
            if i.key_pressed(Key::Escape) {
                if self.show_shortcuts {
                    self.show_shortcuts = false;
                } else if self.show_easy_finder {
                    self.show_easy_finder = false;
                } else if self.show_batch_rename {
                    self.show_batch_rename = false;
                } else if self.single_rename_idx.is_some() {
                    self.single_rename_idx = None;
                }
            }

            // Key M (without Ctrl): Toggle Mute
            if i.key_pressed(Key::M) && !i.modifiers.ctrl {
                let muted = self.audio.toggle_mute();
                let status = if muted {
                    "Audio Muted".to_string()
                } else {
                    format!("Audio Unmuted ({:.0}%)", self.config.volume * 100.0)
                };
                self.set_status(&status);
            }

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
                if self.audio.is_playing() {
                    self.audio.toggle_pause();
                } else if let Some(idx) = self.playlist.current_index {
                    self.play_track_at(idx);
                } else if !self.playlist.tracks.is_empty() {
                    self.play_track_at(0);
                }
            }
            // Ctrl + 1..0: Set volume in 10% increments
            if i.modifiers.ctrl {
                let new_vol = if i.key_pressed(Key::Num1) { Some(0.10) }
                else if i.key_pressed(Key::Num2) { Some(0.20) }
                else if i.key_pressed(Key::Num3) { Some(0.30) }
                else if i.key_pressed(Key::Num4) { Some(0.40) }
                else if i.key_pressed(Key::Num5) { Some(0.50) }
                else if i.key_pressed(Key::Num6) { Some(0.60) }
                else if i.key_pressed(Key::Num7) { Some(0.70) }
                else if i.key_pressed(Key::Num8) { Some(0.80) }
                else if i.key_pressed(Key::Num9) { Some(0.90) }
                else if i.key_pressed(Key::Num0) { Some(1.00) }
                else { None };

                if let Some(vol) = new_vol {
                    self.config.volume = vol;
                    self.audio.set_volume(vol);
                    self.config.save();
                    self.set_status(&format!("Volume set to {:.0}%", vol * 100.0));
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
            // F1: Toggle shortcuts guide
            if i.key_pressed(Key::F1) {
                self.show_shortcuts = !self.show_shortcuts;
            }
            // Delete: Remove selected file(s) from playlist (or Ctrl+Delete to crop)
            if i.key_pressed(Key::Delete) {
                if i.modifiers.ctrl {
                    self.playlist.crop_to_selected();
                    self.set_status("Playlist cropped to selected items");
                } else {
                    let count = self.playlist.remove_selected();
                    if count > 0 {
                        self.set_status(&format!("Removed {} item(s) from playlist", count));
                    }
                }
            }
            // Home / Tilde: Jump to top
            if i.key_pressed(Key::Home) {
                self.set_status("Jumped to top");
            }
            // Slash or F3: Easy Finder
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

        // Track and persist window geometry changes
        let screen_rect = ctx.screen_rect();
        let cur_w = screen_rect.width();
        let cur_h = screen_rect.height();

        if cur_w >= 200.0 && cur_h >= 200.0 {
            let mut changed = false;
            if self.config.window_width != Some(cur_w) {
                self.config.window_width = Some(cur_w);
                changed = true;
            }
            if self.config.window_height != Some(cur_h) {
                self.config.window_height = Some(cur_h);
                changed = true;
            }

            if let Some(outer_rect) = ctx.input(|i| i.viewport().outer_rect) {
                let pos = outer_rect.min;
                if self.config.window_x != Some(pos.x) {
                    self.config.window_x = Some(pos.x);
                    changed = true;
                }
                if self.config.window_y != Some(pos.y) {
                    self.config.window_y = Some(pos.y);
                    changed = true;
                }
            }

            if changed {
                self.config.save();
            }
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
                if ui.button("❓").on_hover_text("View keyboard shortcuts guide [F1]").clicked() {
                    self.show_shortcuts = true;
                }
                if ui.button(RichText::new("❤").color(Color32::from_rgb(245, 75, 75)))
                    .on_hover_text("Support Bobby on Ko-fi (https://ko-fi.com/eremite)")
                    .clicked()
                {
                    let _ = webbrowser::open("https://ko-fi.com/eremite");
                }

                ui.separator();

                ui.heading(RichText::new("Bobby").strong().color(Color32::from_rgb(80, 190, 250)));

                // Right-to-left: [LED VU Meter] [Vol Text] [Slider] [Mute Icon] | [Separator]
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (l, r) = self.audio.get_levels();
                    self.render_led_meter(ui, l, r);

                    ui.add_space(4.0);

                    // Volume text directly adjacent to VU meter
                    let vol_str = format!("{:.0}%", self.config.volume * 100.0);
                    let text_res = ui.add_sized(
                        Vec2::new(32.0, 18.0),
                        egui::Label::new(RichText::new(vol_str).small().monospace())
                    );

                    // Compact volume slider
                    let slider_res = ui.scope(|ui| {
                        ui.spacing_mut().slider_width = 55.0;
                        let mut vol = self.config.volume;
                        let res = ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false));
                        if res.changed() {
                            self.config.volume = vol;
                            self.audio.set_volume(vol);
                            self.config.save();
                        }
                        res
                    }).inner;

                    // Mute icon button
                    let vol_icon = if self.audio.is_muted() || self.config.volume == 0.0 {
                        "🔇"
                    } else if self.config.volume < 0.3 {
                        "🔉"
                    } else {
                        "🔊"
                    };

                    let mute_tooltip = if self.audio.is_muted() { "Unmute (M)" } else { "Mute (M)" };
                    let mute_res = ui.button(vol_icon).on_hover_text(mute_tooltip);
                    if mute_res.clicked() {
                        let muted = self.audio.toggle_mute();
                        let status = if muted {
                            "Audio Muted".to_string()
                        } else {
                            format!("Audio Unmuted ({:.0}%)", self.config.volume * 100.0)
                        };
                        self.set_status(&status);
                    }

                    // Mouse wheel scrolling over volume controls increments/decrements volume by 1%
                    if text_res.hovered() || slider_res.hovered() || mute_res.hovered() {
                        let scroll_y = ui.input(|i| i.raw_scroll_delta.y);
                        if scroll_y != 0.0 {
                            let delta = if scroll_y > 0.0 { 0.01 } else { -0.01 };
                            let new_vol = (self.config.volume + delta).clamp(0.0, 1.0);
                            if (new_vol - self.config.volume).abs() > 0.0001 {
                                self.config.volume = new_vol;
                                self.audio.set_volume(new_vol);
                                self.config.save();
                                self.set_status(&format!("Volume: {:.0}%", new_vol * 100.0));
                            }
                        }
                    }

                    ui.add_space(4.0);
                    ui.separator();
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Audio & Playback Toolbar with Seek Bar
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

                // Seek Bar (Time elapsed on left, dynamic slider in middle, track duration on right)
                let cur_pos = self.audio.get_pos();
                let dur = self.audio.duration();
                let total_secs = dur.map(|d| d.as_secs_f32()).unwrap_or(0.0);
                let cur_secs = cur_pos.as_secs_f32().min(total_secs);

                let display_secs = self.scrubbing_pos.unwrap_or(cur_secs);
                let elapsed_str = format_duration_str(std::time::Duration::from_secs_f32(display_secs));
                let duration_str = dur.map(format_duration_str).unwrap_or_else(|| "--:--".to_string());

                ui.label(RichText::new(elapsed_str).small().monospace().color(Color32::from_rgb(180, 220, 250)));

                if total_secs > 0.0 {
                    let mut seek_val = display_secs;
                    let (slider_res, new_val) = ui.scope(|ui| {
                        let avail_seek_w = (ui.available_width() - 55.0).max(40.0);
                        ui.spacing_mut().slider_width = avail_seek_w;
                        let res = ui.add(egui::Slider::new(&mut seek_val, 0.0..=total_secs).show_value(false));
                        (res, seek_val)
                    }).inner;

                    if slider_res.dragged() {
                        self.scrubbing_pos = Some(new_val);
                    }

                    if slider_res.drag_stopped() {
                        let target_pos = self.scrubbing_pos.take().unwrap_or(new_val);
                        self.audio.seek_to(std::time::Duration::from_secs_f32(target_pos));
                    } else if slider_res.clicked() && !slider_res.dragged() {
                        self.audio.seek_to(std::time::Duration::from_secs_f32(new_val));
                        self.scrubbing_pos = None;
                    }
                } else {
                    self.scrubbing_pos = None;
                    ui.scope(|ui| {
                        let avail_seek_w = (ui.available_width() - 55.0).max(40.0);
                        ui.spacing_mut().slider_width = avail_seek_w;
                        let mut zero = 0.0;
                        ui.add_enabled(false, egui::Slider::new(&mut zero, 0.0..=1.0).show_value(false));
                    });
                }

                ui.label(RichText::new(duration_str).small().monospace().color(Color32::GRAY));
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

            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, 22.0, visible_indices.len(), |ui, row_range| {
                    let avail_w = ui.available_width();
                    let max_line_chars = ((avail_w - 16.0) / 7.5).max(12.0) as usize;

                    for row_i in row_range {
                        if let Some(&track_idx) = visible_indices.get(row_i) {
                            let is_current = self.playlist.current_index == Some(track_idx);
                            let track = &mut self.playlist.tracks[track_idx];
                            let is_selected = track.selected;

                            // Zebra striping background for odd rows
                            let row_rect = Rect::from_min_size(
                                ui.cursor().min,
                                Vec2::new(avail_w, 22.0),
                            );

                            if row_i % 2 == 1 && !is_selected && !is_current {
                                ui.painter().rect_filled(row_rect, 0.0, Color32::from_rgb(24, 30, 39));
                            }

                            let text_color = if is_current {
                                Color32::from_rgb(100, 220, 255)
                            } else {
                                Color32::from_rgb(220, 230, 240)
                            };

                            let icon = if is_current && self.audio.is_playing() { "▶ " } else { "   " };
                            let prefix_str = format!("{}{:03}. ", icon, track_idx + 1);
                            let prefix_chars = prefix_str.chars().count();
                            let name_max_chars = max_line_chars.saturating_sub(prefix_chars);

                            let raw_name = track.display_name(show_parent);
                            let truncated_name = truncate_filename_middle(&raw_name, name_max_chars);
                            let label_text = format!("{}{}", prefix_str, truncated_name);

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
                        if ui.button("Close (Esc)").clicked() || ui.input(|i| i.key_pressed(Key::Escape)) {
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

                        ui.label("F1"); ui.label("Toggle keyboard shortcuts guide"); ui.end_row();
                        ui.label("F4"); ui.label("Open folder select dialog"); ui.end_row();
                        ui.label("F5"); ui.label("Refresh folder for new / removed files"); ui.end_row();
                        ui.label("F8"); ui.label("Toggle parent folder path view"); ui.end_row();
                        ui.label("F2"); ui.label("Rename file or launch batch replacer"); ui.end_row();
                        ui.label("Del"); ui.label("Remove selected file(s) from playlist"); ui.end_row();
                        ui.label("Ctrl + Del"); ui.label("Crop playlist to selected tracks"); ui.end_row();
                        ui.label("Space"); ui.label("Play / Pause audio track"); ui.end_row();
                        ui.label("Ctrl + 1..0"); ui.label("Set volume in 10% increments (Ctrl+1=10%, Ctrl+0=100%)"); ui.end_row();
                        ui.label("Ctrl + M"); ui.label("Cycle playmode (Normal, Single, Repeat All, Repeat 1, Shuffle)"); ui.end_row();
                        ui.label("Home"); ui.label("Jump to top of playlist"); ui.end_row();
                        ui.label("/ or F3"); ui.label("Pop up Easy Finder instant search"); ui.end_row();
                    });
                    ui.add_space(8.0);
                    if ui.button("Close (F1)").clicked() {
                        self.show_shortcuts = false;
                    }
                });
        }
    }
}

fn truncate_filename_middle(name: &str, max_chars: usize) -> String {
    let total_chars = name.chars().count();
    if total_chars <= max_chars || max_chars < 8 {
        return name.to_string();
    }

    if let Some(dot_idx) = name.rfind('.') {
        let ext = &name[dot_idx..];
        let stem = &name[..dot_idx];
        let ext_chars_len = ext.chars().count();

        if ext_chars_len + 3 < max_chars {
            let available_stem = max_chars - 3 - ext_chars_len;
            let stem_chars: Vec<char> = stem.chars().collect();
            if stem_chars.len() > available_stem {
                let prefix: String = stem_chars[..available_stem].iter().collect();
                return format!("{}...{}", prefix, ext);
            }
        }
    }

    let chars: Vec<char> = name.chars().collect();
    let prefix_len = max_chars.saturating_sub(3);
    format!("{}...", chars[..prefix_len].iter().collect::<String>())
}

fn format_duration_str(dur: std::time::Duration) -> String {
    let secs = dur.as_secs();
    let mins = secs / 60;
    let remainder = secs % 60;
    format!("{}:{:02}", mins, remainder)
}


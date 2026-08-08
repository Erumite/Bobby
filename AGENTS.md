# AGENTS.md — Developer & AI Agent Reference for Bobby

## Overview
Bobby is a lightweight, fast, directory-based Linux audio player built with Rust (`eframe`/`egui`, `rodio`).

## Critical Rules for Agents
1. **Tool Commands**: Prefix all shell commands with `rtk` (e.g. `rtk cargo build`, `rtk git status`).
2. **Environment Context**: Linux environment requiring Homebrew paths for compilation:
   ```bash
   export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
   export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/lib/pkgconfig:$PKG_CONFIG_PATH"
   ```
3. **Atomic Installer**: Use `rtk ./install.sh` to compile release binary and update `~/.local/bin/bobby`.

## Architecture & Codebase Map
- `src/main.rs`: Entry point, window initialization, application icon loading, and restoring window geometry from config.
- `src/ui.rs`: GUI rendering (`eframe`/`egui`), header toolbar, 14-LED peak VU meter, interactive seek bar (mouse-release scrubbing), volume controls (1% mouse-wheel scrolling), monospace playlist with pixel-accurate middle truncation (`truncate_filename_middle`), 100% full-width left-aligned rows with zebra striping, and modals (`F1` help, `F2` batch rename, `/` or `F3` easy finder).
- `src/audio.rs`: Audio engine wrapped around `rodio`. Position seeking (`seek_to`), VU level calculation, and `TrackAudioInfo` (extracting container format, calculating Kbps bitrate, and channels).
- `src/playlist.rs`: Tagless filesystem scanner (`walkdir`), search filtering, track selection, batch filename replacement, and play modes (`Normal`, `Single`, `Repeat`, `RepeatOne`, `Shuffle`).
- `src/config.rs`: Config persistence (`~/.config/bobby/bobby_config.json`) saving volume, playmode, last folder, and window geometry (`window_width`, `window_height`, `window_x`, `window_y`).
- `install.sh`: Path-agnostic atomic installer script (`bobby.tmp` -> `bobby`).
- `bobby.desktop`: Freedesktop MIME integration for Linux file managers (Dolphin/KDE/GNOME).

## Key Development Commands
```bash
# Compile debug build
rtk cargo build

# Run application locally
rtk cargo run

# Build release and install locally
rtk ./install.sh
```

## Key Technical Patterns
- **Seek Scrubbing**: Live position tracked via `scrubbing_pos: Option<f32>`. Calls `audio.seek_to()` ONCE on `drag_stopped` or click to avoid audio buffer stutter.
- **Volume Control**: Wrapped in fixed 36px width slot to prevent horizontal layout jitter. Mouse wheel scroll over volume controls increments/decrements volume in 1% steps (`±0.01`).
- **Playlist Truncation**: Monospace font (`FontId::monospace(13.0)`). `truncate_filename_middle` uses binary search on exact font pixel width to keep file stem prefix, `...`, and extension (`stem...ext`).
- **Full-Width Rows**: Rows use `allocate_exact_size` with `Align2::LEFT_CENTER` text anchoring for 100% left-aligned track titles and full-width zebra striping (`Color32::from_rgb(24, 30, 39)`).
- **Audio Stats**: `TrackAudioInfo::display_string()` formats container format, bitrate, and channels (e.g. `MP3 128 Kbps Stereo` or `WAV 1411 Kbps Mono`).

## Release Workflow & Version Bumping
When asked to create a release:
1. **Bump Version**: Update `version = "X.Y.Z"` in `Cargo.toml`.
2. **Stage & Commit**:
   ```bash
   rtk git add .
   rtk git commit -m "Release vX.Y.Z: <summary of features/fixes>"
   ```
3. **Push Branch & Tag**:
   ```bash
   rtk git push origin main
   rtk git tag -a vX.Y.Z -m "Release vX.Y.Z"
   rtk git push origin vX.Y.Z
   ```
4. **CI/CD Build**: Pushing tag `vX.Y.Z` automatically triggers GitHub Actions (`.github/workflows/release.yml`) to compile release binaries and publish artifacts to GitHub Releases.

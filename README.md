# Bobby — No-Nonsense Linux Audio Player

Bobby is a lightweight, fast, directory-based audio player for Linux inspired by [Billy](https://github.com/zQueal/Billy).

---

## Features

- ⚡ **Instant Directory Loading**: Loads thousands of songs in seconds without tag pre-scanning delays.
- 🎵 **Format Support**: Plays MP3, FLAC, OGG Vorbis, WAV, M4A, AAC, and Opus natively.
- 🎛️ **Retro 14-LED Peak Level Meter**: Real-time dual-channel peak VU meters.
- ⌨️ **100% Keyboard Driven**: Full keyboard shortcuts for playback, search, file operations, and navigation.
- 🔍 **Easy Finder (`/` or `F3`)**: Instant search and filter overlay across loaded directories.
- ✏️ **Batch File Replacer (`F2`)**: Multiple file filename replacer directly inside your playlist.
- 🔊 **Quick Volume Attenuation (`V`)**: Instantly drop volume by 30% for incoming calls.
- 📁 **Parent Folder View Toggle (`F8`)**: Easily display parent subfolder names alongside track filenames.

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `F4` | Open directory selection dialog |
| `F5` | Refresh current directory for new/removed files |
| `F8` | Toggle parent subfolder view mode |
| `F2` | Rename single track or launch batch file replacer |
| `Space` | Play / Pause playback |
| `Ctrl + Space` | Reset volume to 100% |
| `Ctrl + M` | Cycle playmodes (Normal, Shuffle, Repeat All, Repeat 1) |
| `Ctrl + Del` | Crop playlist to selected items |
| `V` | Quick lower volume by 30% |
| `Home` | Jump to top of playlist |
| `/` or `F3` | Open Easy Finder instant search modal |

---

## Building & Running

### Requirements
- Rust toolchain (`cargo`, `rustc`)
- `alsa-lib` development headers (e.g. `brew install alsa-lib` or `apt install libasound2-dev`)

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run --release
```

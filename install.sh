#!/usr/bin/env bash
set -e

# Detect script directory and change working directory to project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building Bobby in release mode..."
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/lib/pkgconfig:$PKG_CONFIG_PATH"
# Touch main.rs to ensure Cargo recompiles the embedded assets/bobby.png icon into the binary
touch "$SCRIPT_DIR/src/main.rs"
cargo build --release

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_256_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
ICON_512_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"
PIXMAPS_DIR="$HOME/.local/share/pixmaps"

mkdir -p "$BIN_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$ICON_256_DIR"
mkdir -p "$ICON_512_DIR"
mkdir -p "$PIXMAPS_DIR"

echo "Installing binary to $BIN_DIR/bobby..."
cp "$SCRIPT_DIR/target/release/bobby" "$BIN_DIR/bobby.tmp"
chmod +x "$BIN_DIR/bobby.tmp"
mv -f "$BIN_DIR/bobby.tmp" "$BIN_DIR/bobby"

echo "Installing icon files..."
cp "$SCRIPT_DIR/assets/bobby.png" "$ICON_256_DIR/bobby.png"
cp "$SCRIPT_DIR/assets/bobby.png" "$ICON_512_DIR/bobby.png"
cp "$SCRIPT_DIR/assets/bobby.png" "$PIXMAPS_DIR/bobby.png"
# Remove legacy invalid ico from scalable dir if present
rm -f "$HOME/.local/share/icons/hicolor/scalable/apps/bobby.ico"

echo "Installing desktop entry to $APP_DIR/bobby.desktop..."
cp "$SCRIPT_DIR/bobby.desktop" "$APP_DIR/bobby.desktop"
touch "$APP_DIR/bobby.desktop"

echo "Updating desktop and icon caches..."
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental 2>/dev/null || true
elif command -v kbuildsycoca5 >/dev/null 2>&1; then
    kbuildsycoca5 --noincremental 2>/dev/null || true
fi

echo "Successfully installed Bobby! You can now select Bobby as the default file handler in Dolphin/KDE."

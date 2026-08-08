#!/usr/bin/env bash
set -e

# Detect script directory and change working directory to project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building Bobby in release mode..."
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/lib/pkgconfig:$PKG_CONFIG_PATH"
cargo build --release

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_PNG_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
ICON_ICO_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

mkdir -p "$BIN_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$ICON_PNG_DIR"
mkdir -p "$ICON_ICO_DIR"

echo "Installing binary to $BIN_DIR/bobby..."
cp "$SCRIPT_DIR/target/release/bobby" "$BIN_DIR/bobby.tmp"
chmod +x "$BIN_DIR/bobby.tmp"
mv -f "$BIN_DIR/bobby.tmp" "$BIN_DIR/bobby"

echo "Installing icon files..."
cp "$SCRIPT_DIR/assets/bobby.png" "$ICON_PNG_DIR/bobby.png"
cp "$SCRIPT_DIR/assets/bobby.ico" "$ICON_ICO_DIR/bobby.ico"

echo "Installing desktop entry to $APP_DIR/bobby.desktop..."
cp "$SCRIPT_DIR/bobby.desktop" "$APP_DIR/bobby.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi

echo "Successfully installed Bobby! You can now select Bobby as the default file handler in Dolphin/KDE."

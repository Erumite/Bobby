#!/usr/bin/env bash
set -e

echo "Building Bobby in release mode..."
export PATH="/home/linuxbrew/.linuxbrew/bin:$PATH"
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/lib/pkgconfig:$PKG_CONFIG_PATH"
cargo build --release

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"

mkdir -p "$BIN_DIR"
mkdir -p "$APP_DIR"

echo "Installing binary to $BIN_DIR/bobby..."
cp target/release/bobby "$BIN_DIR/bobby"
chmod +x "$BIN_DIR/bobby"

echo "Installing desktop entry to $APP_DIR/bobby.desktop..."
cp bobby.desktop "$APP_DIR/bobby.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

echo "Successfully installed Bobby! You can now select Bobby as the default file handler in Dolphin/KDE."

#!/bin/bash
# Install script for Niandra music tracker

set -e

echo "Building Niandra..."
cargo build --release --features full

PREFIX="${PREFIX:-$HOME/.local}"
SYSTEMD_USER_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
AUTOSTART_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"
APPLICATIONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"

echo "Installing Niandra to $PREFIX..."

# Create directories
mkdir -p "$PREFIX/bin"
mkdir -p "$SYSTEMD_USER_DIR"
mkdir -p "$AUTOSTART_DIR"
mkdir -p "$APPLICATIONS_DIR"
mkdir -p "$ICONS_DIR"

# Install binaries
echo "Installing binaries..."
install -m755 target/release/niandra "$PREFIX/bin/"
install -m755 target/release/music-tracker "$PREFIX/bin/"

# Install desktop files
echo "Installing desktop entries..."
install -m644 data/io.github.tombleher.Niandra.desktop "$APPLICATIONS_DIR/"
install -m644 data/io.github.tombleher.Niandra.Tracker.desktop "$AUTOSTART_DIR/"

# Install icon
if [ -f data/icons/hicolor/scalable/apps/io.github.tombleher.Niandra.svg ]; then
    echo "Installing icon..."
    install -m644 data/icons/hicolor/scalable/apps/io.github.tombleher.Niandra.svg "$ICONS_DIR/"
fi

# Install and enable systemd service
echo "Installing systemd user service..."
install -m644 music-tracker.service "$SYSTEMD_USER_DIR/"

echo "Enabling music-tracker service..."
systemctl --user daemon-reload
systemctl --user enable music-tracker.service
systemctl --user start music-tracker.service

echo ""
echo "Installation complete!"
echo ""
echo "The music-tracker daemon is now running and will auto-start on login."
echo "You can launch the GUI with: niandra"
echo ""
echo "To check tracker status: systemctl --user status music-tracker"
echo "To view logs: journalctl --user -u music-tracker -f"

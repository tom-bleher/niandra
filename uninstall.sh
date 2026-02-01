#!/bin/bash
# Uninstall script for Niandra music tracker

set -e

PREFIX="${PREFIX:-$HOME/.local}"
SYSTEMD_USER_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
AUTOSTART_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"
APPLICATIONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICONS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/music-analytics"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/music-analytics"

echo "Uninstalling Niandra..."

# Stop and disable systemd service
if systemctl --user is-active --quiet music-tracker.service 2>/dev/null; then
    echo "Stopping music-tracker service..."
    systemctl --user stop music-tracker.service
fi

if systemctl --user is-enabled --quiet music-tracker.service 2>/dev/null; then
    echo "Disabling music-tracker service..."
    systemctl --user disable music-tracker.service
fi

# Remove binaries
echo "Removing binaries..."
rm -f "$PREFIX/bin/niandra"
rm -f "$PREFIX/bin/music-tracker"

# Remove desktop files
echo "Removing desktop entries..."
rm -f "$APPLICATIONS_DIR/io.github.tombleher.Niandra.desktop"
rm -f "$AUTOSTART_DIR/io.github.tombleher.Niandra.Tracker.desktop"

# Remove icon
rm -f "$ICONS_DIR/io.github.tombleher.Niandra.svg"

# Remove systemd service
rm -f "$SYSTEMD_USER_DIR/music-tracker.service"
systemctl --user daemon-reload 2>/dev/null || true

echo ""
echo "Niandra has been uninstalled."
echo ""

# Ask about data removal
if [ -d "$DATA_DIR" ] || [ -d "$CONFIG_DIR" ]; then
    echo "Your listening data and config are still in:"
    [ -d "$DATA_DIR" ] && echo "  - $DATA_DIR"
    [ -d "$CONFIG_DIR" ] && echo "  - $CONFIG_DIR"
    echo ""
    read -p "Remove data and config? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$DATA_DIR"
        rm -rf "$CONFIG_DIR"
        echo "Data and config removed."
    else
        echo "Data and config preserved."
    fi
fi

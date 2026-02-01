#!/bin/bash
# Take professional screenshots of Niandra for Flathub/README
# Creates PNG with transparent background and window shadow

set -e

SCREENSHOT_DIR="/home/tom/niandra/data/screenshots"
mkdir -p "$SCREENSHOT_DIR"

echo "Taking professional screenshot of Niandra..."
echo ""
echo "Instructions:"
echo "1. Position the Niandra window where you want it"
echo "2. Make sure it shows the view you want (Artists, Overview, etc.)"
echo "3. Resize the window to a good size (~800-900px wide)"
echo ""
read -p "Press Enter when ready to take screenshot..."

# Use GNOME's screenshot portal for best quality with shadow
# This triggers the native GNOME screenshot UI
if command -v gnome-screenshot &> /dev/null; then
    echo "Click on the Niandra window to capture it..."
    gnome-screenshot -w -f "$SCREENSHOT_DIR/screenshot-$(date +%s).png"
    echo "Screenshot saved to $SCREENSHOT_DIR/"
else
    echo "gnome-screenshot not found. Using grim..."
    # Fallback to grim + slurp for Wayland
    grim -g "$(slurp)" "$SCREENSHOT_DIR/screenshot-$(date +%s).png"
fi

echo ""
echo "Done! Check $SCREENSHOT_DIR/"
ls -la "$SCREENSHOT_DIR/"

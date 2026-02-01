<p align="center">
  <img src="data/icons/hicolor/scalable/apps/io.github.tombleher.Niandra.svg" alt="Niandra" width="128">
</p>

<h1 align="center">Niandra</h1>

<p align="center">A small and simple music listening tracker for GNOME.</p>

<p align="center">
  <img src="data/screenshots/artists.png" alt="Niandra showing top artists" width="600">
</p>

Niandra tracks what you listen to from any MPRIS-compatible music player on your Linux desktop. It stores your listening history locally and lets you explore your music habits with a clean, native interface.

## Features

- Displays basic listening analytics
- Works with any local player: Amberol, GNOME Music, Spotify, Lollypop, and more
- Background tracker auto-starts on login and when launching the GUI

Note: Spotify tracking is off by default.

## Installation

### Dependencies

Debian/Ubuntu:
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

Fedora/RHEL:
```bash
sudo dnf install gtk4-devel libadwaita-devel
```

### Build and install

```bash
git clone https://github.com/tom-bleher/niandra
cd niandra
cargo build --release --features full
./install.sh
```

The install script will:
- Install binaries to `~/.local/bin/`
- Set up the systemd user service (auto-starts on login)
- Install desktop entries and icons

### Uninstall

```bash
# Stop and disable the tracker
systemctl --user disable --now music-tracker

# Remove binaries and files
rm ~/.local/bin/niandra ~/.local/bin/music-tracker
rm ~/.config/systemd/user/music-tracker.service
rm ~/.local/share/applications/io.github.tombleher.Niandra.desktop
rm ~/.config/autostart/io.github.tombleher.Niandra.Tracker.desktop

# Optional: remove data and config
rm -rf ~/.local/share/music-analytics
rm -rf ~/.config/music-analytics
```

## Why "Niandra"?

The name comes from [*Niandra LaDes and Usually Just a T-Shirt*](https://en.wikipedia.org/wiki/Niandra_LaDes_and_Usually_Just_a_T-Shirt), John Frusciante's 1994 solo album. It was one of my early introductions to music.

## Contributing

Contributions are welcome! Whether it's bug reports, feature suggestions, or pull requests: all are appreciated. Please be kind and respectful in all interactions.

## Disclaimer

This project was largely vibe coded with [Claude Opus 4.5](https://www.anthropic.com/claude).

## License

Niandra is released under the [MIT License](LICENSE).

Copyright 2026 Tom Bleher

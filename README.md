<p align="center">
  <img src="data/icons/hicolor/scalable/apps/io.github.tombleher.Niandra.svg" alt="Niandra" width="128">
</p>

<h1 align="center">Niandra</h1>

<p align="center">A small and simple music listening tracker for GNOME.</p>

<p align="center">
  <img src="data/screenshots/artists.png" alt="Niandra showing top artists" width="600">
</p>

Niandra quietly runs in the background, tracking what you listen to from any MPRIS-compatible music player. Your listening history stays local—nothing is uploaded anywhere.

## Features

- **Top artists, albums, and tracks** — See what you listen to most
- **Listening insights** — Streaks, night owl score, skip rate
- **Hourly heatmap** — When you listen throughout the day
- **Time filters** — View stats for the past week, month, year, or all time
- **Works with any player** — Amberol, GNOME Music, Spotify, Lollypop, and more

## Installation

### Dependencies

Fedora/RHEL:
```bash
sudo dnf install gtk4-devel libadwaita-devel
```

Debian/Ubuntu:
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

### Build from source

```bash
git clone https://github.com/tom-bleher/niandra
cd niandra
cargo build --release --features gui

# Install binaries
cp target/release/niandra ~/.local/bin/
cp target/release/music-tracker ~/.local/bin/

# Install desktop file
cp data/io.github.tombleher.Niandra.desktop ~/.local/share/applications/
```

### Start the tracker

```bash
# Run the background tracker
music-tracker

# Or install as a systemd service
cp music-tracker.service ~/.config/systemd/user/
systemctl --user enable --now music-tracker
```

## Why "Niandra"?

Named after [*Niandra LaDes and Usually Just a T-Shirt*](https://en.wikipedia.org/wiki/Niandra_LaDes_and_Usually_Just_a_T-Shirt), John Frusciante's 1994 solo album—one of my early introductions to music.

## Contributing

Contributions welcome! Bug reports, feature ideas, and pull requests are all appreciated.

## License

[MIT](LICENSE) — Copyright 2025 Tom Bleher

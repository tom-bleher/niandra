# Music Analytics (Rust)

A high-performance personal music listening analytics system for Linux, written in Rust.

Monitors MPRIS-compatible media players via D-Bus and stores rich listening data in a local SQLite database. Generates Spotify Wrapped-style statistics.

## Features

### Tracking
- **Event-driven MPRIS monitoring** - Uses async D-Bus signals (not polling) for efficient, instant response
- **Rich metadata capture** - Artist, album, genre, BPM, release date, MusicBrainz IDs, and more
- **Seek behavior tracking** - Counts seeks, detects intro skipping, tracks forward/backward seeking
- **Volume tracking** - Captures PulseAudio/PipeWire volume levels
- **Context awareness** - Records time of day, day of week, active window, screen state, power state
- **Local-only mode** - Optionally filter out streaming services, only track local files
- **Smart scrobbling** - 30s minimum, 50% or 4-minute threshold (Last.fm compatible)

### Database
- **Local SQLite** - Fast, embedded database via libSQL with no external dependencies
- **Automatic schema migrations** - Database upgrades handled automatically

### Analytics
- Top artists, albums, and tracks with play counts and listening time
- Listening streaks (current and longest)
- Night owl score (midnight-6am listening percentage)
- Skip rate and completion metrics
- Hourly listening heatmap
- Genre and decade breakdowns

### Optional Features
- `pulse` - PulseAudio/PipeWire volume tracking (default)
- `tui` - Terminal UI for live stats (coming soon)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/tom-bleher/music-analytics-rs
cd music-analytics-rs

# Build release binary
cargo build --release

# Install binaries
sudo cp target/release/music-analytics /usr/local/bin/
sudo cp target/release/music-tracker /usr/local/bin/
sudo cp target/release/music-stats /usr/local/bin/

# Or install to user directory
mkdir -p ~/.local/bin
cp target/release/music-* ~/.local/bin/
```

### Systemd Service

```bash
# Copy service file
cp music-tracker.service ~/.config/systemd/user/

# Enable and start
systemctl --user daemon-reload
systemctl --user enable --now music-tracker
```

## Usage

### Combined CLI

```bash
# Show stats (default: current year)
music-analytics

# Start tracker daemon
music-analytics track

# Show stats for different periods
music-analytics stats --week
music-analytics stats --month
music-analytics stats --year 2025
music-analytics stats --all-time

# Configuration
music-analytics config --init   # Create default config
music-analytics config --show   # Show current config

# Database operations
music-analytics db --info       # Show database info
```

### Standalone Binaries

```bash
# Tracker daemon
music-tracker
music-tracker --verbose

# Stats viewer
music-stats
music-stats --week
music-stats --deep              # Advanced analytics
music-stats --full              # Everything
```

## Configuration

Configuration is stored in `~/.config/music-analytics/config.toml`:

```toml
[general]
log_level = "info"
# data_dir = "/custom/path"  # Default: ~/.local/share/music-analytics

[database]
# path = "/custom/path/listens.db"  # Default: ~/.local/share/music-analytics/listens.db

[tracking]
min_play_seconds = 30
min_play_percent = 0.5
local_only = true           # Only track local files
track_seeks = true          # Track seek behavior
track_volume = true         # Track volume levels
track_context = true        # Track time/activity context
idle_timeout_seconds = 30   # Exit after no players for this long

[players]
# Whitelist specific players (empty = all)
whitelist = []

# Blacklist players
blacklist = []

# Known local-only players
local_only_players = [
    "io.bassi.Amberol",
    "org.gnome.Lollypop",
    "org.gnome.Music",
    "audacious",
    "deadbeef",
    "clementine",
    "strawberry",
    "rhythmbox",
]
```

### Environment Variables

```bash
# Override config values
export MUSIC_ANALYTICS_LOG_LEVEL=debug
```

## Architecture

```
music-analytics-rs/
├── src/
│   ├── lib.rs              # Library exports
│   ├── main.rs             # Combined CLI entry point
│   ├── config.rs           # TOML configuration
│   ├── error.rs            # Error types
│   ├── track.rs            # Track metadata and state
│   ├── context.rs          # Listening context capture
│   ├── db/
│   │   ├── mod.rs          # Database wrapper (libSQL/Turso)
│   │   ├── schema.rs       # Schema initialization
│   │   └── queries.rs      # Query implementations
│   ├── mpris/
│   │   ├── mod.rs          # MPRIS module exports
│   │   ├── player.rs       # MPRIS monitor (async D-Bus)
│   │   └── metadata.rs     # Metadata parsing
│   ├── analytics/
│   │   └── mod.rs          # Analytics functions
│   └── bin/
│       ├── tracker.rs      # Standalone tracker daemon
│       └── stats.rs        # Standalone stats viewer
├── Cargo.toml
├── README.md
└── music-tracker.service   # Systemd unit file
```

## Comparison with Python Version

| Feature | Python | Rust |
|---------|--------|------|
| D-Bus Architecture | Event-driven (dbus_next) | Event-driven (zbus) |
| Database | SQLite (sqlite3) | libSQL (SQLite) |
| Performance | Good | Excellent |
| Memory Usage | ~30MB | ~5MB |
| Binary Size | N/A (interpreted) | ~3MB |
| Startup Time | ~200ms | ~10ms |

## Supported Players

Any MPRIS-compatible player, including:
- Amberol
- GNOME Music
- Lollypop
- Rhythmbox
- Audacious
- DeaDBeeF
- Clementine
- Strawberry
- VLC
- mpv
- And many more...

## License

MIT

## Contributing

Contributions welcome! Please open an issue or PR on GitHub.

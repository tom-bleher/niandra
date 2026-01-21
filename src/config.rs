//! Configuration management for music-analytics

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{Error, Result};

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,

    /// Database settings
    pub database: DatabaseConfig,

    /// Tracking settings
    pub tracking: TrackingConfig,

    /// Player filtering
    pub players: PlayerConfig,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,

    /// Data directory (default: ~/.local/share/music-analytics)
    pub data_dir: Option<PathBuf>,
}

/// Database configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    /// Database path (file path for local SQLite)
    pub path: Option<String>,
}

/// Tracking behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrackingConfig {
    /// Minimum seconds to count as a play
    pub min_play_seconds: u64,

    /// Minimum percentage of track to count as a play (0.0-1.0)
    pub min_play_percent: f64,

    /// Only track local files (ignore streaming)
    pub local_only: bool,

    /// Track seek behavior
    pub track_seeks: bool,

    /// Track volume levels
    pub track_volume: bool,

    /// Track listening context (time, active window, etc.)
    pub track_context: bool,

    /// Idle timeout in seconds (0 = never exit)
    pub idle_timeout_seconds: u64,
}

/// Player filtering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    /// Whitelist of player names (empty = all players)
    pub whitelist: Vec<String>,

    /// Blacklist of player names
    pub blacklist: Vec<String>,

    /// Known local-only players (these don't stream)
    pub local_only_players: Vec<String>,
}

// Default implementations

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            data_dir: None,
        }
    }
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            min_play_seconds: 30,
            min_play_percent: 0.5,
            local_only: true,
            track_seeks: true,
            track_volume: true,
            track_context: true,
            idle_timeout_seconds: 30,
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            whitelist: Vec::new(),
            blacklist: Vec::new(),
            local_only_players: vec![
                "io.bassi.Amberol".to_string(),
                "org.gnome.Lollypop".to_string(),
                "org.gnome.Music".to_string(),
                "audacious".to_string(),
                "deadbeef".to_string(),
                "quodlibet".to_string(),
                "clementine".to_string(),
                "strawberry".to_string(),
                "rhythmbox".to_string(),
                "elisa".to_string(),
                "sayonara".to_string(),
                "cantata".to_string(),
            ],
        }
    }
}

impl Config {
    /// Load configuration from the default location
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &std::path::Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to the default location
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;
        Ok(())
    }

    /// Get the default configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| Error::config("Could not determine config directory"))?;
        Ok(config_dir.join("music-analytics").join("config.toml"))
    }

    /// Get the data directory
    pub fn data_dir(&self) -> Result<PathBuf> {
        if let Some(ref dir) = self.general.data_dir {
            Ok(dir.clone())
        } else {
            let data_dir = dirs::data_local_dir()
                .ok_or_else(|| Error::config("Could not determine data directory"))?;
            Ok(data_dir.join("music-analytics"))
        }
    }

    /// Get the database path
    pub fn database_path(&self) -> Result<PathBuf> {
        if let Some(ref path) = self.database.path {
            return Ok(PathBuf::from(path));
        }
        Ok(self.data_dir()?.join("listens.db"))
    }

    /// Validate configuration values.
    ///
    /// Call this after loading to ensure all values are within acceptable ranges.
    pub fn validate(&self) -> Result<()> {
        // Validate min_play_percent is between 0.0 and 1.0
        if !(0.0..=1.0).contains(&self.tracking.min_play_percent) {
            return Err(Error::config(format!(
                "min_play_percent must be between 0.0 and 1.0, got {}",
                self.tracking.min_play_percent
            )));
        }

        // Validate min_play_seconds is reasonable
        if self.tracking.min_play_seconds > 3600 {
            return Err(Error::config(format!(
                "min_play_seconds should not exceed 3600 (1 hour), got {}",
                self.tracking.min_play_seconds
            )));
        }

        // Validate log_level is a known level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.general.log_level.to_lowercase().as_str()) {
            return Err(Error::config(format!(
                "log_level must be one of {:?}, got '{}'",
                valid_levels, self.general.log_level
            )));
        }

        Ok(())
    }
}

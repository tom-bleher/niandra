//! # Music Analytics
//!
//! Personal music listening analytics for Linux.
//!
//! This crate provides:
//! - MPRIS D-Bus monitoring for detecting music playback
//! - Local SQLite/Turso database for storing listening history
//! - Rich metadata tracking (seek behavior, volume, context)
//! - Analytics and statistics generation
//!
//! ## Features
//!
//! - `pulse` - PulseAudio/PipeWire volume tracking (default)
//! - `tui` - Terminal UI for viewing stats
//! - `scrobble` - Last.fm/ListenBrainz scrobbling support
//! - `full` - All features enabled

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub(crate) mod analytics;
pub mod config;
pub(crate) mod context;
pub mod db;
pub mod display;
pub mod error;
pub mod mpris;
pub(crate) mod track;

pub use config::Config;
pub use db::Database;
pub use error::{Error, Result};

/// Application version from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "music-analytics";

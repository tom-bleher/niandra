//! Database module using DuckDB
//!
//! High-performance OLAP database for music listening analytics.
//! DuckDB provides faster analytical queries compared to SQLite.

mod filter;
mod queries;
mod schema;

pub use filter::DateFilter;

use duckdb::Connection;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::DatabaseConfig;
use crate::context::ListeningContext;
use crate::error::Result;
use crate::track::TrackState;

/// Database wrapper for music analytics using DuckDB
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    ///
    /// # Arguments
    /// * `config` - Database configuration
    /// * `data_dir` - Default data directory for local DB
    pub async fn new(config: &DatabaseConfig, data_dir: &Path) -> Result<Self> {
        let db_path = if let Some(ref path) = config.path {
            let path = Path::new(path);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            path.to_path_buf()
        } else {
            std::fs::create_dir_all(data_dir)?;
            data_dir.join("listens.duckdb")
        };

        // Open DuckDB connection (synchronous, so we use spawn_blocking)
        let db_path_str = db_path.to_string_lossy().to_string();
        let conn = tokio::task::spawn_blocking(move || Connection::open(&db_path_str))
            .await
            .map_err(|e| crate::error::Error::other(e.to_string()))??;

        let instance = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        // Initialize schema
        instance.init().await?;

        Ok(instance)
    }

    /// Initialize database schema
    async fn init(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        // Run schema initialization synchronously
        schema::init_schema(&conn)?;
        Ok(())
    }

    /// Log a completed play to the database
    pub async fn log_play(&self, state: &TrackState, context: &ListeningContext) -> Result<()> {
        let conn = self.conn.lock().await;
        queries::insert_play(&conn, state, context)
    }

    /// Get total play count
    pub async fn get_play_count(&self) -> Result<i64> {
        let conn = self.conn.lock().await;
        queries::get_play_count(&conn)
    }

    /// Get top artists by play count
    pub async fn get_top_artists(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<ArtistStats>> {
        // Clone the date strings to avoid lifetime issues
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        queries::get_top_artists(&conn, start.as_deref(), end.as_deref(), limit)
    }

    /// Get top albums by play count
    pub async fn get_top_albums(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<AlbumStats>> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        queries::get_top_albums(&conn, start.as_deref(), end.as_deref(), limit)
    }

    /// Get top tracks by play count
    pub async fn get_top_tracks(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<TrackStats>> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        queries::get_top_tracks(&conn, start.as_deref(), end.as_deref(), limit)
    }

    /// Get listening stats overview
    pub async fn get_overview_stats(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<OverviewStats> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        queries::get_overview_stats(&conn, start.as_deref(), end.as_deref())
    }

    // The following methods are public API for binaries (GUI, music-stats)
    // but not used within the library itself.

    /// Get listening streaks (current and longest)
    #[allow(dead_code)]
    pub async fn get_listening_streaks(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<crate::analytics::StreakInfo> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        crate::analytics::get_listening_streaks(&conn, start.as_deref(), end.as_deref())
    }

    /// Get night owl score (percentage of plays between midnight and 6am)
    #[allow(dead_code)]
    pub async fn get_night_owl_score(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<crate::analytics::NightOwlScore> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        crate::analytics::get_night_owl_score(&conn, start.as_deref(), end.as_deref())
    }

    /// Get hourly listening heatmap
    #[allow(dead_code)]
    pub async fn get_hourly_heatmap(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<crate::analytics::HourlyHeatmap> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        crate::analytics::get_hourly_heatmap(&conn, start.as_deref(), end.as_deref())
    }

    /// Get genre statistics
    #[allow(dead_code)]
    pub async fn get_genre_stats(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<(String, i64, i64)>> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        crate::analytics::get_genre_stats(&conn, start.as_deref(), end.as_deref(), limit)
    }

    /// Get skip rate (percentage of plays with less than 50% completion)
    #[allow(dead_code)]
    pub async fn get_skip_rate(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<(i64, i64, f64)> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        crate::analytics::get_skip_rate(&conn, start.as_deref(), end.as_deref())
    }

    /// Get daily contribution data for the contribution graph
    #[allow(dead_code)]
    pub async fn get_daily_contributions(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<crate::analytics::DailyContribution> {
        let start = start_date.map(String::from);
        let end = end_date.map(String::from);
        let conn = self.conn.lock().await;
        crate::analytics::get_daily_contributions(&conn, start.as_deref(), end.as_deref())
    }
}

/// Aggregated statistics for an artist.
#[derive(Debug, Clone)]
pub struct ArtistStats {
    /// Artist name.
    pub artist: String,
    /// Number of plays.
    pub play_count: i64,
    /// Total listening time in milliseconds.
    pub total_ms: i64,
}

/// Aggregated statistics for an album.
#[derive(Debug, Clone)]
pub struct AlbumStats {
    /// Album name.
    pub album: String,
    /// Primary artist name, if available.
    pub artist: Option<String>,
    /// Number of plays.
    pub play_count: i64,
    /// Total listening time in milliseconds.
    pub total_ms: i64,
    /// Album art URL, if available. Used by GUI for displaying artwork.
    #[allow(dead_code)]
    pub art_url: Option<String>,
}

/// Aggregated statistics for a track.
#[derive(Debug, Clone)]
pub struct TrackStats {
    /// Track title.
    pub title: String,
    /// Artist name, if available.
    pub artist: Option<String>,
    /// Number of plays.
    pub play_count: i64,
    /// Total listening time in milliseconds.
    pub total_ms: i64,
    /// Album art URL, if available. Used by GUI for displaying artwork.
    #[allow(dead_code)]
    pub art_url: Option<String>,
}

/// Overview statistics for a time period.
#[derive(Debug, Clone, Default)]
pub struct OverviewStats {
    /// Total number of plays.
    pub total_plays: i64,
    /// Total listening time in milliseconds.
    pub total_ms: i64,
    /// Count of unique artists.
    pub unique_artists: i64,
    /// Count of unique albums.
    pub unique_albums: i64,
    /// Count of unique tracks.
    pub unique_tracks: i64,
}

//! Database module using libSQL
//!
//! Local SQLite database for music listening analytics.

mod filter;
mod queries;
mod schema;

pub use filter::DateFilter;

use libsql::{Builder, Connection, Database as LibsqlDatabase};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::DatabaseConfig;
use crate::context::ListeningContext;
use crate::error::Result;
use crate::track::TrackState;

/// Database wrapper for music analytics
#[derive(Clone)]
pub struct Database {
    db: Arc<LibsqlDatabase>,
    conn: Arc<RwLock<Option<Connection>>>,
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
            data_dir.join("listens.db")
        };

        let db = Builder::new_local(db_path.to_string_lossy().to_string())
            .build()
            .await?;

        let instance = Self {
            db: Arc::new(db),
            conn: Arc::new(RwLock::new(None)),
        };

        // Initialize schema
        instance.init().await?;

        Ok(instance)
    }

    /// Initialize database schema
    async fn init(&self) -> Result<()> {
        let conn = self.connection().await?;
        schema::init_schema(&conn).await?;
        Ok(())
    }

    /// Get a database connection.
    ///
    /// Uses double-check locking pattern to avoid holding write lock across
    /// potentially blocking operations.
    async fn connection(&self) -> Result<Connection> {
        // Fast path: check with read lock first
        {
            let guard = self.conn.read().await;
            if let Some(ref conn) = *guard {
                return Ok(conn.clone());
            }
        }
        // Read lock released before connect

        // Create connection outside of lock
        let new_conn = self.db.connect()?;

        // Slow path: acquire write lock and double-check
        let mut guard = self.conn.write().await;
        if let Some(ref conn) = *guard {
            // Another task connected while we were waiting
            return Ok(conn.clone());
        }

        *guard = Some(new_conn.clone());
        Ok(new_conn)
    }

    /// Log a completed play to the database
    pub async fn log_play(&self, state: &TrackState, context: &ListeningContext) -> Result<()> {
        let conn = self.connection().await?;
        queries::insert_play(&conn, state, context).await
    }

    /// Get total play count
    pub async fn get_play_count(&self) -> Result<i64> {
        let conn = self.connection().await?;
        queries::get_play_count(&conn).await
    }

    /// Get top artists by play count
    pub async fn get_top_artists(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<ArtistStats>> {
        let conn = self.connection().await?;
        queries::get_top_artists(&conn, start_date, end_date, limit).await
    }

    /// Get top albums by play count
    pub async fn get_top_albums(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<AlbumStats>> {
        let conn = self.connection().await?;
        queries::get_top_albums(&conn, start_date, end_date, limit).await
    }

    /// Get top tracks by play count
    pub async fn get_top_tracks(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<TrackStats>> {
        let conn = self.connection().await?;
        queries::get_top_tracks(&conn, start_date, end_date, limit).await
    }

    /// Get listening stats overview
    pub async fn get_overview_stats(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<OverviewStats> {
        let conn = self.connection().await?;
        queries::get_overview_stats(&conn, start_date, end_date).await
    }

    // The following methods are implemented but not yet exposed in the CLI.
    // They will be integrated in future releases.

    /// Get listening streaks (current and longest)
    #[allow(dead_code)]
    pub async fn get_listening_streaks(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<crate::analytics::StreakInfo> {
        let conn = self.connection().await?;
        crate::analytics::get_listening_streaks(&conn, start_date, end_date).await
    }

    /// Get night owl score (percentage of plays between midnight and 6am)
    #[allow(dead_code)]
    pub async fn get_night_owl_score(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<crate::analytics::NightOwlScore> {
        let conn = self.connection().await?;
        crate::analytics::get_night_owl_score(&conn, start_date, end_date).await
    }

    /// Get hourly listening heatmap
    #[allow(dead_code)]
    pub async fn get_hourly_heatmap(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<crate::analytics::HourlyHeatmap> {
        let conn = self.connection().await?;
        crate::analytics::get_hourly_heatmap(&conn, start_date, end_date).await
    }

    /// Get genre statistics
    #[allow(dead_code)]
    pub async fn get_genre_stats(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: u32,
    ) -> Result<Vec<(String, i64, i64)>> {
        let conn = self.connection().await?;
        crate::analytics::get_genre_stats(&conn, start_date, end_date, limit).await
    }

    /// Get skip rate (percentage of plays with less than 50% completion)
    #[allow(dead_code)]
    pub async fn get_skip_rate(
        &self,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<(i64, i64, f64)> {
        let conn = self.connection().await?;
        crate::analytics::get_skip_rate(&conn, start_date, end_date).await
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
    #[allow(dead_code)]
    pub total_ms: i64,
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
    #[allow(dead_code)]
    pub total_ms: i64,
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

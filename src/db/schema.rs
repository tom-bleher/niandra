//! Database schema initialization

use libsql::Connection;

use crate::error::Result;

/// Initialize the database schema
pub async fn init_schema(conn: &Connection) -> Result<()> {
    // Create main plays table
    conn.execute(
        r"
        CREATE TABLE IF NOT EXISTS plays (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            title TEXT NOT NULL,
            artist TEXT,
            album TEXT,
            duration_ms INTEGER,
            played_ms INTEGER,
            file_path TEXT,

            -- Extended metadata
            genre TEXT,
            album_artist TEXT,
            track_number INTEGER,
            disc_number INTEGER,
            release_date TEXT,
            art_url TEXT,
            user_rating REAL,
            bpm INTEGER,
            composer TEXT,
            musicbrainz_track_id TEXT,

            -- Seek tracking
            seek_count INTEGER,
            intro_skipped INTEGER,
            seek_forward_ms INTEGER,
            seek_backward_ms INTEGER,

            -- Volume tracking
            app_volume REAL,
            system_volume REAL,
            effective_volume REAL,

            -- Context tracking
            hour_of_day INTEGER,
            day_of_week INTEGER,
            is_weekend INTEGER,
            season TEXT,
            active_window TEXT,
            screen_on INTEGER,
            on_battery INTEGER,

            -- Player info
            player_name TEXT,
            is_local INTEGER
        )
        ",
        (),
    )
    .await?;

    // Create indexes for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_plays_timestamp ON plays(timestamp)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_plays_artist ON plays(artist)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_plays_album ON plays(album)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_plays_genre ON plays(genre)",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_plays_title ON plays(title)",
        (),
    )
    .await?;

    // Create audio features table for future audio analysis
    conn.execute(
        r"
        CREATE TABLE IF NOT EXISTS audio_features (
            file_path TEXT PRIMARY KEY,
            tempo REAL,
            energy REAL,
            danceability REAL,
            valence REAL,
            acousticness REAL,
            instrumentalness REAL,
            speechiness REAL,
            loudness REAL,
            key INTEGER,
            mode INTEGER,
            time_signature INTEGER,
            analyzed_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        ",
        (),
    )
    .await?;

    // Create sessions table for session tracking
    conn.execute(
        r"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            track_count INTEGER DEFAULT 0,
            total_ms INTEGER DEFAULT 0,
            player_name TEXT
        )
        ",
        (),
    )
    .await?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_time)",
        (),
    )
    .await?;

    Ok(())
}

//! Database schema initialization for DuckDB

use duckdb::Connection;

use crate::error::Result;

/// Initialize the database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    // Create main plays table
    // DuckDB uses sequences for auto-increment
    conn.execute_batch(
        r"
        CREATE SEQUENCE IF NOT EXISTS plays_id_seq;

        CREATE TABLE IF NOT EXISTS plays (
            id INTEGER PRIMARY KEY DEFAULT nextval('plays_id_seq'),
            timestamp TIMESTAMP DEFAULT current_timestamp,
            title VARCHAR NOT NULL,
            artist VARCHAR,
            album VARCHAR,
            duration_ms BIGINT,
            played_ms BIGINT,
            file_path VARCHAR,

            -- Extended metadata
            genre VARCHAR,
            album_artist VARCHAR,
            track_number INTEGER,
            disc_number INTEGER,
            release_date VARCHAR,
            art_url VARCHAR,
            user_rating DOUBLE,
            bpm INTEGER,
            composer VARCHAR,
            musicbrainz_track_id VARCHAR,

            -- Seek tracking
            seek_count INTEGER,
            intro_skipped INTEGER,
            seek_forward_ms BIGINT,
            seek_backward_ms BIGINT,

            -- Volume tracking
            app_volume DOUBLE,
            system_volume DOUBLE,
            effective_volume DOUBLE,

            -- Context tracking
            hour_of_day INTEGER,
            day_of_week INTEGER,
            is_weekend INTEGER,
            season VARCHAR,
            active_window VARCHAR,
            screen_on INTEGER,
            on_battery INTEGER,

            -- Player info
            player_name VARCHAR,
            is_local INTEGER
        );
        ",
    )?;

    // Create indexes for common queries
    // DuckDB handles IF NOT EXISTS for indexes
    conn.execute_batch(
        r"
        CREATE INDEX IF NOT EXISTS idx_plays_timestamp ON plays(timestamp);
        CREATE INDEX IF NOT EXISTS idx_plays_artist ON plays(artist);
        CREATE INDEX IF NOT EXISTS idx_plays_album ON plays(album);
        CREATE INDEX IF NOT EXISTS idx_plays_genre ON plays(genre);
        CREATE INDEX IF NOT EXISTS idx_plays_title ON plays(title);
        ",
    )?;

    // Create audio features table for future audio analysis
    conn.execute_batch(
        r"
        CREATE TABLE IF NOT EXISTS audio_features (
            file_path VARCHAR PRIMARY KEY,
            tempo DOUBLE,
            energy DOUBLE,
            danceability DOUBLE,
            valence DOUBLE,
            acousticness DOUBLE,
            instrumentalness DOUBLE,
            speechiness DOUBLE,
            loudness DOUBLE,
            key INTEGER,
            mode INTEGER,
            time_signature INTEGER,
            analyzed_at TIMESTAMP DEFAULT current_timestamp
        );
        ",
    )?;

    // Create sessions table for session tracking
    conn.execute_batch(
        r"
        CREATE TABLE IF NOT EXISTS sessions (
            id VARCHAR PRIMARY KEY,
            start_time TIMESTAMP NOT NULL,
            end_time TIMESTAMP,
            track_count INTEGER DEFAULT 0,
            total_ms BIGINT DEFAULT 0,
            player_name VARCHAR
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_time);
        ",
    )?;

    Ok(())
}

//! Migration tool to convert SQLite database to DuckDB
//!
//! Usage: migrate-db [SQLITE_PATH] [DUCKDB_PATH]
//!
//! If paths are not provided, uses default data directory.

use duckdb::Connection;
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = dirs::data_dir()
        .map(|d| d.join("music-analytics"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    let sqlite_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| data_dir.join("listens.db").to_string_lossy().to_string());

    let duckdb_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| data_dir.join("listens.duckdb").to_string_lossy().to_string());

    if !Path::new(&sqlite_path).exists() {
        eprintln!("Error: SQLite database not found at {}", sqlite_path);
        std::process::exit(1);
    }

    if Path::new(&duckdb_path).exists() {
        eprintln!("Warning: DuckDB database already exists at {}", duckdb_path);
        eprint!("Overwrite? [y/N] ");
        std::io::stderr().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("Aborted.");
            std::process::exit(0);
        }
        std::fs::remove_file(&duckdb_path)?;
    }

    println!("Migrating SQLite -> DuckDB");
    println!("  From: {}", sqlite_path);
    println!("  To:   {}", duckdb_path);

    let conn = Connection::open(&duckdb_path)?;

    // Install and load SQLite extension
    println!("\nLoading SQLite extension...");
    conn.execute_batch("INSTALL sqlite; LOAD sqlite;")?;

    // Attach SQLite database
    println!("Attaching SQLite database...");
    conn.execute(
        &format!("ATTACH '{}' AS sqlite_db (TYPE sqlite)", sqlite_path),
        [],
    )?;

    // Create DuckDB schema
    println!("Creating DuckDB schema...");
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
            seek_count INTEGER,
            intro_skipped INTEGER,
            seek_forward_ms BIGINT,
            seek_backward_ms BIGINT,
            app_volume DOUBLE,
            system_volume DOUBLE,
            effective_volume DOUBLE,
            hour_of_day INTEGER,
            day_of_week INTEGER,
            is_weekend INTEGER,
            season VARCHAR,
            active_window VARCHAR,
            screen_on INTEGER,
            on_battery INTEGER,
            player_name VARCHAR,
            is_local INTEGER
        );

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

        CREATE TABLE IF NOT EXISTS sessions (
            id VARCHAR PRIMARY KEY,
            start_time TIMESTAMP NOT NULL,
            end_time TIMESTAMP,
            track_count INTEGER DEFAULT 0,
            total_ms BIGINT DEFAULT 0,
            player_name VARCHAR
        );

        CREATE TABLE IF NOT EXISTS library (
            id INTEGER PRIMARY KEY,
            file_path VARCHAR UNIQUE NOT NULL,
            title VARCHAR,
            artist VARCHAR,
            album VARCHAR,
            album_artist VARCHAR,
            genre VARCHAR,
            composer VARCHAR,
            track_number INTEGER,
            track_total INTEGER,
            disc_number INTEGER,
            disc_total INTEGER,
            duration_ms BIGINT,
            release_date VARCHAR,
            original_date VARCHAR,
            label VARCHAR,
            isrc VARCHAR,
            barcode VARCHAR,
            musicbrainz_track_id VARCHAR,
            musicbrainz_album_id VARCHAR,
            musicbrainz_artist_id VARCHAR,
            musicbrainz_release_group_id VARCHAR,
            release_country VARCHAR,
            release_type VARCHAR,
            replaygain_track_gain DOUBLE,
            replaygain_album_gain DOUBLE,
            bit_rate INTEGER,
            sample_rate INTEGER,
            channels INTEGER,
            file_size BIGINT,
            last_scanned TIMESTAMP DEFAULT current_timestamp
        );
        ",
    )?;

    // Migrate plays
    println!("Migrating plays table...");
    conn.execute("INSERT INTO plays SELECT * FROM sqlite_db.plays", [])?;
    let plays_count: i64 = conn.query_row("SELECT COUNT(*) FROM plays", [], |row| row.get(0))?;
    println!("  Migrated {} plays", plays_count);

    // Advance the sequence past the max id by consuming values
    // This is a workaround since DuckDB doesn't support ALTER SEQUENCE
    let max_id: i64 = conn.query_row(
        "SELECT COALESCE(MAX(id), 0) FROM plays",
        [],
        |row| row.get(0),
    )?;
    if max_id > 0 {
        // Consume sequence values to advance it past max_id
        // We use generate_series to efficiently advance the sequence
        conn.execute(
            &format!(
                "SELECT nextval('plays_id_seq') FROM generate_series(1, {})",
                max_id
            ),
            [],
        )?;
        println!("  Sequence advanced to {}", max_id + 1);
    }

    // Migrate library
    println!("Migrating library table...");
    conn.execute("INSERT INTO library SELECT * FROM sqlite_db.library", [])?;
    let library_count: i64 = conn.query_row("SELECT COUNT(*) FROM library", [], |row| row.get(0))?;
    println!("  Migrated {} library entries", library_count);

    // Create indexes
    println!("Creating indexes...");
    conn.execute_batch(
        r"
        CREATE INDEX IF NOT EXISTS idx_plays_timestamp ON plays(timestamp);
        CREATE INDEX IF NOT EXISTS idx_plays_artist ON plays(artist);
        CREATE INDEX IF NOT EXISTS idx_plays_album ON plays(album);
        CREATE INDEX IF NOT EXISTS idx_plays_genre ON plays(genre);
        CREATE INDEX IF NOT EXISTS idx_plays_title ON plays(title);
        CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_time);
        CREATE INDEX IF NOT EXISTS idx_library_artist ON library(artist);
        CREATE INDEX IF NOT EXISTS idx_library_album ON library(album);
        CREATE INDEX IF NOT EXISTS idx_library_genre ON library(genre);
        CREATE INDEX IF NOT EXISTS idx_library_file_path ON library(file_path);
        ",
    )?;

    // Detach SQLite
    conn.execute("DETACH sqlite_db", [])?;

    println!("\nMigration complete!");

    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("music-analytics/config.toml");
        println!("\nUpdate your config at {}:", config_path.display());
    } else {
        println!("\nUpdate your config file:");
    }
    println!("  [database]");
    println!("  path = \"{}\"", duckdb_path);

    Ok(())
}

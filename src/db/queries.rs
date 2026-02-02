//! Database query implementations for DuckDB

use duckdb::{params, Connection};

use crate::context::ListeningContext;
use crate::error::Result;
use crate::track::TrackState;

use super::filter::DateFilter;
use super::{AlbumStats, ArtistStats, OverviewStats, TrackStats};

/// Insert a play record into the database
pub fn insert_play(
    conn: &Connection,
    state: &TrackState,
    context: &ListeningContext,
) -> Result<()> {
    let track = &state.track;

    conn.execute(
        r"
        INSERT INTO plays (
            title, artist, album, duration_ms, played_ms, file_path,
            genre, album_artist, track_number, disc_number, release_date,
            art_url, user_rating, bpm, composer, musicbrainz_track_id,
            seek_count, intro_skipped, seek_forward_ms, seek_backward_ms,
            app_volume, system_volume, effective_volume,
            hour_of_day, day_of_week, is_weekend, season,
            active_window, screen_on, on_battery, player_name, is_local
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6,
            ?7, ?8, ?9, ?10, ?11,
            ?12, ?13, ?14, ?15, ?16,
            ?17, ?18, ?19, ?20,
            ?21, ?22, ?23,
            ?24, ?25, ?26, ?27,
            ?28, ?29, ?30, ?31, ?32
        )
        ",
        params![
            track.title.as_deref(),
            track.artist.as_deref(),
            track.album.as_deref(),
            track.duration_us.map(|d| d / 1000), // Convert to ms
            state.played_ms(),
            track.file_path.as_deref(),
            track.genre.as_deref(),
            track.album_artist.as_deref(),
            track.track_number,
            track.disc_number,
            track.release_date.as_deref(),
            track.art_url.as_deref(),
            track.user_rating,
            track.bpm,
            track.composer.as_deref(),
            track.musicbrainz_track_id.as_deref(),
            if state.seek_count > 0 {
                Some(state.seek_count as i64)
            } else {
                None
            },
            if state.intro_skipped { Some(1i64) } else { None },
            if state.seek_forward_ms > 0 {
                Some(state.seek_forward_ms)
            } else {
                None
            },
            if state.seek_backward_ms > 0 {
                Some(state.seek_backward_ms)
            } else {
                None
            },
            state.app_volume,
            state.system_volume,
            state.effective_volume(),
            context.hour_of_day,
            context.day_of_week,
            if context.is_weekend { 1i64 } else { 0i64 },
            context.season.as_str(),
            context.active_window.as_deref(),
            context.screen_on.map(|b| if b { 1i64 } else { 0i64 }),
            context.on_battery.map(|b| if b { 1i64 } else { 0i64 }),
            state.player_name.as_deref(),
            if state.is_local { 1i64 } else { 0i64 },
        ],
    )?;

    Ok(())
}

/// Get total play count
pub fn get_play_count(conn: &Connection) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM plays")?;
    let count: i64 = stmt.query_row([], |row| row.get(0))?;
    Ok(count)
}

/// SQL expression to get the primary artist string (before splitting collaborators).
/// Uses album_artist if available, otherwise normalizes artist by stripping featured artists.
/// Normalization handles: "feat.", "ft.", "featuring", "with" patterns.
/// DuckDB uses INSTR for SQLite compatibility.
const PRIMARY_ARTIST_SQL: &str = r"
    CASE
        WHEN album_artist IS NOT NULL AND TRIM(album_artist) != '' THEN TRIM(album_artist)
        WHEN INSTR(LOWER(artist), ' feat. ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), ' feat. ') - 1))
        WHEN INSTR(LOWER(artist), ' feat ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), ' feat ') - 1))
        WHEN INSTR(LOWER(artist), ' ft. ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), ' ft. ') - 1))
        WHEN INSTR(LOWER(artist), ' ft ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), ' ft ') - 1))
        WHEN INSTR(LOWER(artist), '(feat.') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), '(feat.') - 1))
        WHEN INSTR(LOWER(artist), '(feat ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), '(feat ') - 1))
        WHEN INSTR(LOWER(artist), '(ft.') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), '(ft.') - 1))
        WHEN INSTR(LOWER(artist), '(ft ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), '(ft ') - 1))
        WHEN INSTR(LOWER(artist), ' featuring ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), ' featuring ') - 1))
        WHEN INSTR(LOWER(artist), ' with ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), ' with ') - 1))
        WHEN INSTR(LOWER(artist), '(with ') > 0 THEN TRIM(SUBSTR(artist, 1, INSTR(LOWER(artist), '(with ') - 1))
        ELSE artist
    END
";

/// Get top artists by play count.
/// Splits collaborative artists (e.g., "billy woods & Kenny Segal") and counts
/// plays towards individual artists ONLY if they exist independently in the library.
/// If an artist only appears in collabs, the collab name is kept as-is.
pub fn get_top_artists(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: u32,
) -> Result<Vec<ArtistStats>> {
    // Build date filter conditions
    let mut date_conditions = String::new();
    let mut param_values = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut date_conditions, &mut param_values);

    // Strategy:
    // 1. Find "independent artists" - those with plays where album_artist has no " & " or ", "
    // 2. Split collab artists, but only count towards artists that exist independently
    // 3. If NO artist in a collab exists independently, keep the full collab name
    let query = format!(
        r#"
        WITH RECURSIVE
        -- Get primary artist string for each play (strips "feat." etc.)
        play_artists AS (
            SELECT
                id,
                played_ms,
                {PRIMARY_ARTIST_SQL} as primary_artist
            FROM plays
            WHERE artist IS NOT NULL {date_conditions}
        ),
        -- Find independent artists (those with solo plays - no " & " or ", " in their primary_artist)
        independent_artists AS (
            SELECT DISTINCT LOWER(TRIM(primary_artist)) as artist_lower
            FROM play_artists
            WHERE INSTR(primary_artist, ' & ') = 0
              AND INSTR(primary_artist, ', ') = 0
              AND TRIM(primary_artist) != ''
        ),
        -- Recursively split collab artists on " & " and ", "
        split_artists(play_id, played_ms, artist, remaining, original_artist) AS (
            -- Base case: extract first artist, keep original for fallback
            SELECT
                id,
                played_ms,
                TRIM(CASE
                    WHEN INSTR(primary_artist, ' & ') > 0 THEN SUBSTR(primary_artist, 1, INSTR(primary_artist, ' & ') - 1)
                    WHEN INSTR(primary_artist, ', ') > 0 THEN SUBSTR(primary_artist, 1, INSTR(primary_artist, ', ') - 1)
                    ELSE primary_artist
                END),
                CASE
                    WHEN INSTR(primary_artist, ' & ') > 0 THEN SUBSTR(primary_artist, INSTR(primary_artist, ' & ') + 3)
                    WHEN INSTR(primary_artist, ', ') > 0 THEN SUBSTR(primary_artist, INSTR(primary_artist, ', ') + 2)
                    ELSE NULL
                END,
                primary_artist
            FROM play_artists

            UNION ALL

            -- Recursive case: continue splitting remaining string
            SELECT
                play_id,
                played_ms,
                TRIM(CASE
                    WHEN INSTR(remaining, ' & ') > 0 THEN SUBSTR(remaining, 1, INSTR(remaining, ' & ') - 1)
                    WHEN INSTR(remaining, ', ') > 0 THEN SUBSTR(remaining, 1, INSTR(remaining, ', ') - 1)
                    ELSE remaining
                END),
                CASE
                    WHEN INSTR(remaining, ' & ') > 0 THEN SUBSTR(remaining, INSTR(remaining, ' & ') + 3)
                    WHEN INSTR(remaining, ', ') > 0 THEN SUBSTR(remaining, INSTR(remaining, ', ') + 2)
                    ELSE NULL
                END,
                original_artist
            FROM split_artists
            WHERE remaining IS NOT NULL AND TRIM(remaining) != ''
        ),
        -- Filter: only keep split artists that exist independently
        -- For plays where NO split artist exists independently, we'll handle separately
        valid_splits AS (
            SELECT play_id, played_ms, artist, original_artist
            FROM split_artists
            WHERE LOWER(TRIM(artist)) IN (SELECT artist_lower FROM independent_artists)
        ),
        -- Find plays where NO split artist exists independently (keep original collab name)
        plays_without_independent AS (
            SELECT DISTINCT play_id, played_ms, original_artist
            FROM split_artists s
            WHERE NOT EXISTS (
                SELECT 1 FROM valid_splits v WHERE v.play_id = s.play_id
            )
        ),
        -- Combine: valid splits + unsplit collabs
        final_artists AS (
            SELECT artist, played_ms FROM valid_splits
            UNION ALL
            SELECT original_artist as artist, played_ms FROM plays_without_independent
        )
        SELECT
            FIRST(artist) as artist,
            COUNT(*) as play_count,
            SUM(played_ms) as total_ms
        FROM final_artists
        WHERE artist IS NOT NULL AND TRIM(artist) != ''
        GROUP BY LOWER(artist)
        ORDER BY play_count DESC
        LIMIT {limit}
        "#
    );

    let params = DateFilter::params_as_refs(&param_values);
    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(ArtistStats {
            artist: row.get(0)?,
            play_count: row.get(1)?,
            total_ms: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
        })
    })?;

    let mut stats = Vec::new();
    for row in rows {
        stats.push(row?);
    }

    Ok(stats)
}

/// Get top albums by play count
pub fn get_top_albums(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: u32,
) -> Result<Vec<AlbumStats>> {
    // Use album_artist if available, otherwise use the most frequent artist for the album
    // Also fetch the most recent art_url for each album
    let mut query = r"
        SELECT
            FIRST(album) as album,
            COALESCE(
                MAX(album_artist),
                FIRST(artist)
            ) as artist,
            COUNT(*) as play_count,
            SUM(played_ms) as total_ms,
            FIRST(art_url) as art_url
        FROM plays
        WHERE album IS NOT NULL
    "
    .to_string();

    let mut param_values = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut query, &mut param_values);

    query.push_str(&format!(
        " GROUP BY LOWER(album) ORDER BY play_count DESC LIMIT {limit}"
    ));

    let params = DateFilter::params_as_refs(&param_values);
    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(AlbumStats {
            album: row.get(0)?,
            artist: row.get(1)?,
            play_count: row.get(2)?,
            total_ms: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
            art_url: row.get(4)?,
        })
    })?;

    let mut stats = Vec::new();
    for row in rows {
        stats.push(row?);
    }

    Ok(stats)
}

/// Get top tracks by play count
pub fn get_top_tracks(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: u32,
) -> Result<Vec<TrackStats>> {
    // Normalize artist names to aggregate tracks with featuring artists
    // Also fetch the most recent art_url for each track
    let mut query = format!(
        r"
        SELECT
            FIRST(title) as title,
            FIRST({PRIMARY_ARTIST_SQL}) as normalized_artist,
            COUNT(*) as play_count,
            SUM(played_ms) as total_ms,
            FIRST(art_url) as art_url
        FROM plays
        WHERE title IS NOT NULL
    "
    );

    let mut param_values = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut query, &mut param_values);

    query.push_str(&format!(
        " GROUP BY LOWER(title), LOWER({PRIMARY_ARTIST_SQL}) ORDER BY play_count DESC LIMIT {limit}"
    ));

    let params = DateFilter::params_as_refs(&param_values);
    let mut stmt = conn.prepare(&query)?;
    let rows = stmt.query_map(params.as_slice(), |row| {
        Ok(TrackStats {
            title: row.get(0)?,
            artist: row.get(1)?,
            play_count: row.get(2)?,
            total_ms: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
            art_url: row.get(4)?,
        })
    })?;

    let mut stats = Vec::new();
    for row in rows {
        stats.push(row?);
    }

    Ok(stats)
}

/// Get overview statistics
pub fn get_overview_stats(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<OverviewStats> {
    let mut query = r"
        SELECT
            COUNT(*) as play_count,
            COALESCE(SUM(played_ms), 0) as total_ms,
            COUNT(DISTINCT LOWER(artist)) as unique_artists,
            COUNT(DISTINCT LOWER(album)) as unique_albums,
            COUNT(DISTINCT title || '|' || COALESCE(artist, '')) as unique_tracks
        FROM plays
        WHERE 1=1
    "
    .to_string();

    let mut param_values = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut query, &mut param_values);

    let params = DateFilter::params_as_refs(&param_values);
    let mut stmt = conn.prepare(&query)?;
    let result = stmt.query_row(params.as_slice(), |row| {
        Ok(OverviewStats {
            total_plays: row.get(0)?,
            total_ms: row.get(1)?,
            unique_artists: row.get(2)?,
            unique_albums: row.get(3)?,
            unique_tracks: row.get(4)?,
        })
    });

    match result {
        Ok(stats) => Ok(stats),
        Err(_) => Ok(OverviewStats::default()),
    }
}

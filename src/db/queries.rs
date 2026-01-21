//! Database query implementations

use libsql::{params, Connection};

use crate::context::ListeningContext;
use crate::error::Result;
use crate::track::TrackState;

use super::filter::DateFilter;
use super::{AlbumStats, ArtistStats, OverviewStats, TrackStats};

/// Insert a play record into the database
pub async fn insert_play(
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
    )
    .await?;

    Ok(())
}

/// Get total play count
pub async fn get_play_count(conn: &Connection) -> Result<i64> {
    let mut rows = conn.query("SELECT COUNT(*) FROM plays", ()).await?;

    if let Some(row) = rows.next().await? {
        Ok(row.get::<i64>(0)?)
    } else {
        Ok(0)
    }
}

/// Get top artists by play count
pub async fn get_top_artists(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: u32,
) -> Result<Vec<ArtistStats>> {
    let mut query = r"
        SELECT
            artist,
            COUNT(*) as play_count,
            SUM(played_ms) as total_ms
        FROM plays
        WHERE artist IS NOT NULL
    "
    .to_string();

    let mut param_values = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut query, &mut param_values);

    query.push_str(&format!(
        " GROUP BY LOWER(artist) ORDER BY play_count DESC LIMIT {limit}"
    ));

    let mut rows = conn
        .query(&query, DateFilter::to_values(&param_values))
        .await?;
    let mut stats = Vec::new();

    while let Some(row) = rows.next().await? {
        stats.push(ArtistStats {
            artist: row.get(0)?,
            play_count: row.get(1)?,
            total_ms: row.get::<Option<i64>>(2)?.unwrap_or(0),
        });
    }

    Ok(stats)
}

/// Get top albums by play count
pub async fn get_top_albums(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: u32,
) -> Result<Vec<AlbumStats>> {
    let mut query = r"
        SELECT
            album,
            MAX(artist) as artist,
            COUNT(*) as play_count,
            SUM(played_ms) as total_ms
        FROM plays
        WHERE album IS NOT NULL
    "
    .to_string();

    let mut param_values = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut query, &mut param_values);

    query.push_str(&format!(
        " GROUP BY LOWER(album) ORDER BY play_count DESC LIMIT {limit}"
    ));

    let mut rows = conn
        .query(&query, DateFilter::to_values(&param_values))
        .await?;
    let mut stats = Vec::new();

    while let Some(row) = rows.next().await? {
        stats.push(AlbumStats {
            album: row.get(0)?,
            artist: row.get(1)?,
            play_count: row.get(2)?,
            total_ms: row.get::<Option<i64>>(3)?.unwrap_or(0),
        });
    }

    Ok(stats)
}

/// Get top tracks by play count
pub async fn get_top_tracks(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: u32,
) -> Result<Vec<TrackStats>> {
    let mut query = r"
        SELECT
            title,
            artist,
            COUNT(*) as play_count,
            SUM(played_ms) as total_ms
        FROM plays
        WHERE title IS NOT NULL
    "
    .to_string();

    let mut param_values = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut query, &mut param_values);

    query.push_str(&format!(
        " GROUP BY title, artist ORDER BY play_count DESC LIMIT {limit}"
    ));

    let mut rows = conn
        .query(&query, DateFilter::to_values(&param_values))
        .await?;
    let mut stats = Vec::new();

    while let Some(row) = rows.next().await? {
        stats.push(TrackStats {
            title: row.get(0)?,
            artist: row.get(1)?,
            play_count: row.get(2)?,
            total_ms: row.get::<Option<i64>>(3)?.unwrap_or(0),
        });
    }

    Ok(stats)
}

/// Get overview statistics
pub async fn get_overview_stats(
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

    let mut rows = conn
        .query(&query, DateFilter::to_values(&param_values))
        .await?;

    if let Some(row) = rows.next().await? {
        Ok(OverviewStats {
            total_plays: row.get(0)?,
            total_ms: row.get(1)?,
            unique_artists: row.get(2)?,
            unique_albums: row.get(3)?,
            unique_tracks: row.get(4)?,
        })
    } else {
        Ok(OverviewStats::default())
    }
}

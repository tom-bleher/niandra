//! Analytics and statistics module
//!
//! Provides various analytics functions for listening data.
//!
//! Note: These functions are implemented but not yet exposed in the CLI.
//! They will be integrated in future releases.

#![allow(dead_code)]

use chrono::NaiveDate;
use libsql::Connection;
use std::collections::HashMap;

use crate::db::DateFilter;
use crate::error::Result;

/// Streak information
#[derive(Debug, Clone, Default)]
pub struct StreakInfo {
    pub current_streak: i32,
    pub longest_streak: i32,
    pub longest_streak_start: Option<String>,
    pub longest_streak_end: Option<String>,
}

/// Session information
#[derive(Debug, Clone, Default)]
pub struct SessionInfo {
    pub total_sessions: i32,
    pub avg_session_minutes: f64,
    pub longest_session_minutes: f64,
    pub total_listening_minutes: f64,
}

/// Night owl score
#[derive(Debug, Clone, Default)]
pub struct NightOwlScore {
    pub percentage: f64,
    pub night_plays: i64,
    pub total_plays: i64,
}

/// Hourly heatmap data
#[derive(Debug, Clone, Default)]
pub struct HourlyHeatmap {
    pub hours: HashMap<i32, i64>,
    pub peak_hour: i32,
    pub peak_count: i64,
}

/// Get listening streaks
pub async fn get_listening_streaks(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<StreakInfo> {
    let mut query =
        "SELECT DISTINCT DATE(timestamp) as play_date FROM plays WHERE 1=1".to_string();
    let mut params = Vec::new();

    DateFilter::new(start_date, end_date).apply(&mut query, &mut params);

    query.push_str(" ORDER BY play_date ASC");

    let mut rows = conn
        .query(&query, DateFilter::to_values(&params))
        .await?;
    let mut dates: Vec<NaiveDate> = Vec::new();

    while let Some(row) = rows.next().await? {
        let date_str: String = row.get(0)?;
        if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            dates.push(date);
        }
    }

    if dates.is_empty() {
        return Ok(StreakInfo::default());
    }

    // Calculate streaks
    let mut streaks: Vec<(NaiveDate, NaiveDate, i32)> = Vec::new();
    let mut streak_start = dates[0];
    let mut streak_end = dates[0];

    for i in 1..dates.len() {
        let diff = dates[i].signed_duration_since(dates[i - 1]).num_days();
        if diff == 1 {
            streak_end = dates[i];
        } else {
            let length = (streak_end - streak_start).num_days() as i32 + 1;
            streaks.push((streak_start, streak_end, length));
            streak_start = dates[i];
            streak_end = dates[i];
        }
    }

    // Don't forget the last streak
    let length = (streak_end - streak_start).num_days() as i32 + 1;
    streaks.push((streak_start, streak_end, length));

    // Find longest streak
    let longest = streaks
        .iter()
        .max_by_key(|(_, _, len)| *len)
        .cloned()
        .unwrap_or((streak_start, streak_end, 1));

    // Current streak (if ends today or yesterday)
    let today = chrono::Local::now().date_naive();
    let current_streak = if let Some((_, end, len)) = streaks.last() {
        let days_ago = (today - *end).num_days();
        if days_ago <= 1 {
            *len
        } else {
            0
        }
    } else {
        0
    };

    Ok(StreakInfo {
        current_streak,
        longest_streak: longest.2,
        longest_streak_start: Some(longest.0.to_string()),
        longest_streak_end: Some(longest.1.to_string()),
    })
}

/// Get night owl score (percentage of plays between 10 PM and 4 AM)
pub async fn get_night_owl_score(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<NightOwlScore> {
    let mut base_query = "SELECT COUNT(*) FROM plays WHERE 1=1".to_string();
    let mut params = Vec::new();

    DateFilter::new(start_date, end_date).apply(&mut base_query, &mut params);

    let param_values = DateFilter::to_values(&params);

    // Total plays
    let mut rows = conn.query(&base_query, param_values.clone()).await?;
    let total_plays: i64 = if let Some(row) = rows.next().await? {
        row.get(0)?
    } else {
        0
    };

    if total_plays == 0 {
        return Ok(NightOwlScore::default());
    }

    // Night plays (10 PM to 4 AM = hours 22, 23, 0, 1, 2, 3)
    let night_query = format!(
        "{base_query} AND hour_of_day IS NOT NULL AND (hour_of_day >= 22 OR hour_of_day < 4)"
    );
    let mut rows = conn.query(&night_query, param_values).await?;
    let night_plays: i64 = if let Some(row) = rows.next().await? {
        row.get(0)?
    } else {
        0
    };

    let percentage = (night_plays as f64 / total_plays as f64) * 100.0;

    Ok(NightOwlScore {
        percentage,
        night_plays,
        total_plays,
    })
}

/// Get hourly heatmap
pub async fn get_hourly_heatmap(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<HourlyHeatmap> {
    let mut query =
        "SELECT hour_of_day, COUNT(*) FROM plays WHERE hour_of_day IS NOT NULL".to_string();
    let mut params = Vec::new();

    DateFilter::new(start_date, end_date).apply(&mut query, &mut params);

    query.push_str(" GROUP BY hour_of_day ORDER BY hour_of_day");

    let mut rows = conn
        .query(&query, DateFilter::to_values(&params))
        .await?;
    let mut hours: HashMap<i32, i64> = HashMap::new();
    let mut peak_hour = 0;
    let mut peak_count: i64 = 0;

    while let Some(row) = rows.next().await? {
        let hour: i32 = row.get(0)?;
        let count: i64 = row.get(1)?;

        hours.insert(hour, count);

        if count > peak_count {
            peak_count = count;
            peak_hour = hour;
        }
    }

    Ok(HourlyHeatmap {
        hours,
        peak_hour,
        peak_count,
    })
}

/// Get genre statistics
pub async fn get_genre_stats(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
    limit: u32,
) -> Result<Vec<(String, i64, i64)>> {
    let mut query = r"
        SELECT genre, COUNT(*) as play_count, SUM(played_ms) as total_ms
        FROM plays
        WHERE genre IS NOT NULL
    "
    .to_string();

    let mut params = Vec::new();
    DateFilter::new(start_date, end_date).apply(&mut query, &mut params);

    query.push_str(&format!(
        " GROUP BY genre ORDER BY play_count DESC LIMIT {}",
        limit
    ));

    let mut rows = conn
        .query(&query, DateFilter::to_values(&params))
        .await?;
    let mut stats = Vec::new();

    while let Some(row) = rows.next().await? {
        let genre: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        let total_ms: i64 = row.get::<Option<i64>>(2)?.unwrap_or(0);
        stats.push((genre, count, total_ms));
    }

    Ok(stats)
}

/// Get skip rate (plays < 50% completion)
pub async fn get_skip_rate(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<(i64, i64, f64)> {
    let mut base_where = "WHERE duration_ms > 0 AND played_ms IS NOT NULL".to_string();
    let mut params = Vec::new();

    DateFilter::new(start_date, end_date).apply(&mut base_where, &mut params);

    let param_values = DateFilter::to_values(&params);

    // Total plays with duration
    let total_query = format!("SELECT COUNT(*) FROM plays {base_where}");
    let mut rows = conn.query(&total_query, param_values.clone()).await?;
    let total: i64 = if let Some(row) = rows.next().await? {
        row.get(0)?
    } else {
        0
    };

    if total == 0 {
        return Ok((0, 0, 0.0));
    }

    // Skipped plays (< 50% completion)
    let skip_query =
        format!("SELECT COUNT(*) FROM plays {base_where} AND (played_ms * 1.0 / duration_ms) < 0.5");
    let mut rows = conn.query(&skip_query, param_values).await?;
    let skipped: i64 = if let Some(row) = rows.next().await? {
        row.get(0)?
    } else {
        0
    };

    let rate = (skipped as f64 / total as f64) * 100.0;

    Ok((skipped, total, rate))
}

/// Daily contribution data
#[derive(Debug, Clone, Default)]
pub struct DailyContribution {
    /// Map of date string (YYYY-MM-DD) to play count
    pub days: std::collections::HashMap<String, i64>,
    /// Maximum plays in a single day
    pub max_plays: i64,
    /// Total plays in the period
    pub total_plays: i64,
}

/// Get daily play counts for the contribution graph
pub async fn get_daily_contributions(
    conn: &Connection,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<DailyContribution> {
    let mut query =
        "SELECT DATE(datetime(timestamp, 'localtime')) as play_date, COUNT(*) as count FROM plays WHERE 1=1".to_string();
    let mut params = Vec::new();

    DateFilter::new(start_date, end_date).apply(&mut query, &mut params);

    query.push_str(" GROUP BY play_date ORDER BY play_date");

    let mut rows = conn
        .query(&query, DateFilter::to_values(&params))
        .await?;

    let mut days = std::collections::HashMap::new();
    let mut max_plays: i64 = 0;
    let mut total_plays: i64 = 0;

    while let Some(row) = rows.next().await? {
        let date: String = row.get(0)?;
        let count: i64 = row.get(1)?;

        days.insert(date, count);
        max_plays = max_plays.max(count);
        total_plays += count;
    }

    Ok(DailyContribution {
        days,
        max_plays,
        total_plays,
    })
}

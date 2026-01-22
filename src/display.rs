//! Display utilities for formatting statistics output.
//!
//! This module provides shared formatting functions used by CLI tools
//! for displaying music listening statistics in the terminal.
//!
//! # Functions
//!
//! - [`truncate`] - Truncate strings to a maximum length with ellipsis
//! - [`make_bar`] - Create visual bar charts for relative values
//! - [`format_hours`] - Convert milliseconds to hours
//! - [`build_date_range`] - Build date range from CLI flags
//! - [`print_section`] / [`print_section_simple`] - Print section headers
//! - [`display_overview`] / [`display_top_artists`] / etc. - Display formatted stats

use chrono::{Datelike, Local};

use crate::db::{AlbumStats, ArtistStats, OverviewStats, TrackStats};

/// Build date range from CLI flags.
///
/// Returns `(start_date, end_date, period_name)` tuple for filtering and display.
#[must_use]
pub fn build_date_range(
    all_time: bool,
    week: bool,
    month: bool,
    year: Option<i32>,
) -> (Option<String>, Option<String>, String) {
    let now = Local::now();

    if all_time {
        (None, None, "All Time".to_string())
    } else if week {
        let start = (now - chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();
        (Some(start), None, "Last 7 Days".to_string())
    } else if month {
        let start = now
            .with_day(1)
            .expect("day 1 is always valid")
            .format("%Y-%m-%d")
            .to_string();
        (Some(start), None, now.format("%B %Y").to_string())
    } else if let Some(y) = year {
        let start = format!("{y}-01-01");
        let end = format!("{y}-12-31");
        (Some(start), Some(end), y.to_string())
    } else {
        // Default: current year
        let current_year = now.year();
        let start = format!("{current_year}-01-01");
        (Some(start), None, current_year.to_string())
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated.
///
/// Handles Unicode characters correctly by counting graphemes rather than bytes.
/// For `max_len < 3`, truncates without ellipsis since there's no room for "...".
///
/// # Examples
///
/// ```
/// use music_analytics::display::truncate;
///
/// assert_eq!(truncate("hello", 10), "hello");
/// assert_eq!(truncate("hello world", 8), "hello...");
/// assert_eq!(truncate("hello", 2), "he");
/// ```
pub fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len < 3 {
        // No room for ellipsis, just truncate
        s.chars().take(max_len).collect()
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{truncated}...")
    }
}

/// Create a visual bar for displaying relative values.
///
/// Uses Unicode block characters to create a proportional bar chart.
///
/// # Arguments
///
/// * `value` - The value to represent (negative values treated as 0)
/// * `max_value` - The maximum value (determines 100% width)
/// * `width` - The total width of the bar in characters
pub fn make_bar(value: i64, max_value: i64, width: usize) -> String {
    if max_value <= 0 || value <= 0 {
        return " ".repeat(width);
    }
    let ratio = (value as f64 / max_value as f64).clamp(0.0, 1.0);
    let filled = (ratio * width as f64) as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Format milliseconds as hours.
pub fn format_hours(ms: i64) -> f64 {
    ms as f64 / 1000.0 / 3600.0
}

/// Print a section header with equals signs.
#[allow(dead_code)]
pub fn print_section(title: &str) {
    println!("\n{}", "=".repeat(50));
    println!("  {}", title);
    println!("{}", "=".repeat(50));
}

/// Print a simple section header with dashes.
pub fn print_section_simple(title: &str) {
    println!("\n{}", title);
    println!("{}", "-".repeat(30));
}

/// Display overview statistics.
pub fn display_overview(overview: &OverviewStats) {
    let hours = format_hours(overview.total_ms);
    println!("  Total plays:      {:>10}", overview.total_plays);
    println!("  Listening time:   {:>10.1} hours", hours);
    println!("  Unique artists:   {:>10}", overview.unique_artists);
    println!("  Unique albums:    {:>10}", overview.unique_albums);
    println!("  Unique tracks:    {:>10}", overview.unique_tracks);
}

/// Display top artists list.
pub fn display_top_artists(artists: &[ArtistStats], show_bar: bool) {
    let max_plays = artists.first().map(|a| a.play_count).unwrap_or(1);

    for (i, artist) in artists.iter().enumerate() {
        let hours = format_hours(artist.total_ms);
        if show_bar {
            let bar = make_bar(artist.play_count, max_plays, 20);
            println!(
                "  {:2}. {:<30} {} {:>4} plays ({:.1}h)",
                i + 1,
                truncate(&artist.artist, 30),
                bar,
                artist.play_count,
                hours
            );
        } else {
            println!(
                "  {:2}. {:<30} {:>4} plays ({:.1}h)",
                i + 1,
                truncate(&artist.artist, 30),
                artist.play_count,
                hours
            );
        }
    }
}

/// Display top albums list.
pub fn display_top_albums(albums: &[AlbumStats], show_bar: bool) {
    let max_plays = albums.first().map(|a| a.play_count).unwrap_or(1);

    for (i, album) in albums.iter().enumerate() {
        let artist = album.artist.as_deref().unwrap_or("Unknown");
        if show_bar {
            let bar = make_bar(album.play_count, max_plays, 15);
            println!(
                "  {:2}. {:<25} by {:<15} {} {:>3}",
                i + 1,
                truncate(&album.album, 25),
                truncate(artist, 15),
                bar,
                album.play_count
            );
        } else {
            println!(
                "  {:2}. {:<25} by {:<15} {:>3} plays",
                i + 1,
                truncate(&album.album, 25),
                truncate(artist, 15),
                album.play_count
            );
        }
    }
}

/// Display top tracks list.
pub fn display_top_tracks(tracks: &[TrackStats], show_bar: bool) {
    let max_plays = tracks.first().map(|t| t.play_count).unwrap_or(1);

    for (i, track) in tracks.iter().enumerate() {
        let artist = track.artist.as_deref().unwrap_or("Unknown");
        if show_bar {
            let bar = make_bar(track.play_count, max_plays, 15);
            println!(
                "  {:2}. {:<25} - {:<15} {} {:>3}",
                i + 1,
                truncate(&track.title, 25),
                truncate(artist, 15),
                bar,
                track.play_count
            );
        } else {
            println!(
                "  {:2}. {:<25} - {:<15} {:>3} plays",
                i + 1,
                truncate(&track.title, 25),
                truncate(artist, 15),
                track.play_count
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_truncate_unicode() {
        assert_eq!(truncate("日本語テスト", 5), "日本...");
    }

    #[test]
    fn test_make_bar_full() {
        assert_eq!(make_bar(100, 100, 10), "██████████");
    }

    #[test]
    fn test_make_bar_half() {
        assert_eq!(make_bar(50, 100, 10), "█████░░░░░");
    }

    #[test]
    fn test_make_bar_zero_max() {
        assert_eq!(make_bar(50, 0, 10), "          ");
    }

    #[test]
    fn test_truncate_small_max_len() {
        // Edge cases: max_len < 3 means no room for ellipsis
        assert_eq!(truncate("hello", 2), "he");
        assert_eq!(truncate("hello", 1), "h");
        assert_eq!(truncate("hello", 0), "");
    }

    #[test]
    fn test_truncate_exactly_three() {
        // max_len = 3: just enough for ellipsis
        assert_eq!(truncate("hello", 3), "...");
        assert_eq!(truncate("hi", 3), "hi"); // fits without truncation
    }
}

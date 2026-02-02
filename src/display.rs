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

// These functions are public API for CLI binaries
#![allow(dead_code)]

use std::borrow::Cow;

use crate::date_range;
use crate::db::{AlbumStats, ArtistStats, OverviewStats, TrackStats};

/// Trait for types that can be displayed in a ranked list.
///
/// This trait abstracts over the common display pattern for artists, albums, and tracks,
/// allowing a single generic display function to handle all three types.
///
/// Implementors provide data access methods, and the generic display function
/// handles all formatting logic consistently.
pub trait DisplayableItem {
    /// Returns the primary display name for this item.
    ///
    /// For artists, this is the artist name.
    /// For albums, this is the album title.
    /// For tracks, this is the track title.
    fn display_name(&self) -> &str;

    /// Returns an optional secondary display name (e.g., artist for albums/tracks).
    fn secondary_name(&self) -> Option<&str> {
        None
    }

    /// Returns the separator to use between primary and secondary names.
    ///
    /// Default is " - " for tracks, override to " by " for albums.
    fn name_separator(&self) -> &str {
        " - "
    }

    /// Returns the play count for this item.
    fn play_count(&self) -> i64;

    /// Returns the total listening time in milliseconds.
    fn total_ms(&self) -> i64;

    /// Returns the last played timestamp, if available.
    fn last_played(&self) -> Option<&str> {
        None
    }

    /// Returns the width to use for the primary name column.
    fn name_width(&self) -> usize {
        30
    }
}

/// Build date range from CLI flags.
///
/// Returns `(start_date, end_date, period_name)` tuple for filtering and display.
///
/// This is a convenience wrapper around [`date_range::from_cli_flags`] that returns
/// the tuple format expected by existing CLI code.
#[must_use]
pub fn build_date_range(
    all_time: bool,
    week: bool,
    month: bool,
    year: Option<i32>,
) -> (Option<String>, Option<String>, String) {
    let range = date_range::from_cli_flags(all_time, week, month, year);
    let (start, end) = range.to_sql_tuple();
    (start, end, range.display_name)
}

/// Truncate a string to a maximum length, adding "..." if truncated.
///
/// Returns a `Cow<str>` to avoid allocation when the string already fits
/// within the specified length. Only allocates a new `String` when truncation
/// is actually needed.
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
pub fn truncate(s: &str, max_len: usize) -> Cow<'_, str> {
    let char_count = s.chars().count();
    if char_count <= max_len {
        Cow::Borrowed(s)
    } else if max_len < 3 {
        // No room for ellipsis, just truncate
        Cow::Owned(s.chars().take(max_len).collect())
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        Cow::Owned(format!("{truncated}..."))
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

// ============================================================================
// DisplayableItem trait implementations
// ============================================================================

impl DisplayableItem for ArtistStats {
    fn display_name(&self) -> &str {
        &self.artist
    }

    fn play_count(&self) -> i64 {
        self.play_count
    }

    fn total_ms(&self) -> i64 {
        self.total_ms
    }

    fn name_width(&self) -> usize {
        30
    }
}

impl DisplayableItem for AlbumStats {
    fn display_name(&self) -> &str {
        &self.album
    }

    fn secondary_name(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    fn name_separator(&self) -> &str {
        " by "
    }

    fn play_count(&self) -> i64 {
        self.play_count
    }

    fn total_ms(&self) -> i64 {
        self.total_ms
    }

    fn name_width(&self) -> usize {
        25
    }
}

impl DisplayableItem for TrackStats {
    fn display_name(&self) -> &str {
        &self.title
    }

    fn secondary_name(&self) -> Option<&str> {
        self.artist.as_deref()
    }

    fn name_separator(&self) -> &str {
        " - "
    }

    fn play_count(&self) -> i64 {
        self.play_count
    }

    fn total_ms(&self) -> i64 {
        self.total_ms
    }

    fn name_width(&self) -> usize {
        25
    }
}

// ============================================================================
// Generic display function
// ============================================================================

/// Display a ranked list of items implementing [`DisplayableItem`].
///
/// This generic function handles the common display logic for artists, albums, and tracks:
/// 1. Prints a header with count and time period
/// 2. Iterates through items
/// 3. Formats each item with rank, name, play count, hours
/// 4. Handles the "last played" display when available
///
/// # Arguments
/// * `items` - Slice of items to display
/// * `show_bar` - Whether to show visual bar charts
/// * `bar_width` - Width of the bar chart (only used if `show_bar` is true)
pub fn display_top_items<T: DisplayableItem>(items: &[T], show_bar: bool, bar_width: usize) {
    let max_plays = items.first().map(|item| item.play_count()).unwrap_or(1);

    for (i, item) in items.iter().enumerate() {
        let index = i + 1;
        let name_width = item.name_width();
        let name = truncate(item.display_name(), name_width);
        let hours = format_hours(item.total_ms());

        let line = if let Some(secondary) = item.secondary_name() {
            // Format with secondary name (albums/tracks)
            let secondary_truncated = truncate(secondary, 15);
            if show_bar {
                let bar = make_bar(item.play_count(), max_plays, bar_width);
                format!(
                    "  {:2}. {:<width$}{}{:<15} {} {:>3}",
                    index,
                    name,
                    item.name_separator(),
                    secondary_truncated,
                    bar,
                    item.play_count(),
                    width = name_width
                )
            } else {
                format!(
                    "  {:2}. {:<width$}{}{:<15} {:>3} plays ({:.1}h)",
                    index,
                    name,
                    item.name_separator(),
                    secondary_truncated,
                    item.play_count(),
                    hours,
                    width = name_width
                )
            }
        } else {
            // Format without secondary name (artists)
            if show_bar {
                let bar = make_bar(item.play_count(), max_plays, bar_width);
                format!(
                    "  {:2}. {:<width$} {} {:>4} plays ({:.1}h)",
                    index,
                    name,
                    bar,
                    item.play_count(),
                    hours,
                    width = name_width
                )
            } else {
                format!(
                    "  {:2}. {:<width$} {:>4} plays ({:.1}h)",
                    index,
                    name,
                    item.play_count(),
                    hours,
                    width = name_width
                )
            }
        };

        // Handle last played display if available
        if let Some(last_played) = item.last_played() {
            println!("{line}  (last: {last_played})");
        } else {
            println!("{line}");
        }
    }
}

// ============================================================================
// Convenience wrapper functions (for backward compatibility)
// ============================================================================

/// Display top artists list.
///
/// This is a convenience wrapper around [`display_top_items`] with the
/// appropriate bar width for artist display.
pub fn display_top_artists(artists: &[ArtistStats], show_bar: bool) {
    display_top_items(artists, show_bar, 20);
}

/// Display top albums list.
///
/// This is a convenience wrapper around [`display_top_items`] with the
/// appropriate bar width for album display.
pub fn display_top_albums(albums: &[AlbumStats], show_bar: bool) {
    display_top_items(albums, show_bar, 15);
}

/// Display top tracks list.
///
/// This is a convenience wrapper around [`display_top_items`] with the
/// appropriate bar width for track display.
pub fn display_top_tracks(tracks: &[TrackStats], show_bar: bool) {
    display_top_items(tracks, show_bar, 15);
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

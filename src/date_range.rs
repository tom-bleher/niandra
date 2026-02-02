//! Date range utilities for filtering statistics queries.
//!
//! This module provides a unified `DateRange` type used by both CLI and GUI
//! for filtering database queries by time period.

// These types are public API for GUI and CLI binaries
#![allow(dead_code)]

use std::fmt;

use chrono::{Datelike, Local, NaiveDate};

/// A date range for filtering statistics queries.
///
/// Provides a unified representation of date ranges used by both CLI and GUI
/// components. Includes methods for converting to SQL-friendly strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    /// Start date (inclusive), or None for no lower bound
    pub start: Option<NaiveDate>,
    /// End date (inclusive), or None for no upper bound
    pub end: Option<NaiveDate>,
    /// Human-readable name for this period (e.g., "Last 7 Days", "2024")
    pub display_name: String,
}

impl DateRange {
    /// Create a new date range with the given bounds and display name.
    #[must_use]
    pub fn new(
        start: Option<NaiveDate>,
        end: Option<NaiveDate>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            start,
            end,
            display_name: display_name.into(),
        }
    }

    /// Create an unbounded date range (all time).
    #[must_use]
    pub fn all_time() -> Self {
        Self::new(None, None, "All Time")
    }

    /// Create a date range for today only.
    #[must_use]
    pub fn today() -> Self {
        let today = Local::now().date_naive();
        Self::new(Some(today), Some(today), "Today")
    }

    /// Create a date range for the last 7 days.
    #[must_use]
    pub fn last_week() -> Self {
        let today = Local::now().date_naive();
        let start = today - chrono::Duration::days(7);
        Self::new(Some(start), Some(today), "Last 7 Days")
    }

    /// Create a date range for the current calendar month.
    #[must_use]
    pub fn current_month() -> Self {
        let now = Local::now();
        let start = now.date_naive().with_day(1).expect("day 1 is always valid");
        let display = now.format("%B %Y").to_string();
        Self::new(Some(start), None, display)
    }

    /// Create a date range for the last 30 days (rolling month).
    #[must_use]
    pub fn last_month() -> Self {
        let today = Local::now().date_naive();
        let start = today - chrono::Duration::days(30);
        Self::new(Some(start), Some(today), "Past Month")
    }

    /// Create a date range for the current calendar year.
    #[must_use]
    pub fn current_year() -> Self {
        let now = Local::now();
        let year = now.year();
        let start = NaiveDate::from_ymd_opt(year, 1, 1).expect("January 1 is always valid");
        Self::new(Some(start), None, year.to_string())
    }

    /// Create a date range for the last 365 days (rolling year).
    #[must_use]
    pub fn last_year() -> Self {
        let today = Local::now().date_naive();
        let start = today - chrono::Duration::days(365);
        Self::new(Some(start), Some(today), "Past Year")
    }

    /// Create a date range for a specific calendar year.
    #[must_use]
    pub fn year(year: i32) -> Self {
        let start = NaiveDate::from_ymd_opt(year, 1, 1).expect("January 1 is always valid");
        let end = NaiveDate::from_ymd_opt(year, 12, 31).expect("December 31 is always valid");
        Self::new(Some(start), Some(end), year.to_string())
    }

    /// Get the start date as a SQL-friendly string (YYYY-MM-DD format).
    #[must_use]
    pub fn start_sql(&self) -> Option<String> {
        self.start.map(|d| d.format("%Y-%m-%d").to_string())
    }

    /// Get the end date as a SQL-friendly string (YYYY-MM-DD format).
    #[must_use]
    pub fn end_sql(&self) -> Option<String> {
        self.end.map(|d| d.format("%Y-%m-%d").to_string())
    }

    /// Get the end date with time as a SQL-friendly string (YYYY-MM-DD HH:MM:SS format).
    ///
    /// Uses 23:59:59 to include the entire end day.
    #[must_use]
    pub fn end_sql_with_time(&self) -> Option<String> {
        self.end
            .map(|d| format!("{} 23:59:59", d.format("%Y-%m-%d")))
    }

    /// Convert to a tuple of optional SQL strings `(start, end)`.
    ///
    /// This is the format expected by most database query functions.
    #[must_use]
    pub fn to_sql_tuple(&self) -> (Option<String>, Option<String>) {
        (self.start_sql(), self.end_sql())
    }

    /// Convert to a tuple of optional SQL strings with end time `(start, end_with_time)`.
    ///
    /// Uses 23:59:59 for the end date to include the entire day.
    #[must_use]
    pub fn to_sql_tuple_with_end_time(&self) -> (Option<String>, Option<String>) {
        (self.start_sql(), self.end_sql_with_time())
    }

    /// Check if this is an unbounded (all time) range.
    #[must_use]
    pub const fn is_all_time(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }
}

impl fmt::Display for DateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

/// Build a date range from CLI flags.
///
/// This is a convenience function that translates CLI arguments into a `DateRange`.
///
/// # Arguments
///
/// * `all_time` - If true, returns an unbounded range
/// * `week` - If true, returns last 7 days
/// * `month` - If true, returns current calendar month
/// * `year` - If Some, returns that specific year; if None with no other flags, returns current year
///
/// # Priority
///
/// Flags are checked in order: all_time > week > month > year > default (current year)
#[must_use]
pub fn from_cli_flags(all_time: bool, week: bool, month: bool, year: Option<i32>) -> DateRange {
    if all_time {
        DateRange::all_time()
    } else if week {
        DateRange::last_week()
    } else if month {
        DateRange::current_month()
    } else if let Some(y) = year {
        DateRange::year(y)
    } else {
        // Default: current year
        DateRange::current_year()
    }
}

/// Date filter options for the GUI.
///
/// These represent the selectable time periods in the GUI dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateFilter {
    /// Today only
    Today,
    /// Last 7 days (rolling)
    Week,
    /// Last 30 days (rolling)
    Month,
    /// Last 365 days (rolling)
    Year,
    /// All time (no date filter)
    #[default]
    AllTime,
}

impl DateFilter {
    /// Convert to a `DateRange`.
    #[must_use]
    pub fn to_date_range(self) -> DateRange {
        match self {
            Self::Today => DateRange::today(),
            Self::Week => DateRange::last_week(),
            Self::Month => DateRange::last_month(),
            Self::Year => DateRange::last_year(),
            Self::AllTime => DateRange::all_time(),
        }
    }

    /// Get display name for the filter.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Today => "Today",
            Self::Week => "Past Week",
            Self::Month => "Past Month",
            Self::Year => "Past Year",
            Self::AllTime => "All Time",
        }
    }

    /// Get all filter options.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Today,
            Self::Week,
            Self::Month,
            Self::Year,
            Self::AllTime,
        ]
    }
}

impl From<DateFilter> for DateRange {
    fn from(filter: DateFilter) -> Self {
        filter.to_date_range()
    }
}

impl fmt::Display for DateFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_time() {
        let range = DateRange::all_time();
        assert!(range.start.is_none());
        assert!(range.end.is_none());
        assert_eq!(range.display_name, "All Time");
        assert_eq!(range.to_sql_tuple(), (None, None));
    }

    #[test]
    fn test_specific_year() {
        let range = DateRange::year(2024);
        assert_eq!(range.start_sql(), Some("2024-01-01".to_string()));
        assert_eq!(range.end_sql(), Some("2024-12-31".to_string()));
        assert_eq!(range.display_name, "2024");
    }

    #[test]
    fn test_today() {
        let range = DateRange::today();
        let today = Local::now().date_naive();
        assert_eq!(range.start, Some(today));
        assert_eq!(range.end, Some(today));
        assert_eq!(range.display_name, "Today");
    }

    #[test]
    fn test_last_week() {
        let range = DateRange::last_week();
        let today = Local::now().date_naive();
        let week_ago = today - chrono::Duration::days(7);
        assert_eq!(range.start, Some(week_ago));
        assert_eq!(range.end, Some(today));
        assert_eq!(range.display_name, "Last 7 Days");
    }

    #[test]
    fn test_end_sql_with_time() {
        let range = DateRange::year(2024);
        assert_eq!(
            range.end_sql_with_time(),
            Some("2024-12-31 23:59:59".to_string())
        );
    }

    #[test]
    fn test_cli_flags_all_time() {
        let range = from_cli_flags(true, false, false, None);
        assert!(range.start.is_none());
        assert!(range.end.is_none());
    }

    #[test]
    fn test_cli_flags_week() {
        let range = from_cli_flags(false, true, false, None);
        assert!(range.start.is_some());
        assert_eq!(range.display_name, "Last 7 Days");
    }

    #[test]
    fn test_cli_flags_specific_year() {
        let range = from_cli_flags(false, false, false, Some(2023));
        assert_eq!(range.start_sql(), Some("2023-01-01".to_string()));
        assert_eq!(range.end_sql(), Some("2023-12-31".to_string()));
    }

    #[test]
    fn test_cli_flags_priority() {
        // all_time takes precedence
        let range = from_cli_flags(true, true, true, Some(2023));
        assert!(range.start.is_none());

        // week takes precedence over month and year
        let range = from_cli_flags(false, true, true, Some(2023));
        assert_eq!(range.display_name, "Last 7 Days");
    }

    #[test]
    fn test_date_filter_conversion() {
        let filter = DateFilter::AllTime;
        let range = filter.to_date_range();
        assert!(range.start.is_none());
        assert!(range.end.is_none());

        let filter = DateFilter::Week;
        let range = filter.to_date_range();
        assert!(range.start.is_some());
        assert!(range.end.is_some());
    }

    #[test]
    fn test_date_filter_display_names() {
        assert_eq!(DateFilter::Today.display_name(), "Today");
        assert_eq!(DateFilter::Week.display_name(), "Past Week");
        assert_eq!(DateFilter::Month.display_name(), "Past Month");
        assert_eq!(DateFilter::Year.display_name(), "Past Year");
        assert_eq!(DateFilter::AllTime.display_name(), "All Time");
    }

    #[test]
    fn test_date_filter_all() {
        let all = DateFilter::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&DateFilter::Today));
        assert!(all.contains(&DateFilter::AllTime));
    }
}

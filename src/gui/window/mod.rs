//! Main window subclass for Music Analytics GUI

mod imp;

use gtk4::glib;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use libadwaita as adw;

use crate::gui::MusicAnalyticsApplication;

/// Data for the insights view
#[derive(Debug, Clone)]
pub struct InsightsData {
    pub current_streak: i32,
    pub longest_streak: i32,
    pub night_owl_percentage: f64,
    pub skip_rate: f64,
    pub skipped_count: i64,
    pub total_count: i64,
}

/// Data for the heatmap view
#[derive(Debug, Clone)]
pub struct HeatmapData {
    pub hours: [i64; 24],
    pub peak_hour: i32,
    pub peak_count: i64,
}

glib::wrapper! {
    /// The main application window
    pub struct MusicAnalyticsWindow(ObjectSubclass<imp::MusicAnalyticsWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gtk4::gio::ActionGroup, gtk4::gio::ActionMap;
}

impl MusicAnalyticsWindow {
    /// Create a new window for the given application
    #[must_use]
    pub fn new(app: &MusicAnalyticsApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    /// Reload all data from the database
    pub fn reload_data(&self) {
        self.imp().reload_data();
    }

    /// Get the currently selected date filter
    pub fn date_filter(&self) -> DateFilter {
        self.imp().date_filter()
    }
}

/// Date filter options for statistics queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateFilter {
    Week,
    Month,
    Year,
    #[default]
    AllTime,
}

impl DateFilter {
    /// Convert to start/end date strings for database queries
    #[must_use]
    pub fn to_date_range(self) -> (Option<String>, Option<String>) {
        use chrono::{Days, Local, Months};

        let today = Local::now().date_naive();
        let end = today.format("%Y-%m-%d").to_string();

        match self {
            Self::Week => {
                let start = today
                    .checked_sub_days(Days::new(7))
                    .map(|d| d.format("%Y-%m-%d").to_string());
                (start, Some(end))
            }
            Self::Month => {
                let start = today
                    .checked_sub_months(Months::new(1))
                    .map(|d| d.format("%Y-%m-%d").to_string());
                (start, Some(end))
            }
            Self::Year => {
                let start = today
                    .checked_sub_months(Months::new(12))
                    .map(|d| d.format("%Y-%m-%d").to_string());
                (start, Some(end))
            }
            Self::AllTime => (None, None),
        }
    }

    /// Get display name for the filter
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Week => "Past Week",
            Self::Month => "Past Month",
            Self::Year => "Past Year",
            Self::AllTime => "All Time",
        }
    }

    /// Get all filter options
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Week, Self::Month, Self::Year, Self::AllTime]
    }
}

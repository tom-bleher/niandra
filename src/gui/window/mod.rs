//! Main window subclass for Music Analytics GUI

mod imp;

use gtk4::glib;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use libadwaita as adw;

use crate::gui::MusicAnalyticsApplication;

// Re-export DateFilter from the shared date_range module
pub use crate::date_range::DateFilter;

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

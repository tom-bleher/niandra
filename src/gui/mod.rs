//! GTK4/Libadwaita GUI for Music Analytics
//!
//! This module provides a GNOME HIG-compliant graphical interface for viewing
//! listening statistics and analytics.

pub mod application;
pub mod models;
pub mod views;
pub mod widgets;
pub mod window;

use std::sync::OnceLock;

use gtk4 as gtk;
use gtk4::prelude::*;
use tokio::runtime::Runtime;

pub use application::MusicAnalyticsApplication;
pub use window::MusicAnalyticsWindow;

/// Shared tokio runtime for async database operations
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Get or initialize the shared tokio runtime
pub fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("Failed to create tokio runtime")
    })
}

/// Application ID for the GUI application
pub const APP_ID: &str = "com.github.tombleher.MusicAnalytics";

/// Initialize GTK and libadwaita, then run the application
///
/// # Errors
///
/// Returns an error if the application fails to start
pub fn run() -> gtk::glib::ExitCode {
    // Initialize GTK
    gtk::init().expect("Failed to initialize GTK");

    // Create and run the application
    let app = MusicAnalyticsApplication::new();
    app.run()
}

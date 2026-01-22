//! Application subclass for Music Analytics GUI

mod imp;

use gtk4::glib;
use libadwaita as adw;

use crate::gui::APP_ID;

glib::wrapper! {
    /// The main application object for Music Analytics GUI
    pub struct MusicAnalyticsApplication(ObjectSubclass<imp::MusicAnalyticsApplication>)
        @extends adw::Application, gtk4::Application, gtk4::gio::Application,
        @implements gtk4::gio::ActionGroup, gtk4::gio::ActionMap;
}

impl MusicAnalyticsApplication {
    /// Create a new application instance
    #[must_use]
    pub fn new() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gtk4::gio::ApplicationFlags::FLAGS_NONE)
            .build()
    }
}

impl Default for MusicAnalyticsApplication {
    fn default() -> Self {
        Self::new()
    }
}

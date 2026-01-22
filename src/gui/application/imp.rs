//! GObject implementation for MusicAnalyticsApplication

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use libadwaita::subclass::prelude::*;

use crate::gui::MusicAnalyticsWindow;

#[derive(Debug, Default)]
pub struct MusicAnalyticsApplication;

#[glib::object_subclass]
impl ObjectSubclass for MusicAnalyticsApplication {
    const NAME: &'static str = "MusicAnalyticsApplication";
    type Type = super::MusicAnalyticsApplication;
    type ParentType = adw::Application;
}

impl ObjectImpl for MusicAnalyticsApplication {}

impl ApplicationImpl for MusicAnalyticsApplication {
    fn activate(&self) {
        let app = self.obj();

        // Check if we already have a window
        if let Some(window) = app.active_window() {
            window.present();
            return;
        }

        // Create a new window
        let window = MusicAnalyticsWindow::new(&app);
        window.present();
    }

    fn startup(&self) {
        self.parent_startup();

        // Initialize libadwaita
        adw::init().expect("Failed to initialize libadwaita");

        // Set up application actions
        self.setup_actions();
    }
}

impl GtkApplicationImpl for MusicAnalyticsApplication {}

impl AdwApplicationImpl for MusicAnalyticsApplication {}

impl MusicAnalyticsApplication {
    fn setup_actions(&self) {
        let app = self.obj();

        // Quit action
        let quit_action = gtk4::gio::ActionEntry::builder("quit")
            .activate(|app: &super::MusicAnalyticsApplication, _, _| {
                app.quit();
            })
            .build();

        // About action
        let about_action = gtk4::gio::ActionEntry::builder("about")
            .activate(|app: &super::MusicAnalyticsApplication, _, _| {
                Self::show_about_dialog(app);
            })
            .build();

        app.add_action_entries([quit_action, about_action]);

        // Set up keyboard shortcuts
        app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
    }

    fn show_about_dialog(app: &super::MusicAnalyticsApplication) {
        let window = app.active_window();

        let about = adw::AboutDialog::builder()
            .application_name("Niandra")
            .application_icon(crate::gui::APP_ID)
            .developer_name("Tom Bleher")
            .version(crate::VERSION)
            .website("https://github.com/tombleher/niandra")
            .issue_url("https://github.com/tombleher/niandra/issues")
            .license_type(gtk4::License::MitX11)
            .developers(vec!["Tom Bleher"])
            .comments("Personal music listening analytics for Linux")
            .build();

        about.present(window.as_ref());
    }
}

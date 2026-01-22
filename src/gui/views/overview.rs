//! Overview dashboard view showing key statistics

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;

use crate::db::OverviewStats;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct OverviewView {
        pub total_plays_row: RefCell<Option<adw::ActionRow>>,
        pub total_hours_row: RefCell<Option<adw::ActionRow>>,
        pub unique_artists_row: RefCell<Option<adw::ActionRow>>,
        pub unique_albums_row: RefCell<Option<adw::ActionRow>>,
        pub unique_tracks_row: RefCell<Option<adw::ActionRow>>,
        pub spinner: RefCell<Option<gtk4::Spinner>>,
        pub stack: RefCell<Option<gtk4::Stack>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OverviewView {
        const NAME: &'static str = "OverviewView";
        type Type = super::OverviewView;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for OverviewView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for OverviewView {}

    impl adw::subclass::prelude::BinImpl for OverviewView {}

    impl OverviewView {
        fn setup_ui(&self) {
            let obj = self.obj();

            // Main container
            let stack = gtk4::Stack::new();
            stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
            *self.stack.borrow_mut() = Some(stack.clone());

            // Loading spinner
            let spinner = gtk4::Spinner::new();
            spinner.set_halign(gtk4::Align::Center);
            spinner.set_valign(gtk4::Align::Center);
            spinner.set_width_request(32);
            spinner.set_height_request(32);
            stack.add_named(&spinner, Some("loading"));
            *self.spinner.borrow_mut() = Some(spinner);

            // Content container
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let clamp = adw::Clamp::new();
            clamp.set_maximum_size(600);
            clamp.set_margin_top(24);
            clamp.set_margin_bottom(24);
            clamp.set_margin_start(12);
            clamp.set_margin_end(12);

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);

            // Activity group
            let activity_group = adw::PreferencesGroup::new();
            activity_group.set_title("Listening Activity");

            let total_plays_row = adw::ActionRow::new();
            total_plays_row.set_title("Total Plays");
            total_plays_row.set_subtitle("—");
            total_plays_row.add_prefix(&Self::create_icon("media-playback-start-symbolic"));
            activity_group.add(&total_plays_row);
            *self.total_plays_row.borrow_mut() = Some(total_plays_row);

            let total_hours_row = adw::ActionRow::new();
            total_hours_row.set_title("Time Listened");
            total_hours_row.set_subtitle("—");
            total_hours_row.add_prefix(&Self::create_icon("preferences-system-time-symbolic"));
            activity_group.add(&total_hours_row);
            *self.total_hours_row.borrow_mut() = Some(total_hours_row);

            content.append(&activity_group);

            // Library group
            let library_group = adw::PreferencesGroup::new();
            library_group.set_title("Library");

            let unique_artists_row = adw::ActionRow::new();
            unique_artists_row.set_title("Artists");
            unique_artists_row.set_subtitle("—");
            unique_artists_row.add_prefix(&Self::create_icon("avatar-default-symbolic"));
            library_group.add(&unique_artists_row);
            *self.unique_artists_row.borrow_mut() = Some(unique_artists_row);

            let unique_albums_row = adw::ActionRow::new();
            unique_albums_row.set_title("Albums");
            unique_albums_row.set_subtitle("—");
            unique_albums_row.add_prefix(&Self::create_icon("media-optical-symbolic"));
            library_group.add(&unique_albums_row);
            *self.unique_albums_row.borrow_mut() = Some(unique_albums_row);

            let unique_tracks_row = adw::ActionRow::new();
            unique_tracks_row.set_title("Tracks");
            unique_tracks_row.set_subtitle("—");
            unique_tracks_row.add_prefix(&Self::create_icon("emblem-music-symbolic"));
            library_group.add(&unique_tracks_row);
            *self.unique_tracks_row.borrow_mut() = Some(unique_tracks_row);

            content.append(&library_group);

            clamp.set_child(Some(&content));
            scrolled.set_child(Some(&clamp));
            stack.add_named(&scrolled, Some("content"));

            // Start with loading state
            stack.set_visible_child_name("loading");

            obj.set_child(Some(&stack));
        }

        fn create_icon(icon_name: &str) -> gtk4::Image {
            let image = gtk4::Image::from_icon_name(icon_name);
            image.set_icon_size(gtk4::IconSize::Large);
            image
        }
    }
}

glib::wrapper! {
    /// Overview dashboard view
    pub struct OverviewView(ObjectSubclass<imp::OverviewView>)
        @extends adw::Bin, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl OverviewView {
    /// Create a new overview view
    #[must_use]
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set statistics data
    pub fn set_stats(&self, stats: &OverviewStats) {
        let imp = self.imp();

        if let Some(row) = imp.total_plays_row.borrow().as_ref() {
            row.set_subtitle(&format_number(stats.total_plays));
        }

        if let Some(row) = imp.total_hours_row.borrow().as_ref() {
            row.set_subtitle(&format_duration(stats.total_ms));
        }

        if let Some(row) = imp.unique_artists_row.borrow().as_ref() {
            row.set_subtitle(&format_number(stats.unique_artists));
        }

        if let Some(row) = imp.unique_albums_row.borrow().as_ref() {
            row.set_subtitle(&format_number(stats.unique_albums));
        }

        if let Some(row) = imp.unique_tracks_row.borrow().as_ref() {
            row.set_subtitle(&format_number(stats.unique_tracks));
        }

        self.set_loading(false);
    }

    /// Set loading state
    pub fn set_loading(&self, loading: bool) {
        let Some(stack) = self.imp().stack.borrow().as_ref().cloned() else {
            return;
        };

        if loading {
            if let Some(spinner) = self.imp().spinner.borrow().as_ref() {
                spinner.start();
            }
            stack.set_visible_child_name("loading");
        } else {
            if let Some(spinner) = self.imp().spinner.borrow().as_ref() {
                spinner.stop();
            }
            stack.set_visible_child_name("content");
        }
    }
}

impl Default for OverviewView {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a number with thousands separators
fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    result
}

/// Format milliseconds as a human-readable duration
fn format_duration(ms: i64) -> String {
    let total_seconds = ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

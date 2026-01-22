//! Insights view showing listening streaks, night owl score, skip rate

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::gui::window::InsightsData;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct InsightsView {
        pub stack: gtk4::Stack,
        pub spinner: gtk4::Spinner,
        // Streak rows
        pub current_streak_row: adw::ActionRow,
        pub longest_streak_row: adw::ActionRow,
        // Night owl row
        pub night_owl_row: adw::ActionRow,
        pub night_owl_bar: gtk4::ProgressBar,
        // Skip rate rows
        pub skip_rate_row: adw::ActionRow,
        pub skip_rate_bar: gtk4::ProgressBar,
    }

    impl Default for InsightsView {
        fn default() -> Self {
            Self {
                stack: gtk4::Stack::new(),
                spinner: gtk4::Spinner::new(),
                current_streak_row: adw::ActionRow::new(),
                longest_streak_row: adw::ActionRow::new(),
                night_owl_row: adw::ActionRow::new(),
                night_owl_bar: gtk4::ProgressBar::new(),
                skip_rate_row: adw::ActionRow::new(),
                skip_rate_bar: gtk4::ProgressBar::new(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InsightsView {
        const NAME: &'static str = "InsightsView";
        type Type = super::InsightsView;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for InsightsView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for InsightsView {}

    impl adw::subclass::prelude::BinImpl for InsightsView {}

    impl InsightsView {
        fn setup_ui(&self) {
            let obj = self.obj();

            self.stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

            // Loading spinner
            self.spinner.set_halign(gtk4::Align::Center);
            self.spinner.set_valign(gtk4::Align::Center);
            self.spinner.set_width_request(48);
            self.spinner.set_height_request(48);
            self.stack.add_named(&self.spinner, Some("loading"));

            // Content
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let clamp = adw::Clamp::new();
            clamp.set_maximum_size(600);
            clamp.set_margin_top(24);
            clamp.set_margin_bottom(24);
            clamp.set_margin_start(12);
            clamp.set_margin_end(12);

            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);

            // Streaks section
            let streaks_group = adw::PreferencesGroup::new();
            streaks_group.set_title("Listening Streaks");
            streaks_group.set_description(Some("Consecutive days with music played"));

            self.current_streak_row.set_title("Current Streak");
            self.current_streak_row.set_subtitle("—");
            self.current_streak_row.add_prefix(&gtk4::Image::from_icon_name("weather-clear-symbolic"));
            streaks_group.add(&self.current_streak_row);

            self.longest_streak_row.set_title("Longest Streak");
            self.longest_streak_row.set_subtitle("—");
            self.longest_streak_row.add_prefix(&gtk4::Image::from_icon_name("starred-symbolic"));
            streaks_group.add(&self.longest_streak_row);

            content.append(&streaks_group);

            // Night Owl section
            let night_owl_group = adw::PreferencesGroup::new();
            night_owl_group.set_title("Night Owl Score");
            night_owl_group.set_description(Some("Percentage of plays between 10 PM and 4 AM"));

            let night_owl_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            night_owl_box.set_margin_end(12);
            self.night_owl_bar.set_hexpand(true);
            self.night_owl_bar.add_css_class("osd");
            night_owl_box.append(&self.night_owl_bar);

            self.night_owl_row.set_title("Night Owl");
            self.night_owl_row.set_subtitle("—");
            self.night_owl_row.add_prefix(&gtk4::Image::from_icon_name("weather-clear-night-symbolic"));
            self.night_owl_row.add_suffix(&night_owl_box);
            night_owl_group.add(&self.night_owl_row);

            content.append(&night_owl_group);

            // Skip Rate section
            let skip_group = adw::PreferencesGroup::new();
            skip_group.set_title("Skip Behavior");
            skip_group.set_description(Some("How often you skip tracks"));

            let skip_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            skip_box.set_margin_end(12);
            self.skip_rate_bar.set_hexpand(true);
            self.skip_rate_bar.add_css_class("osd");
            skip_box.append(&self.skip_rate_bar);

            self.skip_rate_row.set_title("Skip Rate");
            self.skip_rate_row.set_subtitle("—");
            self.skip_rate_row.add_prefix(&gtk4::Image::from_icon_name("media-skip-forward-symbolic"));
            self.skip_rate_row.add_suffix(&skip_box);
            skip_group.add(&self.skip_rate_row);

            content.append(&skip_group);

            clamp.set_child(Some(&content));
            scrolled.set_child(Some(&clamp));
            self.stack.add_named(&scrolled, Some("content"));

            self.stack.set_visible_child_name("loading");

            obj.set_child(Some(&self.stack));
        }
    }
}

glib::wrapper! {
    /// Insights view showing listening patterns and behaviors
    pub struct InsightsView(ObjectSubclass<imp::InsightsView>)
        @extends adw::Bin, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl InsightsView {
    /// Create a new insights view
    #[must_use]
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set insights data
    pub fn set_data(&self, data: &InsightsData) {
        let imp = self.imp();

        // Streaks
        let current_streak_text = if data.current_streak == 1 {
            "1 day".to_string()
        } else {
            format!("{} days", data.current_streak)
        };
        imp.current_streak_row.set_subtitle(&current_streak_text);

        let longest_streak_text = if data.longest_streak == 1 {
            "1 day".to_string()
        } else {
            format!("{} days", data.longest_streak)
        };
        imp.longest_streak_row.set_subtitle(&longest_streak_text);

        // Night owl
        let night_owl_text = format!("{:.1}%", data.night_owl_percentage);
        imp.night_owl_row.set_subtitle(&night_owl_text);
        imp.night_owl_bar.set_fraction(data.night_owl_percentage / 100.0);

        // Skip rate
        let skip_text = format!(
            "{:.1}% ({} / {})",
            data.skip_rate * 100.0,
            data.skipped_count,
            data.total_count
        );
        imp.skip_rate_row.set_subtitle(&skip_text);
        imp.skip_rate_bar.set_fraction(data.skip_rate);

        self.set_loading(false);
    }

    /// Set loading state
    pub fn set_loading(&self, loading: bool) {
        let imp = self.imp();

        if loading {
            imp.spinner.start();
            imp.stack.set_visible_child_name("loading");
        } else {
            imp.spinner.stop();
            imp.stack.set_visible_child_name("content");
        }
    }
}

impl Default for InsightsView {
    fn default() -> Self {
        Self::new()
    }
}

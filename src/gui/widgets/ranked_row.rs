//! RankedRow widget for displaying ranked list items

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::Cell;

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct RankedRow {
        pub rank_label: gtk4::Label,
        pub title_label: gtk4::Label,
        pub subtitle_label: gtk4::Label,
        pub count_label: gtk4::Label,
        pub hours_label: gtk4::Label,
        pub progress_bar: gtk4::ProgressBar,
        pub rank: Cell<u32>,
    }

    impl Default for RankedRow {
        fn default() -> Self {
            Self {
                rank_label: gtk4::Label::new(None),
                title_label: gtk4::Label::new(None),
                subtitle_label: gtk4::Label::new(None),
                count_label: gtk4::Label::new(None),
                hours_label: gtk4::Label::new(None),
                progress_bar: gtk4::ProgressBar::new(),
                rank: Cell::new(0),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RankedRow {
        const NAME: &'static str = "RankedRow";
        type Type = super::RankedRow;
        type ParentType = gtk4::Box;
    }

    impl ObjectImpl for RankedRow {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for RankedRow {}

    impl BoxImpl for RankedRow {}

    impl RankedRow {
        fn setup_ui(&self) {
            let obj = self.obj();

            obj.set_orientation(gtk4::Orientation::Horizontal);
            obj.set_spacing(12);
            obj.set_margin_top(8);
            obj.set_margin_bottom(8);
            obj.set_margin_start(12);
            obj.set_margin_end(12);

            // Rank label
            self.rank_label.add_css_class("dim-label");
            self.rank_label.add_css_class("numeric");
            self.rank_label.set_width_chars(3);
            self.rank_label.set_xalign(1.0);
            obj.append(&self.rank_label);

            // Main content box
            let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
            main_box.set_hexpand(true);

            // Title
            self.title_label.set_xalign(0.0);
            self.title_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            self.title_label.add_css_class("heading");
            main_box.append(&self.title_label);

            // Progress bar
            self.progress_bar.set_hexpand(true);
            self.progress_bar.add_css_class("osd");
            main_box.append(&self.progress_bar);

            // Subtitle (artist for albums/tracks)
            self.subtitle_label.set_xalign(0.0);
            self.subtitle_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            self.subtitle_label.add_css_class("dim-label");
            self.subtitle_label.add_css_class("caption");
            self.subtitle_label.set_visible(false);
            main_box.append(&self.subtitle_label);

            obj.append(&main_box);

            // Stats box
            let stats_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
            stats_box.set_valign(gtk4::Align::Center);

            self.count_label.set_xalign(1.0);
            self.count_label.add_css_class("numeric");
            stats_box.append(&self.count_label);

            self.hours_label.set_xalign(1.0);
            self.hours_label.add_css_class("dim-label");
            self.hours_label.add_css_class("caption");
            stats_box.append(&self.hours_label);

            obj.append(&stats_box);
        }
    }
}

glib::wrapper! {
    /// A row widget for displaying ranked items in top lists
    pub struct RankedRow(ObjectSubclass<imp::RankedRow>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl RankedRow {
    /// Create a new ranked row
    #[must_use]
    pub fn new() -> Self {
        glib::Object::new()
    }

    /// Set the rank number
    pub fn set_rank(&self, rank: u32) {
        self.imp().rank.set(rank);
        self.imp().rank_label.set_text(&format!("{rank}."));
    }

    /// Set the title text
    pub fn set_title(&self, title: &str) {
        let truncated = crate::display::truncate(title, 50);
        self.imp().title_label.set_text(&truncated);
        self.imp().title_label.set_tooltip_text(Some(title));
    }

    /// Set the subtitle text (e.g., artist name for albums/tracks)
    pub fn set_subtitle(&self, subtitle: Option<&str>) {
        let imp = self.imp();
        match subtitle {
            Some(text) => {
                let truncated = crate::display::truncate(text, 40);
                imp.subtitle_label.set_text(&truncated);
                imp.subtitle_label.set_tooltip_text(Some(text));
                imp.subtitle_label.set_visible(true);
            }
            None => {
                imp.subtitle_label.set_visible(false);
            }
        }
    }

    /// Set the play count
    pub fn set_play_count(&self, count: i64) {
        let text = if count == 1 {
            "1 play".to_string()
        } else {
            format!("{count} plays")
        };
        self.imp().count_label.set_text(&text);
    }

    /// Set the total hours
    pub fn set_hours(&self, ms: i64) {
        let hours = crate::display::format_hours(ms);
        self.imp().hours_label.set_text(&format!("{hours:.1}h"));
    }

    /// Set the progress bar fraction (0.0 to 1.0)
    pub fn set_progress(&self, fraction: f64) {
        self.imp().progress_bar.set_fraction(fraction.clamp(0.0, 1.0));
    }

    /// Configure the row for artist data
    pub fn bind_artist(&self, artist: &crate::gui::models::ArtistObject) {
        self.set_rank(artist.rank());
        self.set_title(&artist.artist());
        self.set_subtitle(None);
        self.set_play_count(artist.play_count());
        self.set_hours(artist.total_ms());
        self.set_progress(artist.progress_fraction());
    }

    /// Configure the row for album data
    pub fn bind_album(&self, album: &crate::gui::models::AlbumObject) {
        self.set_rank(album.rank());
        self.set_title(&album.album());
        self.set_subtitle(album.artist().as_deref());
        self.set_play_count(album.play_count());
        self.set_hours(album.total_ms());
        self.set_progress(album.progress_fraction());
    }

    /// Configure the row for track data
    pub fn bind_track(&self, track: &crate::gui::models::TrackObject) {
        self.set_rank(track.rank());
        self.set_title(&track.title());
        self.set_subtitle(track.artist().as_deref());
        self.set_play_count(track.play_count());
        self.set_hours(track.total_ms());
        self.set_progress(track.progress_fraction());
    }
}

impl Default for RankedRow {
    fn default() -> Self {
        Self::new()
    }
}

//! Top lists view for Artists, Albums, and Tracks

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;

use crate::db::{AlbumStats, ArtistStats, TrackStats};
use crate::gui::models::{AlbumObject, ArtistObject, TrackObject};
use crate::gui::widgets::RankedRow;

/// The type of data displayed in the list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListType {
    #[default]
    Artists,
    Albums,
    Tracks,
}

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct TopListsView {
        pub list_type: RefCell<ListType>,
        pub list_box: gtk4::ListBox,
        pub spinner: gtk4::Spinner,
        pub stack: gtk4::Stack,
        pub empty_status: adw::StatusPage,
        pub model: RefCell<Option<gtk4::gio::ListStore>>,
    }

    impl Default for TopListsView {
        fn default() -> Self {
            Self {
                list_type: RefCell::new(ListType::Artists),
                list_box: gtk4::ListBox::new(),
                spinner: gtk4::Spinner::new(),
                stack: gtk4::Stack::new(),
                empty_status: adw::StatusPage::new(),
                model: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TopListsView {
        const NAME: &'static str = "TopListsView";
        type Type = super::TopListsView;
        type ParentType = adw::Bin;
    }

    impl ObjectImpl for TopListsView {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_ui();
        }
    }

    impl WidgetImpl for TopListsView {}

    impl adw::subclass::prelude::BinImpl for TopListsView {}

    impl TopListsView {
        fn setup_ui(&self) {
            let obj = self.obj();

            self.stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

            // Loading spinner
            self.spinner.set_halign(gtk4::Align::Center);
            self.spinner.set_valign(gtk4::Align::Center);
            self.spinner.set_width_request(48);
            self.spinner.set_height_request(48);
            self.stack.add_named(&self.spinner, Some("loading"));

            // Empty state
            self.empty_status.set_icon_name(Some("emblem-music-symbolic"));
            self.empty_status.set_title("No Data");
            self.empty_status.set_description(Some("No listening history found for this time period"));
            self.stack.add_named(&self.empty_status, Some("empty"));

            // Content
            let scrolled = gtk4::ScrolledWindow::new();
            scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);

            let clamp = adw::Clamp::new();
            clamp.set_maximum_size(800);
            clamp.set_margin_top(12);
            clamp.set_margin_bottom(12);
            clamp.set_margin_start(12);
            clamp.set_margin_end(12);

            self.list_box.set_selection_mode(gtk4::SelectionMode::None);
            self.list_box.add_css_class("boxed-list");

            clamp.set_child(Some(&self.list_box));
            scrolled.set_child(Some(&clamp));
            self.stack.add_named(&scrolled, Some("content"));

            self.stack.set_visible_child_name("loading");

            obj.set_child(Some(&self.stack));
        }
    }
}

glib::wrapper! {
    /// View for displaying ranked top lists
    pub struct TopListsView(ObjectSubclass<imp::TopListsView>)
        @extends adw::Bin, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl TopListsView {
    /// Create a new top lists view for artists
    #[must_use]
    pub fn new_artists() -> Self {
        let obj: Self = glib::Object::new();
        *obj.imp().list_type.borrow_mut() = ListType::Artists;
        obj.imp().empty_status.set_title("No Artists");
        obj.imp().empty_status.set_icon_name(Some("avatar-default-symbolic"));
        obj
    }

    /// Create a new top lists view for albums
    #[must_use]
    pub fn new_albums() -> Self {
        let obj: Self = glib::Object::new();
        *obj.imp().list_type.borrow_mut() = ListType::Albums;
        obj.imp().empty_status.set_title("No Albums");
        obj.imp().empty_status.set_icon_name(Some("media-optical-symbolic"));
        obj
    }

    /// Create a new top lists view for tracks
    #[must_use]
    pub fn new_tracks() -> Self {
        let obj: Self = glib::Object::new();
        *obj.imp().list_type.borrow_mut() = ListType::Tracks;
        obj.imp().empty_status.set_title("No Tracks");
        obj.imp().empty_status.set_icon_name(Some("emblem-music-symbolic"));
        obj
    }

    /// Set artist data
    pub fn set_artist_data(&self, artists: &[ArtistStats]) {
        let imp = self.imp();

        // Clear existing rows
        while let Some(child) = imp.list_box.first_child() {
            imp.list_box.remove(&child);
        }

        if artists.is_empty() {
            imp.stack.set_visible_child_name("empty");
            return;
        }

        let max_plays = artists.first().map_or(1, |a| a.play_count.max(1));

        for (i, artist) in artists.iter().enumerate() {
            let obj = ArtistObject::new(artist, (i + 1) as u32, max_plays);
            let row = RankedRow::new();
            row.bind_artist(&obj);
            imp.list_box.append(&row);
        }

        self.set_loading(false);
    }

    /// Set album data
    pub fn set_album_data(&self, albums: &[AlbumStats]) {
        let imp = self.imp();

        // Clear existing rows
        while let Some(child) = imp.list_box.first_child() {
            imp.list_box.remove(&child);
        }

        if albums.is_empty() {
            imp.stack.set_visible_child_name("empty");
            return;
        }

        let max_plays = albums.first().map_or(1, |a| a.play_count.max(1));

        for (i, album) in albums.iter().enumerate() {
            let obj = AlbumObject::new(album, (i + 1) as u32, max_plays);
            let row = RankedRow::new();
            row.bind_album(&obj);
            imp.list_box.append(&row);
        }

        self.set_loading(false);
    }

    /// Set track data
    pub fn set_track_data(&self, tracks: &[TrackStats]) {
        let imp = self.imp();

        // Clear existing rows
        while let Some(child) = imp.list_box.first_child() {
            imp.list_box.remove(&child);
        }

        if tracks.is_empty() {
            imp.stack.set_visible_child_name("empty");
            return;
        }

        let max_plays = tracks.first().map_or(1, |t| t.play_count.max(1));

        for (i, track) in tracks.iter().enumerate() {
            let obj = TrackObject::new(track, (i + 1) as u32, max_plays);
            let row = RankedRow::new();
            row.bind_track(&obj);
            imp.list_box.append(&row);
        }

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

impl Default for TopListsView {
    fn default() -> Self {
        Self::new_artists()
    }
}

//! GObject wrapper for `AlbumStats`.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::db::AlbumStats;

mod imp {
    use super::*;
    use std::sync::OnceLock;

    #[derive(Debug, Default)]
    pub struct AlbumObject {
        pub album: RefCell<String>,
        pub artist: RefCell<Option<String>>,
        pub play_count: Cell<i64>,
        pub total_ms: Cell<i64>,
        pub rank: Cell<u32>,
        pub max_plays: Cell<i64>,
        pub art_url: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AlbumObject {
        const NAME: &'static str = "AlbumObject";
        type Type = super::AlbumObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for AlbumObject {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                let mut props = vec![
                    glib::ParamSpecString::builder("album").read_only().build(),
                    glib::ParamSpecString::builder("artist").read_only().build(),
                    glib::ParamSpecString::builder("art-url").read_only().build(),
                ];
                props.extend(common_stats_properties!());
                props
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            if let Some(value) = common_stats_property_value!(self, pspec) {
                return value;
            }
            match pspec.name() {
                "album" => self.album.borrow().to_value(),
                "artist" => self.artist.borrow().to_value(),
                "art-url" => self.art_url.borrow().to_value(),
                _ => unimplemented!("Property {} not implemented", pspec.name()),
            }
        }
    }
}

glib::wrapper! {
    /// GObject wrapper for album statistics.
    pub struct AlbumObject(ObjectSubclass<imp::AlbumObject>);
}

impl AlbumObject {
    /// Create a new `AlbumObject` from stats.
    #[must_use]
    pub fn new(stats: &AlbumStats, rank: u32, max_plays: i64) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        *imp.album.borrow_mut() = stats.album.clone();
        *imp.artist.borrow_mut() = stats.artist.clone();
        *imp.art_url.borrow_mut() = stats.art_url.clone();
        set_common_stats!(imp, stats.play_count, stats.total_ms, rank, max_plays);
        obj
    }

    #[must_use]
    pub fn album(&self) -> String {
        self.imp().album.borrow().clone()
    }

    #[must_use]
    pub fn artist(&self) -> Option<String> {
        self.imp().artist.borrow().clone()
    }

    #[must_use]
    pub fn art_url(&self) -> Option<String> {
        self.imp().art_url.borrow().clone()
    }
}

impl_common_accessors!(AlbumObject);
impl_stats_object!(AlbumObject);
impl_gobject_default!(AlbumObject);

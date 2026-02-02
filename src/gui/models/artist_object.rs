//! GObject wrapper for `ArtistStats`.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::db::ArtistStats;

mod imp {
    use super::*;
    use std::sync::OnceLock;

    #[derive(Debug, Default)]
    pub struct ArtistObject {
        pub artist: RefCell<String>,
        pub play_count: Cell<i64>,
        pub total_ms: Cell<i64>,
        pub rank: Cell<u32>,
        pub max_plays: Cell<i64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArtistObject {
        const NAME: &'static str = "ArtistObject";
        type Type = super::ArtistObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ArtistObject {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                let mut props = vec![glib::ParamSpecString::builder("artist")
                    .read_only()
                    .build()];
                props.extend(common_stats_properties!());
                props
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            if let Some(value) = common_stats_property_value!(self, pspec) {
                return value;
            }
            match pspec.name() {
                "artist" => self.artist.borrow().to_value(),
                _ => unimplemented!("Property {} not implemented", pspec.name()),
            }
        }
    }
}

glib::wrapper! {
    /// GObject wrapper for artist statistics.
    pub struct ArtistObject(ObjectSubclass<imp::ArtistObject>);
}

impl ArtistObject {
    /// Create a new `ArtistObject` from stats.
    #[must_use]
    pub fn new(stats: &ArtistStats, rank: u32, max_plays: i64) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        *imp.artist.borrow_mut() = stats.artist.clone();
        set_common_stats!(imp, stats.play_count, stats.total_ms, rank, max_plays);
        obj
    }

    #[must_use]
    pub fn artist(&self) -> String {
        self.imp().artist.borrow().clone()
    }
}

impl_common_accessors!(ArtistObject);
impl_stats_object!(ArtistObject);
impl_gobject_default!(ArtistObject);

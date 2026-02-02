//! GObject wrapper for `TrackStats`.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::db::TrackStats;

mod imp {
    use super::*;
    use std::sync::OnceLock;

    #[derive(Debug, Default)]
    pub struct TrackObject {
        pub title: RefCell<String>,
        pub artist: RefCell<Option<String>>,
        pub play_count: Cell<i64>,
        pub total_ms: Cell<i64>,
        pub rank: Cell<u32>,
        pub max_plays: Cell<i64>,
        pub art_url: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TrackObject {
        const NAME: &'static str = "TrackObject";
        type Type = super::TrackObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for TrackObject {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                let mut props = vec![
                    glib::ParamSpecString::builder("title").read_only().build(),
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
                "title" => self.title.borrow().to_value(),
                "artist" => self.artist.borrow().to_value(),
                "art-url" => self.art_url.borrow().to_value(),
                _ => unimplemented!("Property {} not implemented", pspec.name()),
            }
        }
    }
}

glib::wrapper! {
    /// GObject wrapper for track statistics.
    pub struct TrackObject(ObjectSubclass<imp::TrackObject>);
}

impl TrackObject {
    /// Create a new `TrackObject` from stats.
    #[must_use]
    pub fn new(stats: &TrackStats, rank: u32, max_plays: i64) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        *imp.title.borrow_mut() = stats.title.clone();
        *imp.artist.borrow_mut() = stats.artist.clone();
        *imp.art_url.borrow_mut() = stats.art_url.clone();
        set_common_stats!(imp, stats.play_count, stats.total_ms, rank, max_plays);
        obj
    }

    #[must_use]
    pub fn title(&self) -> String {
        self.imp().title.borrow().clone()
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

impl_common_accessors!(TrackObject);
impl_stats_object!(TrackObject);
impl_gobject_default!(TrackObject);

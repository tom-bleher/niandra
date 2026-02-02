//! GObject wrapper for TrackStats

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::db::TrackStats;

mod imp {
    use super::*;

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
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("title")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("artist")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt64::builder("play-count")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt64::builder("total-ms")
                        .read_only()
                        .build(),
                    glib::ParamSpecUInt::builder("rank")
                        .read_only()
                        .build(),
                    glib::ParamSpecInt64::builder("max-plays")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("art-url")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "artist" => self.artist.borrow().to_value(),
                "play-count" => self.play_count.get().to_value(),
                "total-ms" => self.total_ms.get().to_value(),
                "rank" => self.rank.get().to_value(),
                "max-plays" => self.max_plays.get().to_value(),
                "art-url" => self.art_url.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    /// GObject wrapper for track statistics
    pub struct TrackObject(ObjectSubclass<imp::TrackObject>);
}

impl TrackObject {
    /// Create a new `TrackObject` from stats
    #[must_use]
    pub fn new(stats: &TrackStats, rank: u32, max_plays: i64) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        *imp.title.borrow_mut() = stats.title.clone();
        *imp.artist.borrow_mut() = stats.artist.clone();
        imp.play_count.set(stats.play_count);
        imp.total_ms.set(stats.total_ms);
        imp.rank.set(rank);
        imp.max_plays.set(max_plays);
        *imp.art_url.borrow_mut() = stats.art_url.clone();
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
    pub fn play_count(&self) -> i64 {
        self.imp().play_count.get()
    }

    #[must_use]
    pub fn total_ms(&self) -> i64 {
        self.imp().total_ms.get()
    }

    #[must_use]
    pub fn rank(&self) -> u32 {
        self.imp().rank.get()
    }

    #[must_use]
    pub fn max_plays(&self) -> i64 {
        self.imp().max_plays.get()
    }

    #[must_use]
    pub fn art_url(&self) -> Option<String> {
        self.imp().art_url.borrow().clone()
    }

    /// Get hours as a formatted float
    #[must_use]
    pub fn hours(&self) -> f64 {
        crate::display::format_hours(self.total_ms())
    }

    /// Get progress bar fraction (0.0 to 1.0)
    #[must_use]
    pub fn progress_fraction(&self) -> f64 {
        let max = self.max_plays();
        if max > 0 {
            self.play_count() as f64 / max as f64
        } else {
            0.0
        }
    }
}

impl Default for TrackObject {
    fn default() -> Self {
        glib::Object::new()
    }
}

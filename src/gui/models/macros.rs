//! Macros for reducing GObject model boilerplate.
//!
//! This module provides declarative macros that generate the repetitive
//! GObject property definitions and accessor methods, following the
//! zero-cost abstraction principle by generating code at compile time.

/// Generates `StatsObject` trait implementation for a GObject wrapper.
///
/// This macro implements the `StatsObject` trait by delegating to the
/// inner `imp` struct's accessor methods.
///
/// # Usage
///
/// ```ignore
/// impl_stats_object!(MyObject);
/// ```
macro_rules! impl_stats_object {
    ($type:ty) => {
        impl super::StatsObject for $type {
            fn play_count(&self) -> i64 {
                self.imp().play_count.get()
            }

            fn max_plays(&self) -> i64 {
                self.imp().max_plays.get()
            }

            fn total_ms(&self) -> i64 {
                self.imp().total_ms.get()
            }
        }
    };
}

/// Generates a default implementation for GObject wrappers.
///
/// Creates a `Default` impl that constructs a new GObject using
/// `glib::Object::new()`.
macro_rules! impl_gobject_default {
    ($type:ty) => {
        impl Default for $type {
            fn default() -> Self {
                glib::Object::new()
            }
        }
    };
}

/// Generates common GObject property specs that all stats objects share.
///
/// Returns a vector of `ParamSpec` for: `play-count`, `total-ms`, `rank`, `max-plays`.
macro_rules! common_stats_properties {
    () => {{
        vec![
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
        ]
    }};
}

/// Generates common property value getters for stats objects.
///
/// Handles the common property names: `play-count`, `total-ms`, `rank`, `max-plays`.
/// Returns `Some(value)` if matched, `None` otherwise.
macro_rules! common_stats_property_value {
    ($self:expr, $pspec:expr) => {
        match $pspec.name() {
            "play-count" => Some($self.play_count.get().to_value()),
            "total-ms" => Some($self.total_ms.get().to_value()),
            "rank" => Some($self.rank.get().to_value()),
            "max-plays" => Some($self.max_plays.get().to_value()),
            _ => None,
        }
    };
}

/// Generates common accessor methods for GObject wrappers.
///
/// Creates `play_count()`, `total_ms()`, `rank()`, and `max_plays()` methods.
macro_rules! impl_common_accessors {
    ($type:ty) => {
        impl $type {
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
        }
    };
}

/// Sets common stats fields on a GObject inner struct.
///
/// Used in `new()` constructors to initialize common fields.
macro_rules! set_common_stats {
    ($imp:expr, $play_count:expr, $total_ms:expr, $rank:expr, $max_plays:expr) => {
        $imp.play_count.set($play_count);
        $imp.total_ms.set($total_ms);
        $imp.rank.set($rank);
        $imp.max_plays.set($max_plays);
    };
}

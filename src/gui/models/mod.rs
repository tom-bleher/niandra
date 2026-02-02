//! GObject model wrappers for database types.
//!
//! This module provides GTK4 GObject wrappers for database stat types,
//! enabling them to be used in GTK list models and data binding.

#[macro_use]
mod macros;

mod album_object;
mod artist_object;
mod track_object;

pub use album_object::AlbumObject;
pub use artist_object::ArtistObject;
pub use track_object::TrackObject;

/// Trait for GObject model wrappers that have play statistics.
///
/// Provides default implementations for common calculated properties
/// like `progress_fraction()` and `hours()`. This trait enables generic
/// code to work with any stats object type.
pub trait StatsObject {
    /// Get the play count for this object.
    fn play_count(&self) -> i64;

    /// Get the maximum plays (used for progress calculation).
    fn max_plays(&self) -> i64;

    /// Get the total milliseconds played.
    fn total_ms(&self) -> i64;

    /// Get progress bar fraction (0.0 to 1.0).
    ///
    /// Calculates `play_count / max_plays`, returning 0.0 if max_plays is 0.
    #[must_use]
    fn progress_fraction(&self) -> f64 {
        let max = self.max_plays();
        if max > 0 {
            self.play_count() as f64 / max as f64
        } else {
            0.0
        }
    }

    /// Get hours as a formatted float.
    #[must_use]
    fn hours(&self) -> f64 {
        crate::display::format_hours(self.total_ms())
    }
}

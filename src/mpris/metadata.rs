//! MPRIS metadata parsing

use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

use crate::track::Track;

use super::{extract_f64, extract_i32, extract_i64, extract_string, extract_string_array};

/// Parse MPRIS metadata into a Track
pub fn parse_metadata(metadata: &HashMap<String, OwnedValue>) -> Track {
    let mut track = Track::default();

    // Title
    if let Some(value) = metadata.get("xesam:title") {
        track.title = extract_string(value);
    }

    // Artist (array of strings, take first)
    if let Some(value) = metadata.get("xesam:artist") {
        if let Some(artists) = extract_string_array(value) {
            track.artist = artists.into_iter().next();
        } else {
            track.artist = extract_string(value);
        }
    }

    // Album
    if let Some(value) = metadata.get("xesam:album") {
        track.album = extract_string(value);
    }

    // Duration (microseconds)
    if let Some(value) = metadata.get("mpris:length") {
        track.duration_us = extract_i64(value);
    }

    // URL / file path
    if let Some(value) = metadata.get("xesam:url") {
        track.file_path = extract_string(value);
    }

    // Genre (array of strings -> comma-separated)
    if let Some(value) = metadata.get("xesam:genre") {
        if let Some(genres) = extract_string_array(value) {
            track.genre = Some(genres.join(", "));
        } else {
            track.genre = extract_string(value);
        }
    }

    // Album artist (array of strings, take first)
    if let Some(value) = metadata.get("xesam:albumArtist") {
        if let Some(artists) = extract_string_array(value) {
            track.album_artist = artists.into_iter().next();
        } else {
            track.album_artist = extract_string(value);
        }
    }

    // Track number
    if let Some(value) = metadata.get("xesam:trackNumber") {
        track.track_number = extract_i32(value);
    }

    // Disc number
    if let Some(value) = metadata.get("xesam:discNumber") {
        track.disc_number = extract_i32(value);
    }

    // Release date (ISO 8601 or just year)
    if let Some(value) = metadata.get("xesam:contentCreated") {
        track.release_date = extract_string(value);
    }

    // Art URL
    if let Some(value) = metadata.get("mpris:artUrl") {
        track.art_url = extract_string(value);
    }

    // User rating (0.0 - 1.0)
    if let Some(value) = metadata.get("xesam:userRating") {
        track.user_rating = extract_f64(value);
    }

    // BPM
    if let Some(value) = metadata.get("xesam:audioBPM") {
        track.bpm = extract_i32(value);
    }

    // Composer (array of strings -> comma-separated)
    if let Some(value) = metadata.get("xesam:composer") {
        if let Some(composers) = extract_string_array(value) {
            track.composer = Some(composers.join(", "));
        } else {
            track.composer = extract_string(value);
        }
    }

    // MusicBrainz track ID
    if let Some(value) = metadata.get("xesam:musicBrainzTrackID") {
        track.musicbrainz_track_id = extract_string(value);
    }

    track
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_metadata() {
        let metadata = HashMap::new();
        let track = parse_metadata(&metadata);
        assert!(track.title.is_none());
        assert!(track.artist.is_none());
    }
}

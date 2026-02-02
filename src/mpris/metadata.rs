//! MPRIS metadata parsing

use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

use crate::track::Track;

use super::{extract, extract_first_or_string, extract_or_join_array};

/// Parse MPRIS metadata into a Track
pub fn parse_metadata(metadata: &HashMap<String, OwnedValue>) -> Track {
    let mut track = Track::default();

    // Title
    if let Some(value) = metadata.get("xesam:title") {
        track.title = extract(value);
    }

    // Artist (array of strings, take first)
    if let Some(value) = metadata.get("xesam:artist") {
        track.artist = extract_first_or_string(value);
    }

    // Album
    if let Some(value) = metadata.get("xesam:album") {
        track.album = extract(value);
    }

    // Duration (microseconds)
    if let Some(value) = metadata.get("mpris:length") {
        track.duration_us = extract(value);
    }

    // URL / file path
    if let Some(value) = metadata.get("xesam:url") {
        track.file_path = extract(value);
    }

    // Genre (array of strings -> comma-separated)
    if let Some(value) = metadata.get("xesam:genre") {
        track.genre = extract_or_join_array(value, ", ");
    }

    // Album artist (array of strings, take first)
    if let Some(value) = metadata.get("xesam:albumArtist") {
        track.album_artist = extract_first_or_string(value);
    }

    // Track number
    if let Some(value) = metadata.get("xesam:trackNumber") {
        track.track_number = extract(value);
    }

    // Disc number
    if let Some(value) = metadata.get("xesam:discNumber") {
        track.disc_number = extract(value);
    }

    // Release date (ISO 8601 or just year)
    if let Some(value) = metadata.get("xesam:contentCreated") {
        track.release_date = extract(value);
    }

    // Art URL
    if let Some(value) = metadata.get("mpris:artUrl") {
        track.art_url = extract(value);
    }

    // User rating (0.0 - 1.0)
    if let Some(value) = metadata.get("xesam:userRating") {
        track.user_rating = extract(value);
    }

    // BPM
    if let Some(value) = metadata.get("xesam:audioBPM") {
        track.bpm = extract(value);
    }

    // Composer (array of strings -> comma-separated)
    if let Some(value) = metadata.get("xesam:composer") {
        track.composer = extract_or_join_array(value, ", ");
    }

    // MusicBrainz track ID
    if let Some(value) = metadata.get("xesam:musicBrainzTrackID") {
        track.musicbrainz_track_id = extract(value);
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

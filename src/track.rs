//! Track metadata and state management

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Threshold in microseconds for detecting intro position (5 seconds).
const INTRO_START_THRESHOLD_US: i64 = 5_000_000;

/// Threshold in microseconds for detecting intro skip (15 seconds).
const INTRO_SKIP_THRESHOLD_US: i64 = 15_000_000;

/// Complete track metadata from MPRIS
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Track {
    // Core fields
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_us: Option<i64>,
    pub file_path: Option<String>,

    // Extended metadata
    pub genre: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub release_date: Option<String>,
    pub art_url: Option<String>,
    pub user_rating: Option<f64>,
    pub bpm: Option<i32>,
    pub composer: Option<String>,
    pub musicbrainz_track_id: Option<String>,
}

/// Tracks the current playing state with full metadata
#[derive(Debug, Clone)]
pub struct TrackState {
    /// Current track metadata
    pub track: Track,

    /// When playback started (for calculating play time)
    pub start_time: Option<Instant>,

    /// When playback started (wall clock time for DB)
    pub start_timestamp: Option<DateTime<Local>>,

    /// Whether currently playing
    pub is_playing: bool,

    /// Whether source is a local file
    pub is_local: bool,

    /// Player name (bus name suffix)
    pub player_name: Option<String>,

    // Seek tracking
    pub seek_count: u32,
    pub intro_skipped: bool,
    pub seek_forward_ms: i64,
    pub seek_backward_ms: i64,
    pub last_position_us: i64,

    // Volume tracking
    pub app_volume: Option<f64>,
    pub system_volume: Option<f64>,
}

impl Default for TrackState {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackState {
    /// Create a new empty track state
    #[must_use]
    pub fn new() -> Self {
        Self {
            track: Track::default(),
            start_time: None,
            start_timestamp: None,
            is_playing: false,
            is_local: false,
            player_name: None,
            seek_count: 0,
            intro_skipped: false,
            seek_forward_ms: 0,
            seek_backward_ms: 0,
            last_position_us: 0,
            app_volume: None,
            system_volume: None,
        }
    }

    /// Check if current play meets minimum thresholds for logging
    ///
    /// Rules (similar to Last.fm):
    /// - Must play at least 30 seconds
    /// - AND either: 50% of track, OR 4+ minutes played, OR duration unknown
    #[must_use]
    pub fn should_log(&self, min_seconds: u64, min_percent: f64) -> bool {
        if self.track.title.is_none() || self.start_time.is_none() {
            return false;
        }

        let played = self.played_duration();
        let played_seconds = played.as_secs();

        // Must play at least min_seconds
        if played_seconds < min_seconds {
            return false;
        }

        // If duration unknown, min_seconds is enough
        let Some(duration_us) = self.track.duration_us else {
            return true;
        };

        if duration_us <= 0 {
            return true;
        }

        let duration_seconds = duration_us as f64 / 1_000_000.0;
        let played_seconds_f64 = played.as_secs_f64();

        // Either 50% of track OR 4 minutes played
        played_seconds_f64 >= duration_seconds * min_percent || played_seconds >= 240
    }

    /// Get duration played
    #[must_use]
    pub fn played_duration(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_default()
    }

    /// Get milliseconds played
    #[must_use]
    pub fn played_ms(&self) -> i64 {
        self.played_duration().as_millis() as i64
    }

    /// Handle a seek event
    pub fn on_seeked(&mut self, new_position_us: i64) {
        self.seek_count += 1;

        let delta_us = new_position_us - self.last_position_us;
        let delta_ms = delta_us / 1000;

        if delta_us > 0 {
            self.seek_forward_ms += delta_ms;
        } else {
            self.seek_backward_ms += delta_ms.abs();
        }

        // Check if intro was skipped (seeked past first 15 seconds from near start)
        if self.last_position_us < INTRO_START_THRESHOLD_US
            && new_position_us > INTRO_SKIP_THRESHOLD_US
        {
            self.intro_skipped = true;
        }

        self.last_position_us = new_position_us;
    }

    /// Start playback
    pub fn start_playing(&mut self) {
        self.is_playing = true;
        self.start_time = Some(Instant::now());
        self.start_timestamp = Some(Local::now());
    }

    /// Stop playback
    pub fn stop_playing(&mut self) {
        self.is_playing = false;
    }

    /// Calculate effective volume (app × system)
    #[must_use]
    pub fn effective_volume(&self) -> Option<f64> {
        match (self.app_volume, self.system_volume) {
            (Some(app), Some(sys)) => Some(app * sys),
            (Some(v), None) | (None, Some(v)) => Some(v),
            (None, None) => None,
        }
    }
}

impl Track {
    /// Check if this track appears to be from a local file
    #[must_use]
    pub fn is_local_source(&self, local_players: &[String], player_name: Option<&str>) -> bool {
        // Check if player is a known local-only player
        if let Some(name) = player_name {
            let player_id = name
                .strip_prefix("org.mpris.MediaPlayer2.")
                .unwrap_or(name);
            if local_players.iter().any(|p| player_id.contains(p)) {
                return true;
            }
        }

        // Check URL scheme
        if let Some(ref path) = self.file_path {
            if path.starts_with("file://") || path.starts_with('/') {
                return true;
            }
            // Non-local schemes
            if path.starts_with("http://")
                || path.starts_with("https://")
                || path.starts_with("spotify:")
                || path.starts_with("deezer:")
                || path.starts_with("tidal:")
            {
                return false;
            }
        }

        // No URL - might be local player without URL support
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_playing_state(title: &str, duration_us: Option<i64>) -> TrackState {
        let mut state = TrackState::new();
        state.track.title = Some(title.to_string());
        state.track.duration_us = duration_us;
        state.is_playing = true;
        state.start_playing();
        state
    }

    #[test]
    fn test_should_log_no_title_returns_false() {
        let mut state = TrackState::new();
        state.start_playing();
        std::thread::sleep(Duration::from_millis(50));
        // No title set
        assert!(!state.should_log(0, 0.0));
    }

    #[test]
    fn test_should_log_no_start_time_returns_false() {
        let mut state = TrackState::new();
        state.track.title = Some("Test".to_string());
        // Never called start_playing(), so start_time is None
        assert!(!state.should_log(0, 0.0));
    }

    #[test]
    fn test_should_log_under_min_seconds() {
        let state = make_playing_state("Test", Some(300_000_000)); // 5 min track
        // Just started, hasn't played 30 seconds
        assert!(!state.should_log(30, 0.5));
    }

    #[test]
    fn test_should_log_unknown_duration_only_needs_min_seconds() {
        let mut state = make_playing_state("Test", None);
        // Simulate time passing by manipulating start_time
        state.start_time = Some(Instant::now() - Duration::from_secs(35));
        // With unknown duration, only min_seconds matters
        assert!(state.should_log(30, 0.5));
    }

    #[test]
    fn test_should_log_zero_duration_only_needs_min_seconds() {
        let mut state = make_playing_state("Test", Some(0));
        state.start_time = Some(Instant::now() - Duration::from_secs(35));
        assert!(state.should_log(30, 0.5));
    }

    #[test]
    fn test_should_log_negative_duration_only_needs_min_seconds() {
        let mut state = make_playing_state("Test", Some(-1000));
        state.start_time = Some(Instant::now() - Duration::from_secs(35));
        assert!(state.should_log(30, 0.5));
    }

    #[test]
    fn test_should_log_50_percent_rule() {
        // 60 second track, need 50% = 30 seconds
        let mut state = make_playing_state("Test", Some(60_000_000));
        state.start_time = Some(Instant::now() - Duration::from_secs(35));
        // Played 35s of 60s track (58%) - should log
        assert!(state.should_log(30, 0.5));
    }

    #[test]
    fn test_should_log_under_50_percent_under_4_minutes() {
        // 10 minute track (600 seconds), 50% = 300 seconds = 5 minutes
        let mut state = make_playing_state("Test", Some(600_000_000));
        // Played 200 seconds (33%) - under 50% AND under 4 minutes
        state.start_time = Some(Instant::now() - Duration::from_secs(200));
        assert!(!state.should_log(30, 0.5));

        // Played 239 seconds (39.8%) - still under 50% and just under 4 minutes
        state.start_time = Some(Instant::now() - Duration::from_secs(239));
        assert!(!state.should_log(30, 0.5));

        // Played 240 seconds (40%) - under 50% but exactly 4 minutes
        state.start_time = Some(Instant::now() - Duration::from_secs(240));
        assert!(state.should_log(30, 0.5)); // 240 >= 240, triggers 4-min rule
    }

    #[test]
    fn test_should_log_4_minute_rule() {
        // Very long track (1 hour), 50% = 30 minutes
        let mut state = make_playing_state("Test", Some(3600_000_000));
        // Played 4 minutes = 240 seconds
        state.start_time = Some(Instant::now() - Duration::from_secs(240));
        assert!(state.should_log(30, 0.5));
    }

    #[test]
    fn test_on_seeked_tracks_forward_seek() {
        let mut state = TrackState::new();
        state.last_position_us = 10_000_000; // 10 seconds
        state.on_seeked(20_000_000); // Seek to 20 seconds
        assert_eq!(state.seek_count, 1);
        assert_eq!(state.seek_forward_ms, 10_000);
        assert_eq!(state.seek_backward_ms, 0);
    }

    #[test]
    fn test_on_seeked_tracks_backward_seek() {
        let mut state = TrackState::new();
        state.last_position_us = 30_000_000; // 30 seconds
        state.on_seeked(10_000_000); // Seek back to 10 seconds
        assert_eq!(state.seek_count, 1);
        assert_eq!(state.seek_forward_ms, 0);
        assert_eq!(state.seek_backward_ms, 20_000);
    }

    #[test]
    fn test_on_seeked_detects_intro_skip() {
        let mut state = TrackState::new();
        state.last_position_us = 2_000_000; // 2 seconds (< 5s threshold)
        state.on_seeked(20_000_000); // Seek to 20 seconds (> 15s threshold)
        assert!(state.intro_skipped);
    }

    #[test]
    fn test_on_seeked_no_intro_skip_from_later_position() {
        let mut state = TrackState::new();
        state.last_position_us = 10_000_000; // 10 seconds (> 5s threshold)
        state.on_seeked(30_000_000); // Seek to 30 seconds
        assert!(!state.intro_skipped);
    }

    #[test]
    fn test_is_local_source_known_player() {
        let track = Track::default();
        let local_players = vec!["mpv".to_string(), "cmus".to_string()];
        assert!(track.is_local_source(&local_players, Some("mpv")));
        assert!(track.is_local_source(&local_players, Some("org.mpris.MediaPlayer2.cmus")));
        assert!(!track.is_local_source(&local_players, Some("spotify")));
    }

    #[test]
    fn test_is_local_source_file_url() {
        let mut track = Track::default();
        track.file_path = Some("file:///home/user/music/song.mp3".to_string());
        assert!(track.is_local_source(&[], None));
    }

    #[test]
    fn test_is_local_source_absolute_path() {
        let mut track = Track::default();
        track.file_path = Some("/home/user/music/song.mp3".to_string());
        assert!(track.is_local_source(&[], None));
    }

    #[test]
    fn test_is_local_source_streaming_urls() {
        let local_players: Vec<String> = vec![];
        for url in &[
            "http://stream.example.com/song",
            "https://stream.example.com/song",
            "spotify:track:abc123",
            "deezer:track:123",
            "tidal:track:123",
        ] {
            let mut track = Track::default();
            track.file_path = Some(url.to_string());
            assert!(!track.is_local_source(&local_players, None), "URL {} should not be local", url);
        }
    }

    #[test]
    fn test_effective_volume() {
        let mut state = TrackState::new();

        // Both volumes set
        state.app_volume = Some(0.8);
        state.system_volume = Some(0.5);
        assert_eq!(state.effective_volume(), Some(0.4));

        // Only app volume
        state.system_volume = None;
        assert_eq!(state.effective_volume(), Some(0.8));

        // Only system volume
        state.app_volume = None;
        state.system_volume = Some(0.7);
        assert_eq!(state.effective_volume(), Some(0.7));

        // Neither
        state.system_volume = None;
        assert_eq!(state.effective_volume(), None);
    }
}

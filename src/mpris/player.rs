//! MPRIS player monitoring

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use tokio::sync::{mpsc, RwLock};
use tokio::time::Instant;
use tracing::{debug, error, info};
use zbus::fdo::DBusProxy;
use zbus::zvariant::OwnedValue;
use zbus::message::Type as MessageType;
use zbus::{Connection, MatchRule, MessageStream};

use crate::config::{PlayerConfig, TrackingConfig};
use crate::context::ListeningContext;
use crate::db::Database;
use crate::error::Result;
use crate::track::{Track, TrackState};

use super::{extract_string, parse_metadata, MPRIS_PATH, MPRIS_PLAYER_IFACE, MPRIS_PREFIX};

/// Events emitted by the MPRIS monitor
#[derive(Debug, Clone)]
pub enum MprisEvent {
    /// A new track started playing
    TrackChanged {
        player: String,
        track: Track,
        is_local: bool,
    },
    /// Playback started
    Playing { player: String },
    /// Playback paused
    Paused { player: String },
    /// Playback stopped
    Stopped { player: String },
    /// Player appeared on D-Bus
    PlayerAppeared { player: String },
    /// Player disappeared from D-Bus
    PlayerDisappeared { player: String },
    /// Seek occurred
    Seeked { player: String, position_us: i64 },
}

/// MPRIS player monitor
pub struct MprisMonitor {
    connection: Connection,
    player_config: PlayerConfig,
    tracking_config: TrackingConfig,
    db: Database,
    /// Map from unique bus name (e.g., `:1.500`) to track state
    tracked_players: Arc<RwLock<HashMap<String, TrackState>>>,
    /// Map from unique bus name to well-known name (e.g., `org.mpris.MediaPlayer2.io.bassi.Amberol`)
    bus_name_map: Arc<RwLock<HashMap<String, String>>>,
    /// Atomic flag for stop signaling - more efficient than RwLock for simple bools
    running: Arc<AtomicBool>,
    idle_since: Arc<RwLock<Option<Instant>>>,
}

impl MprisMonitor {
    /// Create a new MPRIS monitor
    pub async fn new(
        player_config: PlayerConfig,
        tracking_config: TrackingConfig,
        db: Database,
    ) -> Result<Self> {
        let connection = Connection::session().await?;

        Ok(Self {
            connection,
            player_config,
            tracking_config,
            db,
            tracked_players: Arc::new(RwLock::new(HashMap::new())),
            bus_name_map: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(true)),
            idle_since: Arc::new(RwLock::new(None)),
        })
    }

    /// Start monitoring MPRIS players
    pub async fn run(&self) -> Result<()> {
        info!("Starting MPRIS monitor...");

        // Discover existing players
        self.discover_players().await?;

        // Check if we found any players
        {
            let players = self.tracked_players.read().await;
            if players.is_empty() {
                info!("No players found, starting idle timer...");
                *self.idle_since.write().await = Some(Instant::now());
            }
        }

        // Set up message stream for D-Bus signals
        let rule = MatchRule::builder()
            .msg_type(MessageType::Signal)
            .build();

        let mut stream = MessageStream::for_match_rule(rule, &self.connection, Some(100)).await?;

        // Create channel for events
        let (tx, mut rx) = mpsc::channel::<MprisEvent>(100);

        // Spawn signal handler
        let connection = self.connection.clone();
        let player_config = self.player_config.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                if let Ok(msg) = msg {
                    Self::handle_dbus_message(&msg, &connection, &player_config, &tx_clone).await;
                }
            }
        });

        // Main event loop
        let idle_timeout = Duration::from_secs(self.tracking_config.idle_timeout_seconds);

        loop {
            // Check if we should stop
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            // Check idle timeout
            if self.tracking_config.idle_timeout_seconds > 0 {
                if let Some(idle_start) = *self.idle_since.read().await {
                    if idle_start.elapsed() >= idle_timeout {
                        info!("Idle timeout reached, shutting down...");
                        break;
                    }
                }
            }

            // Process events with timeout
            tokio::select! {
                Some(event) = rx.recv() => {
                    self.handle_event(event).await;
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    // Periodic check
                }
            }
        }

        // Log any in-progress plays before exiting
        self.finalize().await;

        Ok(())
    }

    /// Stop the monitor.
    ///
    /// This method is synchronous as it only sets an atomic flag.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Discover existing MPRIS players
    async fn discover_players(&self) -> Result<()> {
        let dbus = DBusProxy::new(&self.connection).await?;
        let names = dbus.list_names().await?;

        for name in names {
            let name_str = name.as_str();
            if name_str.starts_with(MPRIS_PREFIX) && self.should_track_player(name_str) {
                self.add_player(name_str).await?;
            }
        }

        Ok(())
    }

    /// Check if a player should be tracked
    fn should_track_player(&self, name: &str) -> bool {
        let player_id = name.strip_prefix(MPRIS_PREFIX).unwrap_or(name);

        // Check blacklist first
        if self
            .player_config
            .blacklist
            .iter()
            .any(|p| player_id.contains(p))
        {
            return false;
        }

        // If whitelist is empty, track all
        if self.player_config.whitelist.is_empty() {
            return true;
        }

        // Check whitelist
        self.player_config
            .whitelist
            .iter()
            .any(|p| player_id.contains(p))
    }

    /// Add a player to tracking
    async fn add_player(&self, well_known_name: &str) -> Result<()> {
        // Get unique bus name for this well-known name
        let dbus = DBusProxy::new(&self.connection).await?;
        let bus_name = well_known_name
            .try_into()
            .map_err(|e| crate::error::Error::other(format!("Invalid bus name: {e}")))?;
        let unique_name = dbus.get_name_owner(bus_name).await?;
        let unique_name_str = unique_name.as_str().to_string();

        let mut players = self.tracked_players.write().await;

        if players.contains_key(&unique_name_str) {
            return Ok(());
        }

        info!("Adding player: {}", well_known_name);

        let mut state = TrackState::new();
        state.player_name = Some(
            well_known_name
                .strip_prefix(MPRIS_PREFIX)
                .unwrap_or(well_known_name)
                .to_string(),
        );

        // Get initial state
        if let Ok(metadata) = self.get_player_metadata(well_known_name).await {
            let track = parse_metadata(&metadata);
            state.track = track.clone();
            state.is_local = track.is_local_source(
                &self.player_config.local_only_players,
                state.player_name.as_deref(),
            );
        }

        if let Ok(status) = self.get_playback_status(well_known_name).await {
            if status == "Playing" {
                state.start_playing();
                info!(
                    "[{}] Already playing: {} - {}",
                    well_known_name,
                    state.track.artist.as_deref().unwrap_or("Unknown"),
                    state.track.title.as_deref().unwrap_or("Unknown")
                );
            }
        }

        players.insert(unique_name_str.clone(), state);

        // Store mapping from unique name to well-known name
        self.bus_name_map
            .write()
            .await
            .insert(unique_name_str, well_known_name.to_string());

        // Cancel idle timer
        *self.idle_since.write().await = None;

        Ok(())
    }

    /// Remove a player from tracking by well-known name
    async fn remove_player(&self, well_known_name: &str) {
        // Find the unique name for this well-known name
        let unique_name = {
            let map = self.bus_name_map.read().await;
            map.iter()
                .find(|(_, v)| *v == well_known_name)
                .map(|(k, _)| k.clone())
        };

        let Some(unique_name) = unique_name else {
            return;
        };

        let mut players = self.tracked_players.write().await;

        if let Some(state) = players.remove(&unique_name) {
            info!("Removing player: {}", well_known_name);

            // Log final play if applicable
            if state.is_playing
                && state.should_log(
                    self.tracking_config.min_play_seconds,
                    self.tracking_config.min_play_percent,
                )
            {
                self.log_play(&state).await;
            }
        }

        // Remove from bus name map
        self.bus_name_map.write().await.remove(&unique_name);

        // Start idle timer if no players remain
        if players.is_empty() {
            info!(
                "No players remaining, will exit in {}s if none appear...",
                self.tracking_config.idle_timeout_seconds
            );
            *self.idle_since.write().await = Some(Instant::now());
        }
    }

    /// Handle a D-Bus message
    async fn handle_dbus_message(
        msg: &zbus::Message,
        _connection: &Connection,
        player_config: &PlayerConfig,
        tx: &mpsc::Sender<MprisEvent>,
    ) {
        let header = msg.header();

        // Handle NameOwnerChanged (player appear/disappear)
        if header.interface().map(|i| i.as_str()) == Some("org.freedesktop.DBus")
            && header.member().map(|m| m.as_str()) == Some("NameOwnerChanged")
        {
            if let Ok((name, old_owner, new_owner)) = msg.body().deserialize::<(String, String, String)>() {
                if name.starts_with(MPRIS_PREFIX) {
                    if new_owner.is_empty() && !old_owner.is_empty() {
                        // Player disappeared
                        let _ = tx
                            .send(MprisEvent::PlayerDisappeared {
                                player: name.clone(),
                            })
                            .await;
                    } else if !new_owner.is_empty() && old_owner.is_empty() {
                        // Player appeared
                        let _ = tx
                            .send(MprisEvent::PlayerAppeared {
                                player: name.clone(),
                            })
                            .await;
                    }
                }
            }
            return;
        }

        // Handle PropertiesChanged
        if header.interface().map(|i| i.as_str()) == Some("org.freedesktop.DBus.Properties")
            && header.member().map(|m| m.as_str()) == Some("PropertiesChanged")
        {
            let sender = header.sender().map(|s| s.as_str().to_string());

            if let Ok((iface, changed, _invalidated)) = msg.body().deserialize::<(
                String,
                HashMap<String, OwnedValue>,
                Vec<String>,
            )>() {
                if iface == MPRIS_PLAYER_IFACE {
                    if let Some(player) = sender {
                        // Handle playback status change
                        if let Some(status) = changed.get("PlaybackStatus") {
                            if let Some(status_str) = extract_string(status) {
                                let event = match status_str.as_str() {
                                    "Playing" => MprisEvent::Playing {
                                        player: player.clone(),
                                    },
                                    "Paused" => MprisEvent::Paused {
                                        player: player.clone(),
                                    },
                                    "Stopped" => MprisEvent::Stopped {
                                        player: player.clone(),
                                    },
                                    _ => return,
                                };
                                let _ = tx.send(event).await;
                            }
                        }

                        // Handle metadata change
                        if let Some(metadata) = changed.get("Metadata") {
                            if let Ok(meta_map) =
                                HashMap::<String, OwnedValue>::try_from(metadata.clone())
                            {
                                let track = parse_metadata(&meta_map);
                                let is_local = track.is_local_source(
                                    &player_config.local_only_players,
                                    Some(&player),
                                );

                                let _ = tx
                                    .send(MprisEvent::TrackChanged {
                                        player: player.clone(),
                                        track,
                                        is_local,
                                    })
                                    .await;
                            }
                        }
                    }
                }
            }
            return;
        }

        // Handle Seeked signal
        if header.interface().map(|i| i.as_str()) == Some(MPRIS_PLAYER_IFACE)
            && header.member().map(|m| m.as_str()) == Some("Seeked")
        {
            if let Some(sender) = header.sender() {
                if let Ok(position) = msg.body().deserialize::<i64>() {
                    let _ = tx
                        .send(MprisEvent::Seeked {
                            player: sender.to_string(),
                            position_us: position,
                        })
                        .await;
                }
            }
        }
    }

    /// Handle an MPRIS event
    async fn handle_event(&self, event: MprisEvent) {
        match event {
            MprisEvent::PlayerAppeared { player } => {
                if self.should_track_player(&player) {
                    if let Err(e) = self.add_player(&player).await {
                        error!("Failed to add player {}: {}", player, e);
                    }
                }
            }

            MprisEvent::PlayerDisappeared { player } => {
                self.remove_player(&player).await;
            }

            MprisEvent::TrackChanged {
                player,
                track,
                is_local,
            } => {
                let display_name = self
                    .bus_name_map
                    .read()
                    .await
                    .get(&player)
                    .cloned()
                    .unwrap_or_else(|| player.clone());

                // Capture state to log before modifying, avoiding race conditions
                let state_to_log = {
                    let mut players = self.tracked_players.write().await;

                    let state_to_log = if let Some(state) = players.get(&player) {
                        // Check if previous track qualifies for logging
                        if state.is_playing
                            && state.should_log(
                                self.tracking_config.min_play_seconds,
                                self.tracking_config.min_play_percent,
                            )
                            && (!self.tracking_config.local_only || state.is_local)
                        {
                            Some(state.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Update state for new track while still holding the lock
                    if let Some(state) = players.get_mut(&player) {
                        let old_title = state.track.title.clone();
                        state.track = track.clone();
                        state.is_local = is_local;

                        // Reset seek tracking for new track
                        state.seek_count = 0;
                        state.intro_skipped = false;
                        state.seek_forward_ms = 0;
                        state.seek_backward_ms = 0;
                        state.last_position_us = 0;

                        if state.is_playing {
                            state.start_playing();
                        }

                        if track.title != old_title {
                            let local_info = if !is_local && self.tracking_config.local_only {
                                " (non-local, won't track)"
                            } else {
                                ""
                            };
                            info!(
                                "[{}] Track changed: {} - {}{}",
                                display_name,
                                track.artist.as_deref().unwrap_or("Unknown"),
                                track.title.as_deref().unwrap_or("Unknown"),
                                local_info
                            );
                        }
                    }

                    state_to_log
                };

                // Log previous track after releasing lock
                if let Some(state) = state_to_log {
                    self.log_play(&state).await;
                }
            }

            MprisEvent::Playing { player } => {
                let mut players = self.tracked_players.write().await;
                let display_name = self
                    .bus_name_map
                    .read()
                    .await
                    .get(&player)
                    .cloned()
                    .unwrap_or_else(|| player.clone());

                if let Some(state) = players.get_mut(&player) {
                    if !state.is_playing {
                        state.start_playing();
                        info!(
                            "[{}] Playing: {} - {}",
                            display_name,
                            state.track.artist.as_deref().unwrap_or("Unknown"),
                            state.track.title.as_deref().unwrap_or("Unknown")
                        );
                    }
                }
            }

            MprisEvent::Paused { player } | MprisEvent::Stopped { player } => {
                let display_name = self
                    .bus_name_map
                    .read()
                    .await
                    .get(&player)
                    .cloned()
                    .unwrap_or_else(|| player.clone());

                // Capture state and update in one lock acquisition to avoid races
                let state_to_log = {
                    let mut players = self.tracked_players.write().await;
                    let state_to_log = if let Some(state) = players.get(&player) {
                        if state.is_playing
                            && state.should_log(
                                self.tracking_config.min_play_seconds,
                                self.tracking_config.min_play_percent,
                            )
                            && (!self.tracking_config.local_only || state.is_local)
                        {
                            Some(state.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Update state while holding lock
                    if let Some(state) = players.get_mut(&player) {
                        state.stop_playing();
                        info!("[{}] Paused", display_name);
                    }

                    state_to_log
                };

                // Log after releasing lock
                if let Some(state) = state_to_log {
                    self.log_play(&state).await;
                }
            }

            MprisEvent::Seeked {
                player,
                position_us,
            } => {
                if self.tracking_config.track_seeks {
                    let display_name = self
                        .bus_name_map
                        .read()
                        .await
                        .get(&player)
                        .cloned()
                        .unwrap_or_else(|| player.clone());

                    let mut players = self.tracked_players.write().await;
                    if let Some(state) = players.get_mut(&player) {
                        state.on_seeked(position_us);
                        debug!(
                            "[{}] Seeked to {}s (total seeks: {})",
                            display_name,
                            position_us / 1_000_000,
                            state.seek_count
                        );
                    }
                }
            }
        }
    }

    /// Get player metadata via D-Bus.
    ///
    /// Times out after 5 seconds to prevent hangs from misbehaving players.
    async fn get_player_metadata(&self, name: &str) -> Result<HashMap<String, OwnedValue>> {
        use zbus::names::InterfaceName;

        const DBUS_TIMEOUT: Duration = Duration::from_secs(5);

        let proxy = tokio::time::timeout(DBUS_TIMEOUT, async {
            zbus::fdo::PropertiesProxy::builder(&self.connection)
                .destination(name)?
                .path(MPRIS_PATH)?
                .build()
                .await
        })
        .await
        .map_err(|_| crate::error::Error::other("D-Bus proxy build timed out"))??;

        let iface = InterfaceName::try_from(MPRIS_PLAYER_IFACE)
            .map_err(|e| crate::error::Error::InvalidMetadata(e.to_string()))?;

        let metadata = tokio::time::timeout(DBUS_TIMEOUT, proxy.get(iface, "Metadata"))
            .await
            .map_err(|_| crate::error::Error::other("D-Bus metadata fetch timed out"))??;

        HashMap::<String, OwnedValue>::try_from(metadata)
            .map_err(|_| crate::error::Error::InvalidMetadata("Failed to parse metadata".into()))
    }

    /// Get playback status via D-Bus.
    ///
    /// Times out after 5 seconds to prevent hangs from misbehaving players.
    async fn get_playback_status(&self, name: &str) -> Result<String> {
        use zbus::names::InterfaceName;

        const DBUS_TIMEOUT: Duration = Duration::from_secs(5);

        let proxy = tokio::time::timeout(DBUS_TIMEOUT, async {
            zbus::fdo::PropertiesProxy::builder(&self.connection)
                .destination(name)?
                .path(MPRIS_PATH)?
                .build()
                .await
        })
        .await
        .map_err(|_| crate::error::Error::other("D-Bus proxy build timed out"))??;

        let iface = InterfaceName::try_from(MPRIS_PLAYER_IFACE)
            .map_err(|e| crate::error::Error::InvalidMetadata(e.to_string()))?;

        let status = tokio::time::timeout(DBUS_TIMEOUT, proxy.get(iface, "PlaybackStatus"))
            .await
            .map_err(|_| crate::error::Error::other("D-Bus status fetch timed out"))??;

        extract_string(&status)
            .ok_or_else(|| crate::error::Error::InvalidMetadata("Failed to get status".into()))
    }

    /// Log a play to the database
    async fn log_play(&self, state: &TrackState) {
        if state.track.title.is_none() {
            return;
        }

        let context = if self.tracking_config.track_context {
            ListeningContext::capture().await
        } else {
            ListeningContext::default()
        };

        let seek_info = if state.seek_count > 0 {
            let mut info = format!(", {} seeks", state.seek_count);
            if state.intro_skipped {
                info.push_str(", intro skipped");
            }
            info
        } else {
            String::new()
        };

        info!(
            "Logging play: {} - {} ({}s played{})",
            state.track.artist.as_deref().unwrap_or("Unknown"),
            state.track.title.as_deref().unwrap_or("Unknown"),
            state.played_ms() / 1000,
            seek_info
        );

        if let Err(e) = self.db.log_play(state, &context).await {
            error!("Failed to log play: {}", e);
        }
    }

    /// Finalize and log any remaining plays
    async fn finalize(&self) {
        let players = self.tracked_players.read().await;

        for (name, state) in players.iter() {
            if state.is_playing
                && state.should_log(
                    self.tracking_config.min_play_seconds,
                    self.tracking_config.min_play_percent,
                )
                && (!self.tracking_config.local_only || state.is_local)
            {
                info!("Logging final play for {}", name);
                self.log_play(state).await;
            }
        }
    }
}

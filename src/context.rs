//! Listening context tracking (time, activity, power state)

use chrono::{Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;
use tracing::debug;

/// Timeout for external command execution (2 seconds).
const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);

/// Run a blocking function with timeout, returning None on failure or timeout.
async fn run_with_timeout<T: Send + 'static>(f: fn() -> Option<T>) -> Option<T> {
    tokio::time::timeout(COMMAND_TIMEOUT, tokio::task::spawn_blocking(f))
        .await
        .ok()
        .and_then(Result::ok)
        .flatten()
}

/// Saturday is day 5 in num_days_from_monday() (0=Monday, 6=Sunday).
const WEEKEND_START_DAY: i32 = 5;

/// Listening context at the time of a play
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListeningContext {
    /// Hour of day (0-23)
    pub hour_of_day: i32,

    /// Day of week (0=Monday, 6=Sunday)
    pub day_of_week: i32,

    /// Whether it's a weekend
    pub is_weekend: bool,

    /// Season (spring, summer, fall, winter)
    pub season: String,

    /// Currently focused window name
    pub active_window: Option<String>,

    /// Whether screen is on
    pub screen_on: Option<bool>,

    /// Whether on battery power
    pub on_battery: Option<bool>,
}

impl ListeningContext {
    /// Capture current listening context asynchronously.
    ///
    /// External command calls are run in a blocking thread pool with timeouts
    /// to avoid blocking the async runtime.
    pub async fn capture() -> Self {
        let now = Local::now();
        let weekday = now.weekday().num_days_from_monday() as i32;

        // Run blocking I/O operations in spawn_blocking with timeouts to avoid
        // indefinite hangs if external commands stall
        let (active_window, screen_on, on_battery) = tokio::join!(
            run_with_timeout(get_active_window),
            run_with_timeout(get_screen_state),
            run_with_timeout(get_power_state),
        );

        Self {
            hour_of_day: now.hour() as i32,
            day_of_week: weekday,
            is_weekend: weekday >= WEEKEND_START_DAY,
            season: get_season(now.month()),
            active_window,
            screen_on,
            on_battery,
        }
    }

    /// Capture current listening context synchronously.
    ///
    /// Use this only in non-async contexts (e.g., tests).
    #[cfg(test)]
    #[must_use]
    pub fn capture_sync() -> Self {
        let now = Local::now();
        let weekday = now.weekday().num_days_from_monday() as i32;

        Self {
            hour_of_day: now.hour() as i32,
            day_of_week: weekday,
            is_weekend: weekday >= WEEKEND_START_DAY,
            season: get_season(now.month()),
            active_window: get_active_window(),
            screen_on: get_screen_state(),
            on_battery: get_power_state(),
        }
    }
}

/// Get the season for a given month (Northern Hemisphere)
fn get_season(month: u32) -> String {
    match month {
        3..=5 => "spring",
        6..=8 => "summer",
        9..=11 => "fall",
        _ => "winter",
    }
    .to_string()
}

/// Get the currently focused window name.
///
/// Tries xdotool (X11) first, then wlrctl (Wayland).
fn get_active_window() -> Option<String> {
    // Try xdotool for X11
    match Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let name = String::from_utf8_lossy(&output.stdout);
            let name = name.trim();
            if !name.is_empty() {
                return Some(name.chars().take(200).collect());
            }
        }
        Ok(_) => debug!("xdotool returned non-zero exit code"),
        Err(e) => debug!("xdotool not available: {}", e),
    }

    // Try wlrctl for Wayland
    match Command::new("wlrctl")
        .args(["toplevel", "focus"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let name = String::from_utf8_lossy(&output.stdout);
            let name = name.trim();
            if !name.is_empty() {
                return Some(name.chars().take(200).collect());
            }
        }
        Ok(_) => debug!("wlrctl returned non-zero exit code"),
        Err(e) => debug!("wlrctl not available: {}", e),
    }

    None
}

/// Check if screen is on.
///
/// Tries gnome-screensaver-command first, then xset (X11).
fn get_screen_state() -> Option<bool> {
    // Try gnome-screensaver
    match Command::new("gnome-screensaver-command").arg("-q").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            return Some(!stdout.contains("is active"));
        }
        Ok(_) => debug!("gnome-screensaver-command returned non-zero exit code"),
        Err(e) => debug!("gnome-screensaver-command not available: {}", e),
    }

    // Try xset for X11
    match Command::new("xset").arg("q").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Some(!stdout.contains("Monitor is Off"));
        }
        Ok(_) => debug!("xset returned non-zero exit code"),
        Err(e) => debug!("xset not available: {}", e),
    }

    None
}

/// Check if on battery power.
///
/// Checks `/sys/class/power_supply` first, then falls back to upower.
fn get_power_state() -> Option<bool> {
    // Check /sys/class/power_supply
    match std::fs::read_dir("/sys/class/power_supply") {
        Ok(entries) => {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("BAT") {
                    let status_path = entry.path().join("status");
                    if let Ok(status) = std::fs::read_to_string(&status_path) {
                        let status = status.trim().to_lowercase();
                        return Some(status == "discharging");
                    }
                    debug!("could not read battery status from {:?}", status_path);
                }
            }
        }
        Err(e) => debug!("/sys/class/power_supply not available: {}", e),
    }

    // Try upower as fallback
    match Command::new("upower")
        .args(["-i", "/org/freedesktop/UPower/devices/battery_BAT0"])
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if stdout.contains("state:") {
                return Some(stdout.contains("discharging"));
            }
        }
        Ok(_) => debug!("upower returned non-zero exit code"),
        Err(e) => debug!("upower not available: {}", e),
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_season() {
        assert_eq!(get_season(1), "winter");
        assert_eq!(get_season(4), "spring");
        assert_eq!(get_season(7), "summer");
        assert_eq!(get_season(10), "fall");
    }

    #[test]
    fn test_context_capture_sync() {
        let ctx = ListeningContext::capture_sync();
        assert!((0..24).contains(&ctx.hour_of_day));
        assert!((0..7).contains(&ctx.day_of_week));
        assert!(!ctx.season.is_empty());
    }
}

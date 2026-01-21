//! MPRIS D-Bus monitoring module
//!
//! Monitors MPRIS-compatible media players via D-Bus signals.
//! Uses async event-driven architecture (not polling).

mod metadata;
mod player;

pub use metadata::parse_metadata;
pub use player::MprisMonitor;

use zbus::zvariant::{OwnedValue, Value};

/// MPRIS D-Bus constants
pub const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
pub const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";
pub const MPRIS_PLAYER_IFACE: &str = "org.mpris.MediaPlayer2.Player";

/// Extract a string value from a D-Bus variant
pub fn extract_string(value: &OwnedValue) -> Option<String> {
    // Try to get as a Value first, then match on Str variant
    if let Ok(Value::Str(s)) = value.try_into() {
        return Some(s.to_string());
    }

    // Try direct string conversion
    if let Ok(s) = <&str>::try_from(value) {
        return Some(s.to_string());
    }

    // Try String
    if let Ok(s) = String::try_from(value.clone()) {
        return Some(s);
    }

    None
}

/// Extract string array from D-Bus variant
pub fn extract_string_array(value: &OwnedValue) -> Option<Vec<String>> {
    // Try as Vec<String> directly
    if let Ok(arr) = Vec::<String>::try_from(value.clone()) {
        if !arr.is_empty() {
            return Some(arr);
        }
    }

    // Try as Value::Array and extract strings
    let val = Value::from(value.clone());
    if let Value::Array(arr) = val {
        let strings: Vec<String> = arr
            .iter()
            .filter_map(|v| {
                if let Value::Str(s) = v {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();
        if !strings.is_empty() {
            return Some(strings);
        }
    }

    None
}

/// Extract i64 from D-Bus variant
pub fn extract_i64(value: &OwnedValue) -> Option<i64> {
    // Try direct i64
    if let Ok(v) = i64::try_from(value.clone()) {
        return Some(v);
    }

    // Try other integer types via Value with proper conversions
    match Value::from(value.clone()) {
        Value::I64(v) => Some(v),
        Value::I32(v) => Some(i64::from(v)),
        Value::U64(v) => i64::try_from(v).ok(), // Returns None if overflow
        Value::U32(v) => Some(i64::from(v)),
        Value::I16(v) => Some(i64::from(v)),
        Value::U16(v) => Some(i64::from(v)),
        _ => None,
    }
}

/// Extract f64 from D-Bus variant
pub fn extract_f64(value: &OwnedValue) -> Option<f64> {
    // Try direct f64
    if let Ok(v) = f64::try_from(value.clone()) {
        return Some(v);
    }

    // Try via Value
    if let Value::F64(v) = Value::from(value.clone()) {
        return Some(v);
    }

    None
}

/// Extract i32 from D-Bus variant
pub fn extract_i32(value: &OwnedValue) -> Option<i32> {
    // Try direct i32
    if let Ok(v) = i32::try_from(value.clone()) {
        return Some(v);
    }

    // Try via Value with proper conversions
    match Value::from(value.clone()) {
        Value::I32(v) => Some(v),
        Value::I64(v) => i32::try_from(v).ok(), // Returns None if out of range
        Value::U32(v) => i32::try_from(v).ok(), // Returns None if out of range
        Value::I16(v) => Some(i32::from(v)),
        Value::U16(v) => Some(i32::from(v)),
        _ => None,
    }
}

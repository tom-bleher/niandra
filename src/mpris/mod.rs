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

/// Trait for extracting typed values from D-Bus variants
pub trait ExtractValue: Sized {
    /// Extract a value from a D-Bus OwnedValue
    fn extract(value: &OwnedValue) -> Option<Self>;
}

impl ExtractValue for String {
    fn extract(value: &OwnedValue) -> Option<Self> {
        if let Ok(Value::Str(s)) = value.try_into() {
            return Some(s.to_string());
        }
        <&str>::try_from(value)
            .map(String::from)
            .or_else(|_| String::try_from(value.clone()))
            .ok()
    }
}

impl ExtractValue for Vec<String> {
    fn extract(value: &OwnedValue) -> Option<Self> {
        Vec::<String>::try_from(value.clone())
            .ok()
            .filter(|arr| !arr.is_empty())
            .or_else(|| {
                if let Value::Array(arr) = Value::from(value.clone()) {
                    let strings: Vec<_> = arr
                        .iter()
                        .filter_map(|v| match v {
                            Value::Str(s) => Some(s.to_string()),
                            _ => None,
                        })
                        .collect();
                    (!strings.is_empty()).then_some(strings)
                } else {
                    None
                }
            })
    }
}

/// Macro to implement `ExtractValue` for integer types with D-Bus variant fallbacks
macro_rules! impl_extract_int {
    ($target:ty, [$($variant:ident => $conv:expr),+ $(,)?]) => {
        impl ExtractValue for $target {
            fn extract(value: &OwnedValue) -> Option<Self> {
                <$target>::try_from(value.clone()).ok().or_else(|| {
                    match Value::from(value.clone()) {
                        $(Value::$variant(v) => $conv(v),)+
                        _ => None,
                    }
                })
            }
        }
    };
}

impl_extract_int!(i64, [
    I64 => Some,
    I32 => |v| Some(i64::from(v)),
    U64 => |v| i64::try_from(v).ok(),
    U32 => |v| Some(i64::from(v)),
    I16 => |v| Some(i64::from(v)),
    U16 => |v| Some(i64::from(v)),
]);

impl_extract_int!(i32, [
    I32 => Some,
    I64 => |v| i32::try_from(v).ok(),
    U32 => |v| i32::try_from(v).ok(),
    I16 => |v| Some(i32::from(v)),
    U16 => |v| Some(i32::from(v)),
]);

impl ExtractValue for f64 {
    fn extract(value: &OwnedValue) -> Option<Self> {
        f64::try_from(value.clone())
            .ok()
            .or_else(|| match Value::from(value.clone()) {
                Value::F64(v) => Some(v),
                _ => None,
            })
    }
}

/// Convenience function to extract a value using the ExtractValue trait
pub fn extract<T: ExtractValue>(value: &OwnedValue) -> Option<T> {
    T::extract(value)
}

/// Extract a string array and join with separator, falling back to a single string
pub fn extract_or_join_array(value: &OwnedValue, separator: &str) -> Option<String> {
    if let Some(arr) = Vec::<String>::extract(value) {
        Some(arr.join(separator))
    } else {
        String::extract(value)
    }
}

/// Extract the first element of a string array, falling back to a single string
pub fn extract_first_or_string(value: &OwnedValue) -> Option<String> {
    if let Some(arr) = Vec::<String>::extract(value) {
        arr.into_iter().next()
    } else {
        String::extract(value)
    }
}

/// Extract a string from a D-Bus value.
pub fn extract_string(value: &OwnedValue) -> Option<String> {
    String::extract(value)
}

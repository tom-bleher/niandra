//! Domain-specific newtypes for type safety.
//!
//! This module provides strongly-typed wrappers for common domain concepts
//! to prevent mixing up values at compile time. Uses `derive_more` to
//! eliminate arithmetic boilerplate while maintaining zero-cost abstractions.

use std::fmt;
use std::ops::{AddAssign, Sub, SubAssign};

use derive_more::{Add as DeriveAdd, From, Into};

// ============================================================================
// Macros for reducing boilerplate
// ============================================================================

/// Generates common methods for numeric newtypes.
macro_rules! impl_newtype_common {
    ($type:ty) => {
        impl $type {
            /// Create a new instance.
            #[must_use]
            pub const fn new(value: i64) -> Self {
                Self(value)
            }

            /// Get the inner value.
            #[must_use]
            pub const fn get(self) -> i64 {
                self.0
            }

            /// Get the inner value (alias for `get`).
            #[must_use]
            pub const fn value(self) -> i64 {
                self.0
            }

            /// Consume and return the inner value.
            #[must_use]
            pub const fn into_inner(self) -> i64 {
                self.0
            }

            /// Check if the value is zero.
            #[must_use]
            pub const fn is_zero(self) -> bool {
                self.0 == 0
            }
        }
    };
}

/// Generates Sub and assignment trait implementations.
macro_rules! impl_sub_traits {
    ($type:ty) => {
        impl Sub for $type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl SubAssign for $type {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl AddAssign for $type {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
    };
}

// ============================================================================
// PlayCount
// ============================================================================

/// A play count value.
///
/// Represents the number of times a track, album, or artist has been played.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, DeriveAdd, From, Into,
)]
pub struct PlayCount(pub i64);

impl_newtype_common!(PlayCount);
impl_sub_traits!(PlayCount);

impl fmt::Display for PlayCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i32> for PlayCount {
    fn from(value: i32) -> Self {
        Self(i64::from(value))
    }
}

// ============================================================================
// Milliseconds
// ============================================================================

/// A duration in milliseconds.
///
/// Used for listening times, play durations, and similar time spans.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, DeriveAdd, From, Into,
)]
pub struct Milliseconds(pub i64);

impl_newtype_common!(Milliseconds);
impl_sub_traits!(Milliseconds);

impl Milliseconds {
    /// Convert to seconds as a floating point value.
    #[must_use]
    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }

    /// Convert to hours as a floating point value.
    #[must_use]
    pub fn as_hours_f64(self) -> f64 {
        self.0 as f64 / 3_600_000.0
    }

    /// Convert to hours as a floating point value (alias for `as_hours_f64`).
    #[must_use]
    pub fn to_hours(self) -> f64 {
        self.as_hours_f64()
    }

    /// Convert to microseconds.
    #[must_use]
    pub const fn to_microseconds(self) -> Microseconds {
        Microseconds(self.0 * 1000)
    }

    /// Create from seconds.
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs * 1000)
    }

    /// Create from minutes.
    #[must_use]
    pub const fn from_mins(mins: i64) -> Self {
        Self(mins * 60 * 1000)
    }

    /// Create from hours.
    #[must_use]
    pub const fn from_hours(hours: i64) -> Self {
        Self(hours * 3_600_000)
    }
}

impl fmt::Display for Milliseconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_seconds = self.0 / 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        match (hours, minutes) {
            (0, 0) => write!(f, "{seconds}s"),
            (0, _) => write!(f, "{minutes}m {seconds}s"),
            _ => write!(f, "{hours}h {minutes}m {seconds}s"),
        }
    }
}

impl From<Microseconds> for Milliseconds {
    fn from(value: Microseconds) -> Self {
        value.to_milliseconds()
    }
}

// ============================================================================
// Microseconds
// ============================================================================

/// A duration in microseconds.
///
/// Used for precise timing, MPRIS positions, and track durations.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, DeriveAdd, From, Into,
)]
pub struct Microseconds(pub i64);

impl_newtype_common!(Microseconds);
impl_sub_traits!(Microseconds);

impl Microseconds {
    /// Convert to seconds as a floating point value.
    #[must_use]
    pub fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }

    /// Convert to milliseconds.
    #[must_use]
    pub const fn to_milliseconds(self) -> Milliseconds {
        Milliseconds(self.0 / 1000)
    }

    /// Create from seconds.
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs * 1_000_000)
    }

    /// Create from milliseconds.
    #[must_use]
    pub const fn from_millis(ms: i64) -> Self {
        Self(ms * 1000)
    }
}

impl fmt::Display for Microseconds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_milliseconds().fmt(f)
    }
}

impl From<Milliseconds> for Microseconds {
    fn from(value: Milliseconds) -> Self {
        value.to_microseconds()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod play_count {
        use super::*;

        #[test]
        fn basic_operations() {
            let count = PlayCount::new(42);
            assert_eq!(count.get(), 42);
            assert!(!count.is_zero());
            assert!(PlayCount::default().is_zero());
        }

        #[test]
        fn display() {
            assert_eq!(format!("{}", PlayCount::new(100)), "100");
        }

        #[test]
        fn arithmetic() {
            let a = PlayCount::new(10);
            let b = PlayCount::new(5);

            assert_eq!(a + b, PlayCount::new(15));
            assert_eq!(a - b, PlayCount::new(5));

            let mut c = PlayCount::new(10);
            c += PlayCount::new(3);
            assert_eq!(c, PlayCount::new(13));
            c -= PlayCount::new(2);
            assert_eq!(c, PlayCount::new(11));
        }

        #[test]
        fn conversions() {
            let count: PlayCount = 42i64.into();
            assert_eq!(count.get(), 42);

            let raw: i64 = count.into();
            assert_eq!(raw, 42);

            let from_i32: PlayCount = 10i32.into();
            assert_eq!(from_i32.get(), 10);
        }
    }

    mod milliseconds {
        use super::*;

        #[test]
        fn basic_operations() {
            let ms = Milliseconds::new(5000);
            assert_eq!(ms.get(), 5000);
            assert!(!ms.is_zero());
            assert!(Milliseconds::default().is_zero());
        }

        #[test]
        fn time_conversions() {
            let ms = Milliseconds::new(3_661_000);
            assert!((ms.as_secs_f64() - 3661.0).abs() < 0.001);
            assert!((ms.as_hours_f64() - 1.0169).abs() < 0.001);
        }

        #[test]
        fn display() {
            assert_eq!(format!("{}", Milliseconds::new(5000)), "5s");
            assert_eq!(format!("{}", Milliseconds::new(65_000)), "1m 5s");
            assert_eq!(format!("{}", Milliseconds::new(3_665_000)), "1h 1m 5s");
        }

        #[test]
        fn factories() {
            assert_eq!(Milliseconds::from_secs(5), Milliseconds::new(5000));
            assert_eq!(Milliseconds::from_mins(2), Milliseconds::new(120_000));
            assert_eq!(Milliseconds::from_hours(1), Milliseconds::new(3_600_000));
        }

        #[test]
        fn arithmetic() {
            let a = Milliseconds::new(1000);
            let b = Milliseconds::new(500);

            assert_eq!(a + b, Milliseconds::new(1500));
            assert_eq!(a - b, Milliseconds::new(500));

            let mut c = Milliseconds::new(1000);
            c += Milliseconds::new(300);
            assert_eq!(c, Milliseconds::new(1300));
            c -= Milliseconds::new(200);
            assert_eq!(c, Milliseconds::new(1100));
        }
    }

    mod microseconds {
        use super::*;

        #[test]
        fn basic_operations() {
            let us = Microseconds::new(5_000_000);
            assert_eq!(us.get(), 5_000_000);
            assert!(!us.is_zero());
            assert!(Microseconds::default().is_zero());
        }

        #[test]
        fn time_conversions() {
            let us = Microseconds::new(5_500_000);
            assert!((us.as_secs_f64() - 5.5).abs() < 0.001);
            assert_eq!(us.to_milliseconds(), Milliseconds::new(5500));
        }

        #[test]
        fn factories() {
            assert_eq!(Microseconds::from_secs(5), Microseconds::new(5_000_000));
            assert_eq!(Microseconds::from_millis(500), Microseconds::new(500_000));
        }

        #[test]
        fn arithmetic() {
            let a = Microseconds::new(1_000_000);
            let b = Microseconds::new(500_000);

            assert_eq!(a + b, Microseconds::new(1_500_000));
            assert_eq!(a - b, Microseconds::new(500_000));

            let mut c = Microseconds::new(1_000_000);
            c += Microseconds::new(300_000);
            assert_eq!(c, Microseconds::new(1_300_000));
            c -= Microseconds::new(200_000);
            assert_eq!(c, Microseconds::new(1_100_000));
        }
    }

    mod cross_type {
        use super::*;

        #[test]
        fn ms_us_conversions() {
            let ms = Milliseconds::new(1500);
            let us = ms.to_microseconds();
            assert_eq!(us, Microseconds::new(1_500_000));

            let back = us.to_milliseconds();
            assert_eq!(back, ms);

            let us_from: Microseconds = ms.into();
            assert_eq!(us_from, Microseconds::new(1_500_000));

            let ms_from: Milliseconds = us.into();
            assert_eq!(ms_from, Milliseconds::new(1500));
        }

        #[test]
        fn ordering() {
            assert!(PlayCount::new(10) > PlayCount::new(5));
            assert!(Milliseconds::new(1000) < Milliseconds::new(2000));
            assert!(Microseconds::new(500) <= Microseconds::new(500));
        }
    }
}

use serde::{Deserialize, Serialize};

/// Serializable representation of `CMTime`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[non_exhaustive]
pub enum Time {
    /// Numeric time represented as `value / timescale` seconds.
    Numeric { value: i64, timescale: i32 },
    /// `kCMTimeInvalid`.
    Invalid,
    /// `kCMTimeIndefinite`.
    Indefinite,
    /// Positive infinity.
    PositiveInfinity,
    /// Negative infinity.
    NegativeInfinity,
}

impl Time {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn new(value: i64, timescale: i32) -> Self {
        Self::Numeric { value, timescale }
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn invalid() -> Self {
        Self::Invalid
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn indefinite() -> Self {
        Self::Indefinite
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn positive_infinity() -> Self {
        Self::PositiveInfinity
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn negative_infinity() -> Self {
        Self::NegativeInfinity
    }

    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn as_numeric(self) -> Option<(i64, i32)> {
        match self {
            Self::Numeric { value, timescale } => Some((value, timescale)),
            Self::Invalid | Self::Indefinite | Self::PositiveInfinity | Self::NegativeInfinity => {
                None
            }
        }
    }

    #[must_use]
    pub(crate) const fn to_raw(self) -> (i64, i32, i32) {
        match self {
            Self::Numeric { value, timescale } => (value, timescale, 0),
            Self::Invalid => (0, 0, 1),
            Self::Indefinite => (0, 0, 2),
            Self::PositiveInfinity => (0, 0, 3),
            Self::NegativeInfinity => (0, 0, 4),
        }
    }

    #[must_use]
    pub(crate) const fn from_raw(value: i64, timescale: i32, kind: i32) -> Self {
        match kind {
            0 => Self::Numeric { value, timescale },
            2 => Self::Indefinite,
            3 => Self::PositiveInfinity,
            4 => Self::NegativeInfinity,
            _ => Self::Invalid,
        }
    }
}

impl From<(i64, i32)> for Time {
    fn from(value: (i64, i32)) -> Self {
        Self::new(value.0, value.1)
    }
}

/// Serializable representation of `CMTimeRange`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    /// Range start.
    pub start: Time,
    /// Range duration.
    pub duration: Time,
}

impl TimeRange {
    /// Mirrors the `AVPlayer` framework constant `fn`.
    #[must_use]
    pub const fn new(start: Time, duration: Time) -> Self {
        Self { start, duration }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_time_round_trips_through_raw_representation() {
        let time = Time::new(1_234, 600);

        assert_eq!(time.as_numeric(), Some((1_234, 600)));
        assert_eq!(time.to_raw(), (1_234, 600, 0));
        assert_eq!(Time::from_raw(1_234, 600, 0), time);
    }

    #[test]
    fn tuple_conversion_creates_numeric_time() {
        assert_eq!(Time::from((7, 3)), Time::new(7, 3));
    }

    #[test]
    fn invalid_time_is_not_numeric() {
        let time = Time::invalid();

        assert_eq!(time.as_numeric(), None);
        assert_eq!(time.to_raw(), (0, 0, 1));
        assert_eq!(Time::from_raw(99, 1, 1), Time::Invalid);
    }

    #[test]
    fn indefinite_time_is_not_numeric() {
        let time = Time::indefinite();

        assert_eq!(time.as_numeric(), None);
        assert_eq!(time.to_raw(), (0, 0, 2));
        assert_eq!(Time::from_raw(99, 1, 2), Time::Indefinite);
    }

    #[test]
    fn positive_infinity_time_is_not_numeric() {
        let time = Time::positive_infinity();

        assert_eq!(time.as_numeric(), None);
        assert_eq!(time.to_raw(), (0, 0, 3));
        assert_eq!(Time::from_raw(99, 1, 3), Time::PositiveInfinity);
    }

    #[test]
    fn negative_infinity_time_is_not_numeric() {
        let time = Time::negative_infinity();

        assert_eq!(time.as_numeric(), None);
        assert_eq!(time.to_raw(), (0, 0, 4));
        assert_eq!(Time::from_raw(99, 1, 4), Time::NegativeInfinity);
    }

    #[test]
    fn time_range_new_preserves_start_and_duration() {
        let range = TimeRange::new(Time::new(5, 1), Time::new(2, 1));

        assert_eq!(range.start, Time::new(5, 1));
        assert_eq!(range.duration, Time::new(2, 1));
    }
}

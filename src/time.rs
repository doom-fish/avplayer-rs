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
    #[must_use]
    pub const fn new(value: i64, timescale: i32) -> Self {
        Self::Numeric { value, timescale }
    }

    #[must_use]
    pub const fn invalid() -> Self {
        Self::Invalid
    }

    #[must_use]
    pub const fn indefinite() -> Self {
        Self::Indefinite
    }

    #[must_use]
    pub const fn positive_infinity() -> Self {
        Self::PositiveInfinity
    }

    #[must_use]
    pub const fn negative_infinity() -> Self {
        Self::NegativeInfinity
    }

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
    #[must_use]
    pub const fn new(start: Time, duration: Time) -> Self {
        Self { start, duration }
    }
}

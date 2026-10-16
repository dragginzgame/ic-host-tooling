//! Positive whole-second duration parsing and compact human-readable display.
//! Callers own deadlines, retention/freshness policy, units in persisted data and IO.

use std::fmt;

/// Error returned when a duration value cannot be parsed into positive seconds.

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DurationParseError {
    /// The input did not produce a positive whole-second value.
    Invalid {
        /// Original rejected input.
        value: String,
    },
}

impl fmt::Display for DurationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Invalid { value } = self;
        write!(
            f,
            "invalid duration {value:?}; use positive seconds or a value ending in s, m, h, or d"
        )
    }
}

impl std::error::Error for DurationParseError {}

/// Parses a positive duration string into seconds.
///
/// Accepts bare seconds or integer values ending in `s`, `m`, `h`, or `d`.
/// Uses Rust's unsigned integer syntax, including an optional leading `+`,
/// without trimming, fractional values or uppercase units.
///
/// # Errors
/// Rejects malformed input, zero seconds and multiplication beyond `u64`.
///
/// ```
/// use ic_host_text::duration::parse_duration_seconds;
/// assert_eq!(parse_duration_seconds("2h").unwrap(), 7_200);
/// ```
pub fn parse_duration_seconds(value: &str) -> Result<u64, DurationParseError> {
    let (number, multiplier) = match value.as_bytes().last().copied() {
        Some(b's') => (&value[..value.len() - 1], 1),
        Some(b'm') => (&value[..value.len() - 1], 60),
        Some(b'h') => (&value[..value.len() - 1], 60 * 60),
        Some(b'd') => (&value[..value.len() - 1], 24 * 60 * 60),
        Some(b'0'..=b'9') => (value, 1),
        _ => {
            return Err(DurationParseError::Invalid {
                value: value.to_string(),
            });
        }
    };
    number
        .parse::<u64>()
        .ok()
        .and_then(|amount| amount.checked_mul(multiplier))
        .filter(|seconds| *seconds > 0)
        .ok_or_else(|| DurationParseError::Invalid {
            value: value.to_string(),
        })
}

/// Renders seconds using the largest readable duration unit.
///
/// Uses `d`, `h`, `m` or `s`, two fractional digits for inexact selected units,
/// and integer half-up rounding. Zero renders `0s`; this display is not a
/// parseable or lossless storage representation.
///
/// ```
/// use ic_host_text::duration::display_duration_seconds;
/// assert_eq!(display_duration_seconds(5_400), "1.50h");
/// ```
#[must_use]
pub fn display_duration_seconds(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;

    if seconds == 0 {
        "0s".to_string()
    } else if seconds >= DAY {
        scaled_duration_unit_text(seconds, DAY, "d")
    } else if seconds >= HOUR {
        scaled_duration_unit_text(seconds, HOUR, "h")
    } else if seconds >= MINUTE {
        scaled_duration_unit_text(seconds, MINUTE, "m")
    } else {
        format!("{seconds}s")
    }
}

fn scaled_duration_unit_text(seconds: u64, unit_seconds: u64, suffix: &str) -> String {
    if seconds.is_multiple_of(unit_seconds) {
        return format!("{}{suffix}", seconds / unit_seconds);
    }
    let hundredths =
        ((u128::from(seconds) * 100) + (u128::from(unit_seconds) / 2)) / u128::from(unit_seconds);
    let whole = hundredths / 100;
    let fractional = hundredths % 100;
    format!("{whole}.{fractional:02}{suffix}")
}

/// Format a duration in compact largest units for CLI tables and summaries.
#[must_use]
pub fn compact_duration(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;

    match seconds {
        0 => "0s".to_string(),
        1..MINUTE => format!("{seconds}s"),
        MINUTE..HOUR => compact_duration_pair(seconds, MINUTE, "m", 1, "s"),
        HOUR..DAY => compact_duration_pair(seconds, HOUR, "h", MINUTE, "m"),
        DAY..WEEK => compact_duration_pair(seconds, DAY, "d", HOUR, "h"),
        _ => compact_duration_pair(seconds, WEEK, "w", DAY, "d"),
    }
}

fn compact_duration_pair(
    seconds: u64,
    major_seconds: u64,
    major_unit: &str,
    minor_seconds: u64,
    minor_unit: &str,
) -> String {
    let major = seconds / major_seconds;
    let minor = (seconds % major_seconds) / minor_seconds;
    if minor == 0 {
        format!("{major}{major_unit}")
    } else {
        format!("{major}{major_unit} {minor}{minor_unit}")
    }
}

#[cfg(test)]
mod tests;

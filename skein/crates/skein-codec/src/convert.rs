//! Scalar translation pairs (skein/docs/design/data/codec/maps.md, 5).

use alloc::boxed::Box;
use skein_lib::{Duration, Time, Token, Wall};

use crate::Reason;

/// Wall-clock nanoseconds for a record.
#[must_use]
pub fn wall_to_record(value: &Wall) -> u64 {
    value.as_nanos()
}

/// A domain wall-clock value.
#[must_use]
pub fn wall_to_domain(value: u64) -> Wall {
    Wall::from_nanos(value)
}

/// Monotonic nanoseconds for a record.
#[must_use]
pub fn time_to_record(value: &Time) -> u64 {
    value.as_nanos()
}

/// A domain monotonic instant.
#[must_use]
pub fn time_to_domain(value: u64) -> Time {
    Time::from_nanos(value)
}

/// Duration nanoseconds for a record.
#[must_use]
pub fn duration_to_record(value: &Duration) -> u64 {
    value.as_nanos()
}

/// A domain duration.
#[must_use]
pub fn duration_to_domain(value: u64) -> Duration {
    Duration::from_nanos(value)
}

/// A token's raw record value.
#[must_use]
pub fn token_to_record(value: &Token) -> u64 {
    value.raw()
}

/// A domain token.
#[must_use]
pub fn token_to_domain(value: u64) -> Token {
    Token::new(value)
}

/// UTF-8 text copied at emission to the record's bytes.
#[expect(clippy::borrowed_box, reason = "the maps converter signature takes the domain value by reference")]
#[must_use]
pub fn text_to_record(value: &Box<str>) -> Box<[u8]> {
    Box::from(value.as_bytes())
}

/// Record bytes interpreted as UTF-8, refusing invalid text.
#[expect(
    clippy::disallowed_methods,
    reason = "the shared text converter validates the record's UTF-8 at the protocol boundary"
)]
pub fn text_to_domain(value: Box<[u8]>) -> Result<Box<str>, Reason> {
    match core::str::from_utf8(&value) {
        Ok(value) => Ok(Box::from(value)),
        Err(_error) => Err(Reason::Utf8),
    }
}

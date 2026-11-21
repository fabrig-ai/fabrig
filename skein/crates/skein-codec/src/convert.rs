//! Scalar translation pairs (skein/docs/design/data/codec/maps.md, 5).

use alloc::boxed::Box;
use skein_lib::{Duration, Time, Token, Wall};

use crate::Reason;

/// Wall-clock nanoseconds for a record.
pub fn wall_to_record(value: &Wall) -> Result<u64, Reason> {
    Ok(value.as_nanos())
}

/// A domain wall-clock value.
pub fn wall_to_domain(value: u64) -> Result<Wall, Reason> {
    Ok(Wall::from_nanos(value))
}

/// Monotonic nanoseconds for a record.
pub fn time_to_record(value: &Time) -> Result<u64, Reason> {
    Ok(value.as_nanos())
}

/// A domain monotonic instant.
pub fn time_to_domain(value: u64) -> Result<Time, Reason> {
    Ok(Time::from_nanos(value))
}

/// Duration nanoseconds for a record.
pub fn duration_to_record(value: &Duration) -> Result<u64, Reason> {
    Ok(value.as_nanos())
}

/// A domain duration.
pub fn duration_to_domain(value: u64) -> Result<Duration, Reason> {
    Ok(Duration::from_nanos(value))
}

/// A token's raw record value.
pub fn token_to_record(value: &Token) -> Result<u64, Reason> {
    Ok(value.raw())
}

/// A domain token.
pub fn token_to_domain(value: u64) -> Result<Token, Reason> {
    Ok(Token::new(value))
}

/// UTF-8 text copied at emission to the record's bytes.
#[expect(clippy::borrowed_box, reason = "the maps converter signature takes the domain value by reference")]
pub fn text_to_record(value: &Box<str>) -> Result<Box<[u8]>, Reason> {
    Ok(Box::from(value.as_bytes()))
}

/// Record bytes interpreted as UTF-8, refusing invalid text.
#[expect(clippy::disallowed_methods, reason = "the shared text converter validates the record's UTF-8 at the protocol boundary")]
pub fn text_to_domain(value: Box<[u8]>) -> Result<Box<str>, Reason> {
    match core::str::from_utf8(&value) {
        Ok(value) => Ok(Box::from(value)),
        Err(_error) => Err(Reason::Utf8),
    }
}

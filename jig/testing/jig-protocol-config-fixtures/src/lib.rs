//! Configuration limits for tests (jig/docs/design/protocol/limits.md, the stock).
#![forbid(unsafe_code)]

mod small;

/// Small limits for tests alone.
#[must_use]
pub fn small() -> jig_protocol_config::Limits {
    small::LIMITS
}

#[cfg(test)]
mod tests;

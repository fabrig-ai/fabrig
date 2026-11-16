//! Configuration limits for tests (jig/docs/design/protocol/limits.md, the stock).
#![forbid(unsafe_code)]

/// Small limits for tests alone.
#[must_use]
pub fn small() -> jig_protocol_config::Limits {
    jig_protocol_config::profile(jig_protocol_config::Profile::Small)
}

#[cfg(test)]
mod tests;

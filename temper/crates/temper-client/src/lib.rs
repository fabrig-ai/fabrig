//! The deployment's client (temper/docs/design/protocol/README.md, section 4).
#![forbid(unsafe_code)]

pub const NAME: &str = "temper-client";

/// The binary's version and build commit (temper/docs/design/release.md, section 3).
#[must_use]
pub fn version() -> String {
    format!("{NAME} {} ({})\n", env!("CARGO_PKG_VERSION"), env!("TEMPER_CLIENT_COMMIT"))
}

#[cfg(test)]
mod tests;

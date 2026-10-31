//! Channel refusal reasons (smith/docs/design/protocol/channel.md, section 2).

/// Channel rule violation.
pub const RULES: u16 = 256;

/// A credential is not permitted on this transport.
pub const UNAUTHORIZED: u16 = 258;

/// A required receive term is absent or too small.
pub const LIMITS: u16 = 2;

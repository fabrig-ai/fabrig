//! Configuration fixtures shared by the box test binaries.
// The process configuration reader admits a MiB, 64K tokens and 32 levels.
pub const DOCUMENT: jig_protocol_config::DocumentLimits =
    jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };

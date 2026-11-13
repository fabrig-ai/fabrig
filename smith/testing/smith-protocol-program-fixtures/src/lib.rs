//! Small program protocol presets for other crates' tests.
//! smith/docs/design/testing.md, section 7; smith/docs/design/protocol/programs.md, section 8.

#![no_std]
#![forbid(unsafe_code)]

/// MCP listener limits for deterministic program scenarios.
pub mod mcp {
    use skein_lib::Time;
    use smith_protocol_program::mcp::Limits;

    /// Two calls with bounded JSON and a short authentication deadline.
    pub const SMALL: Limits = Limits {
        message_bytes: 4096,
        answer_bytes: 4096,
        schema_bytes: 4096,
        calls: 2,
        unauthenticated: Time::from_nanos(10),
    };
}

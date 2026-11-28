//! Durable lifecycle observations for chips (client/domain/objects.md, section 3).
//! Timestamps come from injected transition events; retries name the decision
//! that paused an activation, rather than a counter selected by a reader.
//! The row owns the attempt identity; a new claim clears both observations.
//! Its start report is root placement/adoption timing, not agent/tool timing.
pub use jig_domain_contract::ops::RetryNotice;

/// At most this many bytes of a preparation reason are kept (domain/engine/briefs.md, 1.1).
pub const REASON_BYTES: u32 = 128;

//! Admission paces new work by the journal backlog watermark.
//! Connector and endpoint limits are application configuration (domain/root.md, 10).
use jig_domain_core as core;
use jig_domain_host as host;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;

/// Each child receives its own limits; the root owns its admission pacing.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// The core and its children.
    pub core: core::Limits,
    /// Observability's working set and outbox.
    pub observability: observability::Limits,
    /// Infrastructure's working set and outbox.
    pub infrastructure: infrastructure::Limits,
    /// Engine-slot run hub.
    pub host: host::Limits,
    /// The charter codec's bounds.
    pub charter: smith_charter::v4::Limits,
    /// How many endpoint names the charter codec holds.
    pub endpoints: u32,
    /// Held outputs the journal releases in one call (`domain/root.md` 4).
    pub release: u32,
    /// New external work waits while the backlog is at or past this many
    /// entries (skein's draft `dynamic-memory.md`, 4.4); a pacing limit,
    /// not a memory one.
    pub watermark: u32,
    /// Maximum synchronous continuations for one admitted event.
    pub routes: u32,
}

/// The watermark's default: `commits` in flight plus the `door`'s outputs
/// (`domain/root.md` 10).
#[must_use]
pub fn default_watermark(commits: u32, door: u32) -> u32 {
    commits.checked_add(door).expect("journal commits plus door outputs fit")
}

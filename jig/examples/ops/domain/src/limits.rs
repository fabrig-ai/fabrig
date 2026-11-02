//! Admission paces new work by the journal backlog watermark.
//! Connector and endpoint limits are application configuration (domain/root.md, 10).
use jig_domain_core as core;
use jig_domain_host as host;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;

/// Each child receives its own limits; the root owns its admission pacing.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Deployment policy and translation bounds.
    pub policy: Policy,
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
    /// entries (skein's `programming-model.md`, 7.1); a pacing limit,
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

/// The deployment's policy, in the names of examples.md, 3.3 and 6.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    pub healthy_replica_requirement: u16,
    pub low_load_requirement: u16,
    pub scale_down_load_percent: u8,
    pub scale_down_load_seconds: u64,
    pub verdict_freshness_seconds: u64,
    pub brief_timeout: skein_lib::Duration,
    pub delegate: jig_protocol_tools::Limits,
}
impl Policy {
    /// Reference deployment policy; observed facts are fresh for a minute.
    #[must_use]
    pub const fn reference() -> Self {
        Self {
            healthy_replica_requirement: 1,
            low_load_requirement: 2,
            scale_down_load_percent: 20,
            scale_down_load_seconds: 1800,
            verdict_freshness_seconds: 60,
            brief_timeout: skein_lib::Duration::from_secs(30),
            delegate: jig_protocol_tools::Limits { input_bytes: 65536, words: 65536, batch: 2, depth: 16 },
        }
    }
}

/// The reference deployment's reason for closing a released environment's chat.
pub const ENVIRONMENT_CLOSED_REASON: &[u8] = b"environment torn down";

//! Bounded connector top state (domain/root.md, section 10;
//! protocol/configuration.md, section 7).

/// GitHub App slugs become bot logins with this suffix (setup/lifecycle.rs).
pub const BOT_LOGIN_SUFFIX: &[u8] = b"[bot]";
/// Bounds `stopped_text`'s Debug: skein `connection::Stopped` is fixed-shape (boundary.rs:109–151).
pub const STOPPED_TEXT_BYTES: u64 = 128;

/// Last connect diagnostics are at most 256 UTF-8 bytes (client/forge.md, 5.4).
pub const CONNECT_WHY_BYTES: usize = 256;

/// Admin connects carry names checked at the wire ceiling, independently of the engine's setup name limit.
pub const FIRST_NAME_BYTES: u32 = 1024;

/// Capacity of each durable top table and its child.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    /// Forges kept at most (temper/docs/design/protocol/forge.md, 7).
    pub forges: u32,
    pub repositories: u32,
    pub tasks: u32,
    pub holds: u32,
    pub subscriptions: u32,
    pub entries: u32,
    /// Connector payloads of live named calls, priced independently of outbox entries.
    pub named_calls: u32,
    pub changes: u32,
    pub issues: u32,
    pub resources_per_task: u32,
    pub paths_per_subscription: u32,
    pub name_bytes: u32,
    /// Authority name depth, supplied from `authority.segments` by the root.
    pub authority_segments: u32,
    pub output: u32,
    pub facts: u32,
    pub adoptions: u32,
    pub applications: u32,
    /// Durable member address bindings (protocol/sign-in.md, 5).
    pub bindings: u32,
    pub collaborators: u32,
    /// Live landing procedure rows (domain/landing.md, 12).
    pub landings: u32,
    pub landing_members: u32,
    pub gates: u32,
    pub receipts_per_gate: u32,
    pub ignored_paths: u32,
    pub key_entries: u32,
    pub key_path_bytes: u32,
    pub trees_in_flight: u32,
    pub tiers: TierLimits,
    /// Project judge tables retained beside committed policy.
    pub judge_projects: u32,
    /// Criteria in each deployment or project judge table.
    pub judge_criteria: u32,
    /// Connector brief sections gathering concurrently.
    pub brief_sections: u32,
    /// Maximum bytes retained for one gathered brief section.
    pub brief_bytes: u32,
    pub issue_policy: temper_engine_domain_forge_issues::Limits,
    pub change_policy: temper_engine_domain_forge_change::Limits,
    pub queue_window: skein_lib::Duration,
    pub client: skein_forge_client::Limits,
}

/// Per-repository tiers and reported failures (temper/docs/design/domain/after-landing.md, 10).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TierLimits {
    pub tiers: u32,
    pub failing_tests: u32,
    pub test_id_bytes: u32,
}

/// Whether entrance limits are coherent and retained row counts fit their types.
/// Names and gates fit the panel; client admissions cover the top's rows.
/// Reconciliation and landing capacities must be representable
/// (temper/docs/design/domain/forge.md, sections 4, 8 and 16).
/// Internal output envelopes and retained heap sums do not constrain admission.
#[must_use]
pub fn valid(l: &Limits) -> bool {
    if l.tiers.tiers == 0
        || l.tiers.failing_tests == 0
        // The store's failure and flake lists hold at most 1024 test ids.
        || l.tiers.failing_tests > 1024
        || l.tiers.test_id_bytes == 0
        || l.forges == 0
        || l.repositories == 0
        || l.tasks == 0
        || l.holds == 0
        || l.subscriptions == 0
        || l.entries == 0
        || l.resources_per_task == 0
        || l.name_bytes == 0
        || l.name_bytes > crate::PANEL_NAME_BYTES
        || l.change_policy.gates > crate::PANEL_GATES
        || l.queue_window == skein_lib::Duration::ZERO
        || l.adoptions == 0
        || l.applications == 0
        || l.collaborators == 0
        || l.landings == 0
        || l.brief_sections == 0
        || l.brief_bytes == 0
        || !skein_forge_client::valid(&l.client)
        || l.repositories > l.client.repositories
        // The top's entries, held ones included, are the client's only
        // makes, so a held entry made again always finds the client's room.
        || l.entries > l.client.entries
        || l.holds > l.client.resources
        || crate::reconcile::capacity(l).is_none()
        || !crate::landing::valid(l)
    {
        return false;
    }
    true
}

/// Whether the connection owner's installation row count fits its table capacity.
pub(crate) const fn connection_valid(forges: u32, installations: u32) -> bool {
    forges.checked_mul(installations).is_some()
}

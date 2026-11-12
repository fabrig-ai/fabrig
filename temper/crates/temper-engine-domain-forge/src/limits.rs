//! Bounded connector top state (domain/root.md, section 9;
//! protocol/configuration/limits.md).

/// GitHub App slugs become bot logins with this suffix (setup/lifecycle.rs).
pub const BOT_LOGIN_SUFFIX: &[u8] = b"[bot]";
/// Bounds `stopped_text`'s Debug: skein `connection::Stopped` is fixed-shape (boundary.rs:109–151).
pub const STOPPED_TEXT_BYTES: u64 = 128;

/// Last connect diagnostics are at most 256 UTF-8 bytes (client/forge/deployment.md).
pub const CONNECT_WHY_BYTES: usize = 256;

/// Admin connects carry names checked at the wire ceiling, independently of the engine's setup name limit.
pub const FIRST_NAME_BYTES: u32 = 1024;

/// Capacity of each durable top table and its child.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Forges kept at most (temper/docs/design/protocol/forge/README.md, 6).
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
    /// Repositories being made or failed (temper/docs/design/domain/forge/adoption.md, 5).
    pub makings: u32,
    pub applications: u32,
    /// Durable member address bindings (protocol/sign-in.md, 5).
    pub bindings: u32,
    pub collaborators: u32,
    /// Live landing procedure rows (domain/landing.md, 12).
    pub landings: u32,
    pub landing_members: u32,
    /// Retained landings per repository (domain/after-landing.md, 10).
    pub landings_kept: u32,
    /// Queued rows and recent landed changes in the changes document (client/forge/records.md).
    pub changes_rows: u32,
    pub changes_landed: u32,
    pub gates: u32,
    pub receipts_per_gate: u32,
    pub ignored_paths: u32,
    pub key_entries: u32,
    pub key_path_bytes: u32,
    pub trees_in_flight: u32,
    pub tiers: TierLimits,
    /// Project judge tables retained beside committed policy.
    pub judge_projects: u32,
    /// Retained decided and asked rows per project (temper/docs/design/domain/forge/attention.md, 5).
    pub decided_per_project: u32,
    /// Retained findings per project (temper/docs/design/domain/findings.md, 9).
    pub findings_per_project: u32,
    /// Criteria in each deployment or project judge table.
    pub judge_criteria: u32,
    /// Rules per scope, configured by `forge.landing_rules` (protocol/configuration/settings.md, 2).
    pub landing_rules: u32,
    /// Roles supplied from jig's `authority.roles`.
    pub roles: u32,
    /// Pending proposals of settings, supplied from jig's `settings.proposals`.
    pub setting_proposals: u32,
    /// Revisions whose replaced values are retained, supplied from jig's `settings.history`.
    pub setting_history: u32,
    /// Connector brief sections gathering concurrently.
    pub brief_sections: u32,
    /// Maximum bytes retained for one gathered brief section.
    pub brief_bytes: u32,
    pub issue_policy: temper_engine_domain_forge_issues::Limits,
    pub change_policy: temper_engine_domain_forge_change::Limits,
    pub queue_window: skein_lib::Duration,
    /// Gap between sittings (temper/docs/design/domain/forge/attention.md, 2).
    pub sitting: skein_lib::Duration,
    pub client: skein_forge_client::Limits,
}

impl Limits {
    /// Retained decided and asked rows in the family (temper/docs/design/domain/forge/attention.md, 5).
    #[must_use]
    pub const fn findings(&self) -> u32 {
        self.judge_projects.saturating_mul(self.findings_per_project)
    }
    #[must_use]
    pub const fn decided(&self) -> u32 {
        self.judge_projects.saturating_mul(self.decided_per_project)
    }
    /// Eight times members times projects, saturating (temper/docs/design/domain/forge/attention.md, 1).
    #[must_use]
    pub const fn dismissed(&self) -> u32 {
        self.bindings.saturating_mul(self.judge_projects).saturating_mul(8)
    }
}

/// Per-repository tiers and reported failures (temper/docs/design/domain/after-landing.md, 10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TierLimits {
    pub tiers: u32,
    pub failing_tests: u32,
    pub test_id_bytes: u32,
}

/// Whether entrance limits are coherent and retained row counts fit their types.
/// Names and gates fit the panel; client admissions cover the top's rows.
/// Reconciliation and landing capacities must be representable
/// (temper/docs/design/domain/forge/adoption.md and temper/docs/design/domain/forge/change/README.md and temper/docs/design/domain/forge/README.md, section 9).
/// Internal output envelopes and retained heap sums do not constrain admission.
#[must_use]
pub fn valid(l: &Limits) -> bool {
    if l.tiers.tiers == 0
        || l.tiers.failing_tests == 0
        // The store's failure and flake lists hold at most 1024 test ids.
        || l.tiers.failing_tests > 1024
        || l.tiers.test_id_bytes == 0
        || l.findings_per_project == 0
        || l.judge_projects.checked_mul(l.findings_per_project).is_none()
        || l.decided_per_project == 0
        || l.judge_projects.checked_mul(l.decided_per_project).is_none()
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
        || skein_lib::usize_of(l.change_policy.questions) >= temper_engine_domain_forge_change::QUESTIONS_OPEN
        || l.change_policy.tasks == 0
        || l.queue_window == skein_lib::Duration::ZERO
        || l.sitting == skein_lib::Duration::ZERO
        || l.makings == 0
        || l.makings > 64
        || l.adoptions == 0
        || l.applications == 0
        || l.collaborators == 0
        || l.landings_kept == 0
        || l.landings_kept > 256
        || l.changes_rows == 0
        || l.changes_rows > 32
        || l.changes_landed == 0
        || l.changes_landed > 32
        || l.landings == 0
        || l.landing_rules == 0
        || l.landing_rules.saturating_mul(l.roles).saturating_mul(2) > 8192
        || l.client.inbox > 256
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

//! Project policy and notes (`client/domain/pages.md`, sections 6.1, 6.4, 6.5).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).
//! Notes' bodies are loaded on demand by their child; this module builds a
//! document from that loaded entry without adding a mutable cache.
//! Pool edits target the configured period independently of the observed
//! period. Permission mappings come from the core's retained policy entry;
//! an absent entry remains absent, and each new list states its own loss.

use super::{AuthorityLine, Chip, GrantLine, NoteScope, Part, ProposedTask, Text};
use crate::Limits;
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};

/// The role of a resource adopted into a project.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceRole {
    Owned,
    Fork,
    Participant,
    Unavailable,
    Context,
}
/// One adopted connector resource and its project role.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectResource {
    pub part: Part,
    pub role: ResourceRole,
    pub home: bool,
}
/// One policy role's permission and allotment ceilings.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PolicyRole {
    pub number: u32,
    pub authority: AuthorityLine,
    pub period_spend: u64,
    pub requests: authority::Requests,
    pub decides: authority::Proposals,
}
/// An effect requirement as policy records it, with an exact resource pattern.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Requirement {
    pub grant: GrantLine,
    pub judge: authority::Judge,
    pub guard: authority::Guard,
    pub must_be_guarded: bool,
}
/// Project policy in force, with truncated lists stated.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Policy {
    pub ceiling: AuthorityLine,
    pub period_spend: u64,
    pub escalation_role: Option<u32>,
    pub projections: List<GrantLine>,
    pub roles: List<PolicyRole>,
    pub requirements: List<Requirement>,
    pub permissions: Option<List<people::PermissionRole>>,
    pub permissions_cut: bool,
    pub templates: List<people::TemplateOverride>,
    pub cut: bool,
}
/// A retained party's membership; services carry the same role facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub party: u64,
    pub kind: people::Kind,
    pub role: people::Role,
    pub login: Text,
    pub name: Text,
}
/// Authentic project period or party pool accounting, including superseded rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ledger {
    pub source: tasks::Funder,
    pub parent: Option<tasks::Funder>,
    pub numbers: tasks::Numbers,
    pub left: u64,
    pub closed: bool,
    pub began: Option<Wall>,
    pub next: Option<Wall>,
}
/// A live root's expense against its authentic allotment, largest first.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SpendLine {
    pub chip: Chip,
    pub funder: tasks::Funder,
    pub numbers: tasks::Numbers,
}
/// A recurring template with authentic retained period and supplied boundary facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Recurring {
    pub chip: Chip,
    pub key: u32,
    pub batch: List<ProposedTask>,
    pub budget: u64,
    pub overlap: tasks::RecurringOverlap,
    pub current_period: u64,
    pub began: Option<Wall>,
    pub next: Option<Wall>,
    pub last_period: u64,
    pub pending_period: Option<u64>,
    pub last_batch: List<u64>,
    pub cut: bool,
}
/// One project's adopted resources, current policy, membership and spend.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Project {
    pub acts: List<crate::acts::Offered>,
    pub number: u32,
    pub name: Text,
    pub resources: List<ProjectResource>,
    pub policy: Policy,
    pub members: List<Member>,
    pub current_period: Option<Ledger>,
    pub pools: List<Ledger>,
    pub configured_period: u64,
    pub configured_pools: List<Ledger>,
    pub configured_pools_cut: bool,
    pub spend: List<SpendLine>,
    pub recurring: List<Recurring>,
    pub cut: bool,
}
/// A note scope keeps every connector key byte; oversized keys are refused.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum NoteLocation {
    Deployment,
    Project(u32),
    Goal {
        project: u32,
        task: u64,
    },
    Resources {
        project: u32,
        scope: NoteScope,
    },
    /// A person's own preferences, in no project (domain/engine/briefs.md, 2.1).
    Party {
        party: u64,
    },
}
/// A note made from its child's store-loaded entry. The child currently retains
/// no changed wall time; its revision is carried unchanged.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Note {
    pub acts: List<crate::acts::Offered>,
    pub number: u64,
    pub scope: NoteLocation,
    pub description: Text,
    pub body: Text,
    pub writer: notes::Author,
    pub tasks: List<u64>,
    pub changed: Option<Wall>,
    pub revision: u32,
    pub cut: bool,
}

pub fn document_bound(limits: &super::DocumentLimits) -> Option<u64> {
    // Nested project carriers have no task steering offers of their own.
    let count = u64::from(limits.lines);
    let carriers = count.checked_mul(count)?.checked_add(count.checked_mul(8)?)?.checked_add(8)?;
    super::task_tree::overview_bound(limits)?
        .checked_mul(carriers)?
        .checked_add(super::offers::heap_bound(limits)?)?
        .checked_add(List::<people::PermissionRole>::price(limits.lines)?)?
        .checked_add(List::<Ledger>::price(limits.lines)?)?
        .checked_add(u64::try_from(size_of::<Project>()).ok()?)?
        .checked_add(u64::from(limits.text_bytes))
}

/// A loaded note owns its whole description and body (domain/engine/briefs.md, 2).
pub fn note_bound(limits: &Limits) -> Option<u64> {
    u64::try_from(size_of::<super::Document>())
        .ok()?
        .checked_add(super::offers::heap_bound(&limits.documents)?)?
        .checked_add(List::<u64>::price(limits.documents.lines)?)?
        .checked_add(List::<Box<[u8]>>::price(limits.documents.segments)?)?
        .checked_add(u64::from(limits.notes.pattern_bytes))?
        .checked_add(u64::from(limits.notes.description_bytes))?
        .checked_add(u64::from(limits.notes.body_bytes))
}

//! Forge documents from retained facts (domain/landing.md, 14; domain/forge/attention.md, 1).
use crate::{Panel, PersonFact, ReceiptRun};
use alloc::boxed::Box;
use core::mem::size_of_val;
use temper_engine_domain_forge_landing as policy;
pub mod changes;
pub use changes::{ChangesLanded, ChangesQueue, ChangesQueued, ChangesView, TrainLanding};
pub mod follows;
pub use follows::{DependencyView, FollowLagEntry, FollowView, FollowsView, follows_subject};

/// The changes waiting on a person's approval, the party's own scope (domain/forge/change/questions.md, 2).
pub const APPROVALS_SUBJECT: u64 = (1 << 62) | (1 << 37);

/// A project changes subject lies outside task and landing list subjects.
#[must_use]
pub fn changes_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 33) | u64::from(project)
}

/// A project list subject lies outside the task number space.
#[must_use]
pub fn landings_subject(project: u32, ended: bool) -> u64 {
    (1 << 62) | (u64::from(ended) << 32) | u64::from(project)
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::large_enum_variant, reason = "a change panel carries the bounded Copy landing repair context")]
pub enum Document {
    Findings(crate::Findings),
    Change(Panel),
    Landing(LandingView),
    Landings(LandingsView),
    Follows(FollowsView),
    Changes(ChangesView),
    Status(crate::StatusView),
    Aways(crate::AwaysView),
    Decisions(crate::DecisionsView),
    Dismissals(Box<[PersonFact]>),
    Devices(crate::DevicesView),
    Drafts(crate::DraftsView),
}
/// One landing, its members, gate proofs and the project's turn holder.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingView {
    pub task: u64,
    pub project: u32,
    pub phase: policy::Phase,
    pub members: Box<[LandingMember]>,
    pub preparation: u16,
    pub stale_proofs: u32,
    pub turn: Option<u64>,
    pub train: Option<LandingTrain>,
    pub round: policy::Round,
    pub step: LandingStep,
    pub probe: Option<[u8; 32]>,
}
/// A train is one repository and landing branch (domain/landing.md, 9).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingTrain {
    pub repository: skein_forge::api::Repository,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub branch: Box<[u8]>,
}
/// The phase refined by the round's durable work (domain/landing.md, 14).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LandingStep {
    Waiting,
    Preparing,
    Proving,
    Proved,
    Turn,
    Landed,
    Held,
    Cancelled,
    Judging,
    Leaving,
    Recording,
    Telling,
    Probing,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PrefixJudgement {
    pub prefix: [u8; 32],
    pub verdict: temper_engine_domain_forge_change::Status,
    pub gates: Box<[crate::PanelGate]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingMember {
    pub change: u64,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub branch: Box<[u8]>,
    pub landing: Box<[u8]>,
    pub pull: Option<u64>,
    pub base: Option<[u8; 32]>,
    pub candidate: Option<[u8; 32]>,
    pub landed: Option<[u8; 32]>,
    pub gates: Box<[LandingGate]>,
    pub head: Option<[u8; 32]>,
    pub left: Option<policy::Leave>,
    pub judgement: Option<PrefixJudgement>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingGate {
    pub name: Box<[u8]>,
    pub blocking: bool,
    pub key: Option<[u8; 32]>,
    pub proof: GateProof,
    pub faults: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GateProof {
    Unkeyed,
    Keyed,
    Waiting,
    Dispatched { attempt: u16, run: Option<u64> },
    Passed(ReceiptRun),
    Failed(ReceiptRun),
}
/// A project's retained landings, live first, newest task first within each group.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingsView {
    pub project: u32,
    pub turn: Option<u64>,
    pub landings: Box<[LandingLine]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingLine {
    pub task: u64,
    pub phase: policy::Phase,
    pub changes: Box<[u64]>,
    pub repositories: Box<[Box<[u8]>]>,
    pub landed: Box<[[u8; 32]]>,
    pub train: Option<LandingTrain>,
}
impl Document {
    /// Owned storage retained by the root's asking decision (domain/routing.md, 1).
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "each document charges its owned view evidence beside its record")]
    pub fn owned_bytes(&self) -> usize {
        let mut size = size_of::<Self>();
        match self {
            Self::Findings(view) => size = size.saturating_add(findings_bytes(view)),
            Self::Drafts(view) => {
                size = size.saturating_add(size_of_val(view.rows.as_ref()));
                for draft in &view.rows {
                    size = size
                        .saturating_add(draft.owner.len())
                        .saturating_add(draft.name.len())
                        .saturating_add(draft.row.path.len())
                        .saturating_add(draft.row.branch.len())
                        .saturating_add(draft.row.title.len());
                }
            }
            Self::Follows(view) => {
                size = size.saturating_add(follows::owned_bytes(view));
            }
            Self::Changes(view) => {
                size = size
                    .saturating_add(view.landing.len())
                    .saturating_add(size_of_val(view.queues.as_ref()))
                    .saturating_add(size_of_val(view.landed.as_ref()))
                    .saturating_add(size_of_val(view.landed_default.as_ref()));
                for queue in &view.queues {
                    size = size
                        .saturating_add(queue.repository.len())
                        .saturating_add(queue.branch.len())
                        .saturating_add(size_of_val(queue.changes.as_ref()));
                    if let Some(flight) = &queue.in_flight {
                        size = size.saturating_add(flight.title.len());
                    }
                    for change in &queue.changes {
                        size = size.saturating_add(change.title.len());
                    }
                }
                for change in &view.landed {
                    size = size.saturating_add(change.title.len());
                    size = size.saturating_add(change.signed.len().saturating_mul(size_of::<crate::SignOff>()));
                }
                for change in &view.landed_default {
                    size = size.saturating_add(change.title.len());
                    size = size.saturating_add(change.signed.len().saturating_mul(size_of::<crate::SignOff>()));
                }
            }
            Self::Devices(view) => {
                size = size.saturating_add(size_of_val(view.rows.as_ref()));
                for row in &view.rows {
                    size = size.saturating_add(row.label.len()).saturating_add(row.endpoint.len());
                }
            }
            Self::Aways(view) => {
                size = size.saturating_add(size_of_val(view.rows.as_ref()));
            }
            Self::Decisions(view) => {
                size = size
                    .saturating_add(size_of_val(view.rows.as_ref()))
                    .saturating_add(size_of_val(view.asked.as_ref()));
                for row in &view.rows {
                    size = size
                        .saturating_add(row.question.len())
                        .saturating_add(row.took.len())
                        .saturating_add(row.why.len());
                }
            }
            Self::Status(view) => {
                size = size.saturating_add(view.branch.len());
                if let Some(brief) = &view.brief {
                    size = size.saturating_add(brief.sentence.len()).saturating_add(brief.lede.len());
                }
            }
            Self::Dismissals(rows) => {
                size = size.saturating_add(size_of_val(rows.as_ref()));
            }
            Self::Change(panel) => {
                size = size
                    .saturating_add(panel.owner.len())
                    .saturating_add(panel.name.len())
                    .saturating_add(panel.branch.len())
                    .saturating_add(panel.landing.len())
                    .saturating_add(panel.approval_bytes())
                    .saturating_add(panel.gates.len().saturating_mul(size_of::<crate::PanelGate>()));
                if let crate::PanelStep::Queued { ahead, .. } = &panel.step {
                    size = size.saturating_add(size_of_val(ahead.as_ref()));
                }
            }
            Self::Landing(view) => {
                size = size
                    .saturating_add(phase_bytes(&view.phase))
                    .saturating_add(size_of_val(view.members.as_ref()))
                    .saturating_add(train_bytes(view.train.as_ref()))
                    .saturating_add(view.round.branch.len())
                    .saturating_add(view.round.failure.len());
                for member in &view.members {
                    size = size
                        .saturating_add(member.owner.len())
                        .saturating_add(member.name.len())
                        .saturating_add(member.branch.len())
                        .saturating_add(member.landing.len())
                        .saturating_add(size_of_val(member.gates.as_ref()));
                    if let Some(judgement) = &member.judgement {
                        size = size.saturating_add(size_of_val(judgement.gates.as_ref()));
                    }
                    for gate in &member.gates {
                        size = size.saturating_add(gate.name.len());
                        if let GateProof::Passed(run) | GateProof::Failed(run) = &gate.proof {
                            size = size.saturating_add(run.url.len());
                        }
                    }
                }
            }
            Self::Landings(view) => {
                size = size.saturating_add(size_of_val(view.landings.as_ref()));
                for row in &view.landings {
                    size = size
                        .saturating_add(phase_bytes(&row.phase))
                        .saturating_add(size_of_val(row.changes.as_ref()))
                        .saturating_add(size_of_val(row.repositories.as_ref()))
                        .saturating_add(size_of_val(row.landed.as_ref()))
                        .saturating_add(train_bytes(row.train.as_ref()));
                    for name in &row.repositories {
                        size = size.saturating_add(name.len());
                    }
                }
            }
        }
        size
    }
}
fn train_bytes(train: Option<&LandingTrain>) -> usize {
    match train {
        Some(train) => train.owner.len().saturating_add(train.name.len()).saturating_add(train.branch.len()),
        None => 0,
    }
}
fn findings_bytes(view: &crate::Findings) -> usize {
    let mut size = size_of_val(view.rows.as_ref()).saturating_add(size_of_val(view.areas.as_ref()));
    for row in &view.rows {
        size = size.saturating_add(row.title.len());
        for field in [&row.area, &row.why].into_iter().flatten() {
            size = size.saturating_add(field.len());
        }
    }
    for area in &view.areas {
        size = size.saturating_add(area.name.len()).saturating_add(area.words.len());
    }
    size
}
fn phase_bytes(phase: &policy::Phase) -> usize {
    match phase {
        policy::Phase::Held { was, .. } => size_of::<policy::Phase>().saturating_add(phase_bytes(was)),
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. }
        | policy::Phase::Landed
        | policy::Phase::Cancelled => 0,
    }
}

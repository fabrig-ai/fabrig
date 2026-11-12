//! Forge documents from retained facts (domain/landing.md, 14; domain/forge/attention.md, 1).
use crate::{Check, DISMISSED_SUBJECT, Domain, Panel, PersonFact, ReceiptKey, ReceiptRun};
use alloc::boxed::Box;
use core::mem::size_of_val;
use jig_domain_people::ApplicationScope;
use skein_lib::List;
use temper_engine_domain_forge_landing as policy;
mod changes;
pub use changes::{ChangesLanded, ChangesQueue, ChangesQueued, ChangesView};

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
#[cfg_attr(
    target_pointer_width = "64",
    expect(
        clippy::large_enum_variant,
        reason = "bounded documents move whole into the root decision; owned_bytes charges the inline enum"
    )
)]
pub enum Document {
    Change(Panel),
    Landing(LandingView),
    Landings(LandingsView),
    Changes(ChangesView),
    Status(crate::StatusView),
    Aways(crate::AwaysView),
    Decisions(crate::DecisionsView),
    Dismissals(Box<[PersonFact]>),
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
/// A project's retained landings, turn holder first, then newest task first.
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
}
impl Domain {
    #[must_use]
    pub fn document(
        &self,
        subject: u64,
        scope: ApplicationScope,
        party: u64,
        now: skein_lib::Wall,
        limits: &crate::Limits,
    ) -> Option<Document> {
        let project = match scope {
            ApplicationScope::Own => {
                return if subject == DISMISSED_SUBJECT {
                    Some(Document::Dismissals(self.dismissals(party)))
                } else if subject == crate::AWAY_SUBJECT {
                    Some(Document::Aways(self.aways_view(party)))
                } else {
                    None
                };
            }
            ApplicationScope::Project(project) => project,
            ApplicationScope::Deployment => return None,
        };
        for ended in [false, true] {
            if subject == landings_subject(project, ended) {
                return Some(Document::Landings(self.landings_view(project, ended)?));
            }
        }
        if subject == crate::decided_subject(project) {
            return Some(Document::Decisions(self.decisions_view(project, party)));
        }
        if subject == crate::status_subject(project) {
            return Some(Document::Status(self.status_view(project, party, now, limits)?));
        }
        if subject == changes_subject(project) {
            return Some(Document::Changes(self.changes_view(project, now, limits)?));
        }
        if let Some(row) = self.landing.rows.get(&subject) {
            if row.landing.project != project {
                return None;
            }
            return Some(Document::Landing(self.landing_view(subject)?));
        }
        Some(Document::Change(self.panel(subject, scope, now, limits)?))
    }
    fn turn_holder(&self, project: u32) -> Option<u64> {
        for (task, row) in &self.landing.rows {
            if row.landing.project == project && matches_turn(&row.landing.phase) {
                return Some(*task);
            }
        }
        None
    }
    fn landing_view(&self, task: u64) -> Option<LandingView> {
        let row = &self.landing.rows.get(&task)?.landing;
        let mut members = List::with_capacity(self.landing.limits.landing_members);
        for (member_index, member) in row.members.iter().enumerate() {
            let change = self.changes.get(&member.change)?;
            let repo = self.repositories.get(&change.repository)?;
            let mut gates = List::with_capacity(self.landing.limits.gates);
            for (index, saved) in member.gates.iter().enumerate() {
                let gate = self.landing.gates(row.project, change.repository).get(index)?;
                let key = proof_key(saved.proof);
                let mut proof = match saved.proof {
                    policy::Proof::Unkeyed => GateProof::Unkeyed,
                    policy::Proof::Keyed { .. } | policy::Proof::Passed { .. } | policy::Proof::Failed { .. } => {
                        GateProof::Keyed
                    }
                    policy::Proof::Dispatched { try_, .. } => GateProof::Dispatched { attempt: try_, run: None },
                };
                if let Some(key) = key {
                    let key = ReceiptKey {
                        project: row.project,
                        repository: change.repository,
                        check: Check::Gate(gate.name.clone()),
                        key,
                    };
                    if let Some(receipt) = self.landing.receipts.get(&key) {
                        proof = if receipt.passed {
                            GateProof::Passed(receipt.run.clone())
                        } else {
                            GateProof::Failed(receipt.run.clone())
                        };
                    } else {
                        let flight = match self.landing.flights.get(&key) {
                            Some(flight) => Some(flight),
                            None => self.landing.faults.get(&(
                                task,
                                u16::try_from(member_index).ok()?,
                                u16::try_from(index).ok()?,
                            )),
                        };
                        if let Some(flight) = flight {
                            if flight.task != task {
                                proof = GateProof::Waiting;
                            } else if let GateProof::Dispatched { attempt, .. } = proof {
                                let run = match &flight.run {
                                    Some(run) => Some(run.id),
                                    None => None,
                                };
                                proof = GateProof::Dispatched { attempt, run };
                            }
                        }
                    }
                }
                gates
                    .push(LandingGate {
                        name: gate.name.clone(),
                        blocking: gate.blocking,
                        key,
                        proof,
                        faults: saved.faults,
                    })
                    .expect("admitted gates");
            }
            members
                .push(LandingMember {
                    change: member.change,
                    forge: change.repository.forge,
                    owner: repo.owner.clone(),
                    name: repo.name.clone(),
                    branch: change.branch.clone(),
                    landing: change.base.clone(),
                    pull: change.pull,
                    base: member.base,
                    candidate: member.candidate,
                    landed: member.landed,
                    gates: gates.into_boxed(),
                })
                .expect("admitted members");
        }
        Some(LandingView {
            task,
            project: row.project,
            phase: row.phase.clone(),
            members: members.into_boxed(),
            preparation: row.preparation,
            stale_proofs: row.stale_proofs,
            turn: self.turn_holder(row.project),
        })
    }
    fn landings_view(&self, project: u32, ended: bool) -> Option<LandingsView> {
        let turn = self.turn_holder(project);
        let mut landings = List::with_capacity(self.landing.limits.landings);
        let mut before = None;
        loop {
            let mut next = None;
            for (task, row) in &self.landing.rows {
                if row.landing.project != project || (!ended && is_ended(&row.landing.phase)) {
                    continue;
                }
                if let Some(previous) = before
                    && (Some(*task) == turn || (Some(previous) != turn && *task >= previous))
                {
                    continue;
                }
                if match next {
                    Some(current) => Some(*task) == turn || (Some(current) != turn && *task > current),
                    None => true,
                } {
                    next = Some(*task);
                }
            }
            let Some(task) = next else { break };
            let row = &self.landing.rows.get(&task)?.landing;
            let mut changes = List::with_capacity(self.landing.limits.landing_members);
            let mut repositories = List::with_capacity(self.landing.limits.landing_members);
            let mut landed = List::with_capacity(self.landing.limits.landing_members);
            for member in &row.members {
                let change = self.changes.get(&member.change)?;
                let repo = self.repositories.get(&change.repository)?;
                let mut name = List::with_capacity(
                    u32::try_from(repo.owner.len().checked_add(repo.name.len())?.checked_add(1)?).ok()?,
                );
                for byte in &repo.owner {
                    name.push(*byte).ok()?;
                }
                name.push(b'/').ok()?;
                for byte in &repo.name {
                    name.push(*byte).ok()?;
                }
                changes.push(member.change).ok()?;
                repositories.push(name.into_boxed()).ok()?;
                if let Some(commit) = member.landed {
                    landed.push(commit).ok()?;
                }
            }
            landings
                .push(LandingLine {
                    task,
                    phase: row.phase.clone(),
                    changes: changes.into_boxed(),
                    repositories: repositories.into_boxed(),
                    landed: landed.into_boxed(),
                })
                .ok()?;
            before = Some(task);
        }
        Some(LandingsView { project, turn, landings: landings.into_boxed() })
    }
}
pub(crate) fn matches_turn(phase: &policy::Phase) -> bool {
    match phase {
        policy::Phase::Turn { .. } => true,
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Landed
        | policy::Phase::Held { .. }
        | policy::Phase::Cancelled => false,
    }
}
fn is_ended(phase: &policy::Phase) -> bool {
    match phase {
        policy::Phase::Landed | policy::Phase::Cancelled => true,
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. }
        | policy::Phase::Held { .. } => false,
    }
}
fn proof_key(proof: policy::Proof) -> Option<[u8; 32]> {
    match proof {
        policy::Proof::Unkeyed => None,
        policy::Proof::Keyed { key }
        | policy::Proof::Dispatched { key, .. }
        | policy::Proof::Passed { key }
        | policy::Proof::Failed { key } => Some(key),
    }
}

impl Document {
    /// Owned storage retained by the root's asking decision (domain/routing.md, 1).
    #[must_use]
    pub fn owned_bytes(&self) -> usize {
        let mut size = size_of::<Self>();
        match self {
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
                    for change in &queue.changes {
                        size = size.saturating_add(change.title.len());
                    }
                }
                for change in &view.landed {
                    size = size.saturating_add(change.title.len());
                }
                for change in &view.landed_default {
                    size = size.saturating_add(change.title.len());
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
                size = size.saturating_add(phase_bytes(&view.phase)).saturating_add(size_of_val(view.members.as_ref()));
                for member in &view.members {
                    size = size
                        .saturating_add(member.owner.len())
                        .saturating_add(member.name.len())
                        .saturating_add(member.branch.len())
                        .saturating_add(member.landing.len())
                        .saturating_add(size_of_val(member.gates.as_ref()));
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
                        .saturating_add(size_of_val(row.landed.as_ref()));
                    for name in &row.repositories {
                        size = size.saturating_add(name.len());
                    }
                }
            }
        }
        size
    }
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

/// The adopted repository's project owns its changes document (domain/forge/queues.md).
pub(crate) fn changes_mark(d: &Domain, repository: skein_forge::api::Repository) -> Option<[u64; 3]> {
    let project = d.repositories.get(&repository)?.project;
    Some([changes_subject(project), crate::status_subject(project), APPROVALS_SUBJECT])
}

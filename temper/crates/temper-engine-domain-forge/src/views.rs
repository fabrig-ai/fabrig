//! Forge documents from retained facts (domain/landing.md, 14; domain/forge/attention.md, 1).
use crate::{Check, DISMISSED_SUBJECT, Domain, ReceiptKey};
use jig_domain_people::ApplicationScope;
use skein_lib::List;
use temper_engine_domain_forge_landing as policy;
pub use temper_engine_domain_forge_vocabulary::views::*;
#[expect(hidden_glob_reexports, reason = "the top keeps its private namesake module")]
mod changes;
pub use changes::{ChangesLanded, ChangesQueue, ChangesQueued, ChangesView, TrainLanding};
#[expect(hidden_glob_reexports, reason = "the top keeps its private namesake module")]
mod follows;
pub use follows::{DependencyView, FollowLagEntry, FollowView, FollowsView, follows_subject};

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
                } else if subject == crate::DEVICES_SUBJECT {
                    Some(Document::Devices(self.devices_view(party)))
                } else if subject == crate::AWAY_SUBJECT {
                    Some(Document::Aways(self.aways_view(party)))
                } else {
                    None
                };
            }
            ApplicationScope::Project(project) => project,
            ApplicationScope::Deployment => return None,
        };
        if subject == follows_subject(project) {
            return Some(Document::Follows(self.follows_view(project, limits)?));
        }
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
        if subject == crate::drafts_subject(project) {
            return Some(Document::Drafts(self.drafts_view(project)));
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
                    landing: row.branch.clone(),
                    pull: Some(member.pull),
                    base: member.base,
                    candidate: member.candidate,
                    landed: member.landed,
                    gates: gates.into_boxed(),
                    head: member.head,
                    left: member.left,
                    judgement: self.prefix_judgement(row, member),
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
            train: self.landing_train(row),
            round: round_view(row),
            step: landing_step(self, row),
            probe: landing_probe(self, row),
        })
    }
    fn landing_train(&self, row: &policy::Landing) -> Option<LandingTrain> {
        if row.round.branch.is_empty() {
            return None;
        }
        let member = row.members.first()?;
        let repository =
            skein_forge::api::Repository { forge: member.repository.forge, repository: member.repository.repository };
        let repo = self.repositories.get(&repository)?;
        Some(LandingTrain {
            repository,
            owner: repo.owner.clone(),
            name: repo.name.clone(),
            branch: row.branch.clone(),
        })
    }
    pub(crate) fn prefix_judgement(&self, row: &policy::Landing, member: &policy::Member) -> Option<PrefixJudgement> {
        use temper_engine_domain_forge_change as c;
        let judging = if row.round.branch.is_empty() { row.round.judging } else { member.judging };
        let prefix = member.candidate?;
        let head = member.head?;
        let change = self.changes.get(&member.change)?;
        let mut gates = List::with_capacity(u32::try_from(change.change.gates.len()).ok()?);
        let mut verdict = c::Status::Passed;
        for gate in &change.change.gates {
            let status = if !judging && (gate.kind == c::GateKind::Check || gate.freshness == c::Freshness::Exact) {
                c::Status::Unknown
            } else {
                c::prefix_verdict(&change.change, *gate, &change.verdicts, head, prefix)
            };
            if gate.blocking {
                match status {
                    c::Status::Failed => verdict = c::Status::Failed,
                    c::Status::Unknown | c::Status::Pending if verdict != c::Status::Failed => {
                        verdict = c::Status::Pending;
                    }
                    c::Status::Unknown | c::Status::Pending | c::Status::Passed => {}
                }
            }
            gates
                .push(crate::PanelGate {
                    number: gate.number,
                    kind: gate.kind,
                    blocking: gate.blocking,
                    verdict: status,
                })
                .ok()?;
        }
        Some(PrefixJudgement { prefix, verdict, gates: gates.into_boxed() })
    }
    fn landings_view(&self, project: u32, ended: bool) -> Option<LandingsView> {
        let turn = self.turn_holder(project);
        let mut landings = List::with_capacity(self.landing.limits.landings);
        let mut ordered = skein_lib::Map::with_capacity(self.landing.limits.landings);
        for (task, row) in &self.landing.rows {
            if row.landing.project == project && (ended || !is_ended(&row.landing.phase)) {
                ordered.insert((is_ended(&row.landing.phase), core::cmp::Reverse(*task)), *task).ok()?;
            }
        }
        for (_, task) in &ordered {
            let task = *task;
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
                    train: self.landing_train(row),
                })
                .ok()?;
        }
        Some(LandingsView { project, turn, landings: landings.into_boxed() })
    }
}
fn round_view(row: &policy::Landing) -> policy::Round {
    let mut round = row.round.clone();
    if !round.branch.is_empty() {
        for member in &row.members {
            if member.candidate.is_some() && (member.left.is_none() || round.ending.is_some()) {
                round.branch.clone_from(&member.branch);
                round.combined = member.combined;
            }
        }
    }
    round.failure = crate::preparation::failure(&round.failure);
    round
}
pub(crate) fn landing_step(d: &Domain, row: &policy::Landing) -> LandingStep {
    match row.phase {
        policy::Phase::Held { .. } => LandingStep::Held,
        policy::Phase::Landed => LandingStep::Landed,
        policy::Phase::Cancelled => LandingStep::Cancelled,
        policy::Phase::Waiting
        | policy::Phase::Preparing
        | policy::Phase::Proving
        | policy::Phase::Proved { .. }
        | policy::Phase::Turn { .. } => {
            if row.round.ending.is_some() {
                return LandingStep::Leaving;
            }
            if let Some(step) = landed_step(row) {
                return step;
            }
            match row.phase {
                policy::Phase::Waiting => LandingStep::Waiting,
                policy::Phase::Preparing => LandingStep::Preparing,
                policy::Phase::Proving if policy::probe(row).is_some() => LandingStep::Probing,
                policy::Phase::Proving | policy::Phase::Proved { .. } if judging(d, row) => LandingStep::Judging,
                policy::Phase::Proving => LandingStep::Proving,
                policy::Phase::Proved { .. } => LandingStep::Proved,
                policy::Phase::Turn { .. } => LandingStep::Turn,
                policy::Phase::Held { .. } | policy::Phase::Landed | policy::Phase::Cancelled => {
                    unreachable!("terminal phases returned above")
                }
            }
        }
    }
}
pub(crate) fn landing_probe(d: &Domain, row: &policy::Landing) -> Option<[u8; 32]> {
    if landing_step(d, row) != LandingStep::Probing {
        return None;
    }
    row.members.get(usize::from(policy::probe(row)?))?.candidate
}
fn judging(d: &Domain, row: &policy::Landing) -> bool {
    for member in &row.members {
        if member.left.is_none()
            && member.landed.is_none()
            && member.judging
            && let Some(judgement) = d.prefix_judgement(row, member)
            && judgement.verdict != temper_engine_domain_forge_change::Status::Passed
        {
            return true;
        }
    }
    false
}
fn landed_step(row: &policy::Landing) -> Option<LandingStep> {
    for member in &row.members {
        if member.landed.is_none() {
            continue;
        }
        let legacy = row.round.branch.is_empty();
        if !legacy && member.told {
            continue;
        }
        let combined = if legacy { row.round.combined } else { member.combined };
        let recorded = if legacy { row.round.recorded } else { member.recorded };
        return Some(if combined == Some(policy::Combine::Squash) && !recorded {
            LandingStep::Recording
        } else {
            LandingStep::Telling
        });
    }
    None
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

/// The adopted repository's project owns its changes document (domain/forge/queues.md).
pub(crate) fn changes_mark(d: &Domain, repository: skein_forge::api::Repository) -> Option<[u64; 3]> {
    let project = d.repositories.get(&repository)?.project;
    Some([changes_subject(project), crate::status_subject(project), APPROVALS_SUBJECT])
}

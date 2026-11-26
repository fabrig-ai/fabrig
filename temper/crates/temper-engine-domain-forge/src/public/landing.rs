//! A landing reached through a public member (temper/docs/design/client/forge/README.md, 9.1).
use super::{Names, phase, policy, w};
use crate::{
    Check, Domain, PrefixJudgement, ReceiptKey,
    generated::public::{public_combined_to_record, public_judgement_to_record},
};
use policy::Awaited;
use skein_lib::List;
use temper_client_protocol_wire::v1::Record;
impl Domain {
    fn public_landing_wait(&self, row: &policy::Landing) -> Option<w::PublicLandingWait> {
        if crate::views::is_ended(&row.phase) {
            return None;
        }
        let kept = self.landing.rows.get(&row.task)?;
        if kept.room.is_some() {
            return Some(w::PublicLandingWait::Room);
        }
        Some(match kept.waiting? {
            Awaited::Release => w::PublicLandingWait::Release,
            Awaited::Ready { .. } => w::PublicLandingWait::Ready,
            Awaited::Turn => w::PublicLandingWait::Turn,
            Awaited::Prepare => w::PublicLandingWait::Prepare,
            Awaited::Prefixes => w::PublicLandingWait::Prefixes,
            Awaited::Open => w::PublicLandingWait::Open,
            Awaited::Close => w::PublicLandingWait::Close,
            Awaited::Delete { .. } => w::PublicLandingWait::Delete,
            Awaited::Post { .. } => w::PublicLandingWait::Post,
            Awaited::Run { .. } => w::PublicLandingWait::Run,
            Awaited::Receipt { .. } => w::PublicLandingWait::Receipt,
            Awaited::Judgement { .. } => w::PublicLandingWait::Judgement,
            Awaited::Merge => w::PublicLandingWait::Merge,
            Awaited::Record { .. } => w::PublicLandingWait::Record,
            Awaited::Ancestry { .. } => w::PublicLandingWait::Ancestry,
            Awaited::Read => w::PublicLandingWait::Read,
        })
    }

    pub(super) fn make_landing(&self, row: &policy::Landing, names: &mut Names) -> Option<w::PublicLanding> {
        let l = &w::CEILINGS;
        let mut members = List::with_capacity(l.public_landing_members);
        for (member_index, member) in row.members.iter().enumerate() {
            let change = self.changes.get(&member.change)?;
            let mut gates = List::with_capacity(l.public_member_gates);
            for (index, saved) in member.gates.iter().enumerate() {
                let gate = self.landing.gates(row.project, change.repository).get(index)?;
                let proof = self.public_proof(row, member_index, index, saved.proof)?;
                gates
                    .push(w::PublicGateProof {
                        number: u64::try_from(index).ok()?,
                        kind: w::PublicGateKind::Check,
                        blocks: gate.blocking,
                        proof,
                        faults: saved.faults,
                    })
                    .ok()?;
            }
            members
                .push(w::PublicMember {
                    task: names.name(member.change)?,
                    head: member.head,
                    left: match member.left {
                        Some(value) => Some(left(value)),
                        None => None,
                    },
                    judgement: match self.prefix_judgement(row, member) {
                        Some(value) => Some(make_judgement(&value)?),
                        None => None,
                    },
                    base: member.base,
                    candidate: member.candidate,
                    landed: member.landed,
                    gates,
                })
                .ok()?;
        }
        {
            let value = w::PublicLanding {
                wait: self.public_landing_wait(row),
                search: match &row.round.search {
                    Some(search) => Some(w::PublicSearch {
                        green: search_bound(row, search.green)?,
                        red: search_bound(row, search.red)?,
                        probe: crate::views::landing_probe(self, row),
                    }),
                    None => None,
                },
                phase: phase(&row.phase),
                hold: match row.phase {
                    policy::Phase::Held { why, .. } => match why {
                        policy::Hold::Faults { .. } => Some(w::PublicLandingHold::Faults),
                        policy::Hold::Prepare => Some(w::PublicLandingHold::Prepare),
                        policy::Hold::Record { .. } => Some(w::PublicLandingHold::Record),
                        policy::Hold::Timeout { .. } => Some(w::PublicLandingHold::Timeout),
                        policy::Hold::Turn => Some(w::PublicLandingHold::Turn),
                        policy::Hold::Read | policy::Hold::Claimed { .. } | policy::Hold::Cleanup => None,
                    },
                    policy::Phase::Waiting
                    | policy::Phase::Preparing
                    | policy::Phase::Proving
                    | policy::Phase::Proved { .. }
                    | policy::Phase::Turn { .. }
                    | policy::Phase::Landed
                    | policy::Phase::Cancelled => None,
                },
                preparation: u32::from(row.preparation),
                members,
                stale_proofs: row.stale_proofs,
                holds_turn: crate::views::matches_turn(&row.phase),
                combined: if let Some(combined) = &row.round.combined {
                    let record = public_combined_to_record(combined);
                    Some(record)
                } else {
                    None
                },
                step: Some(step(crate::views::landing_step(self, row))),
            };
            value.measure(l).ok()?;
            Some(value)
        }
    }
    fn public_proof(
        &self,
        row: &policy::Landing,
        member: usize,
        gate: usize,
        saved: policy::Proof,
    ) -> Option<w::PublicProof> {
        let (key, mut proof) = match saved {
            policy::Proof::Unkeyed => return Some(w::PublicProof::Unkeyed),
            policy::Proof::Keyed { key } => (key, w::PublicProof::Keyed),
            policy::Proof::Dispatched { key, .. } => (key, w::PublicProof::Dispatched),
            policy::Proof::Passed { key } => (key, w::PublicProof::Passed),
            policy::Proof::Failed { key } => (key, w::PublicProof::Failed),
        };
        let repository = self.changes.get(&row.members.get(member)?.change)?.repository;
        let name = &self.landing.gates(row.project, repository).get(gate)?.name;
        let key = ReceiptKey { project: row.project, repository, check: Check::Gate(name.clone()), key };
        if let Some(receipt) = self.landing.receipts.get(&key) {
            proof = if receipt.passed { w::PublicProof::Passed } else { w::PublicProof::Failed };
        } else {
            let flight = match self.landing.flights.get(&key) {
                Some(flight) => Some(flight),
                None => self.landing.faults.get(&(row.task, u16::try_from(member).ok()?, u16::try_from(gate).ok()?)),
            };
            if let Some(flight) = flight
                && flight.task != row.task
            {
                proof = w::PublicProof::Waiting;
            }
        }
        Some(proof)
    }
}
fn step(value: crate::LandingStep) -> w::PublicLandingStep {
    use crate::LandingStep as S;
    match value {
        S::Waiting => w::PublicLandingStep::Waiting,
        S::Preparing => w::PublicLandingStep::Preparing,
        S::Proving => w::PublicLandingStep::Proving,
        S::Proved => w::PublicLandingStep::Proved,
        S::Turn => w::PublicLandingStep::Turn,
        S::Landed => w::PublicLandingStep::Landed,
        S::Held => w::PublicLandingStep::Held,
        S::Cancelled => w::PublicLandingStep::Cancelled,
        S::Judging => w::PublicLandingStep::Judging,
        S::Leaving => w::PublicLandingStep::Leaving,
        S::Recording => w::PublicLandingStep::Recording,
        S::Probing | S::ProbingByTest => w::PublicLandingStep::Probing,
        S::Telling => w::PublicLandingStep::Telling,
    }
}
fn left(value: policy::Leave) -> w::PublicLeave {
    match value {
        policy::Leave::Conflict | policy::Leave::EarlierConflict => w::PublicLeave::Conflict,
        policy::Leave::Unit { .. } | policy::Leave::Dependency { .. } | policy::Leave::Held => w::PublicLeave::Held,
        policy::Leave::HeadMoved { .. } => w::PublicLeave::HeadMoved,
        policy::Leave::Cancelled => w::PublicLeave::Cancelled,
        policy::Leave::Refused { gate } => w::PublicLeave::Refused(w::PublicGateNumber { gate }),
        policy::Leave::Failed { gate, .. } => w::PublicLeave::Failed(w::PublicGateNumber { gate: u64::from(gate) }),
        policy::Leave::Blamed { gate, red, green, .. } => {
            w::PublicLeave::Blamed(w::PublicBlamed { gate: u64::from(gate), red, green })
        }
    }
}

fn search_bound(row: &policy::Landing, member: u16) -> Option<w::PublicSearchBound> {
    let prefix = if member == 0 {
        row.round.upstream?
    } else {
        let member = row.members.get(usize::from(member.checked_sub(1)?))?;
        if let Some(prefix) = member.candidate {
            prefix
        } else if let Some(policy::Leave::Blamed { red, .. }) = member.left {
            red
        } else {
            return None;
        }
    };
    {
        let value = w::PublicSearchBound { member, prefix };
        value.measure(&w::CEILINGS).ok()?;
        Some(value)
    }
}

fn make_judgement(value: &PrefixJudgement) -> Option<w::PublicJudgement> {
    let l = &w::CEILINGS;
    if value.gates.len() > skein_lib::usize_of(l.public_judgement_gates) {
        return None;
    }
    let record = public_judgement_to_record(value).ok()?;
    record.measure(l).ok()?;
    Some(record)
}

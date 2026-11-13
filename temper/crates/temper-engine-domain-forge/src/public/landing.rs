//! A landing reached through a public member (temper/docs/design/client/forge/README.md, 9.1).
use super::{Names, phase, policy, w};
use crate::{Check, Domain, ReceiptKey};
use skein_lib::List;
impl Domain {
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
                    .push(
                        w::PublicGateProof::new(
                            l,
                            w::PublicGateProofParts {
                                number: u64::try_from(index).ok()?,
                                kind: w::PublicGateKind::Check,
                                blocks: gate.blocking,
                                proof,
                                faults: saved.faults,
                            },
                        )
                        .ok()?,
                    )
                    .ok()?;
            }
            members
                .push(
                    w::PublicMember::new(
                        l,
                        w::PublicMemberParts {
                            task: names.name(member.change)?,
                            head: member.head,
                            left: match member.left {
                                Some(value) => Some(left(value)?),
                                None => None,
                            },
                            judgement: match self.prefix_judgement(row, member) {
                                Some(value) => Some(make_judgement(value)?),
                                None => None,
                            },
                            base: member.base,
                            candidate: member.candidate,
                            landed: member.landed,
                            gates,
                        },
                    )
                    .ok()?,
                )
                .ok()?;
        }
        w::PublicLanding::new(
            l,
            w::PublicLandingParts {
                phase: phase(&row.phase),
                hold: match row.phase {
                    policy::Phase::Held { why, .. } => Some(match why {
                        policy::Hold::Faults { .. } => w::PublicLandingHold::Faults,
                        policy::Hold::Prepare => w::PublicLandingHold::Prepare,
                        policy::Hold::Record { .. } => w::PublicLandingHold::Record,
                        policy::Hold::Timeout { .. } => w::PublicLandingHold::Timeout,
                        policy::Hold::Turn => w::PublicLandingHold::Turn,
                    }),
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
                combined: match row.round.combined {
                    Some(policy::Combine::Merge) => Some(w::PublicCombined::Merge),
                    Some(policy::Combine::Squash) => Some(w::PublicCombined::Squash),
                    None => None,
                },
                step: Some(step(crate::views::landing_step(self, row))),
            },
        )
        .ok()
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
        S::Telling => w::PublicLandingStep::Telling,
    }
}
fn left(value: policy::Leave) -> Option<w::PublicLeave> {
    Some(match value {
        policy::Leave::Conflict => w::PublicLeave::Conflict,
        policy::Leave::HeadMoved { .. } => w::PublicLeave::HeadMoved,
        policy::Leave::Cancelled => w::PublicLeave::Cancelled,
        policy::Leave::Held => w::PublicLeave::Held,
        policy::Leave::Refused { gate } => {
            w::PublicLeave::Refused(w::PublicGateNumber::new(&w::CEILINGS, w::PublicGateNumberParts { gate }).ok()?)
        }
        policy::Leave::Failed { gate, .. } => w::PublicLeave::Failed(
            w::PublicGateNumber::new(&w::CEILINGS, w::PublicGateNumberParts { gate: u64::from(gate) }).ok()?,
        ),
    })
}

fn make_judgement(value: crate::PrefixJudgement) -> Option<w::PublicJudgement> {
    let l = &w::CEILINGS;
    let mut gates = List::with_capacity(l.public_judgement_gates);
    for gate in value.gates {
        gates
            .push(
                w::PublicGate::new(
                    l,
                    w::PublicGateParts {
                        number: gate.number,
                        kind: match gate.kind {
                            temper_engine_domain_forge_change::GateKind::Agent => w::PublicGateKind::Review,
                            temper_engine_domain_forge_change::GateKind::Person => w::PublicGateKind::Approval,
                            temper_engine_domain_forge_change::GateKind::Check
                            | temper_engine_domain_forge_change::GateKind::Follow => w::PublicGateKind::Check,
                        },
                        blocks: gate.blocking,
                        verdict: match gate.verdict {
                            temper_engine_domain_forge_change::Status::Unknown => w::PublicVerdict::None,
                            temper_engine_domain_forge_change::Status::Pending => w::PublicVerdict::Pending,
                            temper_engine_domain_forge_change::Status::Passed => w::PublicVerdict::Approved,
                            temper_engine_domain_forge_change::Status::Failed => w::PublicVerdict::Changes,
                        },
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    w::PublicJudgement::new(
        l,
        w::PublicJudgementParts {
            prefix: value.prefix,
            verdict: match value.verdict {
                temper_engine_domain_forge_change::Status::Unknown => w::PublicCi::Unknown,
                temper_engine_domain_forge_change::Status::Pending => w::PublicCi::Pending,
                temper_engine_domain_forge_change::Status::Passed => w::PublicCi::Passed,
                temper_engine_domain_forge_change::Status::Failed => w::PublicCi::Failed,
            },
            gates,
        },
    )
    .ok()
}

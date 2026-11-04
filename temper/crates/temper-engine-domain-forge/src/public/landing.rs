//! A landing reached through a public member (temper/docs/design/client/forge.md, 11.1).
use super::{Names, phase, policy, w};
use crate::{Check, Domain, ReceiptKey};
use alloc::boxed::Box;
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
                let (proof, run) = self.public_proof(row, member_index, index, saved.proof)?;
                gates
                    .push(
                        w::PublicGateProof::new(
                            l,
                            w::PublicGateProofParts {
                                number: u64::try_from(index).ok()?,
                                kind: w::PublicGateKind::Check,
                                blocks: gate.blocking,
                                proof,
                                run,
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
                            repository: self.repository_address(change)?,
                            pull: self.public_pull(change),
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
                    policy::Phase::Held { .. } => Some(w::PublicLandingHold::Faults),
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
    ) -> Option<(w::PublicProof, Option<Box<[u8]>>)> {
        let (key, mut proof) = match saved {
            policy::Proof::Unkeyed => return Some((w::PublicProof::Unkeyed, None)),
            policy::Proof::Keyed { key } => (key, w::PublicProof::Keyed),
            policy::Proof::Dispatched { key, .. } => (key, w::PublicProof::Dispatched),
            policy::Proof::Passed { key } => (key, w::PublicProof::Passed),
            policy::Proof::Failed { key } => (key, w::PublicProof::Failed),
        };
        let repository = self.changes.get(&row.members.get(member)?.change)?.repository;
        let name = &self.landing.gates(row.project, repository).get(gate)?.name;
        let key = ReceiptKey { project: row.project, repository, check: Check::Gate(name.clone()), key };
        let mut run = None;
        if let Some(receipt) = self.landing.receipts.get(&key) {
            proof = if receipt.passed { w::PublicProof::Passed } else { w::PublicProof::Failed };
            run = Some(receipt.run.url.clone());
        } else {
            let flight = match self.landing.flights.get(&key) {
                Some(flight) => Some(flight),
                None => self.landing.faults.get(&(row.task, u16::try_from(member).ok()?, u16::try_from(gate).ok()?)),
            };
            if let Some(flight) = flight {
                if flight.task != row.task {
                    proof = w::PublicProof::Waiting;
                } else if let Some(flight) = &flight.run {
                    run = Some(flight.url.clone());
                }
            }
        }
        if let Some(forge) = self.configured.get(&repository.forge)
            && forge.mode == crate::Mode::Fake
            && run.is_some()
        {
            run = Some(Box::default());
        }
        Some((proof, run))
    }
}

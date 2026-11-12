//! The change source table (temper/docs/design/client/forge/README.md, 9.3).
use super::{Names, PublicAsk, w};
use crate::{ChangeRow, Domain, Limits};
use skein_lib::List;
use temper_engine_domain_forge_change as c;
struct Step {
    kind: w::PublicStep,
    repair: Option<w::PublicRepair>,
    merge: Option<[u8; 32]>,
    tiers: u32,
}
impl Domain {
    pub(super) fn make_change(
        &self,
        ask: &PublicAsk<'_>,
        row: &ChangeRow,
        limits: &Limits,
        names: &mut Names,
    ) -> Option<w::PublicChange> {
        let l = &w::CEILINGS;
        let mut state = &row.change.state;
        let mut hold = None;
        while let c::State::Held { was, why } = state {
            hold = Some(hold_kind(*why));
            state = was;
        }
        let step = self.public_step(row, state)?;
        let mut gates = List::with_capacity(l.public_change_gates);
        for gate in &row.change.gates {
            let mut verdict = c::Status::Unknown;
            for report in &row.verdicts {
                if gate.number == report.number
                    && row.change.last_head.is_some()
                    && (Some(report.head) == row.change.last_head
                        || (gate.freshness == c::Freshness::Clean && row.change.clean.contains(&report.head)))
                {
                    verdict = worse(verdict, report.status);
                }
            }
            if let Some((_, c::Delegate::Gate { number, head })) = row.delegate
                && number == gate.number
                && Some(head) == row.change.last_head
            {
                verdict = worse(verdict, c::Status::Pending);
            }
            gates
                .push(
                    w::PublicGate::new(
                        l,
                        w::PublicGateParts {
                            number: gate.number,
                            kind: match gate.kind {
                                c::GateKind::Agent => w::PublicGateKind::Review,
                                c::GateKind::Person => w::PublicGateKind::Approval,
                                c::GateKind::Check => w::PublicGateKind::Check,
                            },
                            blocks: gate.blocking,
                            verdict: match verdict {
                                c::Status::Unknown => w::PublicVerdict::None,
                                c::Status::Pending => w::PublicVerdict::Pending,
                                c::Status::Passed => w::PublicVerdict::Approved,
                                c::Status::Failed => w::PublicVerdict::Changes,
                            },
                        },
                    )
                    .ok()?,
                )
                .ok()?;
        }
        let ci = match if row.terminal() { row.ci.as_ref() } else { self.panel_ci.get(&row.task) } {
            Some(ci) if Some(ci.head) == row.change.last_head => ci.status,
            Some(_) | None => c::Status::Unknown,
        };
        let in_landing = match self.newest_landing(row.task) {
            Some(landing) => self.public_landing(ask, landing),
            None => false,
        };
        w::PublicChange::new(
            l,
            w::PublicChangeParts {
                task: names.name(row.task)?,
                head: row.change.last_head,
                step: step.kind,
                repair: step.repair,
                merge: step.merge,
                tiers: step.tiers,
                hold,
                ci: match ci {
                    c::Status::Unknown => w::PublicCi::Unknown,
                    c::Status::Pending => w::PublicCi::Pending,
                    c::Status::Passed => w::PublicCi::Passed,
                    c::Status::Failed => w::PublicCi::Failed,
                },
                gates,
                repairs: used(row.change.repairs, limits.change_policy.repairs)?,
                resolutions: used(row.change.resolutions, limits.change_policy.resolutions)?,
                updates: used(row.change.updates, limits.change_policy.updates)?,
                way: match row.change.way {
                    c::Way::Queue => w::PublicWay::Queue,
                    c::Way::Turn => w::PublicWay::Turn,
                },
                in_landing,
            },
        )
        .ok()
    }
    fn public_step(&self, row: &ChangeRow, state: &c::State) -> Option<Step> {
        let mut repair = None;
        let mut merge = None;
        let mut tiers = 0;
        let step = match state {
            c::State::NoChange { .. } | c::State::Ended => w::PublicStep::Ended,
            c::State::Producing { .. } | c::State::WaitingProducer { .. } => w::PublicStep::Producing,
            c::State::Opening { .. } => w::PublicStep::Opening,
            c::State::Recreating { .. } => w::PublicStep::Recreating,
            c::State::Reopening { .. } => w::PublicStep::Reopening,
            c::State::Checking { .. } => w::PublicStep::Checking,
            c::State::Judging { .. } | c::State::Gating { .. } | c::State::CancellingGates { .. } => w::PublicStep::Gating,
            c::State::Queued { .. } => w::PublicStep::Queued,
            c::State::First { .. } => w::PublicStep::First,
            c::State::Ready { .. } => w::PublicStep::Ready,
            c::State::Preparing { .. } => w::PublicStep::Preparing,
            c::State::Updating { .. } => w::PublicStep::Updating,
            c::State::Resolving { .. } => w::PublicStep::Resolving,
            c::State::Repairing { why, .. } => {
                repair = Some(match why {
                    c::Repair::Ci => w::PublicRepair::Ci,
                    c::Repair::Review { .. } => w::PublicRepair::Review,
                    c::Repair::Semantic => w::PublicRepair::Semantic,
                    c::Repair::Gate(gate) => w::PublicRepair::Gate(
                        w::PublicGateNumber::new(&w::CEILINGS, w::PublicGateNumberParts { gate: *gate }).ok()?,
                    ),
                    c::Repair::Landing { gate } => w::PublicRepair::Landing(
                        w::PublicGateNumber::new(&w::CEILINGS, w::PublicGateNumberParts { gate: u64::from(*gate) })
                            .ok()?,
                    ),
                });
                w::PublicStep::Repairing
            }
            c::State::Landing { .. } => w::PublicStep::Landing,
            c::State::Landed { merge: hash, .. } => {
                merge = Some(*hash);
                match self.verifications.get(&(row.repository, *hash)) {
                    Some(v) => {
                        tiers = u32::try_from(v.tiers.len()).ok()?;
                        w::PublicStep::Verifying
                    }
                    None => w::PublicStep::Landed,
                }
            }
            c::State::Red { merge: hash, .. } => {
                merge = Some(*hash);
                if let Some(v) = self.verifications.get(&(row.repository, *hash)) {
                    tiers = u32::try_from(v.tiers.len()).ok()?;
                }
                w::PublicStep::Red
            }
            c::State::Cleared { merge: hash } | c::State::Uncleared { merge: hash } => {
                merge = Some(*hash);
                if let Some(v) = self.verifications.get(&(row.repository, *hash)) {
                    tiers = u32::try_from(v.tiers.len()).ok()?;
                }
                w::PublicStep::Cleared
            }
            c::State::Held { .. } => unreachable!("hold unwrapped"),
        };
        Some(Step { kind: step, repair, merge, tiers })
    }
}
fn used(used: u32, limit: u32) -> Option<w::PublicUsed> {
    w::PublicUsed::new(&w::CEILINGS, w::PublicUsedParts { used, limit }).ok()
}
fn hold_kind(why: c::Hold) -> w::PublicHold {
    match why {
        c::Hold::Stalled => w::PublicHold::Stalled,
        c::Hold::Repairs => w::PublicHold::Repairs,
        c::Hold::Resolutions => w::PublicHold::Resolutions,
        c::Hold::Updates => w::PublicHold::Updates,
        c::Hold::BranchMoved => w::PublicHold::Moved,
        c::Hold::BranchMissing => w::PublicHold::Missing,
        c::Hold::PullClosed => w::PublicHold::Closed,
        c::Hold::Retargeted => w::PublicHold::Retargeted,
        c::Hold::Failed => w::PublicHold::Failed,
        c::Hold::Rejected => w::PublicHold::Rejected,
        c::Hold::Authority(_) => w::PublicHold::Authority,
        c::Hold::Questions => w::PublicHold::Questions,
    }
}
fn worse(a: c::Status, b: c::Status) -> c::Status {
    use c::Status::{Failed, Passed, Pending, Unknown};
    match (a, b) {
        (Failed, _) | (_, Failed) => Failed,
        (Pending, _) | (_, Pending) => Pending,
        (Passed, _) | (_, Passed) => Passed,
        (Unknown, Unknown) => Unknown,
    }
}

//! The waits of a closer (temper/docs/design/domain/forge/waits.md, 6).
use crate::{Domain, Limits, closer::Wait};
use jig_domain_core_vocabulary::waits::{AtBound, Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject};
use skein_lib::{Env, List};
use temper_client_protocol_wire::waits::{CLOSER_LANDING, CLOSER_MAKING, CLOSER_REPORTING, THING_PULL};

pub(super) fn waits(d: &Domain, _env: &Env<Limits>, task: u64) -> List<Reported> {
    let mut waits = List::with_capacity(1);
    if let Some(progress) = d.closing.progress.get(&task)
        && let Some(wait) = &progress.wait
    {
        let mut things = List::with_capacity(1);
        let (step, bound) = match wait {
            Wait::Landing { .. } => (CLOSER_LANDING, ReportedBound::Figure(AtBound::Hold)),
            Wait::Reporting(read, _) => {
                if let crate::closer::ReportRead::Pull(number) = read {
                    things.push(Thing { what: THING_PULL, number: *number, owner: None }).expect("one pull read");
                }
                (CLOSER_REPORTING, ReportedBound::Figure(AtBound::Hold))
            }
            Wait::Making => (CLOSER_MAKING, ReportedBound::Figure(AtBound::Hold)),
            Wait::OperatorHold => (CLOSER_LANDING, ReportedBound::None(Unbounded::OperatorHold)),
        };
        waits
            .push(Reported {
                kind: WaitKind::Step,
                object: WaitObject::Procedure { step, things },
                since: progress.since,
                bound,
            })
            .expect("one closer step");
    }
    waits
}

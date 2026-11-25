//! The waits of a landing (temper/docs/design/domain/forge/waits.md, 5).
use crate::{
    Domain, LandingRow, Limits,
    landing::{read_wait, round_effect_top},
    room,
};
use jig_domain_core_vocabulary::waits::{AtBound, Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject};
use skein_lib::{Env, List, Wall};
use temper_client_protocol_wire::waits::{
    LANDING_ANCESTRY, LANDING_CLOSE, LANDING_DELETE, LANDING_JUDGEMENT, LANDING_MERGE, LANDING_OPEN, LANDING_POST,
    LANDING_PREFIXES, LANDING_READ, LANDING_READY, LANDING_RECEIPT, LANDING_RECORD, LANDING_ROOM, LANDING_RUN,
    THING_GATE, THING_PULL, THING_RUN, THING_TASK,
};
use temper_engine_domain_forge_landing::{Awaited, Phase};

pub(super) fn waits(d: &Domain, env: &Env<Limits>, task: u64) -> List<Reported> {
    let mut waits = List::with_capacity(1);
    if let Some(wait) = waited(d, env, task) {
        waits.push(wait).expect("one landing wait");
    }
    waits
}

fn waited(d: &Domain, env: &Env<Limits>, task: u64) -> Option<Reported> {
    let row = d.landing.rows.get(&task)?;
    if d.landing_held() {
        return None;
    }
    match row.landing.phase {
        Phase::Held { .. } | Phase::Landed | Phase::Cancelled => return None,
        Phase::Waiting | Phase::Preparing | Phase::Proving | Phase::Proved { .. } | Phase::Turn { .. } => {}
    }
    if row.room.is_some() {
        return room::waited(d, env, task, LANDING_ROOM);
    }
    let on = row.waiting?;
    let (since, _) = d.landing_advance(task)?;
    let (step, member) = match on {
        Awaited::Release | Awaited::Prepare => return None,
        Awaited::Turn => {
            let holder = d.turn_holder(row.landing.project)?;
            if holder == task {
                return None;
            }
            return Some(Reported {
                kind: WaitKind::Turn,
                object: WaitObject::Task(holder),
                since,
                bound: behind(holder),
            });
        }
        Awaited::Ready { member } | Awaited::Judgement { member } => {
            let change = row.landing.members.get(usize::from(member))?.change;
            let step = if on == (Awaited::Ready { member }) { LANDING_READY } else { LANDING_JUDGEMENT };
            return Some(procedure(step, &[thing(THING_TASK, change)], since, behind(change)));
        }
        Awaited::Receipt { member, gate } => {
            if let Some(flight) = read_wait::gate_flight(d, row, member, gate)
                && flight.task != task
            {
                return Some(procedure(LANDING_RECEIPT, &[thing(THING_TASK, flight.task)], since, behind(flight.task)));
            }
            (LANDING_RECEIPT, Some(member))
        }
        Awaited::Run { member, gate } => {
            if let Some(flight) = read_wait::gate_flight(d, row, member, gate)
                && flight.task == task
                && let Some(at) = flight
                    .gate_deadline(d.landing_gate_timeout(row.landing.project), env.limits.change_policy.runner_wait)
            {
                let gate = thing(THING_GATE, u64::from(gate));
                let bound = ReportedBound::At { at, then: AtBound::Hold };
                return Some(match &flight.run {
                    Some(run) if run.number != 0 => {
                        procedure(LANDING_RUN, &[gate, thing(THING_RUN, run.number)], since, bound)
                    }
                    Some(_) | None => procedure(LANDING_RUN, &[gate], since, bound),
                });
            }
            (LANDING_RUN, Some(member))
        }
        Awaited::Prefixes => (LANDING_PREFIXES, None),
        Awaited::Open => (LANDING_OPEN, None),
        Awaited::Close => (LANDING_CLOSE, None),
        Awaited::Delete { member } => (LANDING_DELETE, Some(member)),
        Awaited::Post { member, .. } => (LANDING_POST, Some(member)),
        Awaited::Merge => (LANDING_MERGE, None),
        Awaited::Record { member } => (LANDING_RECORD, Some(member)),
        Awaited::Ancestry { member } => (LANDING_ANCESTRY, Some(member)),
        Awaited::Read => (LANDING_READ, None),
    };
    let bound = match read_wait::reported_deadline(d, task) {
        Some(at) => ReportedBound::At { at, then: AtBound::Hold },
        None => ReportedBound::None(Unbounded::NoDeadline),
    };
    Some(match forge_thing(d, row, step, member) {
        Some(object) => procedure(step, &[object], since, bound),
        None => procedure(step, &[], since, bound),
    })
}

fn forge_thing(d: &Domain, row: &LandingRow, step: u32, member: Option<u16>) -> Option<Thing> {
    if let Some(member) = member {
        return Some(thing(THING_TASK, row.landing.members.get(usize::from(member))?.change));
    }
    if step == LANDING_READ
        && let Some(change) = read_wait::read_change(d, row.landing.task)
    {
        return Some(thing(THING_TASK, change));
    }
    if let Some(pull) = row.landing.round.pull {
        return Some(thing(THING_PULL, pull));
    }
    Some(thing(THING_TASK, row.landing.members.get(usize::from(round_effect_top(row)))?.change))
}

const fn behind(task: u64) -> ReportedBound {
    ReportedBound::None(Unbounded::Behind(task))
}

const fn thing(what: u32, number: u64) -> Thing {
    Thing { what, number, owner: None }
}

fn procedure(step: u32, things: &[Thing], since: Wall, bound: ReportedBound) -> Reported {
    let things = List::from_boxed(things.into());
    Reported { kind: WaitKind::Step, object: WaitObject::Procedure { step, things }, since, bound }
}

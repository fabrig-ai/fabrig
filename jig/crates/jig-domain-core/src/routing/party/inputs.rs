//! A party's retained inputs, loaded by the root and judged by the core.
use crate::{
    Core, Limits, Now, Request,
    routing::{
        CreationKind,
        party::{dispatch::Routed, holdings::begin_creation},
        refusals::{pool_again, pool_refused},
    },
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};
#[derive(Debug)]
pub(crate) struct Inputs {
    pub person: u64,
    pub project: u32,
    pub role: people::Role,
    pub members: Box<[tasks::New]>,
    pub given: Option<people::Given>,
    pub proposal: Option<(Box<[u8]>, u32)>,
    pub ids: Box<[u64]>,
    pub at: usize,
    pub row: Option<Box<tasks::TaskRecord>>,
    pub stubs: List<tasks::Stub>,
}
pub(crate) fn begin(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, mut flight: Inputs, request: Token) {
    let capacity = env.limits.people.batch.checked_mul(env.limits.tasks.inputs).expect("bounded party inputs");
    let mut ids = List::with_capacity(capacity);
    for member in &flight.members {
        for &task in &member.spec.inputs {
            if !ids.as_slice().contains(&task) {
                ids.push(task).expect("bounded member inputs");
            }
        }
    }
    if ids.is_empty() {
        begin_creation(core, env, routed, request, kind(&mut flight, Box::new([])), flight.members);
        return;
    }
    flight.ids = ids.into_boxed();
    flight.stubs = List::with_capacity(capacity);
    if core.creating_inputs_flights() == core.flight_people_capacity {
        pool_again(&mut routed.work, request, people::Again::Busy);
        return;
    }
    retain(core, &mut routed.out, request, flight);
}
fn retain(core: &mut Core, out: &mut Queue<Request>, request: Token, flight: Inputs) {
    let tasks: Box<[u64]> = match &flight.row {
        Some(row) => Box::new([row.root]),
        None => flight.ids.get(flight.at..).expect("remaining inputs").into(),
    };
    let person = flight.person;
    let project = flight.project;
    let request = core.fly(crate::routing::flight::Flight::CreatingInputs { request, route: flight });
    out.push(Request::Now(Box::new(Now::PartyInputs { request, person, project, tasks })));
}
pub(crate) struct Loaded {
    pub request: Token,
    pub row: Option<Box<tasks::TaskRecord>>,
    pub busy: bool,
}
pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, loaded: Loaded) {
    let Loaded { request, row, busy } = loaded;
    let Some((request, mut flight)) = core.take_creating_inputs(request) else { return };
    if busy {
        pool_again(&mut routed.work, request, people::Again::Busy);
        return;
    }
    let Some(row) = row else {
        pool_refused(&mut routed.work, request, people::Refusal::Unknown);
        return;
    };
    let wanted = *flight.ids.get(flight.at).expect("pending input");
    let input = if let Some(input) = flight.row.take() {
        if row.number != input.root
            || row.project != flight.project
            || row.requester != tasks::Party::Person(flight.person)
        {
            pool_refused(&mut routed.work, request, people::Refusal::Unknown);
            return;
        }
        input
    } else {
        let ended = match row.phase {
            tasks::Phase::Ended(_) => true,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                false
            }
        };
        if row.number != wanted || row.project != flight.project || !ended {
            pool_refused(&mut routed.work, request, people::Refusal::Unknown);
            return;
        }
        if !core.reaches_project(flight.role, flight.project)
            && row.requester != tasks::Party::Person(flight.person)
            && !Core::person_tree(
                flight.person,
                row.root,
                env.limits.tasks.depth,
                &tasks::CompoundAdmission::new(&core.tasks, 0),
            )
        {
            if row.root == wanted || core.tasks.task(row.root).is_some() {
                pool_refused(&mut routed.work, request, people::Refusal::Unknown);
                return;
            }
            flight.row = Some(row);
            retain(core, &mut routed.out, request, flight);
            return;
        }
        row
    };
    let phase = match input.phase {
        tasks::Phase::Ended(tasks::Ending::Done(_)) => tasks::Status::Done,
        tasks::Phase::Ended(tasks::Ending::Failed { .. }) => tasks::Status::Failed,
        tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => tasks::Status::Cancelled,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            unreachable!("retained input checked before its root")
        }
    };
    flight.stubs.push(tasks::Stub { task: wanted, phase, result: Token::new(wanted) }).expect("one stub per input");
    flight.at = flight.at.checked_add(1).expect("bounded input position");
    if flight.at < flight.ids.len() {
        retain(core, &mut routed.out, request, flight);
    } else {
        let stubs = core::mem::replace(&mut flight.stubs, List::with_capacity(0)).into_boxed();
        begin_creation(core, env, routed, request, kind(&mut flight, stubs), flight.members);
    }
}

fn kind(flight: &mut Inputs, stubs: Box<[tasks::Stub]>) -> CreationKind {
    match flight.proposal.take() {
        Some((reason, to)) => CreationKind::Propose { person: flight.person, reason, to, stubs },
        None => CreationKind::Batch { person: flight.person, given: flight.given, stubs },
    }
}

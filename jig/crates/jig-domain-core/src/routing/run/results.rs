//! Ended rows gathered for preparation or amendment admission
//! (`domain/engine.md`, sections 5.3 and 9).

use crate::routing::{
    Work,
    party::amendment_reads::{self, AmendmentRead, Replay},
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};

use crate::{Core, Event, Held, Limits, Now, Request, RetainedResult};

#[derive(Debug)]
pub(crate) struct ResultRead {
    pub why: ReadFor,
    pub ids: Box<[u64]>,
    pub at: usize,
    pub results: List<RetainedResult>,
}

#[derive(Debug)]
pub(crate) enum ReadFor {
    DeploymentClose { request: Token, project: u32 },
    Preparation { task: u64, last_message: u64 },
    Amendment(Box<AmendmentRead>),
}

pub(crate) fn clear(core: &mut Core, task: u64) {
    // Several calls can wait for one run; remove all of its reads.
    for _ in 0..core.flight_tasks_capacity {
        let mut request = None;
        for (id, flight) in &core.flights {
            let crate::routing::flight::Flight::ResultRead(read) = flight else { continue };
            let token = id.token();
            let owner = match &read.why {
                ReadFor::DeploymentClose { .. } => None,
                ReadFor::Preparation { task, .. } => Some(*task),
                ReadFor::Amendment(read) => match &read.replay {
                    Replay::Agent { key, .. } => Some(key.task),
                    Replay::Person { .. } => None,
                },
            };
            if owner == Some(task) {
                request = Some(token);
                break;
            }
        }
        let Some(request) = request else { break };
        if let Some(read) = core.take_result_read(request) {
            match read.why {
                ReadFor::DeploymentClose { .. } | ReadFor::Preparation { .. } => {}
                ReadFor::Amendment(amendment) => match amendment.replay {
                    Replay::Agent { key, .. } => {
                        let _pending = core.pending_calls.remove(&key);
                    }
                    Replay::Person { .. } => {}
                },
            }
        }
    }
}

pub(crate) fn begin(core: &mut Core, work: &mut Queue<Work>, task: u64, ids: Box<[u64]>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    for (_, flight) in &core.flights {
        let crate::routing::flight::Flight::ResultRead(read) = flight else { continue };
        match read.why {
            ReadFor::Preparation { task: owner, .. } if owner == task => {
                return out;
            }
            ReadFor::DeploymentClose { .. } | ReadFor::Preparation { .. } | ReadFor::Amendment(_) => {}
        }
    }
    if core.result_read_flights() == core.flight_tasks_capacity {
        work.push(Work::Event(Event::Tasks(tasks::Event::PreparationFailed { task })));
    } else {
        let read = ResultRead {
            why: ReadFor::Preparation {
                task,
                last_message: core.contexts.get(&task).expect("preparing context").last_message,
            },
            results: List::with_capacity(u32::try_from(ids.len()).expect("bounded dependency results")),
            ids,
            at: 0,
        };
        retain(core, &mut out, read);
    }
    out
}

// Held, so that it sees the commit that ended the dependency (`domain/engine.md`, 5.3).
fn retain(core: &mut Core, out: &mut Queue<Request>, read: ResultRead) {
    let task = *read.ids.get(read.at).expect("pending result ID");
    let request = core.fly(crate::routing::flight::Flight::ResultRead(read));
    out.push(Request::Held(Box::new(Held::Load(Box::new(Now::EndedTask { request, task })))));
}

pub(crate) fn loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    request: Token,
    row: Option<Box<tasks::TaskRecord>>,
    busy: bool,
) -> Queue<Request> {
    let mut out = Queue::uncapped();

    let Some(mut read) = core.take_result_read(request) else { return out };
    match read.why {
        ReadFor::DeploymentClose { request, project } => {
            let outcome = closed_outcome(project, *read.ids.first().expect("one close subject"), row, busy);
            work.push(Work::Event(Event::People(jig_domain_people::Event::Decided { request, outcome })));
        }
        ReadFor::Amendment(mut amendment) => {
            if !amendment_reads::current(core, &amendment.replay) {
                discard_party(core, amendment.replay);
                return out;
            }
            if busy {
                amendment_reads::busy(core, work, &mut out, amendment.replay);
                return out;
            }
            let wanted = *read.ids.first().expect("one pending amendment place");
            amendment.evidence.loaded.push(wanted).expect("bounded loaded places");
            if let Some(row) = row {
                let status = match row.phase {
                    tasks::Phase::Ended(tasks::Ending::Done(_)) => Some(tasks::Status::Done),
                    tasks::Phase::Ended(tasks::Ending::Failed { .. }) => Some(tasks::Status::Failed),
                    tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => Some(tasks::Status::Cancelled),
                    tasks::Phase::Waiting
                    | tasks::Phase::Active(_)
                    | tasks::Phase::Closing(_)
                    | tasks::Phase::Held { .. } => None,
                };
                if row.number == wanted
                    && let Some(status) = status
                {
                    amendment
                        .evidence
                        .places
                        .push(tasks::DependencyPlace {
                            number: row.number,
                            project: row.project,
                            requester: row.requester,
                            root: row.root,
                            depth: row.depth,
                            status,
                        })
                        .expect("bounded loaded places");
                }
            }
            return amendment_reads::resume(core, env, work, *amendment);
        }
        ReadFor::Preparation { task, last_message } => {
            let current = match core.contexts.get(&task) {
                Some(context) if context.last_message == last_message => match core.tasks.task(task) {
                    Some(row) => {
                        row.phase == tasks::Phase::Active(tasks::Active::Preparing)
                            && core.tasks.run_message(task) == Some(last_message)
                    }
                    None => false,
                },
                Some(_) | None => false,
            };
            if !current {
                return out;
            }
            let result = if busy {
                None
            } else {
                match row {
                    Some(row) => core.dependency_result(task, *read.ids.get(read.at).expect("pending result ID"), row),
                    None => None,
                }
            };
            let Some(result) = result else {
                work.push(Work::Event(Event::Tasks(tasks::Event::PreparationFailed { task })));
                return out;
            };
            read.results.push(result).expect("one result per ID");
            read.at = read.at.checked_add(1).expect("bounded result position");
            if read.at < read.ids.len() {
                retain(core, &mut out, read);
            } else {
                assert!(
                    core.dependency_results.insert(task, read.results.into_boxed()).is_ok(),
                    "live task results room"
                );
                work.push(Work::StartBrief { task });
            }
        }
    }
    out
}

/// Holds only transient replay inputs; no call record or task change precedes the load.
pub(crate) fn wait(
    core: &mut Core,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    next: u64,
    amendment: AmendmentRead,
) {
    if core.result_read_flights() == core.flight_tasks_capacity {
        amendment_reads::busy(core, work, out, amendment.replay);
        return;
    }
    match &amendment.replay {
        Replay::Agent { key, .. } => {
            assert!(core.pending_calls.insert(*key, true).is_ok(), "waiting amendment call room");
        }
        Replay::Person { .. } => {}
    }
    let read = ResultRead {
        why: ReadFor::Amendment(Box::new(amendment)),
        ids: Box::new([next]),
        at: 0,
        results: List::with_capacity(0),
    };
    retain(core, out, read);
}

pub(crate) fn clear_person(core: &mut Core, person: u64) {
    for _ in 0..core.flight_tasks_capacity {
        let mut found = None;
        for (id, flight) in &core.flights {
            let crate::routing::flight::Flight::ResultRead(read) = flight else { continue };
            let request = id.token();
            match &read.why {
                ReadFor::Amendment(read) => match &read.replay {
                    Replay::Person { person: owner, .. } if *owner == person => {
                        found = Some(request);
                        break;
                    }
                    Replay::Person { .. } | Replay::Agent { .. } => {}
                },
                ReadFor::DeploymentClose { .. } | ReadFor::Preparation { .. } => {}
            }
        }
        let Some(request) = found else { break };
        if let Some(read) = core.take_result_read(request) {
            match read.why {
                ReadFor::Amendment(read) => discard_party(core, read.replay),
                ReadFor::DeploymentClose { .. } | ReadFor::Preparation { .. } => {}
            }
        }
    }
}

/// A close of an ended chat uses the existing durable ended-task point load.
pub(crate) fn closed_subject(
    core: &mut Core,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    request: Token,
    project: u32,
    task: u64,
) {
    if core.result_read_flights() == core.flight_tasks_capacity {
        work.push(Work::Event(Event::People(jig_domain_people::Event::Decided {
            request,
            outcome: jig_domain_people::Outcome::Again(jig_domain_people::Again::Busy),
        })));
        return;
    }
    let read = ResultRead {
        why: ReadFor::DeploymentClose { request, project },
        ids: Box::new([task]),
        at: 0,
        results: List::with_capacity(0),
    };
    retain(core, out, read);
}

fn closed_outcome(
    project: u32,
    wanted: u64,
    row: Option<Box<tasks::TaskRecord>>,
    busy: bool,
) -> jig_domain_people::Outcome {
    if busy {
        jig_domain_people::Outcome::Again(jig_domain_people::Again::Busy)
    } else {
        let why = match row {
            Some(row) if row.project == project && row.number == wanted => match &row.phase {
                tasks::Phase::Ended(ending) => crate::routing::refusals::ended(ending),
                tasks::Phase::Closing(closing) => crate::routing::refusals::ended(&closing.ending),
                tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {
                    jig_domain_people::Refusal::Unknown
                }
            },
            Some(_) | None => jig_domain_people::Refusal::Unknown,
        };
        jig_domain_people::Outcome::Refused(why)
    }
}

fn discard_party(core: &mut Core, replay: Replay) {
    match replay {
        Replay::Person { request, .. } => {
            drop(core.land_kind(request, crate::routing::flight::Kind::Party));
        }
        Replay::Agent { .. } => {}
    }
}

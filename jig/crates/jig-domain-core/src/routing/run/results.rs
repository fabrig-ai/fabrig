//! Ended rows gathered for preparation or amendment admission
//! (`domain/engine/decisions.md`, section 1.3, and `domain/engine/briefs.md`, section 1).

use crate::routing::{
    Work,
    party::{
        amendment_reads::{self, AmendmentRead, Replay},
        batch_reads::{self, BatchRead},
    },
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};

use crate::{Core, Event, Held, Limits, Now, Request, RetainedResult};

#[derive(Hash, Debug)]
pub(crate) struct ResultRead {
    pub why: ReadFor,
    pub ids: Box<[u64]>,
    pub at: usize,
    pub results: List<RetainedResult>,
}

#[derive(Hash, Debug)]
pub(crate) enum ReadFor {
    Delegate {
        party: crate::routing::party::dispatch::Party,
        task: u64,
        ended: Option<(u64, jig_domain_people::Refusal)>,
    },
    Conclude {
        party: crate::routing::party::dispatch::Party,
        task: u64,
    },
    DeploymentClose {
        request: Token,
        project: u32,
    },
    Preparation {
        task: u64,
        last_message: u64,
    },
    Amendment(Box<AmendmentRead>),
    Batch(Box<BatchRead>),
}

pub(crate) fn clear(core: &mut Core, task: u64) {
    super::preferences::clear(core, task);
    super::now::clear(core, task);
    // Several calls can wait for one run; remove all of its reads.
    for _ in 0..core.flight_tasks_capacity {
        let mut request = None;
        for (id, flight) in &core.flights {
            let crate::routing::flight::Flight::ResultRead(read) = flight else { continue };
            let token = id.token();
            let owner = match &read.why {
                ReadFor::Delegate { .. }
                | ReadFor::Conclude { .. }
                | ReadFor::DeploymentClose { .. }
                | ReadFor::Batch(_) => None,
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
                ReadFor::Delegate { .. }
                | ReadFor::Conclude { .. }
                | ReadFor::DeploymentClose { .. }
                | ReadFor::Preparation { .. }
                | ReadFor::Batch(_) => {}
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
            ReadFor::Delegate { .. }
            | ReadFor::Conclude { .. }
            | ReadFor::DeploymentClose { .. }
            | ReadFor::Preparation { .. }
            | ReadFor::Amendment(_)
            | ReadFor::Batch(_) => {}
        }
    }
    if core.result_read_flights() == core.flight_tasks_capacity {
        work.push(Work::PreparationFailed {
            task,
            failure: jig_domain_brief::GatherFailure {
                fate: jig_domain_brief::GatherFate::Capacity,
                section: b"results",
                words: Box::from(b"no read flight is free".as_slice()),
            },
        });
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

// Held, so that it sees the commit that ended the dependency (`domain/engine/decisions.md`, 1.3).
fn retain(core: &mut Core, out: &mut Queue<Request>, read: ResultRead) {
    let task = *read.ids.get(read.at).expect("pending result ID");
    let request = core.fly(crate::routing::flight::Flight::ResultRead(read));
    out.push(Request::Held(Box::new(Held::Load(Box::new(Now::EndedTask { request, task })))));
}

#[expect(clippy::too_many_lines, reason = "each retained read resumes its owning route")]
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
        ReadFor::Delegate { party, task, ended } => {
            delegate_loaded(core, env, work, &mut out, party, task, ended, row, busy);
        }
        ReadFor::Conclude { party, task } => {
            let outcome = concluded_outcome(core, party, task, row, busy);
            work.push(Work::Event(Event::People(jig_domain_people::Event::Decided {
                request: party.request,
                outcome,
            })));
        }
        ReadFor::DeploymentClose { request, project } => {
            let outcome = closed_outcome(project, *read.ids.first().expect("one close subject"), row, busy);
            work.push(Work::Event(Event::People(jig_domain_people::Event::Decided { request, outcome })));
        }
        ReadFor::Batch(batch) => {
            let wanted = *read.ids.first().expect("one pending batch place");
            batch_reads::loaded(core, work, *batch, wanted, row, busy);
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
                Some(context) if context.last_message == last_message => {
                    super::parts::preparation_current(core, context)
                }
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
                work.push(Work::PreparationFailed {
                    task,
                    failure: jig_domain_brief::GatherFailure {
                        fate: jig_domain_brief::GatherFate::Transient,
                        section: b"results",
                        words: Box::from(b"a dependency's result could not be read".as_slice()),
                    },
                });
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

/// Uses the same held point read as amendments; no new stored or wire form.
pub(crate) fn wait_batch(
    core: &mut Core,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    next: u64,
    batch: BatchRead,
) {
    if core.result_read_flights() == core.flight_tasks_capacity {
        batch_reads::terminal(
            core,
            work,
            batch.request,
            jig_domain_people::Outcome::Again(jig_domain_people::Again::Busy),
        );
        return;
    }
    retain(
        core,
        out,
        ResultRead {
            why: ReadFor::Batch(Box::new(batch)),
            ids: Box::new([next]),
            at: 0,
            results: List::with_capacity(0),
        },
    );
}

pub(crate) fn clear_person(core: &mut Core, person: u64) {
    for _ in 0..core.flight_tasks_capacity {
        let mut found = None;
        for (id, flight) in &core.flights {
            let crate::routing::flight::Flight::ResultRead(read) = flight else { continue };
            let request = id.token();
            match &read.why {
                ReadFor::Batch(read) => {
                    if let Some(admission) = batch_reads::admission(core, read.request)
                        && admission.person == person
                    {
                        found = Some(request);
                        break;
                    }
                }
                ReadFor::Amendment(read) => match &read.replay {
                    Replay::Person { person: owner, .. } if *owner == person => {
                        found = Some(request);
                        break;
                    }
                    Replay::Person { .. } | Replay::Agent { .. } => {}
                },
                ReadFor::Delegate { .. }
                | ReadFor::Conclude { .. }
                | ReadFor::DeploymentClose { .. }
                | ReadFor::Preparation { .. } => {}
            }
        }
        let Some(request) = found else { break };
        if let Some(read) = core.take_result_read(request) {
            match read.why {
                ReadFor::Amendment(read) => discard_party(core, read.replay),
                ReadFor::Batch(read) => batch_reads::discard(core, read.request),
                ReadFor::Delegate { .. }
                | ReadFor::Conclude { .. }
                | ReadFor::DeploymentClose { .. }
                | ReadFor::Preparation { .. } => {}
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

/// Load an ended gathering through the ordinary retained-result read (gatherings.md, 3).
pub(crate) fn conclude(
    core: &mut Core,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    party: crate::routing::party::dispatch::Party,
    task: u64,
) {
    if core.result_read_flights() == core.flight_tasks_capacity {
        crate::routing::refusals::pool_again(work, party.request, jig_domain_people::Again::Busy);
        return;
    }
    retain(
        core,
        out,
        ResultRead {
            why: ReadFor::Conclude { party, task },
            ids: Box::new([task]),
            at: 0,
            results: List::with_capacity(0),
        },
    );
}

fn concluded_outcome(
    core: &Core,
    party: crate::routing::party::dispatch::Party,
    task: u64,
    row: Option<Box<tasks::TaskRecord>>,
    busy: bool,
) -> jig_domain_people::Outcome {
    use jig_domain_people::{Outcome, Refusal, Role};
    if busy {
        return Outcome::Again(jig_domain_people::Again::Busy);
    }
    let Some(row) = row else { return Outcome::Refused(Refusal::Unknown) };
    let allowed = match core.people.role(party.person, party.project) {
        Some(Role::Owner | Role::Maintainer) => true,
        Some(Role::Member | Role::Observer | Role::Policy { .. }) => {
            row.requester == tasks::Party::Person(party.person)
        }
        None => false,
    };
    if !allowed || row.number != task || row.project != party.project {
        return Outcome::Refused(Refusal::Unknown);
    }
    match &row.phase {
        tasks::Phase::Ended(ending) => Outcome::Refused(crate::routing::refusals::ended(ending)),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            Outcome::Refused(Refusal::Unknown)
        }
    }
}

/// An ended Delegate subject is reached on its place and, if needed, its root's
/// (jig/docs/design/domain/tasks/README.md, 4). Stubs never prove reach.
pub(crate) fn delegate_subject(
    core: &mut Core,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    party: crate::routing::party::dispatch::Party,
    task: u64,
) {
    if core.result_read_flights() == core.flight_tasks_capacity {
        crate::routing::refusals::pool_again(work, party.request, jig_domain_people::Again::Busy);
        return;
    }
    retain(
        core,
        out,
        ResultRead {
            why: ReadFor::Delegate { party, task, ended: None },
            ids: Box::new([task]),
            at: 0,
            results: List::with_capacity(0),
        },
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "the retained ended subject and its loaded root carry one reach decision"
)]
fn delegate_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    out: &mut Queue<Request>,
    party: crate::routing::party::dispatch::Party,
    task: u64,
    ended: Option<(u64, jig_domain_people::Refusal)>,
    row: Option<Box<tasks::TaskRecord>>,
    busy: bool,
) {
    use crate::routing::refusals::{pool_again, pool_refused};
    use jig_domain_people::Refusal;
    if busy {
        pool_again(work, party.request, jig_domain_people::Again::Busy);
        return;
    }
    let Some(role) = core.people.role(party.person, party.project) else {
        pool_refused(work, party.request, Refusal::Unknown);
        return;
    };
    let Some(row) = row else {
        pool_refused(work, party.request, Refusal::Unknown);
        return;
    };
    if row.project != party.project {
        pool_refused(work, party.request, Refusal::Unknown);
        return;
    }
    if let Some((root, refusal)) = ended {
        let why = if row.number == root
            && (core.reaches_project(role, party.project) || row.requester == tasks::Party::Person(party.person))
        {
            refusal
        } else {
            Refusal::Unknown
        };
        pool_refused(work, party.request, why);
        return;
    }
    if row.number != task {
        pool_refused(work, party.request, Refusal::Unknown);
        return;
    }
    let refusal = match &row.phase {
        tasks::Phase::Ended(ending) => crate::routing::refusals::ended(ending),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            pool_refused(work, party.request, Refusal::Unknown);
            return;
        }
    };
    if core.reaches_project(role, party.project)
        || row.requester == tasks::Party::Person(party.person)
        || Core::person_tree(
            party.person,
            row.root,
            env.limits.tasks.depth,
            &tasks::CompoundAdmission::new(&core.tasks, 0),
        )
    {
        pool_refused(work, party.request, refusal);
    } else if row.root != task && core.tasks.task(row.root).is_none() {
        retain(
            core,
            out,
            ResultRead {
                why: ReadFor::Delegate { party, task, ended: Some((row.root, refusal)) },
                ids: Box::new([row.root]),
                at: 0,
                results: List::with_capacity(0),
            },
        );
    } else {
        pool_refused(work, party.request, Refusal::Unknown);
    }
}

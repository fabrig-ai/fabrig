//! Party task controls, amendments, messages, answers and moves routed to tasks
//! under current authority (`domain/people.md`, section 5.1).

use crate::{
    Core, Event, Limits, Now, PersonTaskRoute, Request,
    routing::{
        Work,
        party::dispatch::{Amendment, Control, Party, Routed},
        refusals::pool_refused,
    },
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, ReplyTo};

pub(crate) fn amend_route(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    edit: Amendment,
    evidence: super::amendment_reads::Evidence,
) {
    let Amendment { task, revision, change: amendment, release } = edit;
    let Party { request, person, project, .. } = party;

    if let Some(next) = super::amendment_reads::missing(
        core,
        &env.limits.tasks,
        tasks::Party::Person(person),
        task,
        amendment.dependencies.as_deref(),
        &evidence,
    ) {
        crate::routing::run::results::wait(
            core,
            &mut routed.work,
            &mut routed.out,
            next,
            super::amendment_reads::AmendmentRead {
                replay: super::amendment_reads::Replay::Person {
                    request,
                    person,
                    project,
                    task,
                    revision,
                    amendment,
                    release,
                },
                evidence,
            },
        );
        return;
    }
    let Some(mut amendment) =
        crate::admission::amendments::translate(amendment, core.kept_models(task), &env.limits.tasks)
    else {
        pool_refused(&mut routed.work, request, people::Refusal::Limit { which: people::LimitKind::Words });
        return;
    };
    let stop_run = match crate::acts::bounds::amendment_admit_with_places(
        core,
        party.task(task),
        &amendment,
        env.limits.tasks.depth,
        &env.limits.tasks,
        evidence.places.as_slice(),
    ) {
        Ok(admitted) => admitted,
        Err(why) => {
            pool_refused(&mut routed.work, request, why);
            return;
        }
    };
    if let Err(why) = core.party_amend_revision(task, revision, &amendment) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    if let Err(why) = amend_release_admit(core, person, task, release) {
        pool_refused(&mut routed.work, request, why);
        return;
    }
    amendment.release = release;
    let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
        pool_refused(&mut routed.work, request, people::Refusal::Limit { which: people::LimitKind::Arithmetic });
        return;
    };
    let released = match core.tasks.task(task).expect("amend admission found task").phase {
        tasks::Phase::Held { .. } => release.is_some(),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    };
    let request = core.fly_person_task(request, PersonTaskRoute::Amended { task, released });
    routed.work.push(Work::Event(Event::Tasks(tasks::Event::Amend {
        places: evidence.places.into_boxed(),
        funder: None,
        reply_to: ReplyTo::new(request),
        by: tasks::Party::Person(person),
        task,
        message,
        stop_run,
        amendment,
    })));
}

pub(crate) fn person_task_request(
    core: &mut Core,
    routed: &mut Routed,
    party: Party,
    action: crate::admission::person_task::Action,
) {
    let Party { request, person, project, .. } = party;

    let task = action.task();
    if core.tasks.task(task).is_none() {
        routed.out.push(Request::Now(Box::new(Now::EndedPersonTask { request, person, project, task })));
        return;
    }
    match core.person_task(party, action) {
        Ok(event) => routed.work.push(Work::Event(Event::Tasks(event))),
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn stop_request(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, party: Party, task: u64) {
    let Party { request, person, project, .. } = party;

    match core.control_admit(
        party.task(task),
        Control::Stop,
        &env.limits.tasks,
        &tasks::CompoundAdmission::new(&core.tasks, 0),
    ) {
        Ok(task) => {
            let Some(number) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                routed.work.push(Work::Event(Event::People(people::Event::Decided {
                    request,
                    outcome: people::Outcome::Refused(people::Refusal::Limit { which: people::LimitKind::Arithmetic }),
                })));
                return;
            };
            let request = core.fly_person_task(request, PersonTaskRoute::Stop(task));
            routed.work.push(Work::Event(Event::Tasks(tasks::Event::Place {
                reply_to: ReplyTo::new(request),
                hold: tasks::PlacedHold {
                    number,
                    project,
                    owner: tasks::Owner::Party { person },
                    mode: tasks::Mode::Stop,
                    scope: tasks::Scope::Subtree { task },
                    reason: Box::new([]),
                    placed_at: env.wall.as_nanos() / 1_000_000,
                    revision: 1,
                    exempt: Box::new([]),
                    lifted: false,
                },
            })));
        }
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn cancel_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    task: u64,
    reason: Box<[u8]>,
    grace: Option<skein_lib::Duration>,
) {
    let Party { request, person, .. } = party;

    let admitted = core.control_admit(
        party.task(task),
        Control::Cancel { reason_len: reason.len(), grace },
        &env.limits.tasks,
        &tasks::CompoundAdmission::new(&core.tasks, 0),
    );
    match admitted {
        Ok(task) => {
            let message = if grace.is_some() {
                let Some(message) = crate::fresh(&mut core.counters, crate::Family::Message) else {
                    pool_refused(
                        &mut routed.work,
                        request,
                        people::Refusal::Limit { which: people::LimitKind::Arithmetic },
                    );
                    return;
                };
                Some(message)
            } else {
                None
            };
            let request = core.fly_person_task(request, PersonTaskRoute::Cancel(task));
            routed.work.push(Work::Event(Event::Tasks(tasks::Event::Control {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                control: tasks::Control::Cancel { reason, grace, message },
            })));
        }
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn release_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    task: u64,
    revision: u64,
    again: Option<bool>,
) {
    let Party { request, person, .. } = party;

    match core.control_admit(
        party.task(task),
        Control::Release { again },
        &env.limits.tasks,
        &tasks::CompoundAdmission::new(&core.tasks, 0),
    ) {
        Ok(task) => {
            let message = match core.release_admit(task, revision, again) {
                Ok(message) => message,
                Err(why) => {
                    pool_refused(&mut routed.work, request, why);
                    return;
                }
            };
            let request = core.fly_person_task(request, PersonTaskRoute::Release(task));
            routed.work.push(Work::Event(Event::Tasks(tasks::Event::Control {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                task,
                control: tasks::Control::Release { again, message },
            })));
        }
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn say_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    task: u64,
    words: Box<[u8]>,
) {
    let Party { request, person, project, .. } = party;

    match core.person_message(
        &env.limits,
        crate::PersonMessage { request, person, project, task, question: None, words, at: env.wall },
    ) {
        Ok(event) => routed.work.push(Work::Event(Event::Tasks(event))),
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn answer_question_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    task: u64,
    question: u64,
    words: Box<[u8]>,
) {
    let Party { request, person, project, .. } = party;

    match core.person_message(
        &env.limits,
        crate::PersonMessage { request, person, project, task, question: Some(question), words, at: env.wall },
    ) {
        Ok(event) => routed.work.push(Work::Event(Event::Tasks(event))),
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn move_request(
    core: &mut Core,
    env: &Env<Limits>,
    routed: &mut Routed,
    party: Party,
    task: u64,
    reason: Box<[u8]>,
) {
    let Party { request, person, project, .. } = party;

    match core.move_admit(
        party.task(task),
        env.limits.tasks.tree_tasks,
        env.limits.tasks.depth,
        &tasks::CompoundAdmission::new(&core.tasks, 0),
    ) {
        Ok(()) => {
            let token = core.fly(crate::routing::flight::Flight::Moving { request, task });
            routed.work.push(Work::Event(Event::Tasks(tasks::Event::Move {
                reply_to: ReplyTo::new(token),
                task,
                person,
                period: core.period(project),
                pool_budget: core.pool_budget(project, person),
                period_budget: core.period_budget(project),
                reason,
            })));
        }
        Err(why) => {
            routed.work.push(Work::Event(Event::People(people::Event::Decided {
                request,
                outcome: people::Outcome::Refused(why),
            })));
        }
    }
}

pub(crate) fn lift_request(
    core: &mut Core,
    routed: &mut Routed,
    party: Party,
    hold: u64,
    revision: u64,
    with_tasks: bool,
) {
    let Party { request, person, .. } = party;

    let admission = lift_admit(core, person, hold, revision);
    match admission {
        Ok(()) => {
            let request = core.fly_person_task(request, PersonTaskRoute::Lift(hold));
            routed.work.push(Work::Event(Event::Tasks(tasks::Event::Lift {
                reply_to: ReplyTo::new(request),
                by: tasks::Party::Person(person),
                hold,
                revision,
                with_tasks,
            })));
        }
        Err(why) => pool_refused(&mut routed.work, request, why),
    }
}
fn lift_admit(core: &Core, person: u64, hold: u64, revision: u64) -> Result<(), people::Refusal> {
    let hold = core.tasks.placed_hold(hold).ok_or(people::Refusal::Unknown)?;
    if core.people.role(person, hold.project).is_none() {
        return Err(people::Refusal::Unknown);
    }
    if hold.revision != revision {
        return Err(people::Refusal::Moved { now: people::Moved::PlacedHold { revision: hold.revision } });
    }
    core.party_hold_admit(person, hold)
}

fn amend_release_admit(core: &Core, person: u64, task: u64, release: Option<u64>) -> Result<(), people::Refusal> {
    let Some(revision) = release else { return Ok(()) };
    core.hold_revision(task, revision)?;
    match core.tasks.task(task).expect("amend admission found task").phase {
        tasks::Phase::Held { why: tasks::Hold::Uncertain { .. }, .. } => {
            return Err(people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Again { required: true },
            });
        }
        tasks::Phase::Held { .. }
        | tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Closing(_)
        | tasks::Phase::Ended(_) => {}
    }
    if let Some(hold) = core.tasks.placed_on(task) {
        core.party_hold_admit(person, hold)?;
    }
    Ok(())
}

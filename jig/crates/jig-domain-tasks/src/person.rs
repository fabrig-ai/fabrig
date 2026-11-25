//! Person-task lifecycle at the tasks hub (domain/tasks/lifecycle.md, section 5, and domain/tasks/lifecycle-endings.md, section 1;
//! domain/people/README.md, section 7). Root authenticates people and role membership;
//! this child retains the role claim, initial wait, stall notice and current
//! offer. Its alarm projects the active interval; on restart the root rechecks
//! whether a personal offer needs another alarm. Claims and holds keep the wait.
//!
//! | State | Event | Next | Emits |
//! |---|---|---|---|
//! | dependency/resource wait | becomes active | answerable wait | saved start and deadline |
//! | answerable role task | take/hand back | claimed/unclaimed | save, reply |
//! | initial or offered wait | timer expires | awaiting route | request route |
//! | awaiting route | next holder | offered wait | save, requester news at first stall |
//! | awaiting route | no next holder | last wait | no further stall alarm |
//! | answerable task | valid answer | closing | save, reply |
//! | held task | release | answerable wait | prior deadline rearmed |
use crate::domain::{Domain, Task, entrance, publish, record, refused, schedule_alarm, task_mut};
use crate::{
    Active, Closing, Executor, Limits, PersonAddress, PersonOffer, Phase, Refusal, Request, Stage, TaskRecord,
    TaskResult,
};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, ReplyTo, Wall};

pub(crate) fn take(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    person: u64,
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let row = record(domain, number).expect("entrance names task");
    if !role_addressed(row) {
        return refused(to, Some(number), Refusal::Executor, out);
    }
    if !active(&row.phase) || row.taken_by.is_some() || person == 0 {
        return refused(to, Some(number), Refusal::State, out);
    }
    task_mut(domain, number).expect("live person task").record.taken_by = Some(person);
    publish(domain, env, number, out);
    out.push(Request::Done { reply_to: to });
}

pub(crate) fn hand_back(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    person: u64,
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let row = record(domain, number).expect("entrance names task");
    if !role_addressed(row) {
        return refused(to, Some(number), Refusal::Executor, out);
    }
    if !active(&row.phase) || row.taken_by != Some(person) {
        return refused(to, Some(number), Refusal::State, out);
    }
    task_mut(domain, number).expect("live person task").record.taken_by = None;
    publish(domain, env, number, out);
    out.push(Request::Done { reply_to: to });
}

pub(crate) fn answer(
    domain: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    number: u64,
    person: u64,
    result: TaskResult,
    out: &mut Queue<Request>,
) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let row = record(domain, number).expect("entrance names task");
    let address = Executor::Person(PersonAddress::Person(person));
    let addressed = match row.taken_by {
        Some(by) => by == person && role_addressed(row),
        None => row.executor == address || offer(row) == Some(PersonAddress::Person(person)),
    };
    if person == 0 || !addressed || !active(&row.phase) {
        return refused(to, Some(number), Refusal::State, out);
    }
    let failed = match &result {
        TaskResult::Failure { .. } => true,
        TaskResult::Approved { .. }
        | TaskResult::Report { .. }
        | TaskResult::Verdict { .. }
        | TaskResult::Job { .. }
        | TaskResult::NoChange { .. }
        | TaskResult::Change { .. } => false,
    };
    if (!failed || crate::run::ends::kept(domain, number).dependencies)
        && crate::run::ends::kept(domain, number).refuses_done()
    {
        return out.push(Request::Refused {
            reply_to: to,
            problem: domain.waiting_effect_problem(number).expect("kept task names its reason"),
        });
    }
    if !row.delegates.is_empty() {
        return out
            .push(Request::Refused { reply_to: to, problem: crate::Problem::live_delegates(number, &row.delegates) });
    }
    if !crate::run::valid_result(&row.contract, &result, &env.limits) {
        return refused(to, Some(number), Refusal::Contract, out);
    }
    let ending = crate::failed::ending(result, row.executor, &env.limits);
    let row = &mut task_mut(domain, number).expect("live person task").record;
    row.taken_by = None;
    row.person_answer = Some(crate::PersonAnswer { by: person, at: env.wall });
    row.phase = Phase::Closing(Closing { stage: Stage::Delegates, ending });
    publish(domain, env, number, out);
    out.push(Request::Done { reply_to: to });
}

fn role_addressed(row: &TaskRecord) -> bool {
    if let Executor::Person(PersonAddress::Role(_)) = row.executor {
        return true;
    }
    match offer(row) {
        Some(PersonAddress::Role(_)) => true,
        Some(PersonAddress::Person(_)) | None => false,
    }
}

fn offer(row: &TaskRecord) -> Option<PersonAddress> {
    Some(row.person_wait?.offer?.holder)
}

fn active(phase: &Phase) -> bool {
    match phase {
        Phase::Active(Active::Due | Active::Idle) => true,
        Phase::Waiting
        | Phase::Active(
            Active::Preparing | Active::Claimed { .. } | Active::Running { .. } | Active::BackingOff { .. },
        )
        | Phase::Closing(_)
        | Phase::Held { .. }
        | Phase::Ended(_) => false,
    }
}

/// Retain the start of the active wait; taking, handing back and held intervals
/// do not restart the wait. An initial dependency wait has not reached its recipient.
pub(crate) fn waiting(row: &mut TaskRecord, env: &Env<Limits>) {
    match row.executor {
        Executor::Person(_) => {
            if active(&row.phase) && row.person_wait.is_none() {
                row.person_wait = Some(fresh_wait(env, None));
            }
        }
        Executor::Agent { .. } | Executor::Procedure { .. } | Executor::Job { .. } => row.person_wait = None,
    }
}

fn fresh_wait(env: &Env<Limits>, notice: Option<crate::PersonStallNotice>) -> crate::PersonWait {
    crate::PersonWait {
        since: env.wall,
        stall: env.limits.person_stall,
        deadline: Wall::from_nanos(env.wall.as_nanos().saturating_add(env.limits.person_stall.as_nanos())),
        notice,
        offer: None,
    }
}

pub(crate) fn readdress(task: &mut Task, env: &Env<Limits>, address: PersonAddress) {
    task.record.executor = Executor::Person(address);
    task.record.taken_by = None;
    if let Some(wait) = task.record.person_wait {
        task.record.person_wait = Some(fresh_wait(env, wait.notice));
        task.person_stall_clock = true;
        task.alarm = None;
    }
}

pub(crate) fn deadline(task: &Task) -> Option<Wall> {
    let Executor::Person(_) = task.record.executor else { return None };
    let wait = task.record.person_wait?;
    (wait.notice.is_none() || task.person_stall_clock).then_some(wait.stall_deadline())
}

/// A wait whose notice already ran and whose current holder could still pass.
pub(crate) fn restore_clock(row: &TaskRecord) -> bool {
    let Executor::Person(_) = row.executor else { return false };
    let Some(wait) = row.person_wait else { return false };
    if let Phase::Closing(_) | Phase::Ended(_) | Phase::Held { was: crate::Was::Closing(_), .. } = row.phase {
        return false;
    }
    wait.notice.is_some()
        && match wait.offer {
            Some(PersonOffer { holder: PersonAddress::Role(_), .. }) => false,
            Some(PersonOffer { holder: PersonAddress::Person(_), .. }) | None => true,
        }
}

pub(crate) fn rearm(domain: &mut Domain, env: &Env<Limits>, number: u64, may_pass: bool) {
    let Some(task) = task_mut(domain, number) else { return };
    if restore_clock(&task.record)
        && (may_pass || env.wall < task.record.person_wait.expect("restored wait").stall_deadline())
    {
        task.person_stall_clock = true;
        schedule_alarm(domain, env, number);
    }
}

/// One credit per person delegate whose stall has not yet been reported.
pub(crate) fn credit(row: &TaskRecord) -> bool {
    match row.executor {
        Executor::Person(_) => match row.person_wait {
            Some(wait) => wait.notice.is_none(),
            None => true,
        },
        Executor::Agent { .. } | Executor::Procedure { .. } | Executor::Job { .. } => false,
    }
}

pub(crate) fn stalled(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    message: Option<u64>,
    next: Option<PersonAddress>,
    out: &mut Queue<Request>,
) {
    let Some(task) = task_mut(domain, number) else { return };
    let Some(alarm) = task.alarm else { return };
    if !active(&task.record.phase) || deadline(task) != Some(alarm.until) || env.now < alarm.due {
        return;
    }
    let wait = task.record.person_wait.as_mut().expect("person deadline has a wait");
    if wait.notice.is_none() && (message.is_none() || message == Some(0)) {
        return;
    }
    task.alarm = None;
    task.person_stall_clock = match next {
        Some(PersonAddress::Person(_)) => true,
        Some(PersonAddress::Role(_)) | None => false,
    };
    if let Some(holder) = next {
        wait.offer = Some(PersonOffer { holder, since: env.wall });
    }
    if wait.notice.is_some() {
        if next.is_some() {
            publish(domain, env, number, out);
        } else {
            schedule_alarm(domain, env, number);
        }
        return;
    }
    let message = message.expect("first stall was numbered");
    let requester = task.record.requester;
    let word = crate::Word {
        number: message,
        from: crate::Party::Task(number),
        kind: crate::MessageKind::PersonStalled { task: number },
        words: Box::new([]),
        at: env.wall,
        hits: 1,
        eligible: false,
    };
    let row = &mut task_mut(domain, number).expect("waiting person task").record;
    crate::activity::append(row, env.wall, crate::Line::Notified { recipient: requester, word: word.clone() }, out);
    row.person_wait.as_mut().expect("active wait").notice =
        Some(crate::PersonStallNotice { message, at: env.wall, activity: row.activity });
    publish(domain, env, number, out);
    match requester {
        crate::Party::Task(parent) => {
            let prepared = crate::inbox::prepare(domain, &env.limits, parent, word, crate::inbox::Admission::RoomKept)
                .expect("person delegate kept requester news room");
            let parent_row = task_mut(domain, parent).expect("requester remains live");
            parent_row.record.last_message = prepared.word.number;
            parent_row.record.inbox = prepared.inbox;
            crate::activity::message(&mut parent_row.record, &prepared.activity_word, env.limits.inbox_messages, out);
            publish(domain, env, parent, out);
            crate::wake::after_message(domain, env, parent, prepared.previous, prepared.word, out);
        }
        crate::Party::Person(_) | crate::Party::Deployment { .. } => {}
    }
}

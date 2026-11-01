//! Whole ended-task erasure, with its record erased last
//! (jig/docs/design/protocol/store.md, 11.1;
//! jig/docs/design/domain/core.md, 11).
use crate::{Core, Event, Held, Key, Limits, Record, Request, Requests, Write, reads::Sweep};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token};

/// One task's erasure in flight; volatile: a restart begins it again
/// from the record, which is erased last.
#[derive(Debug)]
pub(crate) struct Erasing {
    project: u32,
    record: Box<tasks::TaskRecord>,
}

/// Begin erasing an ended task whole.
pub(crate) fn erase_task(
    core: &mut Core,
    _env: &Env<Limits>,
    _work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    record: Box<tasks::TaskRecord>,
) {
    match record.phase {
        tasks::Phase::Ended(_) => {}
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            unreachable!("only ended tasks are erased");
        }
    }
    assert!(!erasing(core, record.number), "one erasure per task");
    let owner = Token::new(core.next_result_request);
    core.next_result_request = core.next_result_request.checked_add(1).expect("transient sweep owner space");
    let task = record.number;
    let erasure = Erasing { project: record.project, record };
    assert!(core.erasures.insert(owner, erasure).is_ok(), "one erasure per removing project");
    out.push(Request::Write(Write::DropTranscript(task)));
    sweep(out, owner, task);
}

fn sweep(out: &mut Queue<Request>, owner: Token, task: u64) {
    out.push(Request::Held(Box::new(Held::Sweep { owner, sweep: Sweep::Task(task) })));
}

/// A sweep page: erase each row by key, then sweep again or finish.
pub(crate) fn swept(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    owner: Token,
    rows: Box<[Record]>,
    more: bool,
) -> Requests {
    let mut out = Queue::uncapped();
    let Some(erasure) = core.erasures.get(&owner) else {
        return crate::project_rows::swept(core, env, work, owner, rows, more);
    };
    let task = erasure.record.number;
    for row in rows {
        let key = row.key();
        assert!(Sweep::Task(task).admits(&key), "sweep rows belong to the erasing task");
        if let Some(update) = reply_update(&row, erasure.record.requester) {
            erase(&mut out, update.key());
        }
        out.push(Request::Write(Write::Erase(key)));
    }
    if more {
        sweep(&mut out, owner, task);
    } else {
        let erasure = core.erasures.remove(&owner).expect("correlated sweep owner");
        erase_record(core, &mut out, &erasure.record);
        work.push(Event::People(people::Event::TaskErased { task, project: erasure.project }));
        clear_inbox(core, task);
        clear_reads(core, task);
        finished(core, env, work, &mut out, erasure.project, task);
    }
    Requests::Out(out)
}

fn reply_update(row: &Record, requester: tasks::Party) -> Option<tasks::Update> {
    let Record::Tasks(tasks::Stored::Activity(line)) = row else { return None };
    let (party, message) = match &line.line {
        tasks::Line::Turn { reply, .. } => match requester {
            tasks::Party::Person(party) => (party, *reply),
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => return None,
        },
        tasks::Line::Notified { recipient: tasks::Party::Person(party), word } => (*party, word.number),
        tasks::Line::Notified { .. }
        | tasks::Line::Words { .. }
        | tasks::Line::Heard(_)
        | tasks::Line::RunBegan { .. }
        | tasks::Line::RunEnded { .. }
        | tasks::Line::Decided { .. }
        | tasks::Line::Milestone { .. }
        | tasks::Line::Kept { .. } => return None,
    };
    Some(tasks::Update {
        party,
        at: line.at,
        task: line.task,
        object: tasks::UpdateObject::Reply {
            message: tasks::Newest::of(message),
            activity: tasks::Newest::of(line.position),
        },
    })
}

fn erase(out: &mut Queue<Request>, key: tasks::Key) {
    out.push(Request::Write(Write::Erase(Key::Tasks(key))));
}

fn erase_record(core: &Core, out: &mut Queue<Request>, record: &tasks::TaskRecord) {
    let task = record.number;
    let ended_at = record.ended_at.expect("ended task has a time");
    match record.requester {
        tasks::Party::Person(party) => {
            let index = tasks::EndedByParty { party, label: record.label, ended_at, task };
            erase(out, index.key());
            erase(out, tasks::Key::Result { party, position: record.result_position });
            let update = tasks::Update { party, at: ended_at, task, object: tasks::UpdateObject::Result };
            erase(out, update.key());
        }
        tasks::Party::Task(requester) => {
            erase(out, tasks::Key::EndedDelegate { requester, task });
            erase(out, tasks::Key::Delegate { requester, task });
        }
        tasks::Party::Deployment { .. } => {}
    }
    if record.flags.listed {
        let index = tasks::EndedByProject { project: record.project, label: record.label, ended_at, task };
        erase(out, index.key());
    }
    for key in [tasks::Key::Ledger(tasks::Funder::Task(task)), tasks::Key::ClosedLedger(tasks::Funder::Task(task))] {
        erase(out, key);
    }
    if core.tasks.stub(task).is_none() {
        erase(out, tasks::Key::Stub(task));
    }
    erase(out, tasks::Key::Ended(task));
}

fn clear_inbox(core: &mut Core, task: u64) {
    drop(core.archived_chats.remove(&task));
    core.prune_questions(task);
    if let Some((owner, _, _)) = core.census_position
        && owner == task
    {
        core.census_position = None;
    }
    let mut marks = List::with_capacity(core.ended_marks.len());
    for (&token, mark) in &core.ended_marks {
        if mark.task == task {
            marks.push(token).expect("bounded ended marks");
        }
    }
    for token in marks.into_boxed() {
        let _mark = core.ended_marks.remove(&token);
    }
}

fn clear_reads(core: &mut Core, task: u64) {
    crate::routing::result_reads::clear(core, task);
    let mut reads = List::with_capacity(core.result_reads.len());
    for (&token, read) in &core.result_reads {
        let mut named = read.ids.contains(&task);
        for result in &read.results {
            named |= result.task == task;
        }
        if named {
            reads.push(token).expect("bounded result reads");
        }
    }
    for token in reads.into_boxed() {
        if let Some(read) = core.result_reads.remove(&token) {
            match read.why {
                crate::routing::result_reads::ReadFor::Amendment(amendment) => match amendment.replay {
                    crate::routing::amendment_reads::Replay::Agent { key, .. } => {
                        let _pending = core.pending_calls.remove(&key);
                    }
                    crate::routing::amendment_reads::Replay::Person { .. } => {}
                },
                crate::routing::result_reads::ReadFor::DeploymentClose { .. }
                | crate::routing::result_reads::ReadFor::Preparation { .. } => {}
            }
        }
    }
}

/// Whether this task's erasure is in flight.
pub(crate) fn erasing(core: &Core, task: u64) -> bool {
    for (_, erasure) in &core.erasures {
        if erasure.record.number == task {
            return true;
        }
    }
    false
}

/// The erasure's end; `pd-delete` continues the deletion here.
fn finished(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    project: u32,
    task: u64,
) {
    crate::routing::removal::erased(core, env, work, out, project, task);
}

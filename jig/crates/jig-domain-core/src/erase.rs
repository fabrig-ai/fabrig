//! Whole-task erasure (jig/docs/design/protocol/store.md, 7.4, 8.5, 11.1).
use crate::{Core, Key, Limits, Request, Requests, Write, reads};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Token};

/// What selects ended tasks for erasure; retention adds its horizon later.
#[cfg_attr(not(test), expect(dead_code, reason = "project deletion constructs this selection"))]
#[derive(Debug)]
pub(crate) enum Select {
    Project(u32),
}

/// One serialized walk, with subsequent selections waiting behind it.
#[derive(Debug)]
pub(crate) struct Erasures {
    walk: Option<Walk>,
    waiting: Queue<Select>,
}

impl Erasures {
    pub(crate) fn new() -> Self {
        Self { walk: None, waiting: Queue::uncapped() }
    }
}

#[derive(Debug)]
struct Walk {
    select: Select,
    after: Option<u64>,
    task: Option<Erasing>,
}

#[derive(Debug)]
struct Erasing {
    record: Box<tasks::TaskRecord>,
    phase: Phase,
}

#[derive(Debug)]
enum Phase {
    Rows,
    Cleanup,
    /// The preceding decision removed the last rows and indexes.
    Record,
}

/// Walk selected ended tasks, then call `deletion::tasks_erased`.
pub(crate) fn scan(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, select: Select) {
    if core.erasures.walk.is_some() {
        core.erasures.waiting.push(select);
        return;
    }
    core.erasures.walk = Some(Walk { select, after: None, task: None });
    next_record(core, env, out);
}

#[expect(clippy::manual_map, reason = "step code builds the optional ended cursor explicitly")]
fn next_record(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>) {
    let after = core.erasures.walk.as_ref().expect("active erasure walk").after;
    let after = match after {
        Some(task) => Some(reads::Position::Owner(Key::Tasks(tasks::Key::Ended(task)))),
        None => None,
    };
    ask(core, env, out, reads::Scope::EndedTasks, after, 1);
}

fn ask(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    scope: reads::Scope,
    after: Option<reads::Position>,
    rows: u32,
) {
    reads::erase_load(
        core,
        reads::Range { party: 0, scope, after, backwards: false, rows, bytes: env.limits.load_bytes },
        out,
    );
}

/// One page of the walk or of a task's rows, back through `reads::loaded`.
pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, load: Token, result: reads::Loaded) -> Requests {
    let range = reads::erase_range(core, load);
    let mut out = Queue::uncapped();
    let (rows, next) = match result {
        reads::Loaded::Page { rows, next } => (rows, next),
        reads::Loaded::Busy
        | reads::Loaded::Gone
        | reads::Loaded::TooLinked { .. }
        | reads::Loaded::TurnProblem { .. } => {
            reads::erase_load(core, range, &mut out);
            out.push(Request::Decided);
            return Requests::Out(out);
        }
    };
    assert!(rows.len() <= usize::try_from(range.rows).expect("read row bound"), "bounded erasure page");
    let mut walk = core.erasures.walk.take().expect("load belongs to active erasure walk");
    match walk.task.take() {
        None => {
            for row in rows {
                match row.value {
                    crate::Record::Tasks(tasks::Stored::Ended(record)) => {
                        walk.after = Some(record.number);
                        let selected = match walk.select {
                            Select::Project(project) => record.project == project,
                        };
                        if selected {
                            walk.task = Some(Erasing { record, phase: Phase::Rows });
                        }
                    }
                    crate::Record::Tasks(_)
                    | crate::Record::Core(_)
                    | crate::Record::People(_)
                    | crate::Record::Notes(_)
                    | crate::Record::Accounts(_) => {
                        unreachable!("ended task walk loads ended records")
                    }
                }
            }
            if let Some(task) = &walk.task {
                let number = task.record.number;
                core.erasures.walk = Some(walk);
                ask(core, env, &mut out, reads::Scope::TaskRows { task: number }, None, env.limits.documents.rows);
            } else if next.is_some() {
                core.erasures.walk = Some(walk);
                next_record(core, env, &mut out);
            } else {
                finish(core, env, &mut out, walk.select);
            }
        }
        Some(mut task) => {
            let number = task.record.number;
            match task.phase {
                Phase::Rows => {
                    // A page has at most two erases per row: its key and a reply's update.
                    for row in rows {
                        erase_reply(&task.record, &row.value, &mut out);
                        out.push(Request::Write(Write::Erase(row.value.key())));
                    }
                    let after = if next.is_some() {
                        next
                    } else {
                        task.phase = Phase::Cleanup;
                        // A held read is the durability fence between cleanup and the owner erase.
                        range.after
                    };
                    walk.task = Some(task);
                    core.erasures.walk = Some(walk);
                    ask(core, env, &mut out, reads::Scope::TaskRows { task: number }, after, env.limits.documents.rows);
                }
                Phase::Cleanup => {
                    assert!(rows.is_empty() && next.is_none(), "task row cleanup committed");
                    erase_indexes(core, env, &task.record, &mut out);
                    task.phase = Phase::Record;
                    walk.task = Some(task);
                    core.erasures.walk = Some(walk);
                    ask(
                        core,
                        env,
                        &mut out,
                        reads::Scope::TaskRows { task: number },
                        range.after,
                        env.limits.documents.rows,
                    );
                }
                Phase::Record => {
                    assert!(rows.is_empty() && next.is_none(), "task rows are gone before its record");
                    out.push(Request::Write(Write::Erase(Key::Tasks(tasks::Key::Ended(number)))));
                    core.erasures.walk = Some(walk);
                    next_record(core, env, &mut out);
                }
            }
        }
    }
    out.push(Request::Decided);
    Requests::Out(out)
}

fn finish(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, select: Select) {
    match select {
        Select::Project(project) => crate::deletion::tasks_erased(core, env, out, project),
    }
    if let Some(select) = core.erasures.waiting.pop() {
        scan(core, env, out, select);
    }
}

fn erase_reply(record: &tasks::TaskRecord, value: &crate::Record, out: &mut Queue<Request>) {
    let line = match value {
        crate::Record::Tasks(tasks::Stored::Activity(line)) => line,
        crate::Record::Tasks(_)
        | crate::Record::Core(_)
        | crate::Record::People(_)
        | crate::Record::Notes(_)
        | crate::Record::Accounts(_) => return,
    };
    let addressed = match &line.line {
        tasks::Line::Turn { reply, .. } => match record.requester {
            tasks::Party::Person(party) => Some((party, *reply)),
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
        },
        tasks::Line::Notified { recipient: tasks::Party::Person(party), word } => Some((*party, word.number)),
        tasks::Line::Notified { .. }
        | tasks::Line::Words { .. }
        | tasks::Line::Heard(_)
        | tasks::Line::RunBegan { .. }
        | tasks::Line::RunEnded { .. }
        | tasks::Line::Decided { .. }
        | tasks::Line::Kept { .. }
        | tasks::Line::Milestone { .. } => None,
    };
    if let Some((party, message)) = addressed {
        let update = tasks::Update {
            party,
            at: line.at,
            task: line.task,
            object: tasks::UpdateObject::Reply {
                message: tasks::Newest::of(message),
                activity: tasks::Newest::of(line.position),
            },
        };
        out.push(Request::Write(Write::Erase(Key::Tasks(update.key()))));
    }
}

fn erase_indexes(core: &mut Core, env: &Env<Limits>, record: &tasks::TaskRecord, out: &mut Queue<Request>) {
    out.push(Request::Write(Write::DropTranscript { task: record.number }));
    let ended_at = record.ended_at.expect("ended record has its ending time");
    match record.requester {
        tasks::Party::Person(party) => {
            let owner = crate::forget::Owner {
                task: record.number,
                party,
                project: record.project,
                label: record.label,
                ended_at,
                result_position: record.result_position,
                listed: record.flags.listed,
                last_reply: record.flags.conversation.then_some(record.last_reply),
            };
            for key in crate::forget::record_keys(&owner).into_boxed() {
                if key != tasks::Key::Ended(record.number) {
                    out.push(Request::Write(Write::Erase(Key::Tasks(key))));
                }
            }
            // Ended reads are removed even when their party is no longer in memory.
            out.push(Request::Write(Write::Erase(Key::People(people::Key::EndedRead { task: record.number, party }))));
            forget_party(core, env, record, party, out);
        }
        tasks::Party::Task(requester) => {
            for key in [
                tasks::Key::EndedDelegate { requester, task: record.number },
                tasks::Key::Delegate { requester, task: record.number },
            ] {
                out.push(Request::Write(Write::Erase(Key::Tasks(key))));
            }
            erase_project_index(record, out);
        }
        tasks::Party::Deployment { .. } => erase_project_index(record, out),
    }
}

fn erase_project_index(record: &tasks::TaskRecord, out: &mut Queue<Request>) {
    if record.flags.listed {
        let index = tasks::EndedByProject {
            project: record.project,
            label: record.label,
            ended_at: record.ended_at.expect("ended record"),
            task: record.number,
        };
        out.push(Request::Write(Write::Erase(Key::Tasks(index.key()))));
    }
}

fn forget_party(core: &mut Core, env: &Env<Limits>, record: &tasks::TaskRecord, party: u64, out: &mut Queue<Request>) {
    drop(core.archived_chats.remove(&record.number));
    core.touch_party(party);
    let event = people::Event::Forget {
        task: record.number,
        project: record.project,
        party: Some(party),
        last_reply: record.last_reply,
    };
    let mut child = Queue::uncapped();
    people::step(&mut core.people, &Env { now: env.now, wall: env.wall, limits: env.limits.people }, event, &mut child);
    for _ in 0..child.len() {
        match child.pop().expect("forget output count") {
            people::Request::Erase { key } => out.push(Request::Write(Write::Erase(Key::People(key)))),
            people::Request::Act { .. }
            | people::Request::Acted { .. }
            | people::Request::ServiceMade { .. }
            | people::Request::RolesApplied { .. }
            | people::Request::Route { .. }
            | people::Request::Reply { .. }
            | people::Request::Save { .. }
            | people::Request::RolesRefused { .. }
            | people::Request::RestoreRefused { .. } => {
                unreachable!("forget only removes read positions")
            }
        }
    }
}

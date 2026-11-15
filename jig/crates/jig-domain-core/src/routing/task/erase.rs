//! One serialized walk for deletion and retention
//! (jig/docs/design/protocol/store/README.md, 6.1; retention.md, 5–6).
mod retention;
use crate::{Core, Key, Limits, Request, Write, reads};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, Queue, Time, Token};

/// One hour after the last pass ends (protocol/store/retention.md, 5).
pub(crate) const RETENTION_EVERY: Duration = Duration::from_secs(3600);

#[derive(Hash, Debug)]
pub(crate) enum Select {
    Project(u32),
    Retention,
}

/// Deletion and retention queue behind each other; no two walks can own a task.
#[derive(Hash, Debug)]
pub(crate) struct Erasures {
    walk: Option<Walk>,
    waiting: Queue<Select>,
    retention_running: bool,
    next_retention: Option<Time>,
}
impl Erasures {
    pub(crate) fn new() -> Self {
        Self { walk: None, waiting: Queue::uncapped(), retention_running: false, next_retention: Some(Time::ZERO) }
    }
    pub(crate) fn deadline(&self) -> Option<Time> {
        self.next_retention
    }
}

#[derive(Hash, Debug)]
struct Walk {
    select: Select,
    after: Option<u64>,
    task: Option<Erasing>,
    retention: retention::Pass,
}
#[derive(Hash, Debug)]
struct Erasing {
    record: Box<tasks::TaskSummary>,
    mode: Mode,
    phase: Phase,
}
#[derive(Clone, Copy, Hash, Debug)]
enum Mode {
    Whole,
    Record { row: tasks::RetentionRow, whole: bool },
    Forgotten(tasks::RetentionRow),
}
impl Mode {
    fn whole(self) -> bool {
        match self {
            Self::Whole | Self::Forgotten(_) => true,
            Self::Record { whole, .. } => whole,
        }
    }
    fn scope(self, task: u64) -> reads::Scope {
        match self {
            Self::Whole => reads::Scope::TaskRows { task },
            Self::Record { .. } | Self::Forgotten(_) => reads::Scope::TaskOwnRows { task },
        }
    }
}
#[derive(Hash, Debug)]
enum Phase {
    Rows,
    Cleanup,
    Record,
}

pub(crate) fn scan(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, select: Select) {
    if let Select::Retention = select {
        if core.erasures.retention_running {
            return;
        }
        core.erasures.retention_running = true;
        core.erasures.next_retention = None;
    }
    if core.erasures.walk.is_some() {
        core.erasures.waiting.push(select);
        return;
    }
    begin(core, env, out, select);
}
fn begin(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, select: Select) {
    let walk = Walk { select, after: None, task: None, retention: retention::Pass::new() };
    next_record(core, env, out, walk);
}
fn next_record(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, walk: Walk) {
    match walk.select {
        Select::Retention => retention::next(core, env, out, walk),
        Select::Project(_) => {
            let scope = reads::Scope::TaskSummaries;
            let after = match walk.after {
                Some(task) => Some(reads::Position::Owner(Key::Tasks(tasks::Key::Summary(task)))),
                None => None,
            };
            core.erasures.walk = Some(walk);
            ask(core, env, out, scope, after, 1);
        }
    }
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

pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, load: Token, result: reads::Loaded) -> Queue<Request> {
    let range = reads::erase_range(core, load);
    let mut out = Queue::uncapped();
    let (rows, next) = match result {
        reads::Loaded::Page { rows, next } => (rows, next),
        reads::Loaded::Busy
        | reads::Loaded::Gone
        | reads::Loaded::TooLinked { .. }
        | reads::Loaded::TurnProblem { .. } => {
            reads::erase_load(core, range, &mut out);
            return out;
        }
    };
    assert!(rows.len() <= skein_lib::length::usize_of(range.rows), "bounded erasure page");
    let mut walk = core.erasures.walk.take().expect("active erasure walk");
    if let Some(task) = walk.task.take() {
        task_rows(core, env, &mut out, walk, task, range, rows, next);
    } else {
        match walk.select {
            Select::Retention => {
                retention::loaded(core, env, &mut out, walk, rows, range, next);
            }
            Select::Project(project) => {
                for row in rows {
                    match row.value {
                        crate::Record::Tasks(tasks::Stored::Summary(summary)) => {
                            walk.after = Some(summary.number);
                            if summary.project == project {
                                walk.task = Some(Erasing { record: summary, mode: Mode::Whole, phase: Phase::Rows });
                            }
                        }
                        crate::Record::Tasks(_)
                        | crate::Record::Core(_)
                        | crate::Record::People(_)
                        | crate::Record::Notes(_)
                        | crate::Record::Accounts(_) => unreachable!("erasure owners"),
                    }
                }
                if walk.task.is_some() {
                    own_rows(core, env, &mut out, walk, None);
                } else if next.is_some() {
                    next_record(core, env, &mut out, walk);
                } else {
                    finish(core, env, &mut out, walk.select);
                }
            }
        }
    }
    out
}
fn own_rows(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, walk: Walk, after: Option<reads::Position>) {
    let task = walk.task.as_ref().expect("erasing task");
    let scope = task.mode.scope(task.record.number);
    core.erasures.walk = Some(walk);
    ask(core, env, out, scope, after, env.limits.documents.rows);
}
#[expect(clippy::too_many_arguments, reason = "one bounded page and its serialized walk state")]
fn task_rows(
    core: &mut Core,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
    mut walk: Walk,
    mut task: Erasing,
    range: reads::Range,
    rows: Box<[reads::Row]>,
    next: Option<reads::Position>,
) {
    let number = task.record.number;
    match task.phase {
        Phase::Rows => {
            for row in rows {
                erase_reply(&task.record, &row.value, out);
                out.push(Request::Write(Write::Erase(row.value.key())));
            }
            let after = if next.is_some() {
                next
            } else {
                task.phase = Phase::Cleanup;
                range.after
            };
            walk.task = Some(task);
            own_rows(core, env, out, walk, after);
        }
        Phase::Cleanup => {
            assert!(rows.is_empty() && next.is_none(), "task rows committed");
            erase_indexes(core, env, &task.record, task.mode.whole(), out);
            task.phase = Phase::Record;
            walk.task = Some(task);
            own_rows(core, env, out, walk, range.after);
        }
        Phase::Record => {
            assert!(rows.is_empty() && next.is_none(), "rows gone before owner");
            out.push(Request::Write(Write::DropHistory { task: number }));
            if let Mode::Whole | Mode::Record { .. } = task.mode {
                out.push(Request::Write(Write::Erase(Key::Tasks(tasks::Key::Ended(number)))));
                unname(core, number, out);
            }
            if task.mode.whole() {
                out.push(Request::Write(Write::Erase(Key::Tasks(tasks::Key::Summary(number)))));
            }
            match task.mode {
                Mode::Whole => {}
                Mode::Record { row, whole } => {
                    retention::move_row(out, row, if whole { None } else { Some(tasks::RetentionStage::Summary) });
                }
                Mode::Forgotten(row) => retention::move_row(out, row, None),
            }
            next_record(core, env, out, walk);
        }
    }
}
fn unname(core: &Core, task: u64, out: &mut Queue<Request>) {
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask { connector, ask: crate::Ask::Effect(crate::effects::connector::Ask::Unname { task }) });
    }
}
fn finish(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Request>, select: Select) {
    match select {
        Select::Project(project) => crate::admission::deletion::tasks_erased(core, env, out, project),
        Select::Retention => {
            core.erasures.retention_running = false;
            core.erasures.next_retention = Some(env.now.saturating_add(RETENTION_EVERY));
        }
    }
    if let Some(select) = core.erasures.waiting.pop() {
        begin(core, env, out, select);
    }
}

fn erase_reply(record: &tasks::TaskSummary, value: &crate::Record, out: &mut Queue<Request>) {
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
        | tasks::Line::NowWritten { .. }
        | tasks::Line::Stood { .. }
        | tasks::Line::TalkOpened { .. }
        | tasks::Line::TalkEnded { .. }
        | tasks::Line::SessionEnded { .. }
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

fn erase_indexes(
    core: &mut Core,
    env: &Env<Limits>,
    record: &tasks::TaskSummary,
    whole: bool,
    out: &mut Queue<Request>,
) {
    out.push(Request::Write(Write::DropTranscript { task: record.number }));
    if whole {
        erase_ended_indexes(record, out);
    }
    match record.requester {
        tasks::Party::Person(party) => {
            let update =
                tasks::Update { party, at: record.ended_at, task: record.number, object: tasks::UpdateObject::Result };
            for key in [update.key(), tasks::Key::Result { party, position: record.result_position }] {
                out.push(Request::Write(Write::Erase(Key::Tasks(key))));
            }
            // Both positions go even when the party is no longer in memory.
            for key in [
                people::Key::EndedRead { task: record.number, party },
                people::Key::ReadConversation { task: record.number, party },
            ] {
                out.push(Request::Write(Write::Erase(Key::People(key))));
            }
            forget_party(core, env, record, party, out);
        }
        tasks::Party::Task(requester) => {
            out.push(Request::Write(Write::Erase(Key::Tasks(tasks::Key::Delegate { requester, task: record.number }))));
        }
        tasks::Party::Deployment { .. } => {}
    }
}
fn erase_ended_indexes(record: &tasks::TaskSummary, out: &mut Queue<Request>) {
    match record.requester {
        tasks::Party::Person(party) => {
            let index =
                tasks::EndedByParty { party, label: record.label, ended_at: record.ended_at, task: record.number };
            out.push(Request::Write(Write::Erase(Key::Tasks(index.key()))));
        }
        tasks::Party::Task(requester) => {
            out.push(Request::Write(Write::Erase(Key::Tasks(tasks::Key::EndedDelegate {
                requester,
                task: record.number,
            }))));
        }
        tasks::Party::Deployment { .. } => {}
    }
    erase_project_index(record, out);
}

fn erase_project_index(record: &tasks::TaskSummary, out: &mut Queue<Request>) {
    if record.flags.listed {
        let index = tasks::EndedByProject {
            project: record.project,
            label: record.label,
            ended_at: record.ended_at,
            task: record.number,
        };
        out.push(Request::Write(Write::Erase(Key::Tasks(index.key()))));
    }
}

fn forget_party(core: &mut Core, env: &Env<Limits>, record: &tasks::TaskSummary, party: u64, out: &mut Queue<Request>) {
    drop(core.ended_chats.remove(&record.number));
    core.touch_party(party);
    let event = people::Event::Forget {
        task: record.number,
        project: record.project,
        party: Some(party),
        last_reply: record.last_reply,
    };
    let mut child = Queue::uncapped();
    crate::routing::step::step_people(core, env, event, &mut child);
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
            | people::Request::NeedArrived { .. }
            | people::Request::RestoreRefused { .. } => {
                unreachable!("forget only removes read positions")
            }
        }
    }
}

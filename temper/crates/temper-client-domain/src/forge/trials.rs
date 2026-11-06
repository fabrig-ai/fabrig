//! Session trial facts and their run's wait (temper/docs/design/domain/trial.md, 6).
use super::{Limits, Request, Side, wire_limits};
use alloc::boxed::Box;
use jig_client::{Ask, Family, Handle, Latest, Link, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Reader, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub account: u32,
    pub task: Option<u64>,
    pub took: Option<u32>,
    pub step: Step,
    pub state: State,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Starts,
    Answers,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Answered { seconds: u32 },
    Failed { why: Why },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why {
    Ended { class: Class },
    Refused,
    Waited,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    Transient,
    Permanent,
    Run,
    Agent,
    Lost,
    Invalid,
    Refused,
}

#[derive(Debug, PartialEq, Eq)]
enum Watch {
    Unknown,
    Open,
    Ended,
}
#[derive(Debug)]
struct Task {
    number: u64,
    open: bool,
}
#[derive(Debug)]
struct Pending {
    request: Token,
    account: u32,
}
/// One person's trial facts, held independently of the current page.
#[derive(Debug)]
pub struct Trials {
    rows: Box<[Row]>,
    watch: Watch,
    task: Option<Task>,
    pending: Option<Pending>,
    wait: Option<c::TaskRunWait>,
}
impl Trials {
    pub(crate) fn new() -> Self {
        Self { rows: Box::new([]), watch: Watch::Unknown, task: None, pending: None, wait: None }
    }
}
impl Side {
    /// The person's rows, newest first; empty until delivered.
    #[must_use]
    pub fn trials(&self) -> &[Row] {
        &self.trials.rows
    }
    /// The account an ask is out for, until its row is delivered.
    #[must_use]
    pub fn trial_asked(&self) -> Option<u32> {
        Some(self.trials.pending.as_ref()?.account)
    }
    /// The running row's wait, or the last it had: the Task document's `run_wait`.
    #[must_use]
    pub const fn trial_wait(&self) -> Option<&c::TaskRunWait> {
        self.trials.wait.as_ref()
    }
}
#[must_use]
pub fn watch(limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let record = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Trials }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Own,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&record) },
    })
}
#[must_use]
pub fn ask(account: u32, again: bool, limits: &Limits) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let value = w::TryAccount::new(&bounds, w::TryAccountParts { account, again }).ok()?;
    let record = w::Requesting::new(&bounds, w::RequestingParts { request: w::Request::TryAccount(value) }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Own,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}
pub(crate) fn handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::TRIALS }
}
pub(crate) fn task_handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::TRIAL_TASK }
}
pub(crate) fn configure(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        end(side, out);
        side.trials = Trials::new();
        side.changed();
        return;
    }
    if side.party.is_some() && side.trials.watch == Watch::Unknown {
        if let Some(subject) = watch(&limits) {
            side.trials.watch = Watch::Open;
            out.push(Request::Client { asked: jig_client::Event::Watch { handle: handle(), subject } });
        } else {
            side.trials.watch = Watch::Ended;
        }
        side.changed();
    }
}
pub(crate) fn send(side: &mut Side, account: u32, again: bool, limits: Limits, out: &mut Queue<Request>) {
    if side.party.is_none() || side.trials.pending.is_some() {
        return;
    }
    let Some(ask) = ask(account, again, &limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits u64");
    let request = Token::new(side.serial);
    side.trials.pending = Some(Pending { request, account });
    side.trials.wait = None;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}

/// Decoded tells cross the forge's event boundary (client/README.md, 4.6).
#[derive(Debug)]
pub enum Event {
    Rows { rows: Box<[Row]> },
    RowsEnded,
    Wait { task: u64, wait: Option<c::TaskRunWait> },
    TaskEnded,
    Answered { key: jig_client::Key, tried: bool },
}
pub(crate) fn told(client: &jig_client::Client, told: &Told, limits: Limits) -> Option<Event> {
    match told {
        Told::Changed { handle: changed } if *changed == handle() => {
            let document = client.document(*changed)?;
            if document.state == jig_client::State::Ended {
                return Some(Event::RowsEnded);
            }
            match document.latest {
                Some(Latest::Family(record)) => Some(match rows(record, limits) {
                    Some(rows) => Event::Rows { rows },
                    None => Event::RowsEnded,
                }),
                Some(Latest::Jig(_)) => Some(Event::RowsEnded),
                None => None,
            }
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => Some(Event::RowsEnded),
        Told::Changed { handle: changed } if *changed == task_handle() => {
            let document = client.document(*changed)?;
            if document.state == jig_client::State::Ended {
                return Some(Event::TaskEnded);
            }
            match document.latest {
                Some(Latest::Jig(c::Delivery::Task(task))) => {
                    Some(Event::Wait { task: task.chip.task, wait: task.run_wait })
                }
                Some(Latest::Jig(_) | Latest::Family(_)) => Some(Event::TaskEnded),
                None => None,
            }
        }
        Told::Ended { handle: ended, .. } if *ended == task_handle() => Some(Event::TaskEnded),
        Told::Answered { key, outcome } => Some(Event::Answered { key: *key, tried: tried(outcome, limits) }),
        Told::Pending { key } => {
            let pending = client.pending(*key)?;
            if pending.progress == jig_client::Progress::Unknown {
                Some(Event::Answered { key: *key, tried: false })
            } else {
                None
            }
        }
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => None,
    }
}
pub(crate) fn receive(side: &mut Side, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Rows { rows } => {
            match side.trials.watch {
                Watch::Open => {}
                Watch::Unknown | Watch::Ended => return,
            }
            if let Some(pending) = &side.trials.pending {
                for row in &rows {
                    if row.account == pending.account {
                        side.trials.pending = None;
                        break;
                    }
                }
            }
            side.trials.rows = rows;
            follow(side, out);
        }
        Event::RowsEnded => end(side, out),
        Event::Wait { task, wait } => {
            if let Some(watched) = &side.trials.task
                && watched.open
                && watched.number == task
            {
                side.trials.wait = wait;
            } else {
                return;
            }
        }
        Event::TaskEnded => {
            close_task(side, out);
            side.trials.wait = None;
        }
        Event::Answered { key, tried } => {
            let Some(pending) = &side.trials.pending else { return };
            if key.asker != handle().asker || key.number != pending.request.raw() {
                return;
            }
            if !tried {
                side.trials.pending = None;
            }
        }
    }
    side.changed();
}
fn follow(side: &mut Side, out: &mut Queue<Request>) {
    let mut next = None;
    for row in &side.trials.rows {
        match row.state {
            State::Running => {
                if row.task.is_some() {
                    next = row.task;
                    break;
                }
            }
            State::Answered { .. } | State::Failed { .. } => {}
        }
    }
    if let Some(task) = &side.trials.task
        && Some(task.number) == next
    {
        return;
    }
    close_task(side, out);
    side.trials.task = None;
    if let Some(number) = next {
        side.trials.wait = None;
        side.trials.task = Some(Task { number, open: true });
        out.push(Request::Client {
            asked: jig_client::Event::Watch {
                handle: task_handle(),
                subject: Watchable::Jig(c::Subject::Task { task: number }),
            },
        });
    }
}
fn close_task(side: &mut Side, out: &mut Queue<Request>) {
    if let Some(task) = &mut side.trials.task
        && task.open
    {
        task.open = false;
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: task_handle() } });
    }
}
fn end(side: &mut Side, out: &mut Queue<Request>) {
    close_task(side, out);
    if side.trials.watch == Watch::Open {
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
    side.trials.watch = Watch::Ended;
    side.trials.rows = Box::new([]);
    side.trials.task = None;
    side.trials.wait = None;
}
fn rows(record: &Family, limits: Limits) -> Option<Box<[Row]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let bounds = wire_limits(&limits)?;
    let mut reader = Reader::new(&record.bytes);
    let document = w::Trials::decode(&bounds, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let mut rows = List::with_capacity(bounds.trials_rows);
    for value in document.into_parts().rows.into_boxed() {
        let value = value.into_parts();
        let step = match value.step {
            w::TrialStep::Starts => Step::Starts,
            w::TrialStep::Answers => Step::Answers,
        };
        let state = match value.state {
            w::TrialState::Running => State::Running,
            w::TrialState::Answered(value) => State::Answered { seconds: value.seconds() },
            w::TrialState::Failed(value) => State::Failed {
                why: match value.into_parts().why {
                    w::TrialWhy::Refused => Why::Refused,
                    w::TrialWhy::Waited => Why::Waited,
                    w::TrialWhy::Ended(value) => Why::Ended {
                        class: match value.class() {
                            w::TrialClass::Transient => Class::Transient,
                            w::TrialClass::Permanent => Class::Permanent,
                            w::TrialClass::Run => Class::Run,
                            w::TrialClass::Agent => Class::Agent,
                            w::TrialClass::Lost => Class::Lost,
                            w::TrialClass::Invalid => Class::Invalid,
                            w::TrialClass::Refused => Class::Refused,
                        },
                    },
                },
            },
        };
        rows.push(Row { account: value.account, task: value.task, took: value.took, step, state })
            .expect("wire bounds trial rows");
    }
    Some(rows.into_boxed())
}
fn tried(outcome: &jig_client::Outcome, limits: Limits) -> bool {
    let record = match outcome {
        jig_client::Outcome::Answered(jig_client::Answered::Family(record)) => record,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(_))
        | jig_client::Outcome::Unknown
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Unconfirmed
        | jig_client::Outcome::Held
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost => return false,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return false;
    }
    let Some(bounds) = wire_limits(&limits) else { return false };
    let mut reader = Reader::new(&record.bytes);
    let Ok(answer) = w::Answer::decode(&bounds, &mut reader) else { return false };
    reader.is_empty() && *answer.outcome() == w::Outcome::Tried
}

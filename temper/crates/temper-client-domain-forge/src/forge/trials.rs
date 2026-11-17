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
    pub run_wait: Option<c::TaskRunWait>,
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
    Infrastructure,
}

#[derive(Debug, PartialEq, Eq)]
enum Watch {
    Unknown,
    Open,
    Ended,
}
#[derive(Debug)]
struct Pending {
    request: Token,
    account: u32,
}
/// One person's trial facts, held independently of the current page.
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "the root reconciles seen trials; pending requests stay with the forge side"
)]
pub struct Trials {
    pub rows: Box<[Row]>,
    watch: Watch,
    pending: Option<Pending>,
    wait: Option<c::TaskRunWait>,
    last_wait: Option<c::TaskRunWait>,
    pub seen: Option<List<u32>>,
}
impl Trials {
    pub(crate) fn new() -> Self {
        Self { rows: Box::new([]), watch: Watch::Unknown, pending: None, wait: None, last_wait: None, seen: None }
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
    /// The running row's wait, or the last it had: its row's `run_wait`.
    #[must_use]
    pub const fn trial_wait(&self) -> Option<&c::TaskRunWait> {
        if let Some(row) = self.trials.rows.first()
            && let State::Failed { why: Why::Waited } = row.state
        {
            return self.trials.last_wait.as_ref();
        }
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
pub fn configure(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
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
    side.trials.last_wait = None;
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}

/// Decoded tells cross the forge's event boundary (client/forge/records.md).
#[derive(Debug)]
pub enum Event {
    Rows { rows: Box<[Row]> },
    RowsEnded,
    Answered { key: jig_client::Key, tried: bool },
}
pub(crate) fn told(client: &jig_client::Client, told: &Told, limits: &crate::Limits) -> Option<Event> {
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
        Told::Answered { key, outcome } => Some(Event::Answered { key: *key, tried: tried(outcome, limits.forge) }),
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
            let next = running(&rows);
            if let Some(row) = next {
                let old = match running(&side.trials.rows) {
                    Some(old) => old.task,
                    None => None,
                };
                if old != row.task {
                    side.trials.last_wait = None;
                }
                side.trials.wait = row.run_wait;
                if row.run_wait.is_some() {
                    side.trials.last_wait = row.run_wait;
                }
            } else {
                side.trials.wait = None;
            }
            side.trials.rows = rows;
        }
        Event::RowsEnded => end(side, out),
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
#[expect(clippy::manual_find, reason = "step code selects a running row without closures")]
fn running(rows: &[Row]) -> Option<&Row> {
    for row in rows {
        if row.state == State::Running && row.task.is_some() {
            return Some(row);
        }
    }
    None
}
fn end(side: &mut Side, out: &mut Queue<Request>) {
    if side.trials.watch == Watch::Open {
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
    side.trials.watch = Watch::Ended;
    side.trials.rows = Box::new([]);
    side.trials.wait = None;
    side.trials.last_wait = None;
}
fn rows(record: &Family, limits: &crate::Limits) -> Option<Box<[Row]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let bounds = wire_limits(&limits.forge)?;
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).ok()?;
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
                            w::TrialClass::Infrastructure => Class::Infrastructure,
                            w::TrialClass::Refused => Class::Refused,
                        },
                    },
                },
            },
        };
        let run_wait = match value.run_wait {
            Some(bytes) if state == State::Running && value.task.is_some() => run_wait(&bytes, &link),
            Some(_) | None => None,
        };
        rows.push(Row { account: value.account, task: value.task, took: value.took, step, state, run_wait })
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

fn run_wait(bytes: &[u8], limits: &jig_client_protocol_link::Limits) -> Option<c::TaskRunWait> {
    use jig_client_protocol_wire as j;
    let mut reader = Reader::new(bytes);
    let wait = j::TaskRunWait::decode(limits.wire(), &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    jig_client_protocol_link::translate::run_wait(Some(&wait))
}

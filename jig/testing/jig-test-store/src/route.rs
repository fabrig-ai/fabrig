//! The reference root's store transport (protocol/store/README.md, 3 and protocol/store/loads.md).
use crate::{StoreOut, Up, keys, live, records};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use jig_domain_core as core;
use jig_domain_core::engine::store as engine_store;
use jig_protocol_store::{self as store, Event, Request, Restore, Row};
use jig_test_domain as root;
use skein_lib::{List, Queue, Token, length::len_u64};

/// An ordered owner table that grows with admitted store operations.
#[derive(Debug)]
pub struct Routes {
    pending: BTreeMap<Token, Flight>,
    next: u64,
    commit: u64,
    limits: Option<root::Limits>,
    restore_bytes: Option<u64>,
}
#[derive(Debug)]
enum Flight {
    Start { phase: core::start::Phase, census: Option<core::Census> },
    Read(Token),
    Notes(Token),
    Point { load: Token, pending: store::ended::Pending },
    Window { load: Token, task: u64, attempt: u64, turn: u32 },
}
impl Routes {
    /// The supplied size does not limit the retained continuations.
    #[must_use]
    pub fn new(_table: u32) -> Routes {
        Routes { pending: BTreeMap::new(), next: 0, commit: 0, limits: None, restore_bytes: None }
    }
    /// Give the routing the configured decode and page bounds before use.
    pub fn configure(&mut self, limits: &root::Limits) {
        assert!(self.pending.is_empty(), "configure before routing requests");
        self.limits = Some(*limits);
    }
    /// Give restart its writer-sized page allowance before routing requests.
    pub fn configure_restore(&mut self, limits: &store::Limits) -> Option<()> {
        assert!(self.pending.is_empty(), "configure before routing requests");
        self.restore_bytes = Some(store::sizes::restore_bytes(limits)?);
        Some(())
    }
    /// The same decode and page bounds, with no requests or continuations.
    #[must_use]
    pub fn fresh(&self) -> Routes {
        Routes { limits: self.limits, restore_bytes: self.restore_bytes, ..Self::new(0) }
    }
    /// Whether every routed store request has terminated.
    #[must_use]
    pub fn idle(&self) -> bool {
        self.pending.is_empty()
    }
    fn token(&mut self) -> Token {
        self.next = self.next.checked_add(1).expect("store route owner exhausted");
        Token::new(self.next)
    }
    fn keep(&mut self, flight: Flight) -> Token {
        let owner = self.token();
        assert!(self.pending.insert(owner, flight).is_none(), "unique store route owner");
        owner
    }
}
/// Translate one root request, retaining only its continuation's identity.
#[must_use]
pub fn down(routes: &mut Routes, out: StoreOut) -> Option<Request> {
    let limits = routes.limits?;
    routes.restore_bytes.expect("configure restore before routing requests");
    Some(match out {
        StoreOut::Commit { number, writes } => {
            let mut translated = List::with_capacity(u32::try_from(writes.len()).ok()?);
            for write in writes {
                let write = match write {
                    root::Write::DropHistory { task } => store::Write::DropHistory { task },
                    root::Write::DropTranscript { task } => store::Write::DropTranscript { task },
                    root::Write::Wires => store::Write::Wires,
                    root::Write::CopySettingsRevision(number) => {
                        let (from, to) = core::settings::history_copy(number);
                        store::Write::Copy { from, to }
                    }
                    root::Write::Save(root::Record::Core(row)) => store::Write::Save(row),
                    root::Write::Erase(root::Key::Core(key)) => store::Write::Erase(key),
                    root::Write::Save(root::Record::Connector { number, record }) => {
                        store::Write::Put { key: keys::encode(number, &record.key()), value: records::encode(&record) }
                    }
                    root::Write::Erase(root::Key::Connector { number, key }) => {
                        store::Write::Remove { key: keys::encode(number, &key) }
                    }
                };
                translated.push(write).expect("same number of writes");
            }
            routes.commit = number;
            Request::Commit { number, writes: translated.into_boxed() }
        }
        StoreOut::Start { phase, census } => {
            use core::start::Phase;
            match phase {
                Phase::LoadCore | Phase::RestoreConnector { .. } | Phase::Sweep | Phase::Switch => {}
                Phase::LoadInbox => {
                    census.as_ref()?;
                }
                Phase::Link | Phase::Wires | Phase::AdoptRuns | Phase::Open => return None,
            }
            let owner = routes.keep(Flight::Start { phase, census });
            if phase == Phase::Switch {
                return Some(Request::Switch { owner });
            }
            restart(routes, owner, None)?
        }
        StoreOut::Engine(engine_store::Out::Load { load, range: engine_store::Range::Read(range) }) => {
            let owner = routes.keep(Flight::Read(load));
            Request::Read { owner, range: *range }
        }
        StoreOut::Engine(engine_store::Out::Load {
            load,
            range: range @ (engine_store::Range::Record(_) | engine_store::Range::Proposal { .. }),
        }) => {
            let owner = routes.token();
            let (request, pending) = store::ended::start(owner, range).expect("point range");
            assert!(
                routes.pending.insert(owner, Flight::Point { load, pending }).is_none(),
                "unique store route owner"
            );
            request
        }
        StoreOut::Engine(engine_store::Out::Load {
            load,
            range: engine_store::Range::Window { task, from: (attempt, turn) },
        }) => {
            let owner = routes.keep(Flight::Window { load, task, attempt, turn });
            transcript(routes, owner, None)?
        }
        StoreOut::Engine(engine_store::Out::Load { load, range: engine_store::Range::Notes(range) }) => {
            let owner = routes.keep(Flight::Notes(load));
            Request::Load {
                owner,
                range: store::Load::Notes(range),
                after: None,
                rows: limits.core.notes.load_rows,
                bytes: limits.core.load_bytes,
            }
        }
    })
}
fn restart(routes: &Routes, owner: Token, after: Option<store::Cursor>) -> Option<Request> {
    use core::start::Phase;
    let (phase, census) = match routes.pending.get(&owner)? {
        Flight::Start { phase, census } => (*phase, census),
        Flight::Read(_) | Flight::Notes(_) | Flight::Point { .. } | Flight::Window { .. } => return None,
    };
    let range = match phase {
        Phase::LoadCore => Restore::Jig(Box::new(core::RestoreRange::LIVE)),
        Phase::LoadInbox => Restore::Jig(Box::new([core::RestoreRange::Census(census.clone()?)])),
        Phase::RestoreConnector { connector } => Restore::Application(Box::new([live(connector)])),
        Phase::Sweep => Restore::Jig(Box::new([core::RestoreRange::People])),
        Phase::Link | Phase::Switch | Phase::Wires | Phase::AdoptRuns | Phase::Open => return None,
    };
    Some(Request::Restore {
        owner,
        range,
        after,
        rows: 2,
        bytes: routes.restore_bytes.expect("configure restore before routing requests"),
    })
}
fn transcript(routes: &Routes, owner: Token, after: Option<u64>) -> Option<Request> {
    let (task, attempt, turn) = match routes.pending.get(&owner)? {
        Flight::Window { task, attempt, turn, .. } => (task, attempt, turn),
        Flight::Start { .. } | Flight::Read(_) | Flight::Notes(_) | Flight::Point { .. } => {
            return None;
        }
    };
    let limits = routes.limits?;
    Some(Request::Transcript {
        owner,
        read: store::TranscriptRead::Window {
            task: *task,
            attempt: *attempt,
            turn: *turn,
            after,
            limits: Some(store::WindowPage { row_bytes: u32::MAX, bytes: u32::MAX, turn_overhead: 0 }),
        },
        rows: 2,
        bytes: u64::from(limits.core.resume_bytes),
    })
}
/// Translate a terminal, preserving pages and requesting their continuations.
pub fn up(routes: &mut Routes, event: Event, into: &mut Queue<Up>) {
    match event {
        Event::Page { owner, items, next } => page(routes, owner, items, next, into),
        Event::PageRefused { owner, failure } => {
            if let Some(Flight::Start { .. }) = routes.pending.remove(&owner) {
                into.push(Up::Root(root::Event::StartPageRefused { failure }));
            }
        }
        // This route has no job handlers; answer through the ordinary request path.
        Event::Job { owner, .. } => {
            into.push(Up::Ask(Request::Job {
                owner,
                answer: Err(store::JobRefusal { key: None, why: Box::from("no such job in this shell") }),
            }));
        }
        Event::Committed { number, fill } => into.push(Up::Root(root::Event::Committed { number, fill })),
        Event::Failed { number, .. } | Event::FrameTooLarge { number, .. } => {
            into.push(Up::Root(root::Event::Failed { number }));
        }
        Event::Read { owner, result } => {
            if let Some(Flight::Read(load)) = routes.pending.remove(&owner) {
                into.push(Up::Root(root::Event::Store(engine_store::In::Loaded { load, result })));
            }
        }
        Event::Loaded { owner, rows, next } => loaded(routes, owner, rows, next, into),
        Event::Got { owner, row } => {
            if let Some(Flight::Point { load, pending }) = routes.pending.remove(&owner) {
                match store::ended::got(owner, pending, row) {
                    store::ended::Step::Get(request) => {
                        assert!(
                            routes
                                .pending
                                .insert(owner, Flight::Point { load, pending: store::ended::Pending::Record })
                                .is_none(),
                            "point route retained"
                        );
                        into.push(Up::Ask(request));
                    }
                    store::ended::Step::Answer(result) => {
                        into.push(Up::Root(root::Event::Store(engine_store::In::Loaded { load, result })));
                    }
                }
            }
        }
        Event::Turns { owner, turns, next, cut } => window_loaded(routes, owner, turns, next, cut, into),
        Event::Unread { owner } => window_failed(routes, owner, engine_store::Failure::Unread, into),
        Event::WindowFailed { owner } => window_failed(routes, owner, engine_store::Failure::Failed, into),
        Event::SecretsNotReady { .. } => unreachable!("the root store route handles public records only"),
        Event::Corrupt { owner } => {
            if let Some(Flight::Point { load, .. } | Flight::Notes(load)) = routes.pending.remove(&owner) {
                let result = engine_store::Answer::Failed(engine_store::Failure::Failed);
                into.push(Up::Root(root::Event::Store(engine_store::In::Loaded { load, result })));
            } else {
                stop(routes, into);
            }
        }
        Event::OpenRefused { .. } => stop(routes, into),
        Event::Switched { owner } => switched(routes, owner, into),
        // The shell driving open owns progress; this root route prints no lines.
        // This reference route does not consume the census (protocol/store/growth.md, 1.3).
        Event::Census { .. }
        | Event::Checkpointed { .. }
        | Event::CheckpointRefused { .. }
        | Event::Restored { .. }
        | Event::RestoreRefused { .. }
        | Event::CheckpointHeld { .. }
        | Event::CheckpointRemoved { .. }
        | Event::Decoded { .. }
        | Event::Undecodable { .. }
        | Event::Rehearsing { .. }
        | Event::Back { .. }
        | Event::BackRefused { .. }
        | Event::Closed { .. }
        | Event::Ran { .. }
        | Event::Opened { .. }
        | Event::Kept { .. }
        | Event::Carried { .. }
        | Event::Stale { .. }
        | Event::Erased { .. }
        | Event::Secret { .. }
        | Event::SecretsLoaded { .. } => {}
    }
}
fn window_failed(routes: &mut Routes, owner: Token, failure: engine_store::Failure, into: &mut Queue<Up>) {
    if let Some(Flight::Window { load, .. }) = routes.pending.remove(&owner) {
        into.push(Up::Root(root::Event::Store(engine_store::In::Loaded {
            load,
            result: engine_store::Answer::Failed(failure),
        })));
    }
}

fn window_loaded(
    routes: &mut Routes,
    owner: Token,
    turns: Box<[core::TurnRecord]>,
    next: Option<u64>,
    cut: Option<u32>,
    into: &mut Queue<Up>,
) {
    let Some(Flight::Window { load, .. }) = routes.pending.get(&owner) else {
        return;
    };
    let result = if let Some(bytes) = cut {
        let mut turns = turns.into_iter();
        let turn = turns.next().expect("a cut page has one turn");
        assert!(turns.next().is_none(), "a cut turn is alone in its page");
        engine_store::Answer::Cut { turn, bytes: u64::from(bytes), more: next.is_some() }
    } else {
        let mut rows = List::with_capacity(u32::try_from(turns.len()).expect("bounded store page"));
        for turn in turns {
            let limits = jig_protocol_transcript::Limits::new(
                u32::try_from(turn.transcript.len()).expect("stored turn bound"),
                jig_protocol_transcript::TurnLimits {
                    messages: u32::MAX,
                    blocks: u32::MAX,
                    text: u32::try_from(turn.transcript.len()).expect("stored turn bound"),
                    outcome: u32::try_from(turn.transcript.len()).expect("stored turn bound"),
                },
            );
            let text = jig_protocol_transcript::brief_text(&limits, &turn.transcript).ok();
            rows.push((turn, text)).expect("bounded store page");
        }
        engine_store::Answer::Rows(engine_store::Rows::Turns { turns: rows.into_boxed(), more: next.is_some() })
    };
    into.push(Up::Root(root::Event::Store(engine_store::In::Loaded { load: *load, result })));
    if let Some(after) = next {
        if let Some(request) = transcript(routes, owner, Some(after)) {
            into.push(Up::Ask(request));
        }
    } else {
        routes.pending.remove(&owner);
    }
}

fn switched(routes: &mut Routes, owner: Token, into: &mut Queue<Up>) {
    match routes.pending.remove(&owner) {
        Some(Flight::Start { phase: core::start::Phase::Switch, .. }) => {
            into.push(Up::Root(root::Event::StartDone(core::start::Phase::Switch)));
        }
        Some(
            Flight::Start { .. } | Flight::Read(_) | Flight::Notes(_) | Flight::Point { .. } | Flight::Window { .. },
        )
        | None => {}
    }
}
fn loaded(routes: &mut Routes, owner: Token, rows: Box<[Row]>, next: Option<store::Cursor>, into: &mut Queue<Up>) {
    if let Some(Flight::Notes(load)) = routes.pending.remove(&owner) {
        let result = store::notes_answer(rows, next.is_some());
        into.push(Up::Root(root::Event::Store(engine_store::In::Loaded { load, result })));
    }
}
fn decode(row: Row, limits: &root::Limits) -> Result<root::Record, store::restore::Failure> {
    use store::restore::{Cause, DecodeFailure, Failure, RowSite};
    Ok(match row {
        Row::Jig(row) => root::Record::Core(row),
        Row::Application { key, value } => {
            let site = RowSite {
                family: Some(store::Family::Application),
                key: store::keys::application(&key, u32::MAX).expect("bounded application key"),
                key_bytes: len_u64(key.len()).checked_add(1).expect("bounded tagged key"),
                truncated: false,
            };
            let Some((number, _)) = keys::decode(&key, &limits.connector) else {
                return Err(Failure { cause: Cause::Decode(DecodeFailure::Key), row: Some(site) });
            };
            let Ok(record) = records::decode(&value, &limits.connector) else {
                return Err(Failure { cause: Cause::Decode(DecodeFailure::Value), row: Some(site) });
            };
            if keys::encode(number, &record.key()) != key {
                return Err(Failure { cause: Cause::Decode(DecodeFailure::KeyValueDisagree), row: Some(site) });
            }
            root::Record::Connector { number, record }
        }
    })
}
fn page(
    routes: &mut Routes,
    owner: Token,
    items: Box<[store::restore::Item]>,
    next: Option<store::Cursor>,
    into: &mut Queue<Up>,
) {
    let Some(Flight::Start { phase, .. }) = routes.pending.get(&owner) else {
        return;
    };
    let phase = *phase;
    let limits = routes.limits.expect("configured routing");
    for item in items {
        let event = match item {
            store::restore::Item::Row(row) => {
                let record = match decode(row, &limits) {
                    Ok(record) => record,
                    Err(failure) => {
                        routes.pending.remove(&owner);
                        into.push(Up::Root(root::Event::StartPageRefused { failure }));
                        return;
                    }
                };
                root::Event::StartRecord(record)
            }
            store::restore::Item::MissingLink { record, target } => root::Event::StartMissing { record, target },
        };
        into.push(Up::Root(event));
    }
    if let Some(after) = next {
        if let Some(request) = restart(routes, owner, Some(after)) {
            into.push(Up::Ask(request));
        }
    } else {
        routes.pending.remove(&owner);
        into.push(Up::Root(root::Event::StartDone(phase)));
    }
}
fn stop(routes: &Routes, into: &mut Queue<Up>) {
    into.push(Up::Root(root::Event::Failed { number: routes.commit }));
}

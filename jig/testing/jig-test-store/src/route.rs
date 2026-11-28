//! The reference root's store transport (protocol/store/README.md, 3 and protocol/store/loads.md).
use crate::{StoreOut, Up, keys, live, records};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use jig_domain_core as core;
use jig_domain_core::engine::store as engine_store;
use jig_protocol_store::{self as store, Event, Request, Restore, Row};
use jig_test_connector as connector;
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
    Start(Token),
    Connector(u16),
    Switch,
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
/// A switch with no pending generation instead replies directly into `into`.
#[expect(clippy::manual_map, reason = "the step subset translates cursors without closures")]
#[must_use]
pub fn down(routes: &mut Routes, out: StoreOut, generation_pending: bool, into: &mut Queue<Up>) -> Option<Request> {
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
        StoreOut::ConnectorLoad { number, after } => {
            let owner = routes.keep(Flight::Connector(number));
            let after = match after {
                Some(after) => Some(store::Cursor { interval: 0, after }),
                None => None,
            };
            restore(routes, owner, Restore::Application(Box::new([live(number)])), after)
        }
        StoreOut::Engine(engine_store::Out::Switch) => {
            if !generation_pending {
                into.push(Up::Root(root::Event::Store(engine_store::In::Switched)));
                return None;
            }
            Request::Switch { owner: routes.keep(Flight::Switch) }
        }
        StoreOut::Engine(engine_store::Out::Load { load, range: engine_store::Range::Start { ranges, after } }) => {
            let owner = routes.keep(Flight::Start(load));
            let after = match after {
                Some(cursor) => Some(store::Cursor { interval: cursor.interval, after: cursor.after }),
                None => None,
            };
            restore(routes, owner, Restore::Jig(ranges), after)
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
fn restore(routes: &Routes, owner: Token, range: Restore, after: Option<store::Cursor>) -> Request {
    Request::Restore {
        owner,
        range,
        after,
        rows: 2,
        bytes: routes.restore_bytes.expect("configure restore before routing requests"),
    }
}
fn transcript(routes: &Routes, owner: Token, after: Option<u64>) -> Option<Request> {
    let (task, attempt, turn) = match routes.pending.get(&owner)? {
        Flight::Window { task, attempt, turn, .. } => (task, attempt, turn),
        Flight::Start(_)
        | Flight::Connector(_)
        | Flight::Switch
        | Flight::Read(_)
        | Flight::Notes(_)
        | Flight::Point { .. } => {
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
        Event::PageRefused { owner, failure } => match routes.pending.remove(&owner) {
            Some(Flight::Start(load)) => refused(Some(load), failure, into),
            Some(Flight::Connector(_)) => refused(None, failure, into),
            Some(
                Flight::Switch | Flight::Read(_) | Flight::Notes(_) | Flight::Point { .. } | Flight::Window { .. },
            )
            | None => {}
        },
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
        Event::Census { census } => into.push(Up::Root(root::Event::Store(engine_store::In::Census(census)))),
        // The shell driving open owns progress; this root route prints no lines.
        Event::Checkpointed { .. }
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
    if let Some(Flight::Switch) = routes.pending.remove(&owner) {
        into.push(Up::Root(root::Event::Store(engine_store::In::Switched)));
    }
}
fn loaded(routes: &mut Routes, owner: Token, rows: Box<[Row]>, next: Option<store::Cursor>, into: &mut Queue<Up>) {
    if let Some(Flight::Notes(load)) = routes.pending.remove(&owner) {
        let result = store::notes_answer(rows, next.is_some());
        into.push(Up::Root(root::Event::Store(engine_store::In::Loaded { load, result })));
    }
}
fn decode(
    key: &[u8],
    value: &[u8],
    limits: &connector::Limits,
) -> Result<(u16, connector::Record), store::restore::Failure> {
    use store::restore::{Cause, DecodeFailure, Failure, RowSite};
    let site = RowSite {
        family: Some(store::Family::Application),
        key: store::keys::application(key, u32::MAX).expect("bounded application key"),
        key_bytes: len_u64(key.len()).checked_add(1).expect("bounded tagged key"),
        truncated: false,
    };
    let Some((number, _)) = keys::decode(key, limits) else {
        return Err(Failure { cause: Cause::Decode(DecodeFailure::Key), row: Some(site) });
    };
    let Ok(record) = records::decode(value, limits) else {
        return Err(Failure { cause: Cause::Decode(DecodeFailure::Value), row: Some(site) });
    };
    if keys::encode(number, &record.key()).as_ref() != key {
        return Err(Failure { cause: Cause::Decode(DecodeFailure::KeyValueDisagree), row: Some(site) });
    }
    Ok((number, record))
}
fn page(
    routes: &mut Routes,
    owner: Token,
    items: Box<[store::restore::Item]>,
    next: Option<store::Cursor>,
    into: &mut Queue<Up>,
) {
    match routes.pending.remove(&owner) {
        Some(Flight::Start(load)) => start_page(load, items, next, into),
        Some(Flight::Connector(number)) => {
            let limits = routes.limits.expect("configured routing");
            connector_page(number, &limits, items, next, into);
        }
        Some(Flight::Switch | Flight::Read(_) | Flight::Notes(_) | Flight::Point { .. } | Flight::Window { .. })
        | None => {}
    }
}
fn start_page(load: Token, items: Box<[store::restore::Item]>, next: Option<store::Cursor>, into: &mut Queue<Up>) {
    let mut page = List::with_capacity(u32::try_from(items.len()).expect("bounded page"));
    for item in items {
        let item = match item {
            store::restore::Item::Row(Row::Jig(row)) => engine_store::Item::Record(row),
            store::restore::Item::MissingLink { record, target } => engine_store::Item::Missing { record, target },
            store::restore::Item::Row(Row::Application { .. }) => {
                refused(Some(load), wrong_range(), into);
                return;
            }
        };
        page.push(item).expect("one item per row");
    }
    let next = match next {
        Some(cursor) => Some(engine_store::Cursor { interval: cursor.interval, after: cursor.after }),
        None => None,
    };
    let result = engine_store::Answer::Rows(engine_store::Rows::Start { items: page.into_boxed(), next });
    into.push(Up::Root(root::Event::Store(engine_store::In::Loaded { load, result })));
}
fn connector_page(
    number: u16,
    limits: &root::Limits,
    items: Box<[store::restore::Item]>,
    next: Option<store::Cursor>,
    into: &mut Queue<Up>,
) {
    let mut records = List::with_capacity(u32::try_from(items.len()).expect("bounded page"));
    for item in items {
        let (key, value) = match item {
            store::restore::Item::Row(Row::Application { key, value }) => (key, value),
            store::restore::Item::Row(Row::Jig(_)) | store::restore::Item::MissingLink { .. } => {
                refused(None, wrong_range(), into);
                return;
            }
        };
        match decode(&key, &value, &limits.connector) {
            Ok((owner, record)) if owner == number => {
                records.push(record).expect("one entry per row");
            }
            Ok(_) => {
                refused(None, wrong_range(), into);
                return;
            }
            Err(failure) => {
                refused(None, failure, into);
                return;
            }
        }
    }
    let after = match next {
        Some(store::Cursor { interval: 0, after }) => Some(after),
        Some(_) => {
            refused(None, wrong_range(), into);
            return;
        }
        None => None,
    };
    into.push(Up::Root(root::Event::ConnectorPage { number, records: records.into_boxed(), after }));
}
fn wrong_range() -> store::restore::Failure {
    use store::restore::{Cause, Failure, WalkFailure};
    Failure { cause: Cause::Walk(WalkFailure::UnexpectedTerminal), row: None }
}
fn refused(load: Option<Token>, failure: store::restore::Failure, into: &mut Queue<Up>) {
    into.push(Up::Root(root::Event::StartPageRefused { load, failure }));
}
fn stop(routes: &Routes, into: &mut Queue<Up>) {
    into.push(Up::Root(root::Event::Failed { number: routes.commit }));
}

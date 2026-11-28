//! Root store requests and terminals, with temper's decoded families.
//! Checkpoints: temper/docs/design/protocol/checkpoints.md, 3.
//! Window pages carry whole or cut turns (temper/docs/design/domain/root.md, 7).
use super::{
    UnreadableRow, ranges,
    window::{self, Window},
};
use ::core::mem::size_of;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_core::engine::store as engine_store;
use jig_protocol_store as store;
use skein_lib::{List, Map, Queue, Token};
use temper_engine_domain as engine;
use temper_protocol_store as families;

/// A bounded load table and transcript continuations, never decoded rows.
#[derive(Debug)]
pub struct Routes {
    random: skein_lib::CryptoRng,
    pending: Map<Token, Flight>,
    reads: Map<Token, (Token, bool)>,
    points: Map<Token, Point>,
    notes: Option<NotePage>,
    windows: Map<Token, Window>,
    next: u64,
    commit: u64,
    limits: engine::loads::Limits,
    note_rows: u32,
    families: families::Bounds,
    page_failure: Option<store::restore::Failure>,
    row_failure: Option<UnreadableRow>,
}
#[derive(Debug)]
struct Point {
    load: Token,
    pending: store::ended::Pending,
}

#[derive(Debug)]
struct NotePage {
    owner: Token,
    load: Token,
    rows: u32,
    single: bool,
}

#[derive(Debug)]
pub(super) struct Flight {
    root: Token,
    pub(super) range: engine::Range,
    pub(super) after: Option<engine::Key>,
    pub(super) most: u32,
    pub(super) bytes: u32,
    pub(super) offset: Option<u64>,
    waiting: bool,
    pub(super) cursor: Option<store::Cursor>,
}
impl Routes {
    /// Raise application decoders before any work uses the raised domain limits.
    pub fn raise_limits(&mut self, forge: &temper_engine_domain_forge::Limits) {
        self.families.raise(forge);
    }
    /// Size the continuation table from the root's admitted loads.
    #[must_use]
    pub fn new(
        table: u32,
        limits: &engine::Limits,
        families: families::Bounds,
        random: skein_lib::CryptoRng,
    ) -> Routes {
        Routes {
            random,
            pending: Map::with_capacity(table),
            reads: Map::with_capacity(table),
            points: Map::with_capacity(table),
            notes: None,
            windows: Map::uncapped(),
            next: 0,
            commit: 0,
            limits: limits.loads,
            note_rows: limits.notes.load_rows.min(limits.loads.rows),
            families,
            page_failure: None,
            row_failure: None,
        }
    }
    /// The store's typed cause for the refused startup page.
    #[must_use]
    pub fn page_failure(&self) -> Option<&store::restore::Failure> {
        self.page_failure.as_ref()
    }
    /// The application row refused before the root heard it.
    #[must_use]
    pub fn row_failure(&self) -> Option<&UnreadableRow> {
        self.row_failure.as_ref()
    }
    /// Whether every routed load has had its terminal.
    #[must_use]
    pub fn idle(&self) -> bool {
        if !self.reads.is_empty() || !self.points.is_empty() || !self.windows.is_empty() || self.notes.is_some() {
            return false;
        }
        for (_, flight) in &self.pending {
            if flight.waiting {
                return false;
            }
        }
        true
    }
    fn token(&mut self) -> Option<Token> {
        self.next = self.next.checked_add(1)?;
        Some(Token::new(self.next))
    }
    fn continuation(&mut self, range: &engine::Range, after: Option<&engine::Key>) -> Option<u64> {
        let mut found = None;
        let mut retired = None;
        for (owner, flight) in &self.pending {
            if !flight.waiting {
                retired = Some(*owner);
                if flight.range == *range && flight.after.as_ref() == after {
                    found = Some(*owner);
                    break;
                }
            }
        }
        if let Some(owner) = found {
            return self.pending.remove(&owner)?.offset;
        }
        // A continuation that is never asked again must not consume an IO slot.
        if let Some(owner) = retired {
            self.pending.remove(&owner);
        }
        None
    }
}

fn opening_to_store(opening: engine::WorkerOpening) -> store::WorkerOpening {
    let mut wires = List::with_capacity(u32::try_from(opening.statement.wires.len()).expect("opening wires fit u32"));
    for wire in opening.statement.wires {
        wires
            .push(store::OpenedWire { name: wire.name, layout: wire.layout, text: wire.text })
            .expect("one stored wire per opening wire");
    }
    let mut terms = List::with_capacity(u32::try_from(opening.statement.terms.len()).expect("opening terms fit u32"));
    for term in opening.statement.terms {
        terms
            .push(store::OpenedTerm { kind: term.kind, largest: term.largest })
            .expect("one stored term per opening term");
    }
    store::WorkerOpening {
        worker: opening.worker,
        wires: wires.into_boxed(),
        output: opening.statement.output,
        terms: terms.into_boxed(),
        jobs: opening.statement.jobs,
        at: opening.at,
    }
}

/// Why a commit failed: the store's cause or its routing stopping on its own
/// (temper/docs/design/protocol/store-versions.md, 1). In-process only.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CommitStop {
    Store(store::CommitFailure),
    Route(RouteStop),
}

/// Why the engine's store routing stopped (temper/docs/design/protocol/store-versions.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RouteStop {
    /// The store answered a refused open while the engine ran.
    Open,
    /// The store answered its secrets not ready.
    Secrets,
    /// A load's continuation did not translate into its cursor, or did not advance.
    Cursor,
    /// A load's page past the start could not be measured or cut.
    Page,
}

/// A root terminal or the next component page of a filtered load.
#[derive(Debug)]
pub enum Up {
    Root(Box<engine::Event>),
    Ask(store::Request),
    FrameTooLarge {
        number: u64,
        task: u64,
        bytes: u32,
        limit: u32,
    },
    /// Feed the number through the root's Uncommitted policy before stopping.
    Failed {
        number: u64,
        why: CommitStop,
    },
}

/// Route one root request; table saturation refuses before issuing a load.
#[expect(clippy::too_many_lines, reason = "the store adapter lists each routed request and its bound together")]
#[must_use]
pub fn down(routes: &mut Routes, request: engine::StoreRequest) -> Option<store::Request> {
    Some(match request {
        engine::StoreRequest::Checkpoint { token } => {
            let mut draw = [0; 16];
            routes.random.fill(&mut draw);
            store::Request::Checkpoint { owner: token, draw }
        }
        engine::StoreRequest::Catalogue { .. } => unreachable!("the store owner replaces its catalogue directly"),
        engine::StoreRequest::Switch => store::Request::Switch { owner: Token::new(u64::MAX - 2) },
        engine::StoreRequest::Commit { number, writes } => {
            // Every input write translates to one store write or refuses the batch.
            let mut out = List::allocated(u32::try_from(writes.len()).ok()?);
            for write in writes {
                let translated = match write {
                    engine::Write::DropHistory { task } => store::Write::DropHistory { task },
                    engine::Write::DropTranscript { task } => store::Write::DropTranscript { task },
                    engine::Write::Wires => store::Write::Wires,
                    engine::Write::Erase(engine::Key::Wires | engine::Key::Opening(_)) => return None,
                    engine::Write::Opening(opening) => store::Write::Opening(opening_to_store(opening)),
                    engine::Write::CopySettingsRevision(number) => {
                        let (from, to) = core::settings::history_copy(number);
                        store::Write::Copy { from, to }
                    }
                    engine::Write::Save(engine::Record::Core(row)) => store::Write::Save(row),
                    engine::Write::Erase(engine::Key::Core(key)) => store::Write::Erase(key),
                    engine::Write::Save(engine::Record::Forge { row }) => store::Write::Put {
                        key: families::keys::encode(&temper_engine_domain_forge::stored_key(&row), u32::MAX)?,
                        value: families::records::encode(&row),
                    },
                    engine::Write::Erase(engine::Key::Forge(key)) => {
                        store::Write::Remove { key: families::keys::encode(&key, u32::MAX)? }
                    }
                };
                out.push(translated).expect("same write count");
            }
            routes.commit = number;
            store::Request::Commit { number, writes: out.into_boxed() }
        }
        engine::StoreRequest::PartRead { waiter, range } => {
            let owner = routes.token()?;
            routes.reads.insert(owner, (waiter, true)).ok()?;
            store::Request::Read { owner, range: *range }
        }
        engine::StoreRequest::Engine(
            engine_store::Out::Switch | engine_store::Out::Load { range: engine_store::Range::Start { .. }, .. },
        ) => {
            unreachable!("the root drives its own start")
        }
        engine::StoreRequest::Engine(engine_store::Out::Load { load, range: engine_store::Range::Read(range) }) => {
            let owner = routes.token()?;
            routes.reads.insert(owner, (load, false)).ok()?;
            store::Request::Read { owner, range: *range }
        }
        engine::StoreRequest::Engine(engine_store::Out::Load {
            load,
            range: range @ (engine_store::Range::Record(_) | engine_store::Range::Proposal { .. }),
        }) => {
            let owner = routes.token()?;
            let (request, pending) = store::ended::start(owner, range)?;
            routes.points.insert(owner, Point { load, pending }).ok()?;
            request
        }
        engine::StoreRequest::Engine(engine_store::Out::Load {
            load,
            range: engine_store::Range::Window { task, from },
        }) => {
            let owner = routes.token()?;
            let window = Window { load, task, from };
            let request = window::request(owner, &window, None, &routes.limits);
            assert!(
                routes.windows.insert(owner, window).expect("window correlation room").is_none(),
                "unique IO owner"
            );
            request
        }
        engine::StoreRequest::Engine(engine_store::Out::Load { load, range: engine_store::Range::Notes(range) }) => {
            let owner = routes.token()?;
            let single = match &range {
                jig_domain_notes::Range::Entry { .. } => true,
                jig_domain_notes::Range::Lines { .. } | jig_domain_notes::Range::Project { .. } => false,
            };
            let rows = if single { 1 } else { routes.note_rows };
            let page = NotePage { owner, load, rows, single };
            assert!(routes.notes.replace(page).is_none(), "one notes page in flight");
            store::Request::Load {
                owner,
                range: store::Load::Notes(range),
                after: None,
                rows,
                bytes: u64::from(routes.limits.bytes),
            }
        }
        engine::StoreRequest::Load { owner: root, range, after, most, bytes } => {
            let offset = routes.continuation(&range, after.as_ref());
            let owner = routes.token()?;
            let flight = Flight { root, range, after, most, bytes, offset, waiting: true, cursor: None };
            let request = if ranges::starting(&flight.range) {
                ranges::request(owner, &flight).expect("usable translated startup request")
            } else {
                ranges::request(owner, &flight)?
            };
            routes.pending.insert(owner, flight).ok()?;
            request
        }
    })
}

/// Every store terminal is accepted independently of journal room.
pub fn up(routes: &mut Routes, event: store::Event, into: &mut Queue<Up>) {
    match event {
        store::Event::Page { owner, items, next } => {
            startup_page(routes, owner, items, next, into);
        }
        store::Event::PageRefused { owner, failure } => {
            if let Some(flight) = routes.pending.remove(&owner) {
                routes.page_failure = Some(failure);
                root(into, engine::StoreInput::Unloaded { owner: flight.root });
            }
        }
        store::Event::Checkpointed { owner, checkpoint } => {
            root(into, engine::StoreInput::Checkpointed { token: owner, checkpoint: checkpointed(checkpoint) });
        }
        store::Event::CheckpointRefused { owner, why } => {
            root(into, engine::StoreInput::CheckpointRefused { token: owner, why: checkpoint_why(why) });
        }
        store::Event::Census { census } => root(into, engine::StoreInput::Census(census)),
        store::Event::Committed { number, fill } => root(into, engine::StoreInput::Committed { number, fill }),
        store::Event::Failed { number, why } => into.push(Up::Failed { number, why: CommitStop::Store(why) }),
        store::Event::FrameTooLarge { number, task, bytes, limit } => {
            into.push(Up::FrameTooLarge { number, task, bytes, limit });
        }
        store::Event::Loaded { owner, rows, next } => loaded(routes, owner, rows, next, into),
        store::Event::Got { owner, row } => {
            if let Some(Point { load, pending }) = routes.points.remove(&owner) {
                match store::ended::got(owner, pending, row) {
                    store::ended::Step::Get(request) => {
                        routes
                            .points
                            .insert(owner, Point { load, pending: store::ended::Pending::Record })
                            .expect("retained point route");
                        into.push(Up::Ask(request));
                    }
                    store::ended::Step::Answer(result) => {
                        root(into, engine::StoreInput::Engine(engine_store::In::Loaded { load, result }));
                    }
                }
                return;
            }
            let rows: Box<[store::Row]> = match row {
                Some(row) => Box::new([row]),
                None => Box::new([]),
            };
            page(routes, owner, rows, None, None, into);
        }
        store::Event::Turns { owner, turns, next, cut } => {
            if let Some(window) = routes.windows.remove(&owner) {
                window_loaded(routes, owner, window, turns, next, cut, into);
                return;
            }
            assert!(cut.is_none(), "only window pages may cut a turn");
            let mut rows = List::with_capacity(u32::try_from(turns.len()).expect("bounded turns"));
            for turn in turns {
                rows.push(store::Row::Jig(core::Record::Core(core::CoreRecord::Turn(turn)))).expect("same count");
            }
            page(routes, owner, rows.into_boxed(), None, next, into);
        }
        store::Event::Unread { owner } => unread(routes, owner, engine_store::Failure::Unread, into),
        store::Event::WindowFailed { owner } | store::Event::Corrupt { owner } => {
            unread(routes, owner, engine_store::Failure::Failed, into);
        }
        store::Event::Read { owner, result } => {
            if let Some((load, parts)) = routes.reads.remove(&owner) {
                if parts {
                    root(into, engine::StoreInput::PartLoaded { waiter: load, result });
                } else {
                    root(into, engine::StoreInput::Engine(engine_store::In::Loaded { load, result }));
                }
            }
        }
        store::Event::Job { owner, of, job, page } => {
            into.push(Up::Ask(job_answer(owner, of, &job, &page, &routes.families)));
        }
        store::Event::Switched { .. } => root(into, engine::StoreInput::Switched),
        store::Event::OpenRefused { .. } => stop(routes, RouteStop::Open, into),
        store::Event::SecretsNotReady { .. } => stop(routes, RouteStop::Secrets, into),
        store::Event::Decoded { .. }
        | store::Event::Restored { .. }
        | store::Event::RestoreRefused { .. }
        | store::Event::CheckpointHeld { .. }
        | store::Event::CheckpointRemoved { .. }
        | store::Event::Undecodable { .. }
        | store::Event::Rehearsing { .. }
        | store::Event::Back { .. }
        | store::Event::BackRefused { .. }
        | store::Event::Closed { .. }
        | store::Event::Ran { .. }
        | store::Event::Opened { .. }
        | store::Event::Kept { .. }
        | store::Event::Carried { .. }
        | store::Event::Stale { .. }
        | store::Event::Erased { .. }
        | store::Event::Secret { .. }
        | store::Event::SecretsLoaded { .. } => {}
    }
}
fn startup_page(
    routes: &mut Routes,
    owner: Token,
    items: Box<[store::restore::Item]>,
    next: Option<store::Cursor>,
    into: &mut Queue<Up>,
) {
    let Some(flight) = routes.pending.get(&owner) else { return };
    let root_owner = flight.root;
    let mut rows = List::with_capacity(u32::try_from(items.len()).expect("bounded startup items"));
    for item in items {
        match item {
            store::restore::Item::Row(row) => rows.push(row).expect("one row per item"),
            store::restore::Item::MissingLink { record, target } => {
                root(into, engine::StoreInput::MissingLink { owner: root_owner, record, target });
            }
        }
    }
    page(routes, owner, rows.into_boxed(), next, None, into);
}
fn root(into: &mut Queue<Up>, event: engine::StoreInput) {
    into.push(Up::Root(Box::new(engine::Event::Store(event))));
}
fn window_loaded(
    routes: &mut Routes,
    owner: Token,
    window: Window,
    turns: Box<[core::TurnRecord]>,
    next: Option<u64>,
    cut: Option<u32>,
    into: &mut Queue<Up>,
) {
    let result = window::answer(turns, cut, next.is_some(), &routes.limits);
    let continues = match &result {
        engine_store::Answer::Rows(_) | engine_store::Answer::Cut { .. } => true,
        engine_store::Answer::Failed(_) => false,
    };
    engine_answer(into, window.load, result);
    if continues && let Some(after) = next {
        let request = window::request(owner, &window, Some(after), &routes.limits);
        assert!(routes.windows.insert(owner, window).expect("removed window").is_none(), "one window per owner");
        into.push(Up::Ask(request));
    }
}

fn engine_answer(into: &mut Queue<Up>, load: Token, result: engine_store::Answer) {
    root(into, engine::StoreInput::Engine(engine_store::In::Loaded { load, result }));
}

fn unread(routes: &mut Routes, owner: Token, failure: engine_store::Failure, into: &mut Queue<Up>) {
    if let Some(window) = routes.windows.remove(&owner) {
        engine_answer(into, window.load, engine_store::Answer::Failed(failure));
    } else if let Some(Point { load, .. }) = routes.points.remove(&owner) {
        engine_answer(into, load, engine_store::Answer::Failed(engine_store::Failure::Failed));
    } else if let Some(note) = take_note_load(routes, owner) {
        engine_answer(into, note.load, engine_store::Answer::Failed(engine_store::Failure::Failed));
    } else if let Some(flight) = routes.pending.remove(&owner) {
        root(into, engine::StoreInput::Unloaded { owner: flight.root });
    }
}

fn take_note_load(routes: &mut Routes, owner: Token) -> Option<NotePage> {
    if routes.notes.as_ref()?.owner == owner { routes.notes.take() } else { None }
}

fn loaded(
    routes: &mut Routes,
    owner: Token,
    rows: Box<[store::Row]>,
    next: Option<store::Cursor>,
    into: &mut Queue<Up>,
) {
    if let Some(note) = take_note_load(routes, owner) {
        let more = next.is_some();
        let result = if skein_lib::len_u64(rows.len()) > u64::from(note.rows) || (note.single && more) {
            engine_store::Answer::Failed(engine_store::Failure::Failed)
        } else {
            notes_answer(rows, more, &routes.limits)
        };
        engine_answer(into, note.load, result);
    } else {
        page(routes, owner, rows, next, None, into);
    }
}

fn notes_answer(rows: Box<[store::Row]>, more: bool, limits: &engine::loads::Limits) -> engine_store::Answer {
    let failed = engine_store::Answer::Failed(engine_store::Failure::Failed);
    let engine_store::Answer::Rows(engine_store::Rows::Records { records, more }) = store::notes_answer(rows, more)
    else {
        return failed;
    };
    let mut checked = List::with_capacity(u32::try_from(records.len()).expect("bounded page"));
    let mut bytes = 0;
    for record in records {
        let wrapped = engine::Record::Core(record);
        let Some((payload, total)) = page_bytes(&wrapped, bytes) else { return failed };
        if payload > u64::from(limits.transcript_bytes) || total > u64::from(limits.bytes.min(limits.reply_bytes)) {
            return failed;
        }
        bytes = total;
        let engine::Record::Core(record) = wrapped else { unreachable!("checked core record") };
        checked.push(record).expect("one checked record per row");
    }
    engine_store::Answer::Rows(engine_store::Rows::Records { records: checked.into_boxed(), more })
}

fn stop(routes: &Routes, why: RouteStop, into: &mut Queue<Up>) {
    into.push(Up::Failed { number: routes.commit, why: CommitStop::Route(why) });
}

fn page(
    routes: &mut Routes,
    owner: Token,
    rows: Box<[store::Row]>,
    cursor: Option<store::Cursor>,
    offset: Option<u64>,
    into: &mut Queue<Up>,
) {
    let more = cursor.is_some() || offset.is_some();
    let Some(mut flight) = routes.pending.remove(&owner) else {
        return;
    };
    let mut decoded = List::with_capacity(flight.most);
    let mut bytes = 0_u64;
    let mut cutting = false;
    let mut last = flight.after.clone();
    let mut scanned = flight.after.clone();
    for row in rows {
        let record = match decode(row, &routes.families) {
            Ok(record) => record,
            Err(failure) => {
                routes.row_failure = Some(failure);
                root(into, engine::StoreInput::Unloaded { owner: flight.root });
                return;
            }
        };
        let key = record.key();
        scanned = Some(key.clone());
        if !ranges::keeps(&flight.range, &key) {
            continue;
        }
        if !ranges::starting(&flight.range)
            && let Some(after) = &flight.after
        {
            let (Some(position), Some(start)) =
                (ranges::cursor(&flight.range, &key), ranges::cursor(&flight.range, after))
            else {
                stop(routes, RouteStop::Cursor, into);
                return;
            };
            if (position.interval, &position.after) <= (start.interval, &start.after) {
                continue;
            }
        }
        let Some((payload_bytes, total)) = page_bytes(&record, bytes) else {
            page_failed(routes, &flight, into);
            return;
        };
        if cutting
            || decoded.len() >= flight.most
            || total > u64::from(flight.bytes)
            || payload_bytes > u64::from(routes.limits.transcript_bytes)
        {
            cutting = true;
            continue;
        }
        bytes = total;
        last = Some(key);
        decoded.push(record).expect("checked prefix");
    }
    if cutting && (ranges::starting(&flight.range) || decoded.is_empty()) {
        page_failed(routes, &flight, into);
        return;
    }
    let startup_next = if ranges::starting(&flight.range) {
        match &cursor {
            Some(cursor) => {
                let Some(key) = startup_cursor(&flight, cursor, &routes.families) else {
                    root(into, engine::StoreInput::Unloaded { owner: flight.root });
                    return;
                };
                Some(key)
            }
            None => None,
        }
    } else {
        None
    };
    if let engine::Range::Census(_) = &flight.range {
        root(into, engine::StoreInput::Loaded { owner: flight.root, rows: decoded.into_boxed(), next: startup_next });
        return;
    }
    if decoded.is_empty() && !cutting && more {
        continue_filtered(routes, owner, flight, scanned, cursor, offset, into);
        return;
    }
    let next = if ranges::starting(&flight.range) {
        startup_next
    } else if cutting || more {
        last
    } else {
        None
    };
    if offset.is_some() && !cutting {
        flight.after.clone_from(&next);
        flight.offset = offset;
        flight.waiting = false;
        routes.pending.insert(owner, flight).expect("removed slot");
        let root_owner = routes.pending.get(&owner).expect("kept continuation").root;
        root(into, engine::StoreInput::Loaded { owner: root_owner, rows: decoded.into_boxed(), next });
    } else {
        root(into, engine::StoreInput::Loaded { owner: flight.root, rows: decoded.into_boxed(), next });
    }
}

fn continue_filtered(
    routes: &mut Routes,
    owner: Token,
    mut flight: Flight,
    scanned: Option<engine::Key>,
    cursor: Option<store::Cursor>,
    offset: Option<u64>,
    into: &mut Queue<Up>,
) {
    if !ranges::starting(&flight.range)
        && let Some(next) = &cursor
        && (next.after.is_empty() || flight.cursor.as_ref() == Some(next))
    {
        stop(routes, RouteStop::Cursor, into);
        return;
    }
    flight.after = scanned;
    flight.cursor = cursor;
    flight.offset = offset;
    let request = ranges::request(owner, &flight);
    if ranges::starting(&flight.range) {
        assert!(request.is_some(), "usable translated continuation request");
    }
    if let Some(request) = request {
        routes.pending.insert(owner, flight).expect("removed slot");
        into.push(Up::Ask(request));
    } else {
        stop(routes, RouteStop::Cursor, into);
    }
}

/// Translate the store's continuation independently of returned row identities.
/// Filtered startup walks can advance past families not selected by this load.
fn startup_cursor(flight: &Flight, cursor: &store::Cursor, bounds: &families::Bounds) -> Option<engine::Key> {
    let key = match &flight.range {
        engine::Range::Forge | engine::Range::History(_) => {
            let relative = store::keys::of_application(&cursor.after)?;
            engine::Key::Forge(families::keys::decode(relative, bounds.forge())?)
        }
        engine::Range::Core(_) | engine::Range::Census(_) => engine::Key::Core(store::keys::decode(&cursor.after)?),
    };
    let translated = ranges::cursor(&flight.range, &key).expect("usable translated startup cursor");
    assert_eq!(translated, *cursor, "startup cursor keeps its walk interval");
    let start = match (&flight.cursor, &flight.after) {
        (Some(cursor), _) => Some(cursor.clone()),
        (None, Some(after)) => Some(ranges::cursor(&flight.range, after).expect("usable startup starting cursor")),
        (None, None) => None,
    };
    if let Some(start) = start {
        assert!(
            (cursor.interval, &cursor.after) > (start.interval, &start.after),
            "startup continuation advances in walk order"
        );
    }
    Some(key)
}

fn page_failed(routes: &Routes, flight: &Flight, into: &mut Queue<Up>) {
    if ranges::starting(&flight.range) {
        root(into, engine::StoreInput::Unloaded { owner: flight.root });
    } else {
        stop(routes, RouteStop::Page, into);
    }
}

fn decode(row: store::Row, bounds: &families::Bounds) -> Result<engine::Record, UnreadableRow> {
    match row {
        store::Row::Jig(row) => Ok(engine::Record::Core(row)),
        store::Row::Application { key, value } => match families::records::row(&key, &value, bounds) {
            Ok(row) => Ok(engine::Record::Forge { row: Box::new(row) }),
            Err(families::Undecodable::Setting(key)) => {
                Ok(engine::Record::Forge { row: Box::new(temper_engine_domain_forge::Stored::RefusedSetting(key)) })
            }
            Err(families::Undecodable::Malformed) => Err(UnreadableRow { key, lowered: None }),
            Err(families::Undecodable::Lowered(lowered)) => Err(UnreadableRow { key, lowered: Some(lowered) }),
        },
    }
}

fn page_bytes(record: &engine::Record, kept: u64) -> Option<(u64, u64)> {
    let payload = engine::record_bytes(record)?;
    let slot = u64::try_from(size_of::<engine::Record>()).ok()?.checked_add(payload)?;
    Some((payload, kept.checked_add(slot)?))
}

/// Route each job page to its owner's dispatch (jig protocol/store/jobs.md, 3).
fn job_answer(
    owner: Token,
    of: store::Owner,
    job: &str,
    page: &store::JobPage,
    bounds: &families::Bounds,
) -> store::Request {
    let answer = match of {
        store::Owner::Jig => Err(store::JobRefusal { key: None, why: Box::from("jig job store unavailable") }),
        store::Owner::Application => {
            if job == "deleted-repository-forge-rows" {
                Err(store::JobRefusal { key: None, why: Box::from("deleted repository job facts unavailable") })
            } else {
                families::jobs::page(job, page, &families::jobs::Present::default(), bounds)
            }
        }
        store::Owner::Skein | store::Owner::Smith => {
            Err(store::JobRefusal { key: None, why: Box::from("no such job: this owner declares none") })
        }
    };
    store::Request::Job { owner, answer }
}

/// Read a job page's stored facts; later pages inherit no input.
pub fn job(routes: &mut Routes, component: &mut store::Store, event: store::Event, into: &mut Queue<Up>) {
    if let store::Event::Job { owner, of: store::Owner::Jig, job, page } = &event {
        let answer = store::jobs::page(job, page, component);
        into.push(Up::Ask(store::Request::Job { owner: *owner, answer }));
        return;
    }
    if let store::Event::Job { owner, of: store::Owner::Application, job, page } = &event
        && job.as_ref() == "deleted-repository-forge-rows"
    {
        let answer = match families::jobs::read_present(component, &routes.families) {
            Ok(present) => families::jobs::page(job, page, &present, &routes.families),
            Err(why) => Err(why),
        };
        into.push(Up::Ask(store::Request::Job { owner: *owner, answer }));
        return;
    }
    up(routes, event, into);
}

fn checkpointed(value: store::Checkpointed) -> engine::Checkpointed {
    engine::Checkpointed {
        name: value.name.bytes().as_slice().into(),
        store: value.name.store,
        secrets: value.secrets,
        linked: value.linked,
        copied: value.copied,
        bytes: value.bytes,
        paused: value.paused,
        took: value.took,
        held: value.held,
        removed: value.removed,
        skipped: value.skipped,
    }
}
fn checkpoint_why(value: store::CheckpointRefusal) -> engine::CheckpointWhy {
    match value {
        store::CheckpointRefusal::InProgress => engine::CheckpointWhy::InProgress,
        store::CheckpointRefusal::Unavailable => engine::CheckpointWhy::Unavailable,
        store::CheckpointRefusal::Files => engine::CheckpointWhy::Files,
        store::CheckpointRefusal::Unrecognized { path } => engine::CheckpointWhy::Unrecognized { path },
        store::CheckpointRefusal::Vanished { path } => engine::CheckpointWhy::Vanished { path },
        store::CheckpointRefusal::Failed { path, error } => engine::CheckpointWhy::Failed {
            path,
            error: alloc::fmt::format(format_args!("{error:?}")).into_boxed_str(),
        },
        store::CheckpointRefusal::InUse | store::CheckpointRefusal::Missing => {
            unreachable!("taking a checkpoint cannot refuse a reader's lock or name")
        }
    }
}

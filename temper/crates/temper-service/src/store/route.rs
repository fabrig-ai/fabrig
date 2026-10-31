//! Root store requests and terminals, with temper's decoded families.
use super::ranges;
use ::core::mem::size_of;
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_protocol_store as store;
use skein_lib::{List, Map, Queue, Token};
use temper_engine_domain as engine;
use temper_protocol_store as families;

/// A bounded load table and transcript continuations, never decoded rows.
#[derive(Debug)]
pub struct Routes {
    pending: Map<Token, Flight>,
    reads: Map<Token, Token>,
    next: u64,
    commit: u64,
    limits: engine::loads::Limits,
    families: families::Bounds,
    page_failure: Option<store::restore::Failure>,
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
    /// Size the continuation table from the root's admitted loads.
    #[must_use]
    pub fn new(table: u32, limits: &engine::Limits, families: families::Bounds) -> Routes {
        Routes {
            pending: Map::with_capacity(table),
            reads: Map::with_capacity(table),
            next: 0,
            commit: 0,
            limits: limits.loads,
            families,
            page_failure: None,
        }
    }
    /// The store's typed cause for the refused startup page.
    #[must_use]
    pub(crate) fn page_failure(&self) -> Option<&store::restore::Failure> {
        self.page_failure.as_ref()
    }
    /// Whether every routed load has had its terminal.
    #[must_use]
    pub fn idle(&self) -> bool {
        if !self.reads.is_empty() {
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

/// A root terminal or the next component page of a filtered load.
#[derive(Debug)]
pub enum Up {
    Root(Box<engine::Event>),
    Ask(store::Request),
    FrameTooLarge { number: u64, task: u64, bytes: u32, limit: u32 },
}

/// Route one root request; table saturation refuses before issuing a load.
#[must_use]
pub fn down(routes: &mut Routes, request: engine::StoreRequest) -> Option<store::Request> {
    Some(match request {
        engine::StoreRequest::Switch => store::Request::Switch { owner: Token::new(u64::MAX - 2) },
        engine::StoreRequest::Commit { number, writes } => {
            let mut out = List::with_capacity(u32::try_from(writes.len()).ok()?);
            for write in writes {
                let translated = match write {
                    engine::Write::Wires => store::Write::Wires,
                    engine::Write::Erase(engine::Key::Wires) => return None,
                    engine::Write::CopySettingsRevision(number) => store::Write::CopySettingsRevision(number),
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
        engine::StoreRequest::Read { load, range } => {
            let owner = routes.token()?;
            routes.reads.insert(owner, load).ok()?;
            store::Request::Read { owner, range: *range }
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
        store::Event::PageRefused { owner, failure } => {
            if let Some(flight) = routes.pending.remove(&owner) {
                routes.page_failure = Some(failure);
                root(into, engine::StoreInput::Unloaded { owner: flight.root });
            }
        }
        store::Event::Committed { number, fill } => root(into, engine::StoreInput::Committed { number, fill }),
        store::Event::Failed { number } => root(into, engine::StoreInput::Uncommitted { number }),
        store::Event::FrameTooLarge { number, task, bytes, limit } => {
            into.push(Up::FrameTooLarge { number, task, bytes, limit });
        }
        store::Event::Restored { .. } => unreachable!("startup uses typed pages"),
        store::Event::Loaded { owner, rows, next } => {
            page(routes, owner, rows, next, None, into);
        }
        store::Event::Got { owner, mut row } => {
            if let Some(Flight {
                range: engine::Range::Core(engine::CoreRange::TaskProposal { task, proposal }), ..
            }) = routes.pending.get(&owner)
                && let Some(store::Row::Jig(core::Record::Tasks(jig_domain_tasks::Stored::Live(live)))) = &row
            {
                let mut pointer = None;
                for kept in &live.proposal_rows {
                    if kept.proposal == *proposal {
                        pointer = Some(kept.revision);
                        break;
                    }
                }
                if live.number == *task
                    && let Some(revision) = pointer
                {
                    into.push(Up::Ask(store::Request::Get {
                        owner,
                        key: store::Get::Jig(core::Key::Tasks(jig_domain_tasks::Key::History {
                            task: *task,
                            revision,
                        })),
                    }));
                    return;
                }
                row = None;
            }
            let rows: Box<[store::Row]> = match row {
                Some(row) => Box::new([row]),
                None => Box::new([]),
            };
            page(routes, owner, rows, None, None, into);
        }
        store::Event::Turns { owner, turns, next } => {
            let mut rows = List::with_capacity(u32::try_from(turns.len()).expect("bounded turns"));
            for turn in turns {
                rows.push(store::Row::Jig(core::Record::Core(core::CoreRecord::Turn(turn)))).expect("same count");
            }
            page(routes, owner, rows.into_boxed(), None, next, into);
        }
        store::Event::Unread { owner } | store::Event::Corrupt { owner } => {
            if let Some(flight) = routes.pending.remove(&owner) {
                root(into, engine::StoreInput::Unloaded { owner: flight.root });
            }
        }
        store::Event::Read { owner, result } => {
            if let Some(load) = routes.reads.remove(&owner) {
                root(into, engine::StoreInput::ReadLoaded { load, result });
            }
        }
        store::Event::Site { owner, of, site, patcher, .. } => {
            into.push(Up::Ask(dispatch(owner, of, site, patcher)));
        }
        store::Event::Switched { .. } => root(into, engine::StoreInput::Switched),
        store::Event::OpenRefused { .. } | store::Event::SecretsNotReady { .. } => stop(routes, into),
        store::Event::Patched { .. }
        | store::Event::Opened { .. }
        | store::Event::Kept { .. }
        | store::Event::Carried { .. }
        | store::Event::Stale { .. }
        | store::Event::Erased { .. }
        | store::Event::Secret { .. }
        | store::Event::SecretsLoaded { .. } => {}
    }
}
fn root(into: &mut Queue<Up>, event: engine::StoreInput) {
    into.push(Up::Root(Box::new(engine::Event::Store(event))));
}
fn stop(routes: &Routes, into: &mut Queue<Up>) {
    root(into, engine::StoreInput::Uncommitted { number: routes.commit });
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
        let Some(record) = decode(row, &routes.families) else {
            root(into, engine::StoreInput::Unloaded { owner: flight.root });
            return;
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
                stop(routes, into);
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
        root(
            into,
            engine::StoreInput::Loaded {
                texts: None,
                owner: flight.root,
                rows: decoded.into_boxed(),
                next: startup_next,
            },
        );
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
    let texts = ranges::texts(&flight.range, decoded.as_slice());
    // For a cut transcript page restart from its beginning and filter by key;
    // the component's offset names the end of the whole incoming page.
    if offset.is_some() && !cutting {
        flight.after.clone_from(&next);
        flight.offset = offset;
        flight.waiting = false;
        routes.pending.insert(owner, flight).expect("removed slot");
        let root_owner = routes.pending.get(&owner).expect("kept continuation").root;
        root(into, engine::StoreInput::Loaded { texts, owner: root_owner, rows: decoded.into_boxed(), next });
    } else {
        root(into, engine::StoreInput::Loaded { texts, owner: flight.root, rows: decoded.into_boxed(), next });
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
        stop(routes, into);
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
        stop(routes, into);
    }
}

/// Translate the store's continuation independently of returned row identities.
/// Filtered startup walks can advance past families not selected by this load.
fn startup_cursor(flight: &Flight, cursor: &store::Cursor, bounds: &families::Bounds) -> Option<engine::Key> {
    let key = match &flight.range {
        engine::Range::Forge => {
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
        stop(routes, into);
    }
}

fn decode(row: store::Row, bounds: &families::Bounds) -> Option<engine::Record> {
    Some(match row {
        store::Row::Jig(row) => engine::Record::Core(row),
        store::Row::Application { key, value } => {
            engine::Record::Forge { row: Box::new(families::records::row(&key, &value, bounds).ok()?) }
        }
    })
}

fn page_bytes(record: &engine::Record, kept: u64) -> Option<(u64, u64)> {
    let payload = engine::record_bytes(record)?;
    let slot = u64::try_from(size_of::<engine::Record>()).ok()?.checked_add(payload)?;
    Some((payload, kept.checked_add(slot)?))
}

/// No owner has registered a written patch site in this build. Hand the
/// machine back with the named refusal, so the active generation is preserved.
fn dispatch(
    owner: Token,
    of: store::Owner,
    site: skein_codec::patch::Site,
    patcher: Box<skein_codec::patch::Patcher>,
) -> store::Request {
    let result = match of {
        store::Owner::Skein | store::Owner::Smith | store::Owner::Jig | store::Owner::Application => {
            Err(skein_codec::patch::Halt::NoSite(site))
        }
    };
    store::Request::Site { owner, patcher, result }
}

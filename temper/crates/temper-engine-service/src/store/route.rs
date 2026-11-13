//! Root store requests and terminals, with temper's decoded families.
//! Window pages carry whole or cut turns (temper/docs/design/domain/root.md, 7).
use super::{UnreadableRow, ranges};
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
    row_failure: Option<UnreadableRow>,
    pub(crate) patch_context: store::patches::Context,
    application_patch_context: families::patches::Context,
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
            row_failure: None,
            patch_context: store::patches::Context::default(),
            application_patch_context: families::patches::Context::default(),
        }
    }
    /// The store's typed cause for the refused startup page.
    #[must_use]
    pub(crate) fn page_failure(&self) -> Option<&store::restore::Failure> {
        self.page_failure.as_ref()
    }
    /// The application row refused before the root heard it.
    #[must_use]
    pub(crate) fn row_failure(&self) -> Option<&UnreadableRow> {
        self.row_failure.as_ref()
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
        engine::StoreRequest::Catalogue { .. } => unreachable!("the store owner replaces its catalogue directly"),
        engine::StoreRequest::Switch => store::Request::Switch { owner: Token::new(u64::MAX - 2) },
        engine::StoreRequest::Commit { number, writes } => {
            let mut out = List::with_capacity(u32::try_from(writes.len()).ok()?);
            for write in writes {
                let translated = match write {
                    engine::Write::DropTranscript { task } => store::Write::DropTranscript { task },
                    engine::Write::Wires => store::Write::Wires,
                    engine::Write::Erase(engine::Key::Wires) => return None,
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
            startup_page(routes, owner, items, next, into);
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
        store::Event::Loaded { owner, rows, next } => {
            page(routes, owner, rows, next, None, None, into);
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
            page(routes, owner, rows, None, None, None, into);
        }
        store::Event::Turns { owner, turns, next, cut } => {
            let mut rows = List::with_capacity(u32::try_from(turns.len()).expect("bounded turns"));
            for turn in turns {
                rows.push(store::Row::Jig(core::Record::Core(core::CoreRecord::Turn(turn)))).expect("same count");
            }
            page(routes, owner, rows.into_boxed(), None, next, cut, into);
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
        store::Event::Site { owner, of, site, at, patcher, key: _ } => {
            into.push(Up::Ask(dispatch(
                owner,
                of,
                site,
                at,
                patcher,
                &mut routes.patch_context,
                &mut routes.application_patch_context,
            )));
        }
        store::Event::Switched { .. } => root(into, engine::StoreInput::Switched),
        store::Event::OpenRefused { .. } | store::Event::SecretsNotReady { .. } => stop(routes, into),
        store::Event::Decoded { .. }
        | store::Event::Undecodable { .. }
        | store::Event::Rehearsing { .. }
        | store::Event::Back { .. }
        | store::Event::BackRefused { .. }
        | store::Event::Closed { .. }
        | store::Event::Patched { .. }
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
    page(routes, owner, rows.into_boxed(), next, None, None, into);
}
fn root(into: &mut Queue<Up>, event: engine::StoreInput) {
    into.push(Up::Root(Box::new(engine::Event::Store(event))));
}
fn stop(routes: &Routes, into: &mut Queue<Up>) {
    root(into, engine::StoreInput::Uncommitted { number: routes.commit });
}

#[expect(clippy::too_many_lines, reason = "the page bounds its prefix before handing off its terminal")]
fn page(
    routes: &mut Routes,
    owner: Token,
    rows: Box<[store::Row]>,
    cursor: Option<store::Cursor>,
    offset: Option<u64>,
    mut cut: Option<u32>,
    into: &mut Queue<Up>,
) {
    let more = cursor.is_some() || offset.is_some();
    let Some(mut flight) = routes.pending.remove(&owner) else {
        return;
    };
    let page_bound = if ranges::window(&flight.range) { flight.bytes.min(routes.limits.bytes) } else { flight.bytes };
    let mut decoded = List::with_capacity(flight.most);
    let mut bytes = 0_u64;
    let mut cutting = false;
    let mut last = flight.after.clone();
    let mut scanned = flight.after.clone();
    for row in rows {
        let mut record = match decode(row, &routes.families) {
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
        if decoded.is_empty()
            && cut.is_none()
            && !cutting
            && flight.most != 0
            && (payload_bytes > u64::from(routes.limits.transcript_bytes) || total > u64::from(routes.limits.bytes))
            && ranges::window(&flight.range)
            && let Some(length) = cut_turn(&mut record)
        {
            cut = Some(length);
            last = Some(key);
            decoded.push(record).expect("first row fits the count");
            cutting = true;
            continue;
        }
        if cutting
            || decoded.len() >= flight.most
            || total > u64::from(page_bound)
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
    let texts = ranges::texts(&flight.range, decoded.as_slice(), cut);
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

fn cut_turn(record: &mut engine::Record) -> Option<u32> {
    match record {
        engine::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) => {
            let length = u32::try_from(turn.transcript.len()).expect("bounded stored turn");
            turn.transcript = Box::new([]);
            Some(length)
        }
        engine::Record::Core(
            core::Record::Core(
                core::CoreRecord::ProposalDecision(_)
                | core::CoreRecord::Settings(_)
                | core::CoreRecord::Call(_)
                | core::CoreRecord::EscalationDecision(_)
                | core::CoreRecord::Deployment(_)
                | core::CoreRecord::RunProof(_)
                | core::CoreRecord::Terminal(_)
                | core::CoreRecord::Projection(_),
            )
            | core::Record::Tasks(_)
            | core::Record::People(_)
            | core::Record::Accounts(_)
            | core::Record::Notes(_),
        )
        | engine::Record::Forge { .. } => None,
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

fn decode(row: store::Row, bounds: &families::Bounds) -> Result<engine::Record, UnreadableRow> {
    match row {
        store::Row::Jig(row) => Ok(engine::Record::Core(row)),
        store::Row::Application { key, value } => match families::records::row(&key, &value, bounds) {
            Ok(row) => Ok(engine::Record::Forge { row: Box::new(row) }),
            Err(families::Undecodable::Setting(key)) => {
                Ok(engine::Record::Forge { row: Box::new(temper_engine_domain_forge::Stored::RefusedSetting(key)) })
            }
            Err(families::Undecodable::Malformed) => Err(UnreadableRow { key }),
        },
    }
}

fn page_bytes(record: &engine::Record, kept: u64) -> Option<(u64, u64)> {
    let payload = engine::record_bytes(record)?;
    let slot = u64::try_from(size_of::<engine::Record>()).ok()?.checked_add(payload)?;
    Some((payload, kept.checked_add(slot)?))
}

/// Route each written site to its owner with the deployment's admitted inputs.
fn dispatch(
    owner: Token,
    of: store::Owner,
    site: skein_codec::patch::Site,
    at: skein_codec::value::Node,
    mut patcher: Box<skein_codec::patch::Patcher>,
    context: &mut store::patches::Context,
    application: &mut families::patches::Context,
) -> store::Request {
    let (old, new) = patcher.parts();
    let result = match of {
        store::Owner::Skein => {
            skein_oauth::patches::written(site, old, new, at, &mut skein_oauth::patches::Context::default())
        }
        store::Owner::Smith => {
            smith_transcript::patches::written(site, old, new, at, &mut smith_transcript::patches::Context::default())
        }
        store::Owner::Jig => store::patches::written(site, old, new, at, context),
        store::Owner::Application => families::patches::written(site, old, new, at, application),
    };
    store::Request::Site { owner, patcher, result }
}

/// Read stored adoption settings for this landing alone; a later record inherits no input.
pub(crate) fn site(routes: &mut Routes, component: &mut store::Store, mut event: store::Event, into: &mut Queue<Up>) {
    routes.application_patch_context = families::patches::Context::default();
    if let store::Event::Site { of: store::Owner::Application, site, at, patcher, .. } = &mut event
        && *site == skein_codec::patch::Site::new(5, 0)
    {
        let (old, _) = patcher.parts();
        if let Ok((project, repository)) = families::patches::Context::landing_project(old, *at) {
            routes.application_patch_context.landing_branch =
                landing_branch(component, &routes.families, project, repository);
        }
    }
    up(routes, event, into);
}

fn landing_branch(
    component: &mut store::Store,
    bounds: &families::Bounds,
    project: u32,
    repository: Option<skein_forge::api::Repository>,
) -> Option<Box<[u8]>> {
    let mut after = None;
    let mut branch = None;
    // Every nonempty page consumes at least one adopted repository.
    for _ in 0..=bounds.forge().repositories {
        let page = component
            .patch_page(skein_kv::Range { start: Box::new([2]), end: Some(Box::new([3])) }, after.as_ref(), 1)
            .ok()?;
        for row in page.rows {
            let temper_engine_domain_forge::Stored::Repository(adopted) =
                families::records::row(&row.key, &row.value, bounds).ok()?
            else {
                return None;
            };
            if adopted.project != project || (repository.is_some() && repository != Some(adopted.provider)) {
                continue;
            }
            if let Some(prior) = &branch
                && *prior != adopted.settings.default_branch
            {
                return None;
            }
            branch = Some(adopted.settings.default_branch);
        }
        after = page.next;
        if after.is_none() {
            return branch;
        }
    }
    None
}

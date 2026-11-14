//! The reference root's store transport (protocol/store/README.md, 3 and protocol/store/loads.md).
use crate::{StoreOut, Up, keys, live, records};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use jig_domain_core as core;
use jig_domain_notes as notes;
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
    Ended(core::Now),
    Window { task: u64, attempt: u64, turn: u32 },
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
    /// Whether every routed store request has terminated.
    #[must_use]
    pub fn idle(&self) -> bool {
        self.pending.is_empty()
    }
    fn keep(&mut self, flight: Flight) -> Token {
        self.next = self.next.checked_add(1).expect("store route owner exhausted");
        let owner = Token::new(self.next);
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
                Phase::Link
                | Phase::Wires
                | Phase::AdoptRuns
                | Phase::ReadAfresh { .. }
                | Phase::SettleOutbox { .. }
                | Phase::Open => return None,
            }
            let owner = routes.keep(Flight::Start { phase, census });
            if phase == Phase::Switch {
                return Some(Request::Switch { owner });
            }
            restart(routes, owner, None)?
        }
        StoreOut::ReadLoad { load, range } => {
            let owner = routes.keep(Flight::Read(load));
            Request::Read { owner, range: *range }
        }
        StoreOut::NotesLoad { owner, range } => {
            let owner = routes.keep(Flight::Notes(owner));
            Request::Load {
                owner,
                range: store::Load::Notes(range),
                after: None,
                rows: limits.core.notes.load_rows,
                bytes: limits.core.load_bytes,
            }
        }
        StoreOut::Now(now) => {
            if let Some(key) = store::ended::key(&now) {
                let owner = routes.keep(Flight::Ended(now));
                return Some(Request::Get { owner, key: store::Get::Jig(key) });
            }
            let (task, attempt, turn) = window(now)?;
            let owner = routes.keep(Flight::Window { task, attempt, turn });
            transcript(routes, owner, None)?
        }
    })
}
fn restart(routes: &Routes, owner: Token, after: Option<store::Cursor>) -> Option<Request> {
    use core::start::Phase;
    let (phase, census) = match routes.pending.get(&owner)? {
        Flight::Start { phase, census } => (*phase, census),
        Flight::Read(_) | Flight::Notes(_) | Flight::Ended(_) | Flight::Window { .. } => return None,
    };
    let range = match phase {
        Phase::LoadCore => Restore::Jig(Box::new(core::RestoreRange::LIVE)),
        Phase::LoadInbox => Restore::Jig(Box::new([core::RestoreRange::Census(census.clone()?)])),
        Phase::RestoreConnector { connector } => Restore::Application(Box::new([live(connector)])),
        Phase::Sweep => Restore::Jig(Box::new([core::RestoreRange::People])),
        Phase::Link
        | Phase::Switch
        | Phase::Wires
        | Phase::AdoptRuns
        | Phase::ReadAfresh { .. }
        | Phase::SettleOutbox { .. }
        | Phase::Open => return None,
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
        Flight::Window { task, attempt, turn } => (task, attempt, turn),
        Flight::Start { .. } | Flight::Read(_) | Flight::Notes(_) | Flight::Ended(_) => {
            return None;
        }
    };
    let limits = routes.limits?;
    Some(Request::Transcript {
        owner,
        read: store::TranscriptRead::Window { task: *task, attempt: *attempt, turn: *turn, after },
        rows: 2,
        bytes: u64::from(limits.core.resume_bytes),
    })
}
fn window(now: core::Now) -> Option<(u64, u64, u32)> {
    match now {
        core::Now::StartPreparation { task, from: (attempt, turn), .. } => Some((task, attempt, turn)),
        core::Now::RelayBusy { .. }
        | core::Now::CallBusy { .. }
        | core::Now::Link(_)
        | core::Now::EffectAnswer { .. }
        | core::Now::ProposalRead { .. }
        | core::Now::SettingsAnswer { .. }
        | core::Now::PersonAnswer { .. }
        | core::Now::SettledCallRefused { .. }
        | core::Now::Ask { .. }
        | core::Now::Call { .. }
        | core::Now::DropCall { .. }
        | core::Now::Undelivered { .. }
        | core::Now::SignInRefused { .. }
        | core::Now::WatchRefused { .. }
        | core::Now::TaskRead { .. }
        | core::Now::Account(_)
        | core::Now::RunText { .. }
        | core::Now::NotesIndexed { .. }
        | core::Now::NotesRecalled { .. }
        | core::Now::NotesRefused { .. }
        | core::Now::NoteBusy { .. }
        | core::Now::DropAssignment { .. }
        | core::Now::Activate { .. }
        | core::Now::CompleteBrief { .. }
        | core::Now::BriefCorePlanned { .. }
        | core::Now::WorkspaceRequest { .. }
        | core::Now::RunPreparationFailed { .. }
        | core::Now::BriefFailed { .. }
        | core::Now::RunPrepared { .. }
        | core::Now::RestoreRefused
        | core::Now::SeenConversation { .. }
        | core::Now::LoadEndedRead { .. }
        | core::Now::EndedPersonTask { .. }
        | core::Now::EndedTask { .. }
        | core::Now::PartyInputs { .. }
        | core::Now::EndedProposal { .. }
        | core::Now::DecidedProposal { .. }
        | core::Now::EndedEscalation { .. } => None,
    }
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
        // The reference shells never hand patch sets. Return the unknown
        // written site to its open's asker through their ordinary request path.
        Event::Site { owner, patcher, site, .. } => {
            into.push(Up::Ask(Request::Site { owner, patcher, result: Err(skein_codec::patch::Halt::NoSite(site)) }));
        }
        Event::Committed { number, fill } => into.push(Up::Root(root::Event::Committed { number, fill })),
        Event::Failed { number } | Event::FrameTooLarge { number, .. } => {
            into.push(Up::Root(root::Event::Failed { number }));
        }
        Event::Read { owner, result } => {
            if let Some(Flight::Read(load)) = routes.pending.remove(&owner) {
                into.push(Up::Root(root::Event::Core(core::Event::ReadLoaded { load, result })));
            }
        }
        Event::Loaded { owner, rows, next } => loaded(routes, owner, rows, next, into),
        Event::Got { owner, row } => {
            if let Some(Flight::Ended(now)) = routes.pending.remove(&owner) {
                let row = match row {
                    Some(Row::Jig(row)) => Some(row),
                    None => None,
                    Some(Row::Application { .. }) => {
                        stop(routes, into);
                        return;
                    }
                };
                if let Some(key) = store::ended::next(&now, &row) {
                    assert!(routes.pending.insert(owner, Flight::Ended(now)).is_none(), "ended owner retained");
                    into.push(Up::Ask(Request::Get { owner, key: store::Get::Jig(key) }));
                    return;
                }
                if let Some(event) = store::ended::answer(now, row) {
                    into.push(Up::Root(root::Event::Core(event)));
                }
            }
        }
        Event::Turns { owner, turns, next, cut } => window_loaded(routes, owner, turns, next, cut, into),
        Event::Unread { owner } => {
            if let Some(Flight::Window { task, .. }) = routes.pending.remove(&owner) {
                into.push(Up::Root(root::Event::Core(core::Event::TranscriptLoaded {
                    task,
                    load: core::TranscriptLoad::Unread,
                })));
            }
        }
        Event::SecretsNotReady { .. } => unreachable!("the root store route handles public records only"),
        Event::Corrupt { owner } => {
            routes.pending.remove(&owner);
            stop(routes, into);
        }
        Event::OpenRefused { .. } => stop(routes, into),
        Event::Switched { owner } => switched(routes, owner, into),
        // The shell driving open owns progress; this root route prints no lines.
        // This reference route does not consume the census (protocol/store/README.md, 6.3).
        Event::Census { .. }
        | Event::Decoded { .. }
        | Event::Undecodable { .. }
        | Event::Rehearsing { .. }
        | Event::Back { .. }
        | Event::BackRefused { .. }
        | Event::Closed { .. }
        | Event::Patched { .. }
        | Event::Opened { .. }
        | Event::Kept { .. }
        | Event::Carried { .. }
        | Event::Stale { .. }
        | Event::Erased { .. }
        | Event::Secret { .. }
        | Event::SecretsLoaded { .. } => {}
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
    let Some(Flight::Window { task, .. }) = routes.pending.get(&owner) else {
        return;
    };
    if let Some(bytes) = cut {
        let mut turns = turns.into_iter();
        let turn = turns.next().expect("a cut page has one turn");
        assert!(turns.next().is_none(), "a cut turn is alone in its page");
        into.push(Up::Root(root::Event::Core(core::Event::TranscriptLoaded {
            task: *task,
            load: core::TranscriptLoad::Cut { turn, bytes: u64::from(bytes), done: next.is_none() },
        })));
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
        into.push(Up::Root(root::Event::TranscriptLoaded {
            task: *task,
            rows: rows.into_boxed(),
            done: next.is_none(),
        }));
    }
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
        Some(Flight::Start { .. } | Flight::Read(_) | Flight::Notes(_) | Flight::Ended(_) | Flight::Window { .. })
        | None => {}
    }
}
fn loaded(routes: &mut Routes, owner: Token, rows: Box<[Row]>, next: Option<store::Cursor>, into: &mut Queue<Up>) {
    if let Some(Flight::Notes(owner)) = routes.pending.remove(&owner) {
        let mut records = List::with_capacity(u32::try_from(rows.len()).expect("bounded page"));
        for row in rows {
            match row {
                Row::Jig(core::Record::Notes(record)) => {
                    records.push(record).expect("same count");
                }
                Row::Jig(_) | Row::Application { .. } => {
                    stop(routes, into);
                    return;
                }
            }
        }
        let event = notes::Event::Loaded { owner, rows: notes::Rows { records }, more: next.is_some() };
        into.push(Up::Root(root::Event::Core(core::Event::Notes(event))));
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

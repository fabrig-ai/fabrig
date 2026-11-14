//! Serial durable chat operations (protocol/host.md, 5.3; skein skein/docs/design/runtime/io/files.md, 2).
//! Compatibility, loading, fresh-history recovery and writes own their sequence
//! data. Record versions survive operations; conversation policy is never known.
mod compatibility;
mod ephemeral;
mod fresh;
mod load;
mod selection;
mod write;

use super::boundary::{Action, History, RecordName, StoreFault, StoreLimits, StoreRequest};
use super::{File, decode_state, decode_turns_bounded, save_state, save_turn, turn_opens_window_bounded};
use alloc::boxed::Box;
use compatibility::Compatibility;
use core::mem::replace;
use fresh::{Fresh, Resume};
use load::Loading;
use selection::Selection;
use skein_io::{
    digest::{Digest, digest},
    file::{self, Expect},
    kernel,
};
use skein_lib::bytes::decimal;
use skein_lib::{Duration, List, Map, Queue, Time, Token};
use smith_local_domain::{self as local, ChatState, DeliveryRecord};
use smith_protocol_translate::Endpoints;
use write::{AfterWrite, Writing};

#[derive(Debug)]
enum State {
    Idle,
    Selecting(Selection),
    Compatibility(Compatibility),
    Load(Loading),
    Fresh(Fresh),
    Write(Writing),
    Cleaning,
    Closing,
    Closed,
}

/// One serial chat store over an independently adopted root.
#[derive(Debug)]
pub struct Store {
    owner: Token,
    root: Token,
    descriptor: kernel::Fd,
    chats_root: Token,
    chats_descriptor: kernel::Fd,
    limits: StoreLimits,
    endpoints: Endpoints,
    phase: State,
    checked: bool,
    closing: bool,
    versions: Map<Box<[u8]>, Digest>,
    ephemeral: Option<ephemeral::Ephemeral>,
}

impl Store {
    /// The stored record format version this codec writes and reads.
    pub const FORMAT_VERSION: u64 = super::CHAT_STORE_VERSION;

    /// Adopt a chat directory's token and descriptor; the store closes it explicitly.
    #[must_use]
    pub fn new(
        owner: Token,
        root: Token,
        descriptor: kernel::Fd,
        limits: StoreLimits,
        endpoints: Endpoints,
    ) -> Option<Self> {
        if !Self::valid(&limits) {
            return None;
        }
        let records = limits.turns.checked_add(limits.deliveries)?.checked_add(2)?;
        Some(Self {
            owner,
            root,
            descriptor,
            chats_root: root,
            chats_descriptor: descriptor,
            limits,
            endpoints,
            phase: State::Idle,
            checked: false,
            closing: false,
            versions: Map::with_capacity(records),
            ephemeral: None,
        })
    }

    /// Whether record capacities and the format marker fit the store's file and scan edges.
    #[must_use]
    pub fn valid(limits: &StoreLimits) -> bool {
        let Some(records) = limits.turns.checked_add(limits.deliveries) else { return false };
        records.checked_add(4).is_some()
            && limits.file_bytes >= crate::format::max_bytes()
            && limits.turns > 0
            && limits.deliveries > 0
            && limits.scan_bytes >= Self::marker_entry_bytes()
    }

    /// Room for exactly one serial domain request.
    #[must_use]
    pub fn takes(&self) -> bool {
        self.idle() && !self.closing
    }

    fn idle(&self) -> bool {
        match self.phase {
            State::Idle => true,
            State::Selecting(_)
            | State::Compatibility(_)
            | State::Load(_)
            | State::Fresh(_)
            | State::Write(_)
            | State::Cleaning
            | State::Closing
            | State::Closed => false,
        }
    }

    /// Admit one request while idle; its IO requests carry a bounded deadline.
    pub fn request(&mut self, request: StoreRequest, now: Time, out: &mut Queue<Action>) {
        assert!(self.takes(), "the service reserves a serial store slot");
        self.phase = match request {
            StoreRequest::SelectChat { name, create } => self.select_chat(name, create, out),
            StoreRequest::Load if self.ephemeral.is_some() => Self::empty(out),
            StoreRequest::Load => {
                self.versions = Map::with_capacity(self.versions.capacity());
                if self.checked { State::Load(Loading::Journal) } else { State::Compatibility(Compatibility::Format) }
            }
            StoreRequest::SaveState { state, history } => {
                assert!(self.checked, "Load must check the selected root before any save");
                match history {
                    History::Kept => self.begin_write(
                        RecordName::State,
                        save_state(state),
                        AfterWrite::Answer(local::Event::StateSaved),
                        out,
                    ),
                    History::Fresh if self.ephemeral.is_some() => self.begin_write(
                        RecordName::State,
                        save_state(state),
                        AfterWrite::Answer(local::Event::StateSaved),
                        out,
                    ),
                    History::Fresh => {
                        let mut journal = save_state(state);
                        journal.name = b"fresh".as_slice().into();
                        self.begin_write(
                            RecordName::Journal,
                            journal,
                            AfterWrite::Journal { state, resume: Resume::Save },
                            out,
                        )
                    }
                }
            }
            StoreRequest::SaveTurn { number, read, opens_window, turn } => {
                assert!(self.checked, "Load must check the selected root before any save");
                let record = RecordName::Turn(turn.sequence);
                match save_turn(turn.sequence, read, opens_window, &turn, &smith_transcript::CEILINGS, &self.endpoints)
                {
                    Ok(file) => {
                        self.begin_write(record, file, AfterWrite::Answer(local::Event::TurnSaved { number }), out)
                    }
                    Err(_) => Self::failure(record, StoreFault::Malformed { file: record }, false, out),
                }
            }
            StoreRequest::SaveDelivery { record } => {
                assert!(self.checked, "Load must check the selected root before any save");
                let name = RecordName::Delivery(record.name);
                match crate::save_delivery(&record, self.limits.file_bytes.min(DELIVERY_BYTES)) {
                    Ok(file) => self.begin_write(
                        name,
                        file,
                        AfterWrite::Answer(local::Event::DeliverySaved { name: record.name }),
                        out,
                    ),
                    Err(_) => Self::failure(name, StoreFault::Malformed { file: name }, false, out),
                }
            }
        };
        self.follow(now, out);
    }

    /// Consume one terminal of the last file request, reserving three output slots.
    pub fn from_io(&mut self, event: file::Event, now: Time, out: &mut Queue<Action>) {
        assert_eq!(event.owner(), self.owner, "store IO echoes its owner");
        match self.phase {
            State::Cleaning => {
                self.cleanup_event(event, now, out);
                return;
            }
            State::Idle
            | State::Selecting(_)
            | State::Compatibility(_)
            | State::Load(_)
            | State::Fresh(_)
            | State::Write(_)
            | State::Closing
            | State::Closed => {}
        }
        let state = replace(&mut self.phase, State::Closed);
        self.phase = match state {
            State::Selecting(state) => self.selection_event(state, event, out),
            State::Compatibility(state) => self.compatibility_event(state, terminal(event), out),
            State::Load(state) => self.load_event(state, terminal(event), out),
            State::Fresh(state) => self.fresh_event(state, terminal(event), out),
            State::Write(writing) => self.write_event(writing, terminal(event), out),
            State::Closing => self.close_event(terminal(event), out),
            State::Idle | State::Cleaning | State::Closed => {
                unreachable!("IO terminal has an outstanding request")
            }
        };
        self.follow(now, out);
    }

    fn follow(&mut self, now: Time, out: &mut Queue<Action>) {
        match &self.phase {
            State::Fresh(Fresh::Remove { entries, index, .. }) => {
                self.versions.remove(&entries.get(*index).expect("selected removal").name);
            }
            State::Idle
            | State::Selecting(_)
            | State::Compatibility(_)
            | State::Load(_)
            | State::Fresh(_)
            | State::Write(_)
            | State::Cleaning
            | State::Closing
            | State::Closed => {}
        }
        let request = match &self.phase {
            State::Selecting(state) => Some(self.selection_request(state)),
            State::Compatibility(state) => Some(self.compatibility_request(state)),
            State::Load(state) => Some(self.load_request(state)),
            State::Fresh(state) => Some(self.fresh_request(state)),
            State::Write(writing) => Some(writing.request(self.owner, self.root)),
            State::Closing => Some(file::Request::Close { owner: self.owner, file: self.root }),
            State::Idle => {
                if self.closing {
                    self.close(now, out);
                }
                None
            }
            State::Cleaning | State::Closed => None,
        };
        if let Some(request) = request {
            Self::io(request, now, out);
        }
    }

    fn close_event(&mut self, event: Terminal, out: &mut Queue<Action>) -> State {
        match event {
            Terminal::Closed => {
                if self.root == self.chats_root {
                    out.push(Action::Closed);
                    State::Closed
                } else {
                    self.root = self.chats_root;
                    self.descriptor = self.chats_descriptor;
                    State::Closing
                }
            }
            event @ (Terminal::Loaded(_)
            | Terminal::Scanned { .. }
            | Terminal::Stored(_)
            | Terminal::Removed
            | Terminal::Synced
            | Terminal::Missing
            | Terminal::Committed(_)
            | Terminal::Fault(_)) => {
                out.push(Action::Fault(event_fault(RecordName::State, event)));
                out.push(Action::Closed);
                State::Closed
            }
        }
    }

    fn io(request: file::Request, now: Time, out: &mut Queue<Action>) {
        out.push(Action::Io { request, deadline: now.saturating_add(Duration::from_secs(30)) });
    }

    fn marker_entry_bytes() -> u64 {
        u64::try_from(size_of::<file::Entry>())
            .expect("entry size fits u64")
            .checked_add(6)
            .expect("marker name fits scan accounting")
    }

    fn scan_request(&self, reserve_marker: bool) -> file::Request {
        let max = self
            .limits
            .turns
            .checked_add(self.limits.deliveries)
            .expect("admitted scan count")
            .checked_add(4)
            .expect("admitted metadata count")
            .checked_sub(u32::from(reserve_marker))
            .expect("marker entry is reserved");
        let max_bytes = if reserve_marker {
            self.limits.scan_bytes.checked_sub(Self::marker_entry_bytes()).expect("admitted marker scan room")
        } else {
            self.limits.scan_bytes
        };
        file::Request::Scan {
            owner: self.owner,
            root: self.root,
            path: b".".as_slice().into(),
            max,
            max_bytes,
            no_follow: true,
        }
    }

    fn load_file(&self, path: Box<[u8]>, max: u32) -> file::Request {
        file::Request::Load { owner: self.owner, root: self.root, path, max, no_follow: true }
    }

    fn remove_file(&self, name: Box<[u8]>) -> file::Request {
        file::Request::Remove { owner: self.owner, root: self.descriptor, name, directory: false }
    }

    fn finished(event: local::Event, out: &mut Queue<Action>) -> State {
        out.push(Action::Domain(event));
        State::Idle
    }

    fn empty(out: &mut Queue<Action>) -> State {
        Self::finished(local::Event::Loaded { state: None, transcript: None, deliveries: Box::new([]) }, out)
    }

    fn failure(record: RecordName, fault: StoreFault, loading: bool, out: &mut Queue<Action>) -> State {
        out.push(Action::Fault(fault));
        let turn = match record {
            RecordName::Turn(number) => Some(number),
            RecordName::State | RecordName::Delivery(_) | RecordName::Journal => None,
        };
        let reason = if loading { local::StoreFailure::Read } else { local::StoreFailure::Write };
        Self::finished(local::Event::StoreFailed { turn, reason }, out)
    }
}

const DELIVERY_BYTES: u32 = 1_u32 << 20_u32;

#[derive(Debug)]
enum Terminal {
    Loaded(Box<[u8]>),
    Scanned { entries: Box<[file::Entry]>, more: u64 },
    Stored(Digest),
    Removed,
    Synced,
    Closed,
    Missing,
    Committed(kernel::Error),
    Fault(Fault),
}

#[derive(Debug)]
enum Fault {
    Io(kernel::Error),
    Conflict,
    TooLarge,
    Malformed,
}

fn terminal(event: file::Event) -> Terminal {
    match event {
        file::Event::Loaded { bytes, .. } => Terminal::Loaded(bytes),
        file::Event::Scanned { entries, more, .. } => Terminal::Scanned { entries, more },
        file::Event::Stored { digest, .. } => Terminal::Stored(digest),
        file::Event::Removed { .. } => Terminal::Removed,
        file::Event::Synced { .. } => Terminal::Synced,
        file::Event::Closed { .. } => Terminal::Closed,
        file::Event::Failed { error: kernel::Error::NotFound, committed: false, .. } => Terminal::Missing,
        file::Event::Failed { error, committed: true, .. } => Terminal::Committed(error),
        file::Event::Failed { error, .. } => Terminal::Fault(Fault::Io(error)),
        file::Event::Conflict { .. } => Terminal::Fault(Fault::Conflict),
        file::Event::TooLarge { .. } => Terminal::Fault(Fault::TooLarge),
        file::Event::Cancelled { .. } => Terminal::Fault(Fault::Io(kernel::Error::TimedOut)),
        file::Event::Refused { .. } => Terminal::Fault(Fault::Malformed),
        file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Made { .. }
        | file::Event::Opened { .. }
        | file::Event::Stated { .. }
        | file::Event::Written { .. }
        | file::Event::Read { .. }
        | file::Event::Linked { .. }
        | file::Event::Renamed { .. }
        | file::Event::Cut { .. }
        | file::Event::Listed { .. } => unreachable!("store requests have whole-file terminals"),
    }
}

fn event_fault(record: RecordName, event: Terminal) -> StoreFault {
    let fault = match event {
        Terminal::Missing => Fault::Io(kernel::Error::NotFound),
        Terminal::Committed(error) => Fault::Io(error),
        Terminal::Fault(fault) => fault,
        Terminal::Loaded(_)
        | Terminal::Scanned { .. }
        | Terminal::Stored(_)
        | Terminal::Removed
        | Terminal::Synced
        | Terminal::Closed => unreachable!("terminal matches its store request"),
    };
    match fault {
        Fault::Io(error) => StoreFault::Io { file: record, error },
        Fault::Conflict => StoreFault::Conflict { file: record },
        Fault::TooLarge => StoreFault::TooLarge { file: record },
        Fault::Malformed => StoreFault::Malformed { file: record },
    }
}

fn record_name(name: &[u8]) -> Option<RecordName> {
    if name == b"state" {
        return Some(RecordName::State);
    }
    if name == b"fresh" {
        return Some(RecordName::Journal);
    }
    if name.len() == 15 && name.ends_with(b".turn") {
        return Some(RecordName::Turn(u32::try_from(decimal(name.get(..10)?)?).ok()?));
    }
    if name.len() == 42 && name.get(20) == Some(&b'.') && name.get(31) == Some(&b'.') {
        return Some(RecordName::Delivery(smith_domain::run::CallName {
            activation: decimal(name.get(..20)?)?,
            completion: u32::try_from(decimal(name.get(21..31)?)?).ok()?,
            position: u32::try_from(decimal(name.get(32..)?)?).ok()?,
        }));
    }
    None
}

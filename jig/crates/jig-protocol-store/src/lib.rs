//! The engine's store component (protocol/store/README.md, sections 2 and 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
mod baseline;

#[path = "../patches/mod.rs"]
pub mod patches;

mod back;
pub mod boundary;
pub mod codec;
mod commits;
mod directory;
pub mod ended;
pub mod format;
mod generations;
mod gets;
pub mod host;
pub mod intervals;
mod loads;
mod open;
mod pages;
mod patching;
mod rehearsal;
pub use patching::Counts;
pub use patching::baseline::{SKEIN_PATCH_SET, SMITH_PATCH_SET};
pub use patching::sets::{JIG, KeptFamily, KeptMap, Keys, Owner, PatchSet, PatchSets};
mod reads;
pub mod restore;
pub mod retained;
mod secrets;
pub mod sizes;
pub mod transcripts;
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Time, Wall};

/// The binary's patch level of each owner, `None` before its first patch
/// (jig/docs/design/protocol/store/opening.md, 5, and jig/docs/design/protocol/store/records.md, 3).
#[must_use]
pub fn levels(sets: &PatchSets) -> codec::component::Levels {
    patching::plan::binary(sets)
}

/// The store component's bounds (protocol/store/README.md, section 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub core: jig_domain_core::Limits,
    pub store: skein_kv::Limits,
    pub secrets: skein_kv::Limits,
    /// The clients component's carried values at once (protocol/store/secrets.md).
    pub held: u32,
    /// The store's commit edge (protocol/store/records.md, 4).
    pub commits: sizes::Commits,
    /// Point gets available to a party page's joins (protocol/store/README.md, 8).
    pub joins: u32,
    pub queue: u32,
    pub files: u32,
    pub reads: u32,
    pub orphan_erases: u32,
}

impl Limits {
    /// The generated families' bounds, derived from the component's source (protocol/store/README.md, 8).
    #[must_use]
    pub fn codec_view(&self) -> Option<codec::View> {
        codec::limits(&self.core)
    }
}

/// Stores made by the caller, kept across a world's crash (protocol/store/README.md, 9).
#[derive(Debug)]
pub struct Stores {
    pub store: skein_kv::Store,
    pub secrets: skein_kv::Store,
}

/// The component's open and stopped states (protocol/store/README.md, 5 and protocol/store/opening.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Closed,
    Ready { next: u64, format_due: bool },
    Stopped,
}

#[derive(Debug)]
pub struct Store {
    limits: Limits,
    application: sizes::Application,
    codec_limits: codec::View,
    layouts: codec::component::Layouts,
    /// Today's catalogue, overlaid at open and replaced before each catalogue commit.
    names: Box<[jig_domain_core::settings::ModelName]>,
    stores: Stores,
    phase: Phase,
    directory: directory::Directory,
    secret_state: secrets::State,
    transcripts: transcripts::State,
    /// The installation's identifier (protocol/store/README.md 4.4): drawn by the shell and
    /// kept by an empty store's first commit, or restored from the store.
    installation: [u8; 16],
    patch_sets: PatchSets,
    wires: &'static [Wire],
    /// This start's first Wires commit, and its durability (protocol/store/records.md 3).
    mark: Mark,
    start_open_failure: Option<restore::OpenFailure>,
    rehearsal: Option<rehearsal::State>,
    back: Option<back::State>,
}

#[derive(Debug)]
enum Mark {
    Waiting,
    Committing(u64),
    Durable,
}
impl Mark {
    fn accept(&mut self, wires: bool, number: u64) {
        if wires {
            match self {
                Mark::Waiting => *self = Mark::Committing(number),
                Mark::Committing(_) | Mark::Durable => {}
            }
        }
    }
    fn committed(&mut self, number: u64) {
        if let Mark::Committing(mark) = self
            && *mark == number
        {
            *self = Mark::Durable;
        }
    }
}

/// A worker wire's name and version (protocol/store/records.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wire {
    pub name: &'static str,
    pub version: u32,
}

/// Proof that the store's three file drivers were seeded: io's `Store` names
/// its temporaries from a driver's randomness and refuses unseeded (protocol/store/opening.md 1).
/// It proves three drivers were seeded, not that they are the three the shell
/// serves the store with.
#[derive(Clone, Copy, Debug)]
pub struct Seeded(());

/// Seeds each store driver from `seed` mixed with its index.
#[must_use]
pub fn seed(drivers: [&mut skein_io::file_layer::FileIo; 3], seed: u64) -> Seeded {
    for (index, driver) in drivers.into_iter().enumerate() {
        let index = u64::try_from(index).expect("three driver indices");
        driver.seed_randomness(
            seed ^ 0x9e37_79b9_7f4a_7c15_u64.wrapping_mul(index.checked_add(1).expect("three drivers")),
        );
    }
    Seeded(())
}

impl Store {
    /// A component over open stores; `models` are the configuration's, in its order.
    #[must_use]
    pub fn over(
        limits: &Limits,
        application: sizes::Application,
        models: &[jig_domain_core::Model],
        stores: Stores,
    ) -> Store {
        let codec_limits = checked_codec(limits).expect("the store's limits are valid");
        Store {
            limits: *limits,
            application,
            codec_limits,
            layouts: format::layouts(),
            names: codec::names(models),
            stores,
            phase: Phase::Closed,
            directory: directory::Directory::new(),
            secret_state: secrets::State::new(limits.secrets.queued, limits.held),
            transcripts: transcripts::State::new(limits),
            installation: [0; 16],
            patch_sets: PatchSets::NONE,
            wires: &[],
            mark: Mark::Waiting,
            start_open_failure: None,
            rehearsal: None,
            back: None,
        }
    }

    #[must_use]
    pub fn on_directory(
        limits: &Limits,
        application: sizes::Application,
        models: &[jig_domain_core::Model],
        _seeded: Seeded,
    ) -> Store {
        let stores =
            Stores { store: skein_kv::Store::new(&limits.store), secrets: skein_kv::Store::new(&limits.secrets) };
        let mut store = Store::over(limits, application, models, stores);
        store.directory.disk = true;
        store
    }

    /// Today's models, before a commit that holds them (domain/engine/settings.md, 11).
    pub fn catalogue(&mut self, names: Box<[jig_domain_core::settings::ModelName]>) {
        self.names = names;
    }

    /// The first open failure, with no stored values (protocol/store/secrets.md).
    #[must_use]
    pub fn start_open_failure(&self) -> Option<&restore::OpenFailure> {
        self.start_open_failure.as_ref()
    }

    fn open_failed(&mut self, failure: restore::OpenFailure) {
        if self.start_open_failure.is_none() {
            self.start_open_failure = Some(failure);
        }
    }

    /// Supplies the owners' static patch data (protocol/store/opening.md 3).
    pub fn patch_sets(&mut self, sets: PatchSets) {
        self.patch_sets = sets;
    }

    /// The start's wires (protocol/store/records.md 3), at most 16 with names of at most
    /// 32 bytes. Refused input is not retained. No wires means an empty row.
    #[must_use]
    pub fn wires(&mut self, wires: &'static [Wire]) -> bool {
        if wires.len() > 16 {
            return false;
        }
        for wire in wires {
            if wire.name.len() > 32 {
                return false;
            }
        }
        self.wires = wires;
        true
    }

    /// The first Wires commit accepted in this start, while awaiting durability.
    #[must_use]
    pub fn pending_mark(&self) -> Option<u64> {
        match self.mark {
            Mark::Committing(number) => Some(number),
            Mark::Waiting | Mark::Durable => None,
        }
    }

    /// Whether this start's first Wires commit is durable (protocol/store/records.md 3).
    #[must_use]
    pub fn mark_durable(&self) -> bool {
        match self.mark {
            Mark::Durable => true,
            Mark::Waiting | Mark::Committing(_) => false,
        }
    }

    #[must_use]
    pub fn into_stores(self) -> Stores {
        self.stores
    }
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    checked_codec(limits).is_some()
}

// Validate and retain one family mapping for both admission and the live store (protocol/store/README.md, 8).
fn checked_codec(limits: &Limits) -> Option<codec::View> {
    let codec = limits.codec_view()?;
    if valid_with_codec(limits, &codec) { Some(codec) } else { None }
}

fn valid_with_codec(limits: &Limits, codec: &codec::View) -> bool {
    limits.queue > 0
        && limits.queue <= limits.store.queued
        && limits.files > 0
        && limits.orphan_erases > 0
        && limits.store.is_usable()
        && limits.secrets.is_usable()
        && decoded_records_bound(limits, codec).is_some()
        && frame_bound(codec).is_some()
        && transcripts::reservation_count(limits).is_some()
        && commits::most_ops(limits.commits.writes, limits.orphan_erases).is_some()
        && limits.commits.writes.checked_add(1).is_some()
        && limits.core.tasks.tasks.checked_mul(limits.core.tasks.inbox_messages).is_some()
        && match limits.core.load_slots.checked_add(limits.core.tasks.tasks) {
            Some(readers) => limits.reads >= readers,
            None => false,
        }
}

fn frame_bound(codec: &codec::View) -> Option<u64> {
    core_v6::TurnRecord::encoded_bound(&codec.core)?.checked_add(u64::from(transcripts::frame::WRAPPER_BYTES))
}

// Bound for two decoded records, including their wire values and the census's joined task.
fn decoded_records_bound(limits: &Limits, codec: &codec::View) -> Option<u64> {
    core_v6::Row::decoded_bound(&codec.core)?
        .checked_add(tasks_v10::Row::decoded_bound(&codec.tasks)?)?
        .checked_add(people_v14::Row::decoded_bound(&codec.people)?)?
        .checked_add(notes_v1::Row::decoded_bound(&codec.notes)?)?
        .checked_add(accounts_v2::Row::decoded_bound(&accounts_v2::CEILINGS)?)?
        .checked_add(settings_v2::Revision::decoded_bound(&codec.settings)?)?
        .checked_add(jig_domain_core::settings::revision_bound(&limits.core)?)?
        .checked_add(u64::try_from(size_of::<jig_domain_tasks::TaskRecord>()).ok()?)?
        .checked_mul(2)
}

/// The memory component's requests (protocol/store/opening.md).
pub fn down(store: &mut Store, env: &Env<Limits>, request: Request, above: &mut Queue<Event>, below: &mut Below) {
    let lengths = rehearsal::lengths(below);
    down_inner(store, env, request, above, below);
    accounted(store, lengths, above, below);
}
fn accounted(store: &mut Store, lengths: [u32; 3], above: &mut Queue<Event>, below: &mut Below) {
    rehearsal::issued(store, below, lengths);
    let lengths = rehearsal::lengths(below);
    rehearsal::advance_close(store, above, below);
    rehearsal::issued(store, below, lengths);
}
fn down_inner(store: &mut Store, env: &Env<Limits>, request: Request, above: &mut Queue<Event>, below: &mut Below) {
    if let Request::Close { owner } = request {
        rehearsal::close(store, owner, above);
        return;
    }
    if store.phase == Phase::Stopped {
        return;
    }
    match request {
        Request::Rehearse { owner, directory, scratch, installation } => {
            rehearsal::begin(store, owner, directory, scratch, installation, above, below);
        }
        Request::Back { owner, root } => back::begin(store, owner, root, above, below),
        Request::Close { .. } => unreachable!("close routed before stopped guard"),
        Request::Switch { owner } => patching::switch::ask(store, env, owner, above, below),
        Request::Open { owner, root, installation } => {
            directory::open(store, owner, root, installation, above, below);
        }
        Request::Site { owner, patcher, result } => match store.phase {
            Phase::Ready { .. } => transcripts::read::site(store, env, owner, patcher, result, above, below),
            Phase::Closed | Phase::Stopped => patching::phase::site(store, env, owner, patcher, result, above, below),
        },
        Request::Commit { number, writes } => transcripts::write::begin(store, env, number, writes, above, below),
        Request::Restore { owner, range, after, rows, bytes } => {
            restore::restore(store, env, owner, restore::Page { range, after, rows, bytes }, above, below);
        }
        Request::Load { owner, range, after, rows, bytes } => {
            loads::load(store, owner, range, after, rows, bytes, above);
        }
        Request::Get { owner, key } => gets::get(store, owner, key, above),
        Request::Read { owner, range } => reads::read(store, owner, range, above, below),
        Request::Secrets { owner, request } => secrets::secrets(store, env, owner, request, above, below),
        Request::Transcript { owner, read, rows, bytes } => {
            transcripts::read::transcript(store, owner, read, rows, bytes, above, below);
        }
    }
}

/// File terminals routed through their independent drivers.
pub fn up(store: &mut Store, env: &Env<Limits>, io: Io, above: &mut Queue<Event>, below: &mut Below) {
    let lengths = rehearsal::lengths(below);
    rehearsal::arriving(store, &io);
    if rehearsal::closing(store) {
        rehearsal::closing_up(store, io, above, below);
    } else if rehearsal::preparing(store) {
        let arrived = match io {
            Io::Store(a) | Io::Secrets(a) | Io::Transcripts(a) => a,
        };
        rehearsal::up(store, arrived, above, below);
    } else if store.back.is_some() {
        back::up(store, env, io, above, below);
    } else {
        up_inner(store, env, io, above, below);
    }
    accounted(store, lengths, above, below);
}
fn up_inner(store: &mut Store, env: &Env<Limits>, io: Io, above: &mut Queue<Event>, below: &mut Below) {
    directory::up(store, env, io, above, below);
    if store.phase != Phase::Stopped {
        transcripts::read::fire(store, above, below);
        transcripts::write::fire(store, env, above, below);
        patching::switch::resume(store, env, below);
    }
}

/// Bounded background store work and the orphan sweep.
pub fn fire(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    let lengths = rehearsal::lengths(below);
    fire_inner(store, env, above, below);
    accounted(store, lengths, above, below);
}
fn fire_inner(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    if store.phase != Phase::Stopped && !rehearsal::preparing(store) && store.back.is_none() {
        directory::fire(store, env, above, below);
        secrets::sweep(store, env, above, below);
        transcripts::read::fire(store, above, below);
        transcripts::write::fire(store, env, above, below);
        patching::switch::resume(store, env, below);
    }
}

fn memory_env(limits: skein_kv::Limits) -> Env<skein_kv::Limits> {
    // Memory gets and commits use no clock or file operation.
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits }
}

#[rustfmt::skip]
#[path = "generated/accounts_v2.rs"]
pub mod accounts_v2;
#[rustfmt::skip]
#[path = "generated/notes_v1.rs"]
pub mod notes_v1;
#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "generated StoredActed retains its domain fields act and at")]
#[path = "generated/people_v14.rs"]
pub mod people_v14;
#[rustfmt::skip]
#[path = "generated/tasks_v10.rs"]
pub mod tasks_v10;
#[rustfmt::skip]
#[path = "generated/core_v6.rs"]
pub mod core_v6;
pub mod keys;
pub use boundary::{
    Arrived, BackRefusal, Below, CARRIED, CarriedKind, Check, Cursor, Event, Family, Generations, Get, HELD, Io,
    KeptWire, Load, Older, OpenRefusal, Request, Restore, Root, Row, SecretOwner, SecretRow, SecretValue, Secrets,
    TranscriptRead, Write,
};

#[cfg(test)]
mod tests;

#[rustfmt::skip]
#[path = "generated/settings_v2.rs"]
pub mod settings_v2;
#[rustfmt::skip]
#[path = "generated/component_v3.rs"]
pub mod component_v3;
#[rustfmt::skip]
#[path = "generated/layouts.rs"]
mod layouts;

#[rustfmt::skip]
#[path = "generated/secrets_v1.rs"]
pub mod secrets_v1;

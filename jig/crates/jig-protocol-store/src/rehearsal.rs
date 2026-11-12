//! Scratch starts and their cancellable close (protocol/store/opening.md 5).
use alloc::boxed::Box;
use crate::{Arrived, Below, Event, Io, KeptWire, OpenRefusal, Phase, Root, Store, directory, generations, open};
use generations::{Copying, Progress, Removing};
use skein_io::file;
use skein_lib::{List, Queue, Token};

#[derive(Debug)]
enum Stage {
    Locking,
    Reading,
    Emptying(Removing),
    Making,
    Copying(Copying),
    Active,
    Opening,
    Closing,
    Cleaning(Removing),
    Finished,
}
#[derive(Debug)]
pub(crate) struct State {
    source: Root,
    scratch: Root,
    owner: Token,
    installation: [u8; 16],
    active: Option<generations::Active>,
    stage: Stage,
    announced: bool,
    close: Option<Token>,
    fault: bool,
    pending: u32,
    handles: List<(crate::host::Lane, Token)>,
}

pub(crate) fn begin(store: &mut Store, owner: Token, source: Root, scratch: Root, installation: [u8; 16], above: &mut Queue<Event>, below: &mut Below) {
    if store.phase != Phase::Closed || store.directory.root.is_some() || store.rehearsal.is_some() || !store.directory.disk || source.fd == scratch.fd {
        return open::refuse(store, owner, OpenRefusal::Failed, above);
    }
    store.rehearsal = Some(State { source, scratch, owner, installation, active: None, stage: Stage::Locking, announced: false, close: None, fault: false, pending: 0, handles: List::with_capacity(24) });
    store.directory.owner = owner;
    below.store.push(file::Request::Lock { owner: directory::OWNER, root: source.token });
}
pub(crate) fn source(store: &Store) -> Root { store.rehearsal.as_ref().expect("rehearsal retained").source }
fn state(store: &mut Store) -> &mut State { store.rehearsal.as_mut().expect("rehearsal routing") }

pub(crate) fn preparing(store: &Store) -> bool {
    let Some(state) = &store.rehearsal else { return false };
    match state.stage {
        Stage::Locking | Stage::Reading | Stage::Emptying(_) | Stage::Making | Stage::Copying(_) | Stage::Active => true,
        Stage::Opening | Stage::Closing | Stage::Cleaning(_) | Stage::Finished => false,
    }
}
pub(crate) fn up(store: &mut Store, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let transition = core::mem::replace(&mut state(store).stage, Stage::Opening);
    match transition {
        Stage::Locking => {
            if let file::Event::Busy { .. } = arrived.event {
                return refused(store, OpenRefusal::Generations { why: crate::Generations::Busy }, above);
            }
            let file::Event::Locked { .. } = arrived.event else { return refused(store, OpenRefusal::Failed, above) };
            let state = state(store);
            below.store.push(file::Request::Load { owner: directory::OWNER, root: state.source.token, path: Box::from(generations::ACTIVE), max: generations::ACTIVE_BYTES, no_follow: true });
            state.stage = Stage::Reading;
        }
        Stage::Reading => {
            let file::Event::Loaded { bytes, .. } = arrived.event else { return refused(store, OpenRefusal::Generations { why: crate::Generations::Unreadable }, above) };
            let Some(number) = generations::parse(&bytes) else { return refused(store, OpenRefusal::Generations { why: crate::Generations::Unreadable }, above) };
            let state = state(store);
            state.active = Some(generations::Active { number, digest: skein_io::digest::digest(&bytes) });
            state.stage = Stage::Emptying(generations::empty(state.scratch, &mut below.transcripts));
        }
        Stage::Emptying(mut removing) => match generations::removed(&mut removing, arrived, &mut below.transcripts) {
            Progress::Pending => state(store).stage = Stage::Emptying(removing),
            Progress::Failed => refused(store, OpenRefusal::Failed, above),
            Progress::Done => {
                let state = state(store);
                below.transcripts.push(file::Request::MakeDirectory { owner: directory::OWNER, root: state.scratch.token, path: Box::from(generations::GENERATIONS), mode: 0o700 });
                state.stage = Stage::Making;
            }
        },
        Stage::Making => {
            let file::Event::Made { .. } = arrived.event else { return refused(store, OpenRefusal::Failed, above) };
            let chunk = store.limits.store.chunk;
            let state = state(store);
            let number = state.active.expect("active read before scratch copy").number;
            state.stage = Stage::Copying(generations::copy_between(state.source, state.scratch, number, number, chunk, below));
        }
        Stage::Copying(mut copying) => match generations::copied(&mut copying, arrived, below) {
            Progress::Pending => state(store).stage = Stage::Copying(copying),
            Progress::Failed => refused(store, OpenRefusal::Failed, above),
            Progress::Done => {
                let state = state(store);
                below.transcripts.push(file::Request::Store { owner: directory::OWNER, root: state.scratch.token, path: Box::from(generations::ACTIVE), bytes: generations::encode(state.active.expect("copied active").number), expected: file::Expect::Absent, no_follow: true });
                state.stage = Stage::Active;
            }
        },
        Stage::Active => {
            let file::Event::Stored { .. } = arrived.event else { return refused(store, OpenRefusal::Failed, above) };
            let state = state(store);
            let (owner, scratch, installation) = (state.owner, state.scratch, state.installation);
            directory::open(store, owner, scratch, installation, above, below);
        }
        Stage::Opening | Stage::Closing | Stage::Cleaning(_) | Stage::Finished => unreachable!("preparing routed"),
    }
}
fn refused(store: &mut Store, why: OpenRefusal, above: &mut Queue<Event>) {
    let owner = state(store).owner;
    open::refuse(store, owner, why, above);
}

/// Read the source's last wires from its scratch copy, before any checks or patches.
pub(crate) fn announce(store: &mut Store, above: &mut Queue<Event>) {
    let Some(state) = &mut store.rehearsal else { return };
    if state.announced { return; }
    state.announced = true;
    let owner = state.owner;
    let wires = kept_wires(store, owner);
    let fresh = fresh(store, owner);
    above.push(Event::Rehearsing { owner, wires, fresh });
}
fn kept_wires(store: &mut Store, owner: Token) -> Option<Box<[KeptWire]>> {
    let key = crate::keys::encode_own(&crate::keys::Own::Wires, store.limits.store.key)?;
    let bytes = open::get(store, owner, key).ok()??;
    crate::codec::component::kept_wires(&bytes)
}

fn fresh(store: &mut Store, owner: Token) -> bool {
    let Some(key) = crate::keys::encode_own(&crate::keys::Own::Format, store.limits.store.key) else { return false };
    let Ok(Some(bytes)) = open::get(store, owner, key) else { return false };
    let Ok(format) = crate::codec::component::Format::decode(&bytes) else { return false };
    let Ok(plan) = crate::patching::plan::plan(&store.patch_sets, &format.levels, &crate::levels(&store.patch_sets)) else { return false };
    plan.close()
}

pub(crate) fn close(store: &mut Store, owner: Token, above: &mut Queue<Event>) {
    let Some(state) = &mut store.rehearsal else { return open::refuse(store, owner, OpenRefusal::Failed, above) };
    if state.close.is_some() { return; }
    state.close = Some(owner);
    state.stage = Stage::Closing;
    store.phase = Phase::Stopped;
}
pub(crate) fn closing(store: &Store) -> bool {
    match &store.rehearsal { Some(state) => state.close.is_some(), None => false }
}

/// Account every single-terminal file ask, so Close can drain any interrupted phase.
pub(crate) fn lengths(below: &Below) -> [u32; 3] { [below.store.len(), below.secrets.len(), below.transcripts.len()] }
pub(crate) fn issued(store: &mut Store, below: &Below, lengths: [u32; 3]) {
    let Some(state) = &mut store.rehearsal else { return };
    for ((lane, queue), length) in [(crate::host::Lane::Store, &below.store), (crate::host::Lane::Secrets, &below.secrets), (crate::host::Lane::Transcripts, &below.transcripts)].into_iter().zip(lengths) {
        for request in queue.iter().skip(skein_lib::length::usize_of(length)) {
            state.pending = state.pending.checked_add(1).expect("bounded file asks");
            if let file::Request::Close { file, .. } = request {
                let mut kept = List::with_capacity(24);
                for handle in &state.handles {
                    if *handle != (lane, *file) { kept.push(*handle).expect("retained handles fit"); }
                }
                state.handles = kept;
            }
        }
    }
}
pub(crate) fn arriving(store: &mut Store, io: &Io) {
    let Some(state) = &mut store.rehearsal else { return };
    let (lane, arrived) = match io { Io::Store(a) => (crate::host::Lane::Store, a), Io::Secrets(a) => (crate::host::Lane::Secrets, a), Io::Transcripts(a) => (crate::host::Lane::Transcripts, a) };
    state.pending = state.pending.checked_sub(1).expect("file terminal has an ask");
    if let file::Event::Opened { file, .. } = &arrived.event { state.handles.push((lane, *file)).expect("three drivers' open handles fit"); }
}
pub(crate) fn closing_up(store: &mut Store, io: Io, above: &mut Queue<Event>, below: &mut Below) {
    let arrived = match io { Io::Store(a) | Io::Secrets(a) | Io::Transcripts(a) => a };
    let state = state(store);
    if let Stage::Cleaning(removing) = &mut state.stage {
        match generations::removed(removing, arrived, &mut below.transcripts) {
            Progress::Pending => {},
            Progress::Done => ended(store, true, above),
            Progress::Failed => ended(store, false, above),
        }
    } else if let file::Event::Failed { .. } = arrived.event { state.fault = true; }
}
pub(crate) fn advance_close(store: &mut Store, above: &mut Queue<Event>, below: &mut Below) {
    let Some(state) = &mut store.rehearsal else { return };
    if state.close.is_none() || state.pending != 0 { return; }
    if let Stage::Closing = state.stage {
        if let Some((lane, file)) = state.handles.iter().next().copied() {
            let queue = match lane { crate::host::Lane::Store => &mut below.store, crate::host::Lane::Secrets => &mut below.secrets, crate::host::Lane::Transcripts => &mut below.transcripts };
            queue.push(file::Request::Close { owner: directory::OWNER, file });
        } else if state.fault { ended(store, false, above); }
        else { state.stage = Stage::Cleaning(generations::empty(state.scratch, &mut below.transcripts)); }
    }
}
fn ended(store: &mut Store, removed: bool, above: &mut Queue<Event>) {
    let state = state(store);
    state.stage = Stage::Finished;
    above.push(Event::Closed { owner: state.close.expect("close owner retained"), removed });
}

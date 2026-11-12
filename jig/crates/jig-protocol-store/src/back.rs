//! Going back before an active generation's first own commit (protocol/store/opening.md 5).
use alloc::boxed::Box;
use crate::{Arrived, BackRefusal, Below, Event, Io, Limits, Phase, Root, Store, codec, directory, generations};
use skein_io::file;
use skein_lib::{Env, Queue, Token, Writer};

#[derive(Debug)]
enum Stage { Locking, Active, Manifest, Previous, PreviousClosed, Map, Recovering, Replacing, Finished }
#[derive(Debug)]
pub(crate) struct State {
    owner: Token,
    root: Root,
    active: Option<generations::Active>,
    to: u64,
    position: u64,
    stage: Stage,
}
pub(crate) fn begin(store: &mut Store, owner: Token, root: Root, above: &mut Queue<Event>, below: &mut Below) {
    if store.phase != Phase::Closed || store.directory.root.is_some() || store.back.is_some() || store.rehearsal.is_some() || !store.directory.disk {
        above.push(Event::BackRefused { owner, why: BackRefusal::Busy });
        return;
    }
    store.back = Some(State { owner, root, active: None, to: 0, position: 0, stage: Stage::Locking });
    below.store.push(file::Request::Lock { owner: directory::OWNER, root: root.token });
}
fn state(store: &mut Store) -> &mut State { store.back.as_mut().expect("back routing") }
fn refuse(store: &mut Store, why: BackRefusal, above: &mut Queue<Event>) {
    let state = state(store);
    state.stage = Stage::Finished;
    above.push(Event::BackRefused { owner: state.owner, why });
    store.phase = Phase::Stopped;
}
pub(crate) fn up(store: &mut Store, env: &Env<Limits>, io: Io, above: &mut Queue<Event>, below: &mut Below) {
    let arrived = match io { Io::Store(a) | Io::Secrets(a) | Io::Transcripts(a) => a };
    if arrived.event.owner() != directory::OWNER {
        let mut events = Queue::uncapped();
        skein_kv::up(&mut store.stores.store, &env.with(store.limits.store), arrived.event, &mut events, &mut below.store);
        recovered(store, &mut events, above, below);
        return;
    }
    file_up(store, env, arrived, above, below);
}
fn file_up(store: &mut Store, env: &Env<Limits>, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let transition = core::mem::replace(&mut state(store).stage, Stage::Finished);
    match transition {
        Stage::Locking => {
            if let file::Event::Busy { .. } = arrived.event { return refuse(store, BackRefusal::Busy, above); }
            let file::Event::Locked { .. } = arrived.event else { return refuse(store, BackRefusal::Unreadable, above) };
            let state = state(store);
            below.store.push(file::Request::Load { owner: directory::OWNER, root: state.root.token, path: Box::from(generations::ACTIVE), max: generations::ACTIVE_BYTES, no_follow: true });
            state.stage = Stage::Active;
        }
        Stage::Active => {
            let file::Event::Loaded { bytes, .. } = arrived.event else { return refuse(store, BackRefusal::Unreadable, above) };
            let Some(number) = generations::parse(&bytes) else { return refuse(store, BackRefusal::Unreadable, above) };
            if number == generations::FIRST { return refuse(store, BackRefusal::NoPrevious, above); }
            let state = state(store);
            state.active = Some(generations::Active { number, digest: skein_io::digest::digest(&bytes) });
            let generation = generations::generation(number);
            let mut path = Writer::new(generation.len().checked_add(9).expect("bounded manifest path"));
            path.put(&generation).expect("measured generation");
            path.put(b"/manifest").expect("measured manifest path");
            below.store.push(file::Request::Load { owner: directory::OWNER, root: state.root.token, path: path.finish(), max: crate::sizes::MANIFEST_BYTES, no_follow: true });
            state.stage = Stage::Manifest;
        }
        Stage::Manifest => {
            let file::Event::Loaded { bytes, .. } = arrived.event else { return refuse(store, BackRefusal::Unreadable, above) };
            let Ok(manifest) = codec::component::Manifest::decode(&bytes) else { return refuse(store, BackRefusal::Unreadable, above) };
            let state = state(store);
            let number = state.active.expect("active read").number;
            let Some(to) = manifest.from else { return refuse(store, BackRefusal::NoPrevious, above) };
            if manifest.generation != number || to == 0 || Some(to) != number.checked_sub(1) { return refuse(store, BackRefusal::Unreadable, above); }
            state.to = to;
            state.position = manifest.store;
            below.store.push(file::Request::OpenDirectory { owner: directory::OWNER, root: state.root.fd, name: generations::map(to, generations::Map::Store), no_follow: true });
            state.stage = Stage::Previous;
        }
        Stage::Previous => {
            let Some(root) = directory::opened(arrived) else { return refuse(store, BackRefusal::NoPrevious, above) };
            below.store.push(file::Request::Close { owner: directory::OWNER, file: root.token });
            state(store).stage = Stage::PreviousClosed;
        }
        Stage::PreviousClosed => {
            let file::Event::Closed { .. } = arrived.event else { return refuse(store, BackRefusal::Unreadable, above) };
            let state = state(store);
            below.store.push(file::Request::OpenDirectory { owner: directory::OWNER, root: state.root.fd, name: generations::map(state.active.expect("active read").number, generations::Map::Store), no_follow: true });
            state.stage = Stage::Map;
        }
        Stage::Map => {
            let Some(root) = directory::opened(arrived) else { return refuse(store, BackRefusal::Unreadable, above) };
            state(store).stage = Stage::Recovering;
            let mut events = Queue::uncapped();
            skein_kv::down(&mut store.stores.store, &env.with(store.limits.store), skein_kv::Request::Open { owner: directory::OWNER, root: root.fd }, &mut events, &mut below.store);
            recovered(store, &mut events, above, below);
        }
        Stage::Replacing => {
            let file::Event::Stored { .. } = arrived.event else { return refuse(store, BackRefusal::Unreadable, above) };
            let state = state(store);
            above.push(Event::Back { owner: state.owner, to: state.to });
            store.phase = Phase::Stopped;
        }
        Stage::Recovering | Stage::Finished => refuse(store, BackRefusal::Unreadable, above),
    }
}
fn recovered(store: &mut Store, events: &mut Queue<skein_kv::Event>, above: &mut Queue<Event>, below: &mut Below) {
    for _ in 0..events.len() {
        let event = events.pop().expect("bounded terminals");
        match event {
            skein_kv::Event::Opened { .. } => {
                let last = store.stores.store.last();
                let state = state(store);
                if last != state.position { let manifest = state.position; return refuse(store, BackRefusal::Committed { manifest, last }, above); }
                below.store.push(file::Request::Store { owner: directory::OWNER, root: state.root.token, path: Box::from(generations::ACTIVE), bytes: generations::encode(state.to), expected: file::Expect::Digest(state.active.expect("active read").digest), no_follow: true });
                state.stage = Stage::Replacing;
            }
            skein_kv::Event::OpenFailed { .. } | skein_kv::Event::Refused { .. } | skein_kv::Event::Failed { .. } | skein_kv::Event::Got { .. } | skein_kv::Event::Loaded { .. } | skein_kv::Event::Committed { .. } | skein_kv::Event::Closed { .. } | skein_kv::Event::Recovered { .. } | skein_kv::Event::RecoveryFailed { .. } => refuse(store, BackRefusal::Unreadable, above),
        }
    }
}

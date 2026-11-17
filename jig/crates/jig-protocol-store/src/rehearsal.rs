//! Scratch starts and their cancellable close (protocol/store/opening.md 5).
use crate::{Arrived, Below, Event, Io, KeptWire, OpenRefusal, Phase, Root, Store, directory, generations, open};
use alloc::boxed::Box;
use generations::{Copying, Progress, Removing};
use skein_io::file;
use skein_lib::{Queue, Token};
mod decode;
pub(crate) use decode::records;

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
    files: crate::file_handles::Owned,
}

pub(crate) fn begin(
    store: &mut Store,
    owner: Token,
    source: Root,
    scratch: Root,
    installation: [u8; 16],
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    if store.phase != Phase::Closed
        || store.directory.root.is_some()
        || store.rehearsal.is_some()
        || !store.directory.disk
        || source.fd == scratch.fd
    {
        return open::refuse(store, owner, OpenRefusal::Failed, above);
    }
    store.rehearsal = Some(State {
        source,
        scratch,
        owner,
        installation,
        active: None,
        stage: Stage::Locking,
        announced: false,
        close: None,
        fault: false,
        files: crate::file_handles::Owned::new(),
    });
    store.directory.owner = owner;
    below.store.push(file::Request::Lock { owner: directory::OWNER, root: source.token });
}
pub(crate) fn source(store: &Store) -> Root {
    store.rehearsal.as_ref().expect("rehearsal retained").source
}
fn state(store: &mut Store) -> &mut State {
    store.rehearsal.as_mut().expect("rehearsal routing")
}

pub(crate) fn preparing(store: &Store) -> bool {
    let Some(state) = &store.rehearsal else { return false };
    match state.stage {
        Stage::Locking | Stage::Reading | Stage::Emptying(_) | Stage::Making | Stage::Copying(_) | Stage::Active => {
            true
        }
        Stage::Opening | Stage::Closing | Stage::Cleaning(_) | Stage::Finished => false,
    }
}
pub(crate) fn waiting(store: &Store) -> Option<&'static str> {
    let state = store.rehearsal.as_ref()?;
    Some(match state.stage {
        Stage::Locking => "lock source",
        Stage::Reading => "read source active",
        Stage::Emptying(_) => "empty scratch",
        Stage::Making => "make scratch generations",
        Stage::Copying(_) => "copy source generation",
        Stage::Active => "write scratch active",
        Stage::Opening => "open scratch store",
        Stage::Closing => "close scratch handles",
        Stage::Cleaning(_) => "remove scratch entries",
        Stage::Finished => "finished",
    })
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
            below.store.push(file::Request::Load {
                owner: directory::OWNER,
                root: state.source.token,
                path: Box::from(generations::ACTIVE),
                max: generations::ACTIVE_BYTES,
                no_follow: true,
            });
            state.stage = Stage::Reading;
        }
        Stage::Reading => {
            let file::Event::Loaded { bytes, .. } = arrived.event else {
                return refused(store, OpenRefusal::Generations { why: crate::Generations::Unreadable }, above);
            };
            let Some(number) = generations::parse(&bytes) else {
                return refused(store, OpenRefusal::Generations { why: crate::Generations::Unreadable }, above);
            };
            let state = state(store);
            state.active = Some(generations::Active { number, digest: skein_io::digest::digest(&bytes) });
            state.stage = Stage::Emptying(generations::empty(state.scratch, &mut below.transcripts));
        }
        Stage::Emptying(mut removing) => match generations::removed(&mut removing, arrived, &mut below.transcripts) {
            Progress::Pending => state(store).stage = Stage::Emptying(removing),
            Progress::Failed => refused(store, OpenRefusal::Failed, above),
            Progress::Done => {
                let state = state(store);
                below.transcripts.push(file::Request::MakeDirectory {
                    owner: directory::OWNER,
                    root: state.scratch.token,
                    path: Box::from(generations::GENERATIONS),
                    mode: 0o700,
                });
                state.stage = Stage::Making;
            }
        },
        Stage::Making => {
            let file::Event::Made { .. } = arrived.event else { return refused(store, OpenRefusal::Failed, above) };
            let chunk = store.limits.store.chunk;
            let state = state(store);
            let number = state.active.expect("active read before scratch copy").number;
            state.stage =
                Stage::Copying(generations::copy_between(state.source, state.scratch, number, number, chunk, below));
        }
        Stage::Copying(mut copying) => match generations::copied(&mut copying, arrived, below) {
            Progress::Pending => state(store).stage = Stage::Copying(copying),
            Progress::Failed => refused(store, OpenRefusal::Failed, above),
            Progress::Done => {
                let state = state(store);
                below.transcripts.push(file::Request::Store {
                    owner: directory::OWNER,
                    root: state.scratch.token,
                    path: Box::from(generations::ACTIVE),
                    bytes: generations::encode(state.active.expect("copied active").number),
                    expected: file::Expect::Absent,
                    no_follow: true,
                });
                state.stage = Stage::Active;
            }
        },
        Stage::Active => {
            let file::Event::Stored { .. } = arrived.event else { return refused(store, OpenRefusal::Failed, above) };
            let state = state(store);
            let (owner, scratch, installation) = (state.owner, state.scratch, state.installation);
            directory::open(store, owner, scratch, installation, crate::Purpose::Serve, above, below);
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
    if state.announced {
        return;
    }
    state.announced = true;
    let owner = state.owner;
    let wires = kept_wires(store, owner);
    let openings = crate::openings::kept(store).unwrap_or(Box::new([]));
    above.push(Event::Rehearsing { owner, wires, openings });
}
fn kept_wires(store: &mut Store, owner: Token) -> Option<Box<[KeptWire]>> {
    let key = crate::keys::encode_own(&crate::keys::Own::Wires, store.limits.store.key)?;
    let bytes = open::get(store, owner, key).ok()??;
    crate::rows::kept_wires(&bytes)
}

pub(crate) fn close(store: &mut Store, owner: Token, above: &mut Queue<Event>) {
    let Some(state) = &mut store.rehearsal else { return open::refuse(store, owner, OpenRefusal::Failed, above) };
    if state.close.is_some() {
        return;
    }
    state.close = Some(owner);
    state.stage = Stage::Closing;
    store.phase = Phase::Stopped;
}
pub(crate) fn closing(store: &Store) -> bool {
    match &store.rehearsal {
        Some(state) => state.close.is_some(),
        None => false,
    }
}

/// Account every single-terminal file ask, so Close can drain any interrupted phase.
pub(crate) fn lengths(below: &Below) -> [u32; 4] {
    [below.store.len(), below.secrets.len(), below.transcripts.len(), below.checkpoints.len()]
}
pub(crate) fn issued(store: &mut Store, below: &Below, lengths: [u32; 4]) {
    let Some(state) = &mut store.rehearsal else { return };
    state.files.issued(below, lengths);
}
pub(crate) fn arriving(store: &mut Store, io: &Io) {
    let Some(state) = &mut store.rehearsal else { return };
    state.files.arriving(io);
}

pub(crate) fn closing_up(store: &mut Store, io: Io, above: &mut Queue<Event>, below: &mut Below) {
    let arrived = match io {
        Io::Store(a) | Io::Secrets(a) | Io::Transcripts(a) | Io::Checkpoints(a) => a,
    };
    let state = state(store);
    match &mut state.stage {
        Stage::Cleaning(removing) => match generations::removed(removing, arrived, &mut below.transcripts) {
            Progress::Pending => {}
            Progress::Done => ended(store, true, above),
            Progress::Failed => ended(store, false, above),
        },
        Stage::Closing => {
            if let file::Event::Failed { .. } = arrived.event {
                state.fault = true;
            }
        }
        Stage::Locking
        | Stage::Reading
        | Stage::Emptying(_)
        | Stage::Making
        | Stage::Copying(_)
        | Stage::Active
        | Stage::Opening
        | Stage::Finished => {}
    }
}
pub(crate) fn advance_close(store: &mut Store, above: &mut Queue<Event>, below: &mut Below) {
    let Some(state) = &mut store.rehearsal else { return };
    if state.close.is_none() || state.files.pending != 0 {
        return;
    }
    match state.stage {
        Stage::Closing => {
            if state.files.drain(below) {
                if state.fault {
                    ended(store, false, above);
                } else {
                    state.stage = Stage::Cleaning(generations::empty(state.scratch, &mut below.transcripts));
                }
            }
        }
        Stage::Locking
        | Stage::Reading
        | Stage::Emptying(_)
        | Stage::Making
        | Stage::Copying(_)
        | Stage::Active
        | Stage::Opening
        | Stage::Cleaning(_)
        | Stage::Finished => {}
    }
}
fn ended(store: &mut Store, removed: bool, above: &mut Queue<Event>) {
    let state = state(store);
    state.stage = Stage::Finished;
    above.push(Event::Closed { owner: state.close.expect("close owner retained"), removed });
}

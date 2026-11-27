//! Restoring over a stopped deployment (protocol/store/restoring.md, 4).
use crate::{
    Arrived, Below, CheckpointName, CheckpointRefusal, Event, Limits, Phase, RestoreRefusal, Restored, Root, Store,
    checkpoints,
    codec::component::{Checkpoint, Manifest},
    directory, generations,
};
use alloc::boxed::Box;
use skein_codec::Record as _;
use skein_io::{file, kernel::Error};
use skein_lib::{Env, List, Queue, Time, Token};
mod removal;
pub(crate) mod roll;
pub(crate) mod staging;
pub(crate) use checkpoints::files::{error, join};
pub(crate) const OWNER: Token = Token::new(0x5253_5452);
#[derive(Debug)]
enum Step {
    Lock,
    Root,
    Source,
    SourceLock,
    Record,
    Manifest,
    Opening,
    Taking,
    Closing,
    Safety,
    SafetyLock,
    Staging(Box<staging::Staging>),
    Intent,
    IntentCheck { error: Error },
    Rolling(roll::Rolling),
    GenerationCleanup(removal::Generation),
    Cleaning(generations::Removing),
    Drain,
    RootClose,
}
#[derive(Debug)]
pub(crate) struct State {
    owner: Token,
    root: Root,
    lane: Option<Root>,
    source: Option<Root>,
    name: CheckpointName,
    draw: [u8; 16],
    began: Time,
    active: Option<generations::Active>,
    record: Option<Checkpoint>,
    manifest: Option<Manifest>,
    safety: Option<CheckpointName>,
    handles: List<Token>,
    step: Step,
    terminal: Option<Result<Restored, RestoreRefusal>>,
    counts: (u32, u32, u64),
    path: Box<[u8]>,
}
#[expect(clippy::too_many_arguments, reason = "the host supplies the adoption's roots, name and draw")]
pub(crate) fn begin(
    store: &mut Store,
    env: &Env<Limits>,
    owner: Token,
    root: Root,
    name: CheckpointName,
    draw: [u8; 16],
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    if store.phase != Phase::Closed
        || store.directory.root.is_some()
        || store.restoring.is_some()
        || !store.directory.disk
    {
        above.push(Event::RestoreRefused { owner, why: RestoreRefusal::Busy });
        return;
    }
    store.restoring = Some(Box::new(State {
        owner,
        root,
        lane: None,
        source: None,
        name,
        draw,
        began: env.now,
        active: None,
        record: None,
        manifest: None,
        safety: None,
        handles: List::with_capacity(3),
        step: Step::Lock,
        terminal: None,
        counts: (0, 0, 0),
        path: Box::from(b".".as_slice()),
    }));
    below.store.push(file::Request::Lock { owner: OWNER, root: root.token });
}
fn source_path(state: &State) -> Box<[u8]> {
    join(b"checkpoints/", &state.name.bytes())
}
fn open_dir(state: &mut State, path: Box<[u8]>, below: &mut Below) {
    state.path.clone_from(&path);
    below.checkpoints.push(file::Request::OpenDirectory {
        owner: OWNER,
        root: state.root.fd,
        name: path,
        no_follow: true,
    });
}
fn opened(state: &mut State, arrived: Arrived) -> Option<Root> {
    let root = directory::opened(arrived)?;
    state.handles.push(root.token).expect("lane, source, safety");
    Some(root)
}
fn load(state: &mut State, path: &[u8], max: u32, below: &mut Below) {
    state.path = join(&join(&source_path(state), b"/"), path);
    below.checkpoints.push(file::Request::Load {
        owner: OWNER,
        root: state.source.expect("source locked").token,
        path: Box::from(path),
        max,
        no_follow: true,
    });
}
fn terminal(state: &mut State, why: RestoreRefusal) {
    state.terminal = Some(Err(why));
}
fn drain(state: &mut State, below: &mut Below) {
    if let Some(index) = state.handles.len().checked_sub(1) {
        let file = state.handles.swap_remove(index).expect("last restore handle");
        below.checkpoints.push(file::Request::Close { owner: OWNER, file });
        state.step = Step::Drain;
    } else {
        below.store.push(file::Request::Close { owner: OWNER, file: state.root.token });
        state.step = Step::RootClose;
    }
}
fn staging_failure(state: &mut State, path: Box<[u8]>, error: Option<Error>, below: &mut Below) {
    terminal(
        state,
        RestoreRefusal::Staging {
            safety: state.safety.expect("safety checkpoint taken"),
            path: checkpoints::files::text(&path),
            error,
        },
    );
    state.path = generations::generation(state.active.expect("active read").number.saturating_add(1));
    state.step = Step::GenerationCleanup(removal::Generation::begin(
        state.lane.expect("lane root"),
        state.active.expect("active read").number.saturating_add(1),
        OWNER,
        &mut below.checkpoints,
    ));
}
pub(crate) fn owns(io: &crate::Io) -> bool {
    let arrived = match io {
        crate::Io::Store(a) | crate::Io::Secrets(a) | crate::Io::Transcripts(a) | crate::Io::Checkpoints(a) => a,
    };
    arrived.event.owner() == OWNER
}
#[expect(
    clippy::too_many_lines,
    clippy::manual_filter,
    reason = "one restore transition table; optional validation is spelled without a closure"
)]
pub(crate) fn up(store: &mut Store, env: &Env<Limits>, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let mut state = store.restoring.take().expect("restore file terminal");
    let error = error(&arrived.event);
    match core::mem::replace(&mut state.step, Step::Drain) {
        Step::Lock => {
            if let file::Event::Locked { .. } = arrived.event {
                open_dir(&mut state, Box::from(b".".as_slice()), below);
                state.step = Step::Root;
            } else {
                let why = if let file::Event::Busy { .. } = arrived.event {
                    RestoreRefusal::Busy
                } else {
                    RestoreRefusal::Open { why: crate::OpenRefusal::Failed }
                };
                terminal(&mut state, why);
                drain(&mut state, below);
            }
        }
        Step::Root => {
            if let Some(root) = opened(&mut state, arrived) {
                state.lane = Some(root);
                let path = source_path(&state);
                open_dir(&mut state, path, below);
                state.step = Step::Source;
            } else {
                terminal(&mut state, RestoreRefusal::Open { why: crate::OpenRefusal::Failed });
                drain(&mut state, below);
            }
        }
        Step::Source => {
            if let Some(root) = opened(&mut state, arrived) {
                state.source = Some(root);
                below.checkpoints.push(file::Request::Lock { owner: OWNER, root: root.token });
                state.step = Step::SourceLock;
            } else {
                let why = if error == Some(Error::NotFound) {
                    CheckpointRefusal::Missing
                } else {
                    CheckpointRefusal::Failed {
                        path: checkpoints::files::text(&state.path),
                        error: error.unwrap_or(Error::Other(0)),
                    }
                };
                terminal(&mut state, RestoreRefusal::Source { why });
                drain(&mut state, below);
            }
        }
        Step::SourceLock => {
            if let file::Event::Locked { .. } = arrived.event {
                let max = u32::try_from(
                    crate::component_v3::Checkpoint::encoded_bound(&crate::component_v3::CEILINGS)
                        .expect("record bound"),
                )
                .expect("io whole file bound");
                load(&mut state, b"checkpoint", max, below);
                state.step = Step::Record;
            } else {
                let why = if let file::Event::Busy { .. } = arrived.event {
                    CheckpointRefusal::InUse
                } else {
                    CheckpointRefusal::Failed {
                        path: checkpoints::files::text(&state.path),
                        error: error.unwrap_or(Error::Other(0)),
                    }
                };
                terminal(&mut state, RestoreRefusal::Source { why });
                drain(&mut state, below);
            }
        }
        Step::Record => {
            let record = match arrived.event {
                file::Event::Loaded { bytes, .. } => Checkpoint::decode(&bytes).ok(),
                file::Event::Opened { .. }
                | file::Event::Refused { .. }
                | file::Event::Locked { .. }
                | file::Event::Busy { .. }
                | file::Event::Scanned { .. }
                | file::Event::Stored { .. }
                | file::Event::Conflict { .. }
                | file::Event::TooLarge { .. }
                | file::Event::Stated { .. }
                | file::Event::Written { .. }
                | file::Event::Read { .. }
                | file::Event::Cut { .. }
                | file::Event::Synced { .. }
                | file::Event::Closed { .. }
                | file::Event::Linked { .. }
                | file::Event::Renamed { .. }
                | file::Event::Removed { .. }
                | file::Event::Listed { .. }
                | file::Event::Cancelled { .. }
                | file::Event::Made { .. }
                | file::Event::Failed { .. } => None,
            };
            let record = match record {
                Some(record) => {
                    if valid(&record) {
                        Some(record)
                    } else {
                        None
                    }
                }
                None => None,
            };
            if let Some(record) = record {
                state.record = Some(record);
                load(&mut state, b"generations/1/manifest", crate::sizes::MANIFEST_BYTES, below);
                state.step = Step::Manifest;
            } else {
                let path = checkpoints::files::text(&state.path);
                terminal(&mut state, RestoreRefusal::Unreadable { path });
                drain(&mut state, below);
            }
        }
        Step::Manifest => {
            let manifest = match arrived.event {
                file::Event::Loaded { bytes, .. } => Manifest::decode(&bytes).ok(),
                file::Event::Opened { .. }
                | file::Event::Refused { .. }
                | file::Event::Locked { .. }
                | file::Event::Busy { .. }
                | file::Event::Scanned { .. }
                | file::Event::Stored { .. }
                | file::Event::Conflict { .. }
                | file::Event::TooLarge { .. }
                | file::Event::Stated { .. }
                | file::Event::Written { .. }
                | file::Event::Read { .. }
                | file::Event::Cut { .. }
                | file::Event::Synced { .. }
                | file::Event::Closed { .. }
                | file::Event::Linked { .. }
                | file::Event::Renamed { .. }
                | file::Event::Removed { .. }
                | file::Event::Listed { .. }
                | file::Event::Cancelled { .. }
                | file::Event::Made { .. }
                | file::Event::Failed { .. } => None,
            };
            if let Some(manifest) = manifest {
                state.manifest = Some(manifest);
                state.step = Step::Opening;
                let (owner, root) = (state.owner, state.root);
                store.restoring = Some(state);
                directory::open_locked(store, owner, root, below);
                return;
            }
            let path = checkpoints::files::text(&state.path);
            terminal(&mut state, RestoreRefusal::Unreadable { path });
            drain(&mut state, below);
        }
        Step::Safety => {
            if let Some(root) = opened(&mut state, arrived) {
                below.checkpoints.push(file::Request::Lock { owner: OWNER, root: root.token });
                state.step = Step::SafetyLock;
            } else {
                let path = state.path.clone();
                staging_failure(&mut state, path, error, below);
            }
        }
        Step::SafetyLock => {
            if let file::Event::Locked { .. } = arrived.event {
                let Some(generation) = state.active.expect("checkpoint open read active").number.checked_add(1) else {
                    let path = Box::from(generations::GENERATIONS);
                    staging_failure(&mut state, path, Some(Error::InvalidArgument), below);
                    store.restoring = Some(state);
                    return;
                };
                state.step = Step::Staging(Box::new(staging::Staging::begin(
                    state.source.expect("source locked"),
                    state.lane.expect("lane root"),
                    state.record.take().expect("source record"),
                    state.manifest.take().expect("source manifest"),
                    generation,
                    OWNER,
                    &mut below.checkpoints,
                )));
            } else {
                let path = state.path.clone();
                staging_failure(&mut state, path, error, below);
            }
        }
        Step::Staging(mut staging) => match staging.arrived(arrived, env.wall.as_nanos(), &mut below.checkpoints) {
            generations::Progress::Pending => state.step = Step::Staging(staging),
            generations::Progress::Failed => {
                let mut path = staging.path();
                if path.starts_with(b"generations/1/") || path.starts_with(b"transcripts/") {
                    path = join(&join(&source_path(&state), b"/"), &path);
                }
                staging_failure(&mut state, path, staging.failure(), below);
            }
            generations::Progress::Done => {
                state.counts = (staging.linked(), staging.copied(), staging.bytes());
                let active = state.active.expect("active read");
                let intent = roll::Intent {
                    generation: active.number.checked_add(1).expect("staging checked next generation"),
                    active: active.digest,
                    source: state.name,
                    safety: state.safety.expect("safety checkpoint"),
                    made: env.wall.as_nanos(),
                };
                state.path = Box::from(roll::INTENT);
                below.checkpoints.push(file::Request::Store {
                    owner: OWNER,
                    root: state.lane.expect("lane root").token,
                    path: state.path.clone(),
                    bytes: intent.encode(),
                    expected: file::Expect::Absent,
                    no_follow: true,
                });
                state.step = Step::Intent;
            }
        },
        Step::Intent => {
            if let file::Event::Stored { .. } = arrived.event {
                state.step =
                    Step::Rolling(roll::Rolling::begin(state.lane.expect("lane root"), OWNER, &mut below.checkpoints));
            } else {
                below.checkpoints.push(file::Request::Load {
                    owner: OWNER,
                    root: state.lane.expect("lane root").token,
                    path: Box::from(roll::INTENT),
                    max: 256,
                    no_follow: true,
                });
                state.step = Step::IntentCheck { error: error.unwrap_or(Error::Other(0)) };
            }
        }
        Step::IntentCheck { error } => {
            if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
                staging_failure(&mut state, Box::from(roll::INTENT), Some(error), below);
            } else {
                let safety = state.safety.expect("safety checkpoint");
                terminal(&mut state, RestoreRefusal::Interrupted { safety, path: Box::from("restore"), error });
                drain(&mut state, below);
            }
        }

        Step::Rolling(mut rolling) => match rolling.arrived(arrived, &mut below.checkpoints) {
            roll::Progress::Pending => state.step = Step::Rolling(rolling),
            roll::Progress::Done => {
                state.terminal = Some(Ok(Restored {
                    name: state.name,
                    safety: state.safety.expect("safety checkpoint"),
                    generation: state.active.expect("active read").number.saturating_add(1),
                    linked: state.counts.0,
                    copied: state.counts.1,
                    bytes: state.counts.2,
                    took: env.now.as_nanos().saturating_sub(state.began.as_nanos()),
                }));
                drain(&mut state, below);
            }
            roll::Progress::Failed | roll::Progress::Unmatched => {
                let safety = state.safety.expect("safety checkpoint");
                terminal(
                    &mut state,
                    RestoreRefusal::Interrupted {
                        safety,
                        path: checkpoints::files::text(&rolling.path()),
                        error: rolling.error().unwrap_or(Error::Other(0)),
                    },
                );
                drain(&mut state, below);
            }
        },
        Step::GenerationCleanup(mut removing) => match removing.arrived(arrived, &mut below.checkpoints) {
            generations::Progress::Pending => state.step = Step::GenerationCleanup(removing),
            generations::Progress::Done | generations::Progress::Failed => {
                state.step = Step::Cleaning(generations::remove_checkpoint_owned(
                    state.lane.expect("lane root"),
                    Box::from(roll::STAGED),
                    OWNER,
                    &mut below.checkpoints,
                ));
            }
        },
        Step::Cleaning(mut removing) => match generations::removed(&mut removing, arrived, &mut below.checkpoints) {
            generations::Progress::Pending => state.step = Step::Cleaning(removing),
            generations::Progress::Done | generations::Progress::Failed => drain(&mut state, below),
        },
        Step::Drain => {
            close_failure(&mut state, error);
            drain(&mut state, below);
        }
        Step::RootClose => {
            close_failure(&mut state, error);
            let terminal = match state.terminal.take().expect("restore terminal before releasing locks") {
                Ok(mut restored) => {
                    restored.took = env.now.as_nanos().saturating_sub(state.began.as_nanos());
                    Event::Restored { owner: state.owner, restored }
                }
                Err(why) => Event::RestoreRefused { owner: state.owner, why },
            };
            above.push(terminal);
            store.phase = Phase::Stopped;
            return;
        }
        Step::Opening | Step::Taking | Step::Closing => unreachable!("store terminals drive the safety checkpoint"),
    }
    store.restoring = Some(state);
}
fn close_failure(state: &mut State, error: Option<Error>) {
    let Some(error) = error else {
        return;
    };
    if let Some(Ok(_)) = &state.terminal {
        terminal(
            state,
            RestoreRefusal::Interrupted {
                safety: state.safety.expect("restore completed"),
                path: Box::from("."),
                error,
            },
        );
    }
}
/// The source is compared as soon as the safety open decodes its format row (restoring.md, 4.2).
pub(crate) fn deployment(store: &mut Store, id: [u8; 16]) -> bool {
    let Some(state) = &mut store.restoring else {
        return true;
    };
    let same = state.record.as_ref().expect("source read before safety open").deployment == id;
    if !same {
        terminal(state, RestoreRefusal::Deployment);
    }
    same
}
fn valid(record: &Checkpoint) -> bool {
    for row in &record.files {
        if !(1..=6).contains(&row.part) || !(1..=3).contains(&row.way) || row.way == 3 && row.part != 4 {
            return false;
        }
    }
    true
}
/// The host asks for an adoption, but the component owns the safety open and close.
pub(crate) fn events(
    store: &mut Store,
    env: &Env<Limits>,
    events: &mut Queue<Event>,
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    for _ in 0..events.len() {
        let event = events.pop().expect("store terminals");
        let Some(mut state) = store.restoring.take() else {
            above.push(event);
            continue;
        };
        match event {
            Event::Opened { .. } => {
                state.active = store.directory.active;
                state.step = Step::Taking;
                let (owner, draw) = (state.owner, state.draw);
                store.restoring = Some(state);
                checkpoints::taking::ask(store, env, owner, draw, events, below);
            }
            Event::Checkpointed { checkpoint, .. } => {
                state.safety = Some(checkpoint.name);
                close(store, state, events);
            }
            Event::CheckpointRefused { why, .. } => {
                terminal(&mut state, RestoreRefusal::Safety { why });
                close(store, state, events);
            }
            Event::OpenRefused { why, .. } => {
                if state.terminal.is_none() {
                    terminal(&mut state, RestoreRefusal::Open { why });
                }
                close(store, state, events);
            }
            Event::Closed { .. } => {
                if state.terminal.is_some() {
                    drain(&mut state, below);
                } else {
                    let path = join(b"checkpoints/", &state.safety.expect("safety checkpoint").bytes());
                    open_dir(&mut state, path, below);
                    state.step = Step::Safety;
                }
                store.restoring = Some(state);
            }
            Event::Corrupt { .. } => {
                terminal(&mut state, RestoreRefusal::Open { why: crate::OpenRefusal::Failed });
                close(store, state, events);
            }
            Event::Census { .. } => {
                store.restoring = Some(state);
            }
            other @ (Event::Restored { .. }
            | Event::RestoreRefused { .. }
            | Event::CheckpointHeld { .. }
            | Event::CheckpointRemoved { .. }
            | Event::Rehearsing { .. }
            | Event::Decoded { .. }
            | Event::Undecodable { .. }
            | Event::Back { .. }
            | Event::BackRefused { .. }
            | Event::Switched { .. }
            | Event::Committed { .. }
            | Event::Failed { .. }
            | Event::FrameTooLarge { .. }
            | Event::Page { .. }
            | Event::PageRefused { .. }
            | Event::Loaded { .. }
            | Event::Read { .. }
            | Event::Got { .. }
            | Event::Turns { .. }
            | Event::WindowFailed { .. }
            | Event::Unread { .. }
            | Event::SecretsNotReady { .. }
            | Event::Kept { .. }
            | Event::Carried { .. }
            | Event::Stale { .. }
            | Event::Erased { .. }
            | Event::Secret { .. }
            | Event::SecretsLoaded { .. }
            | Event::Job { .. }
            | Event::Ran { .. }) => {
                store.restoring = Some(state);
                above.push(other);
            }
        }
    }
}
fn close(store: &mut Store, mut state: Box<State>, events: &mut Queue<Event>) {
    state.step = Step::Closing;
    let owner = state.owner;
    store.restoring = Some(state);
    crate::checkpoint_open::close(store, owner, events);
}

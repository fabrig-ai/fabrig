//! One serving checkpoint lifecycle (protocol/store/checkpoints.md, 4).
use super::{CheckpointName, FILES, files, selection::Selection, sweep};
use crate::{
    Arrived, Below, CheckpointRefusal, Checkpointed, Event, Limits, Phase, Root, Store,
    codec::component::{Checkpoint, Format},
    directory, generations, keys, open,
};
use alloc::boxed::Box;
use skein_io::file;
use skein_lib::{Env, List, Queue, Time, Token, Wall};
pub(crate) const OWNER: Token = Token::new(0x434b_5054);

#[derive(Debug)]
pub(crate) struct Ask {
    pub owner: Token,
    pub draw: [u8; 16],
    pub began: Time,
}
#[derive(Debug)]
pub(crate) struct Session {
    pub ask: Ask,
    pub root: Root,
    /// Every owned handle, in opening order; failure closes them in reverse order.
    pub handles: List<Token>,
    pub path: Box<[u8]>,
}
#[derive(Debug)]
pub(crate) struct Cut {
    pub session: Session,
    pub name: CheckpointName,
    pub temporary: Box<[u8]>,
    pub record: Checkpoint,
    pub sources: List<super::selection::Source>,
    pub deferred: Queue<u64>,
    pub paused: u64,
    pub linked: u32,
    pub copied: u32,
    pub bytes: u64,
    pub removed: u32,
    pub skipped: u32,
}
#[derive(Debug)]
pub(crate) enum Taking {
    Idle,
    Closing { owner: Token },
    Opening(Ask),
    Preparing { session: Session, step: Prepare },
    Queued(Session),
    Holding { session: Session, began: Time, made: Wall, main: HoldAnswer, secrets: HoldAnswer },
    Selecting { session: Session, began: Time, made: Wall, store: u64, secrets: u64, selection: Selection },
    Files { cut: Cut, step: files::Step },
    Cleaning { session: Session, why: CheckpointRefusal, temporary: Option<Box<[u8]>>, step: Cleanup },
}
#[derive(Debug)]
pub(crate) enum Prepare {
    Make,
    Sweep(sweep::Sweep),
}
#[derive(Debug)]
pub(crate) enum Cleanup {
    Close,
    ParentOpen,
    Remove(generations::Removing),
    Done,
}
#[derive(Debug)]
pub(crate) enum HoldAnswer {
    Waiting,
    Refused,
    Granted { last: u64, snapshot: bool, segments: Box<[skein_kv::HeldSegment]> },
}
impl Taking {
    pub(crate) fn new() -> Self {
        Self::Idle
    }
}
pub(crate) fn busy(store: &Store) -> bool {
    match store.checkpoint.as_ref() {
        Taking::Idle => false,
        Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Queued(_)
        | Taking::Holding { .. }
        | Taking::Selecting { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { .. } => true,
    }
}
pub(crate) fn barrier(store: &Store) -> bool {
    match store.checkpoint.as_ref() {
        Taking::Holding { .. } | Taking::Selecting { .. } => true,
        Taking::Idle
        | Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Queued(_)
        | Taking::Files { .. }
        | Taking::Cleaning { .. } => false,
    }
}
pub(crate) fn prepared(store: &Store) -> bool {
    match store.checkpoint.as_ref() {
        Taking::Queued(_) | Taking::Idle => true,
        Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Holding { .. }
        | Taking::Selecting { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { .. } => false,
    }
}
pub(crate) fn ready(store: &Store) -> bool {
    match store.checkpoint.as_ref() {
        Taking::Selecting { .. } | Taking::Cleaning { step: Cleanup::Done, .. } => true,
        Taking::Idle
        | Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Queued(_)
        | Taking::Holding { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { step: Cleanup::Close | Cleanup::ParentOpen | Cleanup::Remove(_), .. } => false,
    }
}
pub(crate) fn ask(
    store: &mut Store,
    env: &Env<Limits>,
    owner: Token,
    draw: [u8; 16],
    above: &mut Queue<Event>,
    below: &mut Below,
) {
    let why = if !store.directory.disk || !matches_ready(store) {
        Some(CheckpointRefusal::Unavailable)
    } else if busy(store) {
        Some(CheckpointRefusal::InProgress)
    } else {
        None
    };
    if let Some(why) = why {
        above.push(Event::CheckpointRefused { owner, why });
        return;
    }
    *store.checkpoint = Taking::Opening(Ask { owner, draw, began: env.now });
    below.checkpoints.push(file::Request::OpenDirectory {
        owner: OWNER,
        root: store.directory.root.expect("serving deployment").fd,
        name: Box::from(b".".as_slice()),
        no_follow: true,
    });
    crate::view::checkpoint(store);
}
fn matches_ready(store: &Store) -> bool {
    match store.phase {
        Phase::Ready { .. } => !crate::rehearsal::closing(store),
        Phase::Closed | Phase::Stopped => false,
    }
}
/// Holds are requested in the same step; their terminals can arrive in either order.
pub(crate) fn begin(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    let state = core::mem::replace(store.checkpoint.as_mut(), Taking::Idle);
    *store.checkpoint = match state {
        Taking::Queued(session) => {
            let mut events = Queue::uncapped();
            *store.checkpoint = Taking::Holding {
                session,
                began: env.now,
                made: env.wall,
                main: HoldAnswer::Waiting,
                secrets: HoldAnswer::Waiting,
            };
            for secret in [false, true] {
                let kv_env = directory::env(store, env, secret);
                if secret {
                    skein_kv::down(
                        &mut store.stores.secrets,
                        &kv_env,
                        skein_kv::Request::Hold { owner: OWNER },
                        &mut events,
                        &mut below.secrets,
                    );
                } else {
                    skein_kv::down(
                        &mut store.stores.store,
                        &kv_env,
                        skein_kv::Request::Hold { owner: OWNER },
                        &mut events,
                        &mut below.store,
                    );
                }
                directory::events_up(store, secret, &mut events, above, below);
            }
            return;
        }
        Taking::Idle => Taking::Idle,
        Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Holding { .. }
        | Taking::Selecting { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { .. } => unreachable!("only prepared checkpoint reaches its barrier"),
    };
}
pub(crate) fn event(
    store: &mut Store,
    secret: bool,
    event: skein_kv::Event,
    _above: &mut Queue<Event>,
    below: &mut Below,
) {
    let answer = match event {
        skein_kv::Event::Held { last, snapshot, segments, .. } => HoldAnswer::Granted { last, snapshot, segments },
        skein_kv::Event::Refused { .. } => HoldAnswer::Refused,
        skein_kv::Event::Released { .. } => return,
        skein_kv::Event::Opened { .. }
        | skein_kv::Event::Committed { .. }
        | skein_kv::Event::Closed { .. }
        | skein_kv::Event::Failed { .. }
        | skein_kv::Event::OpenFailed { .. }
        | skein_kv::Event::Recovered { .. }
        | skein_kv::Event::RecoveryFailed { .. }
        | skein_kv::Event::Got { .. }
        | skein_kv::Event::Loaded { .. }
        | skein_kv::Event::LogRead { .. }
        | skein_kv::Event::LogUnread { .. } => unreachable!("checkpoint routes hold terminals only"),
    };
    let state = core::mem::replace(store.checkpoint.as_mut(), Taking::Idle);
    *store.checkpoint = match state {
        Taking::Holding { session, began, made, mut main, mut secrets } => {
            if secret {
                secrets = answer;
            } else {
                main = answer;
            }
            if waiting(&main) || waiting(&secrets) {
                Taking::Holding { session, began, made, main, secrets }
            } else {
                let mut selection = Selection::new();
                let selected = match held(&mut selection, false, &main) {
                    Ok(last) => match held(&mut selection, true, &secrets) {
                        Ok(secrets) => Ok((last, secrets)),
                        Err(why) => Err(why),
                    },
                    Err(why) => Err(why),
                };
                match selected {
                    Ok((store, secrets)) => Taking::Selecting { session, began, made, store, secrets, selection },
                    Err(why) => {
                        release(store, held_ok(&main), held_ok(&secrets), below);
                        fail(session, None, why, below)
                    }
                }
            }
        }
        Taking::Idle
        | Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Queued(_)
        | Taking::Selecting { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { .. } => unreachable!("hold terminal belongs to holding checkpoint"),
    };
}
fn waiting(held: &HoldAnswer) -> bool {
    match held {
        HoldAnswer::Waiting => true,
        HoldAnswer::Refused | HoldAnswer::Granted { .. } => false,
    }
}
fn held_ok(held: &HoldAnswer) -> bool {
    match held {
        HoldAnswer::Granted { .. } => true,
        HoldAnswer::Waiting | HoldAnswer::Refused => false,
    }
}
fn held(selection: &mut Selection, secret: bool, held: &HoldAnswer) -> Result<u64, CheckpointRefusal> {
    match held {
        HoldAnswer::Granted { last, snapshot, segments } => {
            selection.held(secret, *snapshot, segments)?;
            Ok(*last)
        }
        HoldAnswer::Refused => Err(CheckpointRefusal::Unavailable),
        HoldAnswer::Waiting => unreachable!("both hold terminals received"),
    }
}
pub(super) fn release(store: &mut Store, main: bool, secret: bool, below: &mut Below) {
    for (release, secrets) in [(main, false), (secret, true)] {
        if !release {
            continue;
        }
        let mut events = Queue::uncapped();
        let env = crate::memory_env(if secrets { store.limits.secrets } else { store.limits.store });
        if secrets {
            skein_kv::down(
                &mut store.stores.secrets,
                &env,
                skein_kv::Request::Release { owner: OWNER },
                &mut events,
                &mut below.secrets,
            );
        } else {
            skein_kv::down(
                &mut store.stores.store,
                &env,
                skein_kv::Request::Release { owner: OWNER },
                &mut events,
                &mut below.store,
            );
        }
        // Release answers at once; any held file work follows on its own lane.
        for event in &events {
            match event {
                skein_kv::Event::Released { .. } | skein_kv::Event::Refused { .. } => {}
                skein_kv::Event::Opened { .. }
                | skein_kv::Event::Committed { .. }
                | skein_kv::Event::Closed { .. }
                | skein_kv::Event::Failed { .. }
                | skein_kv::Event::OpenFailed { .. }
                | skein_kv::Event::Recovered { .. }
                | skein_kv::Event::RecoveryFailed { .. }
                | skein_kv::Event::Got { .. }
                | skein_kv::Event::Loaded { .. }
                | skein_kv::Event::Held { .. }
                | skein_kv::Event::LogRead { .. }
                | skein_kv::Event::LogUnread { .. } => unreachable!("release is immediate"),
            }
        }
    }
}
pub(crate) fn fire(store: &mut Store, env: &Env<Limits>, above: &mut Queue<Event>, below: &mut Below) {
    if store.phase == Phase::Stopped && !closing(store) {
        let pending = match store.checkpoint.as_ref() {
            Taking::Opening(_)
            | Taking::Preparing { .. }
            | Taking::Files { .. }
            | Taking::Cleaning { step: Cleanup::Close | Cleanup::ParentOpen | Cleanup::Remove(_), .. } => true,
            Taking::Idle
            | Taking::Closing { .. }
            | Taking::Queued(_)
            | Taking::Holding { .. }
            | Taking::Selecting { .. }
            | Taking::Cleaning { step: Cleanup::Done, .. } => false,
        };
        cancel(store, pending, above);
        return;
    }
    if !ready(store) {
        return;
    }
    let state = core::mem::replace(store.checkpoint.as_mut(), Taking::Idle);
    *store.checkpoint = match state {
        Taking::Selecting { session, began, made, store: last, secrets, mut selection } => {
            match selection.page(store) {
                Ok(false) => Taking::Selecting { session, began, made, store: last, secrets, selection },
                Ok(true) => {
                    let key = keys::encode_own(&keys::Own::Format, store.limits.store.key).expect("format key fits");
                    let format = match open::get(store, OWNER, key) {
                        Ok(Some(bytes)) => Format::decode(&bytes).ok(),
                        Ok(None) | Err(_) => None,
                    };
                    match format {
                        Some(format) => {
                            let (name, suffix) = CheckpointName::from_draw(last, session.ask.draw);
                            let temporary = files::join(b"checkpoints/", &name.temporary(suffix));
                            let record = Checkpoint {
                                store: last,
                                secrets,
                                made: made.as_nanos(),
                                build: Box::from(store.build_name.as_bytes()),
                                deployment: format.deployment,
                                levels: format.levels,
                                files: List::with_capacity(FILES),
                            };
                            let mut cut = Cut {
                                session,
                                name,
                                temporary,
                                record,
                                sources: selection.files,
                                deferred: Queue::with_capacity(FILES),
                                paused: env.now.as_nanos().saturating_sub(began.as_nanos()),
                                linked: 0,
                                copied: 0,
                                bytes: 0,
                                removed: 0,
                                skipped: 0,
                            };
                            let step = files::begin(&mut cut, below);
                            Taking::Files { cut, step }
                        }
                        None => {
                            release(store, true, true, below);
                            fail(session, None, CheckpointRefusal::Unavailable, below)
                        }
                    }
                }
                Err(why) => {
                    release(store, true, true, below);
                    fail(session, None, why, below)
                }
            }
        }
        Taking::Cleaning { session, why, temporary: _, step: Cleanup::Done } => {
            above.push(Event::CheckpointRefused { owner: session.ask.owner, why });
            Taking::Idle
        }
        state @ (Taking::Idle
        | Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Queued(_)
        | Taking::Holding { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { step: Cleanup::Close | Cleanup::ParentOpen | Cleanup::Remove(_), .. }) => state,
    };
}
pub(crate) fn defer(store: &mut Store, task: u64) -> bool {
    match store.checkpoint.as_mut() {
        Taking::Files { cut, step } if files::holding(step) => {
            for source in &cut.sources {
                if source.row.part == 4 && source.row.number == task {
                    let mut found = false;
                    for deferred in &cut.deferred {
                        if *deferred == task {
                            found = true;
                        }
                    }
                    if !found {
                        cut.deferred.push(task);
                    }
                    return true;
                }
            }
            false
        }
        Taking::Idle
        | Taking::Closing { .. }
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Queued(_)
        | Taking::Holding { .. }
        | Taking::Selecting { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { .. } => false,
    }
}
pub(crate) fn up(store: &mut Store, env: &Env<Limits>, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let state = core::mem::replace(store.checkpoint.as_mut(), Taking::Idle);
    *store.checkpoint = match state {
        Taking::Closing { owner } => {
            above.push(Event::CheckpointRefused { owner, why: CheckpointRefusal::Unavailable });
            Taking::Idle
        }
        Taking::Opening(ask) => {
            let why = files::failure(super::DIRECTORY, &arrived.event);
            match directory::opened(arrived) {
                Some(root) => {
                    let mut handles = List::with_capacity(16);
                    handles.push(root.token).expect("root handle fits");
                    let session = Session { ask, root, handles, path: Box::from(super::DIRECTORY) };
                    below.checkpoints.push(file::Request::MakeDirectory {
                        owner: OWNER,
                        root: root.token,
                        path: Box::from(super::DIRECTORY),
                        mode: 0o700,
                    });
                    Taking::Preparing { session, step: Prepare::Make }
                }
                None => {
                    above.push(Event::CheckpointRefused { owner: ask.owner, why });
                    Taking::Idle
                }
            }
        }
        Taking::Preparing { session, step } => match step {
            Prepare::Make => {
                if files::made(&arrived.event, true) {
                    let step = Prepare::Sweep(sweep::begin(session.root, &mut below.checkpoints));
                    Taking::Preparing { session, step }
                } else {
                    let why = files::failure(&session.path, &arrived.event);
                    fail(session, None, why, below)
                }
            }
            Prepare::Sweep(mut sweep) => {
                let why = files::failure(super::DIRECTORY, &arrived.event);
                match sweep::arrived(&mut sweep, arrived, &mut below.checkpoints) {
                    generations::Progress::Pending => Taking::Preparing { session, step: Prepare::Sweep(sweep) },
                    generations::Progress::Done => Taking::Queued(session),
                    generations::Progress::Failed => fail(session, None, why, below),
                }
            }
        },
        Taking::Files { mut cut, step } => {
            let holding = files::holding(&step);
            let published = files::published(&step);
            match files::up(store, &mut cut, step, arrived, below) {
                Ok(files::Progress::Pending(step)) => Taking::Files { cut, step },
                Ok(files::Progress::Done { held }) => {
                    let checkpoint = Checkpointed {
                        name: cut.name,
                        secrets: cut.record.secrets,
                        linked: cut.linked,
                        copied: cut.copied,
                        bytes: cut.bytes,
                        paused: cut.paused,
                        took: env.now.as_nanos().saturating_sub(cut.session.ask.began.as_nanos()),
                        held,
                        removed: cut.removed,
                        skipped: cut.skipped,
                    };
                    above.push(Event::Checkpointed { owner: cut.session.ask.owner, checkpoint });
                    Taking::Idle
                }
                Err(why) => {
                    if holding {
                        release(store, true, true, below);
                    }
                    // Deferred removals are handed to the ordinary transcript writer on either outcome.
                    crate::transcripts::drop::released(store, cut.deferred);
                    let temporary = if published { None } else { Some(cut.temporary) };
                    fail(cut.session, temporary, why, below)
                }
            }
        }
        Taking::Cleaning { session, why, temporary, step } => {
            cleanup(session, why, temporary, step, arrived, above, below)
        }
        Taking::Idle | Taking::Queued(_) | Taking::Holding { .. } | Taking::Selecting { .. } => {
            unreachable!("checkpoint file terminal without a file ask")
        }
    };
}
pub(super) fn fail(
    mut session: Session,
    temporary: Option<Box<[u8]>>,
    why: CheckpointRefusal,
    below: &mut Below,
) -> Taking {
    let step = if session.handles.len() == 1 && temporary.is_some() {
        {
            below.checkpoints.push(file::Request::OpenDirectory {
                owner: OWNER,
                root: session.root.fd,
                name: Box::from(super::DIRECTORY),
                no_follow: true,
            });
            Cleanup::ParentOpen
        }
    } else {
        match pop_handle(&mut session) {
            Some(file) => {
                below.checkpoints.push(file::Request::Close { owner: OWNER, file });
                Cleanup::Close
            }
            None => Cleanup::Done,
        }
    };
    Taking::Cleaning { session, why, temporary, step }
}
fn cleanup(
    mut session: Session,
    why: CheckpointRefusal,
    temporary: Option<Box<[u8]>>,
    step: Cleanup,
    arrived: Arrived,
    above: &mut Queue<Event>,
    below: &mut Below,
) -> Taking {
    match step {
        Cleanup::Close => {
            // Keep the deployment clone until the temporary tree is removed.
            if session.handles.len() == 1 && temporary.is_some() {
                below.checkpoints.push(file::Request::OpenDirectory {
                    owner: OWNER,
                    root: session.root.fd,
                    name: Box::from(super::DIRECTORY),
                    no_follow: true,
                });
                Taking::Cleaning { session, why, temporary, step: Cleanup::ParentOpen }
            } else if let Some(file) = pop_handle(&mut session) {
                below.checkpoints.push(file::Request::Close { owner: OWNER, file });
                Taking::Cleaning { session, why, temporary, step: Cleanup::Close }
            } else {
                above.push(Event::CheckpointRefused { owner: session.ask.owner, why });
                Taking::Idle
            }
        }
        Cleanup::ParentOpen => {
            if let Some(root) = directory::opened(arrived) {
                session.handles.push(root.token).expect("cleanup parent handle");
                let leaf = Box::from(
                    temporary
                        .as_ref()
                        .expect("temporary retained")
                        .strip_prefix(b"checkpoints/")
                        .expect("temporary path"),
                );
                let removing = generations::remove_checkpoint(root, leaf, &mut below.checkpoints);
                Taking::Cleaning { session, why, temporary, step: Cleanup::Remove(removing) }
            } else {
                let file = pop_handle(&mut session).expect("deployment root retained");
                below.checkpoints.push(file::Request::Close { owner: OWNER, file });
                Taking::Cleaning { session, why, temporary: None, step: Cleanup::Close }
            }
        }
        Cleanup::Remove(mut removing) => match generations::removed(&mut removing, arrived, &mut below.checkpoints) {
            generations::Progress::Pending => {
                Taking::Cleaning { session, why, temporary, step: Cleanup::Remove(removing) }
            }
            generations::Progress::Done | generations::Progress::Failed => {
                let file = pop_handle(&mut session).expect("checkpoint parent retained");
                below.checkpoints.push(file::Request::Close { owner: OWNER, file });
                Taking::Cleaning { session, why, temporary: None, step: Cleanup::Close }
            }
        },
        Cleanup::Done => {
            above.push(Event::CheckpointRefused { owner: session.ask.owner, why });
            Taking::Idle
        }
    }
}

fn pop_handle(session: &mut Session) -> Option<Token> {
    let index = session.handles.len().checked_sub(1)?;
    session.handles.swap_remove(index)
}

/// Physical close owns the handles and the unfinished temporary from this point.
pub(crate) fn cancel(store: &mut Store, in_flight: bool, above: &mut Queue<Event>) {
    let state = core::mem::replace(store.checkpoint.as_mut(), Taking::Idle);
    let owner = match state {
        Taking::Idle => None,
        Taking::Closing { owner } => {
            *store.checkpoint = Taking::Closing { owner };
            return;
        }
        Taking::Opening(ask) => Some(ask.owner),
        Taking::Preparing { session, .. }
        | Taking::Queued(session)
        | Taking::Holding { session, .. }
        | Taking::Selecting { session, .. }
        | Taking::Cleaning { session, .. } => Some(session.ask.owner),
        Taking::Files { cut, .. } => Some(cut.session.ask.owner),
    };
    if let Some(owner) = owner {
        if in_flight {
            *store.checkpoint = Taking::Closing { owner };
        } else {
            above.push(Event::CheckpointRefused { owner, why: CheckpointRefusal::Unavailable });
        }
    }
}
pub(crate) fn closing(store: &Store) -> bool {
    match store.checkpoint.as_ref() {
        Taking::Closing { .. } => true,
        Taking::Idle
        | Taking::Opening(_)
        | Taking::Preparing { .. }
        | Taking::Queued(_)
        | Taking::Holding { .. }
        | Taking::Selecting { .. }
        | Taking::Files { .. }
        | Taking::Cleaning { .. } => false,
    }
}

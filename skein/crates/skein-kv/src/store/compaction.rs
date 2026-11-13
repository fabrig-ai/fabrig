//! A snapshot's replacement lifecycle (skein/docs/design/data/kv.md, 5).

use alloc::boxed::Box;
use core::mem;
use skein_io::file;
use skein_io::kernel::{Error, Fd};
use skein_lib::{Env, List, Queue, Token, usize_of};

use super::{Dir, IO_OWNER, Log, Phase, Repair, State, Step, Store, announce_recovered, commits_failed, log_number};
use crate::boundary::Row;
use crate::snapshot;
use crate::{Census, Limits};

#[derive(Debug)]
pub(super) enum Compaction {
    Idle,
    Due { start: u64 },
    Create { start: u64 },
    RemoveTemp { start: u64 },
    Header { start: u64, file: Token },
    Chunk { written: Written, cursor: Option<Box<[u8]>> },
    Trailer { written: Written },
    Sync { start: u64, file: Token },
    Close { start: u64, file: Token },
    Rename { start: u64 },
    SyncDirectory { start: u64 },
    RemoveOld { start: u64 },
    CleanupSyncDirectory,
}

/// What the snapshot file holds so far.
#[derive(Debug)]
pub(super) struct Written {
    start: u64,
    file: Token,
    offset: u64,
    rows: u64,
    payload: u64,
}

impl Compaction {
    /// The snapshot's open file, in each state that holds one.
    pub(super) fn file(&self) -> Option<Token> {
        match self {
            Self::Header { file, .. }
            | Self::Chunk { written: Written { file, .. }, .. }
            | Self::Trailer { written: Written { file, .. } }
            | Self::Sync { file, .. }
            | Self::Close { file, .. } => Some(*file),
            Self::Idle
            | Self::Due { .. }
            | Self::Create { .. }
            | Self::RemoveTemp { .. }
            | Self::Rename { .. }
            | Self::SyncDirectory { .. }
            | Self::RemoveOld { .. }
            | Self::CleanupSyncDirectory => None,
        }
    }

    pub(super) fn is_idle(&self) -> bool {
        match self {
            Self::Idle => true,
            Self::Due { .. }
            | Self::Create { .. }
            | Self::RemoveTemp { .. }
            | Self::Header { .. }
            | Self::Chunk { .. }
            | Self::Trailer { .. }
            | Self::Sync { .. }
            | Self::Close { .. }
            | Self::Rename { .. }
            | Self::SyncDirectory { .. }
            | Self::RemoveOld { .. }
            | Self::CleanupSyncDirectory => false,
        }
    }

    pub(super) fn announced(self, log_bytes: &mut u64, census: &mut Census) -> Self {
        match self {
            Self::Due { start } => {
                *log_bytes = 0;
                census.restart(start);
                Self::Create { start }
            }
            state @ (Self::Idle
            | Self::Create { .. }
            | Self::RemoveTemp { .. }
            | Self::Header { .. }
            | Self::Chunk { .. }
            | Self::Trailer { .. }
            | Self::Sync { .. }
            | Self::Close { .. }
            | Self::Rename { .. }
            | Self::SyncDirectory { .. }
            | Self::RemoveOld { .. }
            | Self::CleanupSyncDirectory) => state,
        }
    }
}

pub(super) fn fire_ready(
    mut dir: Dir,
    log: Log,
    store: &mut Store,
    env: &Env<Limits>,
    below: &mut Queue<file::Request>,
) -> State {
    if !store.queue.is_empty() {
        return store.start_batch(dir, log, &env.limits, below);
    }
    dir.compaction = match mem::replace(&mut dir.compaction, Compaction::Idle) {
        Compaction::Idle => return idle(store, dir, log, env, below),
        Compaction::Due { .. } => unreachable!("the starting segment is announced before Ready"),
        Compaction::RemoveTemp { .. } => {
            unreachable!("create_failed sends the removal and finished answers it before Ready")
        }
        Compaction::Create { start } => create(start, dir.root, below),
        Compaction::Header { start, file } => header(start, file, below),
        Compaction::Chunk { written, cursor } => chunk(written, cursor, store, &env.limits, below),
        Compaction::Trailer { written } => trailer(written, below),
        Compaction::Sync { start, file } => sync(start, file, below),
        Compaction::Close { start, file } => close(start, file, below),
        Compaction::Rename { start } => rename(start, dir.root, below),
        Compaction::SyncDirectory { start } => sync_directory(start, dir.root, below),
        Compaction::RemoveOld { start } => remove_old(start, &mut dir, below),
        Compaction::CleanupSyncDirectory => cleanup_sync_directory(dir.root, below),
    };
    dir.at(Phase::SnapIo { log })
}

fn idle(store: &Store, mut dir: Dir, log: Log, env: &Env<Limits>, below: &mut Queue<file::Request>) -> State {
    if dir.close_owner.is_some() {
        below.push(file::Request::Close { owner: IO_OWNER, file: log.file });
        dir.at(Phase::Closing)
    } else if dir.log_bytes >= env.limits.snapshot_bound(store.map.live()) {
        dir.compaction = Compaction::Due { start: store.last.checked_add(1).expect("commit number did not overflow") };
        below.push(file::Request::Close { owner: IO_OWNER, file: log.file });
        dir.at(Phase::RolloverClosing)
    } else {
        dir.at(Phase::Ready { log })
    }
}

fn create(start: u64, root: Fd, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::Create { owner: IO_OWNER, root, name: Box::from(&b"snapshot.tmp"[..]), mode: 0o600 });
    Compaction::Create { start }
}

fn remove_temp(start: u64, root: Fd, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::Remove {
        owner: IO_OWNER,
        root,
        name: Box::from(&b"snapshot.tmp"[..]),
        directory: false,
    });
    Compaction::RemoveTemp { start }
}

fn header(start: u64, file: Token, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::WriteAt { owner: IO_OWNER, file, offset: 0, bytes: snapshot::header(start) });
    Compaction::Header { start, file }
}

fn chunk(
    mut written: Written,
    cursor: Option<Box<[u8]>>,
    store: &Store,
    limits: &Limits,
    below: &mut Queue<file::Request>,
) -> Compaction {
    let target = usize_of(limits.chunk_target());
    let mut rows = List::with_capacity(limits.chunk_target() / 8);
    let mut size: usize = 12;
    let mut payload: u64 = 0;
    for (key, value) in store.map.after(cursor.as_deref()) {
        let row_payload = key.len().checked_add(value.len()).expect("payload fits");
        let row_size = 8_usize.checked_add(row_payload).expect("row size fits");
        let Some(next) = size.checked_add(row_size) else { break };
        // A row past the target has a chunk of its own, up to the ceiling (data/kv.md, 5).
        if next > target && !rows.is_empty() {
            break;
        }
        size = next;
        payload = payload.checked_add(skein_lib::len_u64(row_payload)).expect("payload total fits u64");
        rows.push(Row { key: key.clone(), value: value.clone() }).expect("each row takes at least eight chunk bytes");
    }
    if rows.is_empty() {
        return trailer(written, below);
    }
    let cursor = Some(rows.last().expect("a chunk has rows").key.clone());
    let bytes = snapshot::chunk(rows.as_slice(), limits.chunk).expect("limits hold one row per chunk");
    let len = skein_lib::len_u64(bytes.len());
    below.push(file::Request::WriteAt { owner: IO_OWNER, file: written.file, offset: written.offset, bytes });
    written.offset = written.offset.checked_add(len).expect("snapshot offset fits");
    written.rows = written.rows.checked_add(u64::from(rows.len())).expect("snapshot rows fit");
    written.payload = written.payload.checked_add(payload).expect("snapshot payload fits");
    Compaction::Chunk { written, cursor }
}

fn trailer(written: Written, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::WriteAt {
        owner: IO_OWNER,
        file: written.file,
        offset: written.offset,
        bytes: snapshot::trailer(written.rows, written.payload),
    });
    Compaction::Trailer { written }
}

fn sync(start: u64, file: Token, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::Sync { owner: IO_OWNER, file });
    Compaction::Sync { start, file }
}

fn close(start: u64, file: Token, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::Close { owner: IO_OWNER, file });
    Compaction::Close { start, file }
}

fn rename(start: u64, root: Fd, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::Rename {
        owner: IO_OWNER,
        from_root: root,
        from: Box::from(&b"snapshot.tmp"[..]),
        to_root: root,
        to: Box::from(&b"snapshot"[..]),
    });
    Compaction::Rename { start }
}

fn sync_directory(start: u64, root: Fd, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::SyncDirectory { owner: IO_OWNER, root });
    Compaction::SyncDirectory { start }
}

fn remove_old(start: u64, dir: &mut Dir, below: &mut Queue<file::Request>) -> Compaction {
    let mut old = None;
    for (index, name) in dir.segments.iter().enumerate() {
        match log_number(name) {
            Some(number) if number < start => {
                old = Some(index);
                break;
            }
            Some(_) | None => {}
        }
    }
    let Some(index) = old else { return cleanup_sync_directory(dir.root, below) };
    let name = dir.segments.remove(index);
    below.push(file::Request::Remove { owner: IO_OWNER, root: dir.root, name, directory: false });
    Compaction::RemoveOld { start }
}

fn cleanup_sync_directory(root: Fd, below: &mut Queue<file::Request>) -> Compaction {
    below.push(file::Request::SyncDirectory { owner: IO_OWNER, root });
    Compaction::CleanupSyncDirectory
}

pub(super) fn opened(store: &mut Store, mut dir: Dir, log: Log, file: Token, step: &mut Step<'_>) -> State {
    dir.compaction = match mem::replace(&mut dir.compaction, Compaction::Idle) {
        Compaction::Create { start } => created(start, file),
        Compaction::Idle
        | Compaction::Due { .. }
        | Compaction::RemoveTemp { .. }
        | Compaction::Header { .. }
        | Compaction::Chunk { .. }
        | Compaction::Trailer { .. }
        | Compaction::Sync { .. }
        | Compaction::Close { .. }
        | Compaction::Rename { .. }
        | Compaction::SyncDirectory { .. }
        | Compaction::RemoveOld { .. }
        | Compaction::CleanupSyncDirectory => unreachable!("only snapshot creation opens a file"),
    };
    store.start_batch(dir, log, &step.env.limits, step.below)
}

pub(super) fn finished(store: &mut Store, mut dir: Dir, log: Log, step: &mut Step<'_>) -> State {
    dir.compaction = match mem::replace(&mut dir.compaction, Compaction::Idle) {
        Compaction::RemoveTemp { start } => temp_removed(start),
        Compaction::Header { start, file } => header_written(start, file),
        Compaction::Chunk { written, cursor } => chunk_written(written, cursor),
        Compaction::Trailer { written } => trailer_written(written),
        Compaction::Sync { start, file } => synced(start, file),
        Compaction::Close { start, .. } => closed(start),
        Compaction::Rename { start } => renamed(start),
        Compaction::SyncDirectory { start } => directory_synced(store, &mut dir, start, step),
        Compaction::RemoveOld { start } => old_removed(start),
        Compaction::CleanupSyncDirectory => cleanup_synced(),
        Compaction::Idle | Compaction::Due { .. } | Compaction::Create { .. } => {
            unreachable!("snapshot creation opens a file; Idle and Due have no snapshot request")
        }
    };
    store.start_batch(dir, log, &step.env.limits, step.below)
}

fn created(start: u64, file: Token) -> Compaction {
    Compaction::Header { start, file }
}

fn temp_removed(start: u64) -> Compaction {
    Compaction::Create { start }
}

fn header_written(start: u64, file: Token) -> Compaction {
    let offset = skein_lib::len_u64(snapshot::HEADER);
    Compaction::Chunk { written: Written { start, file, offset, rows: 0, payload: 0 }, cursor: None }
}

fn chunk_written(written: Written, cursor: Option<Box<[u8]>>) -> Compaction {
    Compaction::Chunk { written, cursor }
}

fn trailer_written(written: Written) -> Compaction {
    Compaction::Sync { start: written.start, file: written.file }
}

fn synced(start: u64, file: Token) -> Compaction {
    Compaction::Close { start, file }
}

fn closed(start: u64) -> Compaction {
    Compaction::Rename { start }
}

fn renamed(start: u64) -> Compaction {
    Compaction::SyncDirectory { start }
}

fn directory_synced(store: &mut Store, dir: &mut Dir, start: u64, step: &mut Step<'_>) -> Compaction {
    match store.repair {
        Repair::Clean => {}
        Repair::Required => {
            store.repair = Repair::Clean;
            announce_recovered(dir, store.last, step.above);
        }
    }
    Compaction::RemoveOld { start }
}

fn old_removed(start: u64) -> Compaction {
    Compaction::RemoveOld { start }
}

fn cleanup_synced() -> Compaction {
    Compaction::Idle
}

pub(super) fn failed(store: &mut Store, mut dir: Dir, log: Log, error: Error, step: &mut Step<'_>) -> State {
    let compaction = mem::replace(&mut dir.compaction, Compaction::Idle);
    if let Some(file) = compaction.file() {
        return failure_closing(dir, log, file, step.below);
    }
    match compaction {
        Compaction::Create { start } => create_failed(store, dir, log, start, error, step),
        Compaction::RemoveTemp { .. }
        | Compaction::Rename { .. }
        | Compaction::SyncDirectory { .. }
        | Compaction::RemoveOld { .. }
        | Compaction::CleanupSyncDirectory => commits_failed(store, dir, Some(log.file), Queue::with_capacity(0), step),
        Compaction::Idle | Compaction::Due { .. } => unreachable!("no snapshot request is outstanding"),
        Compaction::Header { .. }
        | Compaction::Chunk { .. }
        | Compaction::Trailer { .. }
        | Compaction::Sync { .. }
        | Compaction::Close { .. } => unreachable!("the snapshot's file is closed before recovery"),
    }
}

fn create_failed(store: &mut Store, mut dir: Dir, log: Log, start: u64, error: Error, step: &mut Step<'_>) -> State {
    if error == Error::Exists {
        dir.compaction = remove_temp(start, dir.root, step.below);
        dir.at(Phase::SnapIo { log })
    } else {
        commits_failed(store, dir, Some(log.file), Queue::with_capacity(0), step)
    }
}

fn failure_closing(dir: Dir, log: Log, file: Token, below: &mut Queue<file::Request>) -> State {
    below.push(file::Request::Close { owner: IO_OWNER, file });
    dir.at(Phase::SnapshotFailureClosing { log })
}

//! The checkpoint's single-flight file lane and bounded prefix copier (protocol/store/checkpoints.md, 4.5–4.8).
use super::taking::{Cut, OWNER, Session};
use crate::{Arrived, Below, CheckpointRefusal, Root, Store, generations};
use alloc::boxed::Box;
use skein_io::{file, kernel::Error};
use skein_lib::{Decimal, Writer};
pub(crate) const CHUNK: u32 = 1_048_576;

#[derive(Debug)]
pub(crate) enum Step {
    Make { index: u8 },
    Private,
    PrivateClose,
    CheckOpen { secret: bool },
    CheckPage { secret: bool, root: Root },
    CheckClose { secret: bool },
    Shard,
    ReadOpen,
    LinkSource { reader: skein_lib::Token },
    LinkDestination { reader: skein_lib::Token, source: Root },
    Link { reader: skein_lib::Token, source: Root, destination: Root },
    LinkDestinationClose { reader: skein_lib::Token, source: Root, way: LinkWay },
    LinkSourceClose { reader: skein_lib::Token, way: LinkWay },
    WriteOpen { reader: skein_lib::Token },
    Read { reader: skein_lib::Token, writer: skein_lib::Token, offset: u64 },
    Write { reader: skein_lib::Token, writer: skein_lib::Token, offset: u64, count: u64 },
    FileSync { reader: skein_lib::Token, writer: skein_lib::Token },
    WriteClose { reader: skein_lib::Token },
    ReadClose,
    SyncOpen { index: u32 },
    Sync { index: u32, root: Root },
    SyncClose { index: u32 },
    Manifest,
    Active,
    Record,
    TopOpen { index: u8 },
    TopSync { index: u8, root: Root },
    TopClose { index: u8 },
    Rename { root: Root },
    ParentOpen,
    ParentSync { root: Root },
    ParentClose,
    Retaining(Box<super::retention::Retention>),
    MeasureOpen { index: u32, held: u64 },
    MeasureStat { index: u32, held: u64, file: skein_lib::Token },
    MeasureClose { index: u32, held: u64 },
    Finished { held: u64 },
}
#[derive(Debug)]
pub(super) enum Progress {
    Pending(Step),
    Done { held: u64 },
}
pub(super) fn holding(step: &Step) -> bool {
    match step {
        Step::Make { .. }
        | Step::Private
        | Step::PrivateClose
        | Step::CheckOpen { .. }
        | Step::CheckPage { .. }
        | Step::CheckClose { .. }
        | Step::Shard
        | Step::ReadOpen
        | Step::LinkSource { .. }
        | Step::LinkDestination { .. }
        | Step::Link { .. }
        | Step::LinkDestinationClose { .. }
        | Step::LinkSourceClose { .. }
        | Step::WriteOpen { .. }
        | Step::Read { .. }
        | Step::Write { .. }
        | Step::FileSync { .. }
        | Step::WriteClose { .. }
        | Step::ReadClose
        | Step::SyncOpen { .. }
        | Step::Sync { .. }
        | Step::SyncClose { .. } => true,
        Step::Manifest
        | Step::Active
        | Step::Record
        | Step::TopOpen { .. }
        | Step::TopSync { .. }
        | Step::TopClose { .. }
        | Step::Rename { .. }
        | Step::ParentOpen
        | Step::ParentSync { .. }
        | Step::ParentClose
        | Step::Retaining(_)
        | Step::MeasureOpen { .. }
        | Step::MeasureStat { .. }
        | Step::MeasureClose { .. }
        | Step::Finished { .. } => false,
    }
}
pub(super) fn published(step: &Step) -> bool {
    match step {
        Step::ParentSync { .. }
        | Step::ParentClose
        | Step::Retaining(_)
        | Step::MeasureOpen { .. }
        | Step::MeasureStat { .. }
        | Step::MeasureClose { .. }
        | Step::Finished { .. } => true,
        Step::ParentOpen
        | Step::Make { .. }
        | Step::Private
        | Step::PrivateClose
        | Step::CheckOpen { .. }
        | Step::CheckPage { .. }
        | Step::CheckClose { .. }
        | Step::Shard
        | Step::ReadOpen
        | Step::LinkSource { .. }
        | Step::LinkDestination { .. }
        | Step::Link { .. }
        | Step::LinkDestinationClose { .. }
        | Step::LinkSourceClose { .. }
        | Step::WriteOpen { .. }
        | Step::Read { .. }
        | Step::Write { .. }
        | Step::FileSync { .. }
        | Step::WriteClose { .. }
        | Step::ReadClose
        | Step::SyncOpen { .. }
        | Step::Sync { .. }
        | Step::SyncClose { .. }
        | Step::Manifest
        | Step::Active
        | Step::Record
        | Step::TopOpen { .. }
        | Step::TopSync { .. }
        | Step::TopClose { .. }
        | Step::Rename { .. } => false,
    }
}
pub(super) fn join(a: &[u8], b: &[u8]) -> Box<[u8]> {
    let mut out = Writer::new(a.len().checked_add(b.len()).expect("bounded checkpoint path"));
    out.put(a).expect("measured prefix");
    out.put(b).expect("measured suffix");
    out.finish()
}
fn under(a: &[u8], b: &[u8]) -> Box<[u8]> {
    join(&join(a, b"/"), b)
}
fn segment(number: u64) -> Box<[u8]> {
    let decimal = Decimal::of(number);
    let mut out = Writer::new(24);
    out.put(b"log-").expect("segment prefix");
    for _ in decimal.as_bytes().len()..20 {
        out.put(b"0").expect("twenty digit field");
    }
    out.put(decimal.as_bytes()).expect("segment number");
    out.finish()
}
fn relative(part: u8, number: u64) -> Box<[u8]> {
    file_path(1, part, number)
}
fn source(store: &Store, part: u8, number: u64) -> Box<[u8]> {
    file_path(store.directory.active.expect("serving active generation").number, part, number)
}
fn file_path(generation: u64, part: u8, number: u64) -> Box<[u8]> {
    match part {
        1 => under(&generations::map(generation, generations::Map::Store), b"snapshot"),
        2 => under(&generations::map(generation, generations::Map::Store), &segment(number)),
        3 => under(&generations::map(generation, generations::Map::Store), &skein_kv::log_path(number)),
        4 => under(b"transcripts", &crate::transcripts::path(number)),
        5 => under(&generations::map(generation, generations::Map::Secrets), b"snapshot"),
        6 => under(&generations::map(generation, generations::Map::Secrets), &segment(number)),
        _ => unreachable!("selection has six parts"),
    }
}
fn current(cut: &Cut) -> &super::selection::Source {
    cut.sources.get(cut.record.files.len()).expect("active selected source")
}
fn current_mut(cut: &mut Cut) -> &mut super::selection::Source {
    cut.sources.get_mut(cut.record.files.len()).expect("active selected source")
}
fn destination(cut: &Cut) -> Box<[u8]> {
    let row = &current(cut).row;
    under(&cut.temporary, &relative(row.part, row.number))
}
fn shard(part: u8, number: u64) -> Option<Box<[u8]>> {
    match part {
        3 => Some(under(b"generations/1/store", &skein_kv::log_directory(number))),
        4 => Some(under(b"transcripts", &crate::transcripts::name(number >> 12))),
        1 | 2 | 5 | 6 => None,
        _ => unreachable!("selected part"),
    }
}
fn ask(cut: &mut Cut, path: Box<[u8]>, request: file::Request, below: &mut Below) {
    cut.session.path = path;
    below.checkpoints.push(request);
}
fn make(cut: &mut Cut, index: u8, below: &mut Below) -> Step {
    const DIRS: [&[u8]; 6] =
        [b"", b"generations", b"generations/1", b"generations/1/store", b"generations/1/store/logs", b"transcripts"];
    let suffix = DIRS.get(usize::from(index)).expect("six temporary directories");
    let path = if suffix.is_empty() { cut.temporary.clone() } else { under(&cut.temporary, suffix) };
    let request =
        file::Request::MakeDirectory { owner: OWNER, root: cut.session.root.token, path: path.clone(), mode: 0o700 };
    ask(cut, path, request, below);
    Step::Make { index }
}
pub(super) fn begin(cut: &mut Cut, below: &mut Below) -> Step {
    make(cut, 0, below)
}
fn open_dir(cut: &mut Cut, path: Box<[u8]>, below: &mut Below) {
    let request =
        file::Request::OpenDirectory { owner: OWNER, root: cut.session.root.fd, name: path.clone(), no_follow: true };
    ask(cut, path, request, below);
}
fn check_open(store: &Store, cut: &mut Cut, secret: bool, below: &mut Below) -> Step {
    let generation = store.directory.active.expect("serving active").number;
    open_dir(
        cut,
        generations::map(generation, if secret { generations::Map::Secrets } else { generations::Map::Store }),
        below,
    );
    Step::CheckOpen { secret }
}
fn opened(session: &mut Session, arrived: Arrived) -> Result<(skein_lib::Token, u64), CheckpointRefusal> {
    match arrived.event {
        file::Event::Opened { file, len, .. } => {
            session.handles.push(file).expect("bounded checkpoint handles");
            Ok((file, len))
        }
        event @ (file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Loaded { .. }
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
        | file::Event::Failed { .. }) => Err(failure(&session.path, &event)),
    }
}
pub(super) fn directory(cut: &mut Cut, arrived: Arrived) -> Result<Root, CheckpointRefusal> {
    let fd = arrived.fd;
    let (token, _) = opened(&mut cut.session, arrived)?;
    Ok(Root { token, fd: fd.expect("file lane reports opened descriptor") })
}
pub(super) fn close(cut: &mut Cut, file: skein_lib::Token, below: &mut Below) {
    let index = cut.session.handles.len().checked_sub(1).expect("closing owned handle");
    assert!(cut.session.handles.swap_remove(index) == Some(file), "checkpoint closes in reverse opening order");
    below.checkpoints.push(file::Request::Close { owner: OWNER, file });
}
fn read_open(store: &Store, cut: &mut Cut, below: &mut Below) -> Step {
    let row = &current(cut).row;
    let path = source(store, row.part, row.number);
    let request = file::Request::OpenReadNoFollow { owner: OWNER, root: cut.session.root.fd, name: path.clone() };
    ask(cut, path, request, below);
    Step::ReadOpen
}
fn next_file(store: &Store, cut: &mut Cut, below: &mut Below) -> Step {
    if cut.record.files.len() == cut.sources.len() {
        return sync_next(cut, 0, below);
    }
    let row = &current(cut).row;
    match shard(row.part, row.number) {
        Some(path) => {
            let path = under(&cut.temporary, &path);
            let request = file::Request::MakeDirectory {
                owner: OWNER,
                root: cut.session.root.token,
                path: path.clone(),
                mode: 0o700,
            };
            ask(cut, path, request, below);
            Step::Shard
        }
        None => read_open(store, cut, below),
    }
}
fn write_open(cut: &mut Cut, reader: skein_lib::Token, below: &mut Below) -> Step {
    current_mut(cut).row.way = 2;
    let path = destination(cut);
    let request =
        file::Request::CreateNoFollow { owner: OWNER, root: cut.session.root.fd, name: path.clone(), mode: 0o600 };
    ask(cut, path, request, below);
    Step::WriteOpen { reader }
}
fn read(
    store: &Store,
    cut: &mut Cut,
    reader: skein_lib::Token,
    writer: skein_lib::Token,
    offset: u64,
    below: &mut Below,
) -> Step {
    let length = current(cut).row.length;
    if offset == length {
        below.checkpoints.push(file::Request::Sync { owner: OWNER, file: writer });
        Step::FileSync { reader, writer }
    } else {
        let row = &current(cut).row;
        cut.session.path = source(store, row.part, row.number);
        let max = u32::try_from(length.saturating_sub(offset).min(u64::from(CHUNK))).expect("bounded copy chunk");
        below.checkpoints.push(file::Request::ReadAt { owner: OWNER, file: reader, offset, max });
        Step::Read { reader, writer, offset }
    }
}
fn finished_file(store: &Store, cut: &mut Cut, below: &mut Below) -> Step {
    let row = current(cut).row.clone();
    match row.way {
        1 => cut.linked = cut.linked.saturating_add(1),
        2 => {
            cut.copied = cut.copied.saturating_add(1);
            cut.bytes = cut.bytes.saturating_add(row.length);
        }
        3 => {}
        _ => unreachable!("selected way"),
    }
    cut.record
        .files
        .push(
            crate::codec::component::CheckpointFile::new(&crate::component_v3::CEILINGS, row)
                .expect("plain file fields"),
        )
        .expect("selection bounded record");
    next_file(store, cut, below)
}
// Selected shards are in map order, so only their first entry requests a sync.
fn sync_path(cut: &Cut, index: u32) -> Option<Box<[u8]>> {
    const DIRS: [&[u8]; 5] = [
        b"generations/1/store/logs",
        b"generations/1/store",
        b"generations/1/secrets",
        b"transcripts",
        b"generations/1",
    ];
    if index < cut.sources.len() {
        let row = &cut.sources.get(index)?.row;
        let path = shard(row.part, row.number)?;
        if index > 0 {
            let previous = &cut.sources.get(index.checked_sub(1)?)?.row;
            if shard(previous.part, previous.number).as_deref() == Some(&path) {
                return None;
            }
        }
        Some(under(&cut.temporary, &path))
    } else {
        let index = usize::try_from(index.checked_sub(cut.sources.len())?).ok()?;
        Some(under(&cut.temporary, DIRS.get(index)?))
    }
}
fn sync_next(cut: &mut Cut, start: u32, below: &mut Below) -> Step {
    for index in start..cut.sources.len().saturating_add(5) {
        if let Some(path) = sync_path(cut, index) {
            open_dir(cut, path, below);
            return Step::SyncOpen { index };
        }
    }
    // This marker is turned into the manifest request by the transition's owner.
    Step::Manifest
}
fn store_file(cut: &mut Cut, suffix: &[u8], bytes: Box<[u8]>, below: &mut Below) {
    let path = under(&cut.temporary, suffix);
    let request = file::Request::Store {
        owner: OWNER,
        root: cut.session.root.token,
        path: path.clone(),
        bytes,
        expected: file::Expect::Absent,
        no_follow: true,
    };
    ask(cut, path, request, below);
}
fn manifest(cut: &mut Cut, below: &mut Below) {
    let bytes = crate::codec::component::Manifest {
        generation: 1,
        from: None,
        before: cut.record.levels,
        after: cut.record.levels,
        counts: skein_lib::List::with_capacity(0),
        store: cut.record.store,
        secrets: Some(cut.record.secrets),
        written: cut.record.made,
    }
    .encode();
    store_file(cut, b"generations/1/manifest", bytes, below);
}
fn top(cut: &mut Cut, index: u8, below: &mut Below) -> Step {
    let path = if index == 0 { under(&cut.temporary, b"generations") } else { cut.temporary.clone() };
    open_dir(cut, path, below);
    Step::TopOpen { index }
}
fn measure(cut: &mut Cut, start: u32, mut held: u64, below: &mut Below) -> Step {
    for index in start..cut.record.files.len() {
        let row = cut.record.files.get(index).expect("measured file");
        match row.way() {
            1 => {
                let path = under(&join(b"checkpoints/", &cut.name.bytes()), &relative(row.part(), row.number()));
                let request =
                    file::Request::OpenReadNoFollow { owner: OWNER, root: cut.session.root.fd, name: path.clone() };
                ask(cut, path, request, below);
                return Step::MeasureOpen { index, held };
            }
            2 => held = held.saturating_add(row.length()),
            3 => {}
            _ => unreachable!("selected way"),
        }
    }
    close(cut, cut.session.root.token, below);
    Step::Finished { held }
}
pub(super) fn made(event: &file::Event, existing: bool) -> bool {
    kind(event) == Kind::Made || existing && error(event) == Some(Error::Exists)
}
fn known(name: &[u8], secret: bool) -> bool {
    if name == b"snapshot" || name == b"snapshot.tmp" || !secret && name == b"logs" {
        return true;
    }
    if let Some(digits) = name.strip_prefix(b"log-") {
        if digits.len() != 20 {
            return false;
        }
        for byte in digits {
            if !byte.is_ascii_digit() {
                return false;
            }
        }
        return true;
    }
    if let Some(hex) = name.strip_prefix(b"snapshot.skein-") {
        if hex.len() != 16 {
            return false;
        }
        for byte in hex {
            if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(byte) {
                return false;
            }
        }
        return true;
    }
    false
}
#[expect(clippy::disallowed_methods, reason = "the file protocol renders bounded directory names in refusals")]
pub(super) fn text(path: &[u8]) -> Box<str> {
    let prefix = path.get(..path.len().min(128)).expect("bounded path prefix");
    Box::from(core::str::from_utf8(prefix).unwrap_or("<non-UTF8 path>"))
}
pub(crate) fn failure(path: &[u8], event: &file::Event) -> CheckpointRefusal {
    let error = error(event).unwrap_or(Error::Other(0));
    CheckpointRefusal::Failed { path: text(path), error }
}
fn vanished(store: &Store, cut: &Cut) -> CheckpointRefusal {
    let row = &current(cut).row;
    CheckpointRefusal::Vanished { path: text(&source(store, row.part, row.number)) }
}
#[expect(clippy::too_many_lines, reason = "the total single-flight checkpoint file transition table")]
pub(super) fn up(
    store: &mut Store,
    cut: &mut Cut,
    step: Step,
    arrived: Arrived,
    below: &mut Below,
) -> Result<Progress, CheckpointRefusal> {
    let next = match step {
        Step::Make { index } => {
            if !made(&arrived.event, false) {
                return Err(failure(&cut.session.path, &arrived.event));
            }
            if index < 5 {
                make(cut, index.checked_add(1).expect("six directories"), below)
            } else {
                let path = under(&cut.temporary, b"generations/1/secrets");
                let request =
                    file::Request::OpenPrivate { owner: OWNER, root: cut.session.root.token, path: path.clone() };
                ask(cut, path, request, below);
                Step::Private
            }
        }
        Step::Private => {
            let (file, _) = opened(&mut cut.session, arrived)?;
            close(cut, file, below);
            Step::PrivateClose
        }
        Step::PrivateClose => {
            success(&arrived.event, Success::Closed, cut)?;
            check_open(store, cut, false, below)
        }
        Step::CheckOpen { secret } => {
            let root = directory(cut, arrived)?;
            below.checkpoints.push(file::Request::ListPage {
                owner: OWNER,
                directory: root.token,
                max: generations::PAGE,
            });
            Step::CheckPage { secret, root }
        }
        Step::CheckPage { secret, root } => {
            let file::Event::Listed { entries, .. } = arrived.event else {
                return Err(failure(&cut.session.path, &arrived.event));
            };
            for entry in &entries {
                if !known(&entry.name, secret) {
                    return Err(CheckpointRefusal::Unrecognized { path: text(&under(&cut.session.path, &entry.name)) });
                }
            }
            if entries.is_empty() {
                close(cut, root.token, below);
                Step::CheckClose { secret }
            } else {
                below.checkpoints.push(file::Request::ListPage {
                    owner: OWNER,
                    directory: root.token,
                    max: generations::PAGE,
                });
                Step::CheckPage { secret, root }
            }
        }
        Step::CheckClose { secret } => {
            success(&arrived.event, Success::Closed, cut)?;
            if secret { next_file(store, cut, below) } else { check_open(store, cut, true, below) }
        }
        Step::Shard => {
            if !made(&arrived.event, true) {
                return Err(failure(&cut.session.path, &arrived.event));
            }
            read_open(store, cut, below)
        }
        Step::ReadOpen => {
            if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
                if current(cut).row.part != 4 {
                    return Err(vanished(store, cut));
                }
                current_mut(cut).row.way = 3;
                finished_file(store, cut, below)
            } else {
                let (reader, length) = opened(&mut cut.session, arrived)?;
                let source = current_mut(cut);
                source.row.length = source.end.unwrap_or(length);
                if length < source.row.length {
                    return Err(vanished(store, cut));
                }
                if source.row.way == 1 {
                    let row = &source.row;
                    let from = source_path(store, row.part, row.number);
                    let (parent, _) = split(&from);
                    open_dir(cut, parent, below);
                    Step::LinkSource { reader }
                } else {
                    write_open(cut, reader, below)
                }
            }
        }
        Step::LinkSource { reader } => {
            let source = directory(cut, arrived)?;
            let (parent, _) = split(&destination(cut));
            open_dir(cut, parent, below);
            Step::LinkDestination { reader, source }
        }
        Step::LinkDestination { reader, source } => {
            let destination_root = directory(cut, arrived)?;
            let row = &current(cut).row;
            let from = source_path(store, row.part, row.number);
            let (_, from_leaf) = split(&from);
            let (_, to) = split(&destination(cut));
            let request = file::Request::HardLink {
                owner: OWNER,
                from_root: source.fd,
                from: from_leaf,
                to_root: destination_root.fd,
                to,
            };
            ask(cut, from, request, below);
            Step::Link { reader, source, destination: destination_root }
        }
        Step::Link { reader, source, destination } => {
            let way = if kind(&arrived.event) == Kind::Linked {
                LinkWay::Linked
            } else {
                match error(&arrived.event) {
                    Some(Error::Permission | Error::TooManyLinks | Error::Other(_)) => LinkWay::Copy,
                    Some(Error::NotFound) => {
                        if current(cut).row.part == 4 {
                            LinkWay::Missing
                        } else {
                            return Err(vanished(store, cut));
                        }
                    }
                    Some(
                        Error::Busy
                        | Error::Refused
                        | Error::Reset
                        | Error::BrokenPipe
                        | Error::NotConnected
                        | Error::AddressInUse
                        | Error::AddressNotAvailable
                        | Error::Unreachable
                        | Error::TimedOut
                        | Error::TooManyOpenFiles
                        | Error::NoBufferSpace
                        | Error::Cancelled
                        | Error::TooLate
                        | Error::Exists
                        | Error::NotADirectory
                        | Error::IsADirectory
                        | Error::NotEmpty
                        | Error::ReadOnly
                        | Error::NoSpace
                        | Error::NameTooLong
                        | Error::Escape
                        | Error::NotAFile
                        | Error::InvalidArgument,
                    )
                    | None => return Err(failure(&cut.session.path, &arrived.event)),
                }
            };
            close(cut, destination.token, below);
            Step::LinkDestinationClose { reader, source, way }
        }
        Step::LinkDestinationClose { reader, source, way } => {
            success(&arrived.event, Success::Closed, cut)?;
            close(cut, source.token, below);
            Step::LinkSourceClose { reader, way }
        }
        Step::LinkSourceClose { reader, way } => {
            success(&arrived.event, Success::Closed, cut)?;
            match way {
                LinkWay::Copy => write_open(cut, reader, below),
                LinkWay::Linked | LinkWay::Missing => {
                    if way == LinkWay::Missing {
                        current_mut(cut).row.way = 3;
                    }
                    close(cut, reader, below);
                    Step::ReadClose
                }
            }
        }
        Step::WriteOpen { reader } => {
            let (writer, _) = opened(&mut cut.session, arrived)?;
            read(store, cut, reader, writer, 0, below)
        }
        Step::Read { reader, writer, offset } => {
            let file::Event::Read { bytes, .. } = arrived.event else {
                return Err(failure(&cut.session.path, &arrived.event));
            };
            let count = skein_lib::length::len_u64(bytes.len());
            if count == 0 {
                return Err(vanished(store, cut));
            }
            cut.session.path = destination(cut);
            below.checkpoints.push(file::Request::WriteAt { owner: OWNER, file: writer, offset, bytes });
            Step::Write { reader, writer, offset, count }
        }
        Step::Write { reader, writer, offset, count } => {
            success(&arrived.event, Success::Written, cut)?;
            read(store, cut, reader, writer, offset.checked_add(count).expect("copy bounded by end"), below)
        }
        Step::FileSync { reader, writer } => {
            success(&arrived.event, Success::Synced, cut)?;
            close(cut, writer, below);
            Step::WriteClose { reader }
        }
        Step::WriteClose { reader } => {
            success(&arrived.event, Success::Closed, cut)?;
            close(cut, reader, below);
            Step::ReadClose
        }
        Step::ReadClose => {
            success(&arrived.event, Success::Closed, cut)?;
            finished_file(store, cut, below)
        }
        Step::SyncOpen { index } => {
            let root = directory(cut, arrived)?;
            below.checkpoints.push(file::Request::SyncDirectory { owner: OWNER, root: root.fd });
            Step::Sync { index, root }
        }
        Step::Sync { index, root } => {
            success(&arrived.event, Success::Synced, cut)?;
            close(cut, root.token, below);
            Step::SyncClose { index }
        }
        Step::SyncClose { index } => {
            success(&arrived.event, Success::Closed, cut)?;
            sync_next(cut, index.saturating_add(1), below)
        }
        Step::Manifest => {
            success(&arrived.event, Success::Stored, cut)?;
            store_file(cut, b"active", Box::from(b"1\n".as_slice()), below);
            Step::Active
        }
        Step::Active => {
            success(&arrived.event, Success::Stored, cut)?;
            let bytes = cut.record.clone().encode().expect("selection within record bounds");
            store_file(cut, b"checkpoint", bytes, below);
            Step::Record
        }
        Step::Record => {
            success(&arrived.event, Success::Stored, cut)?;
            top(cut, 0, below)
        }
        Step::TopOpen { index } => {
            let root = directory(cut, arrived)?;
            below.checkpoints.push(file::Request::SyncDirectory { owner: OWNER, root: root.fd });
            Step::TopSync { index, root }
        }
        Step::TopSync { index, root } => {
            success(&arrived.event, Success::Synced, cut)?;
            close(cut, root.token, below);
            Step::TopClose { index }
        }
        Step::TopClose { index } => {
            success(&arrived.event, Success::Closed, cut)?;
            if index == 0 {
                top(cut, 1, below)
            } else {
                open_dir(cut, Box::from(b"checkpoints".as_slice()), below);
                Step::ParentOpen
            }
        }
        Step::ParentOpen => {
            let root = directory(cut, arrived)?;
            let (_, from) = split(&cut.temporary);
            let to = Box::from(cut.name.bytes());
            let request = file::Request::Rename { owner: OWNER, from_root: root.fd, from, to_root: root.fd, to };
            let path = join(b"checkpoints/", &cut.name.bytes());
            ask(cut, path, request, below);
            Step::Rename { root }
        }
        Step::Rename { root } => {
            success(&arrived.event, Success::Renamed, cut)?;
            below.checkpoints.push(file::Request::SyncDirectory { owner: OWNER, root: root.fd });
            Step::ParentSync { root }
        }
        Step::ParentSync { root } => {
            success(&arrived.event, Success::Synced, cut)?;
            Step::Retaining(Box::new(super::retention::Retention::begin(root, store.limits.checkpoints.keep, below)))
        }
        Step::Retaining(mut retention) => {
            if retention.up(cut, arrived, below)? {
                let parent = *cut.session.handles.iter().last().expect("checkpoint parent retained");
                close(cut, parent, below);
                Step::ParentClose
            } else {
                Step::Retaining(retention)
            }
        }
        Step::ParentClose => {
            success(&arrived.event, Success::Closed, cut)?;
            measure(cut, 0, 0, below)
        }
        Step::MeasureOpen { index, held } => {
            if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
                measure(cut, index.saturating_add(1), held, below)
            } else {
                let (file, _) = opened(&mut cut.session, arrived)?;
                below.checkpoints.push(file::Request::Stat { owner: OWNER, file });
                Step::MeasureStat { index, held, file }
            }
        }
        Step::MeasureStat { index, mut held, file } => {
            let file::Event::Stated { stat, .. } = arrived.event else {
                return Err(failure(&cut.session.path, &arrived.event));
            };
            if stat.links == 1 {
                held = held.saturating_add(stat.size);
            }
            close(cut, file, below);
            Step::MeasureClose { index, held }
        }
        Step::MeasureClose { index, held } => {
            success(&arrived.event, Success::Closed, cut)?;
            measure(cut, index.saturating_add(1), held, below)
        }
        Step::Finished { held } => {
            success(&arrived.event, Success::Closed, cut)?;
            return Ok(Progress::Done { held });
        }
    };
    match next {
        Step::Manifest => {
            super::taking::release(store, true, true, below);
            crate::transcripts::drop::released(
                store,
                core::mem::replace(&mut cut.deferred, skein_lib::Queue::with_capacity(0)),
            );
            manifest(cut, below);
            Ok(Progress::Pending(Step::Manifest))
        }
        next @ (Step::Make { .. }
        | Step::Private
        | Step::PrivateClose
        | Step::CheckOpen { .. }
        | Step::CheckPage { .. }
        | Step::CheckClose { .. }
        | Step::Shard
        | Step::ReadOpen
        | Step::LinkSource { .. }
        | Step::LinkDestination { .. }
        | Step::Link { .. }
        | Step::LinkDestinationClose { .. }
        | Step::LinkSourceClose { .. }
        | Step::WriteOpen { .. }
        | Step::Read { .. }
        | Step::Write { .. }
        | Step::FileSync { .. }
        | Step::WriteClose { .. }
        | Step::ReadClose
        | Step::SyncOpen { .. }
        | Step::Sync { .. }
        | Step::SyncClose { .. }
        | Step::Active
        | Step::Record
        | Step::TopOpen { .. }
        | Step::TopSync { .. }
        | Step::TopClose { .. }
        | Step::Rename { .. }
        | Step::ParentOpen
        | Step::ParentSync { .. }
        | Step::ParentClose
        | Step::Retaining(_)
        | Step::MeasureOpen { .. }
        | Step::MeasureStat { .. }
        | Step::MeasureClose { .. }
        | Step::Finished { .. }) => Ok(Progress::Pending(next)),
    }
}
fn source_path(store: &Store, part: u8, number: u64) -> Box<[u8]> {
    source(store, part, number)
}
#[derive(PartialEq, Eq)]
enum Kind {
    Linked,
    Closed,
    Written,
    Synced,
    Stored,
    Renamed,
    Made,
    Failed,
    Other,
}
type Success = Kind;
fn success(event: &file::Event, expected: Success, cut: &Cut) -> Result<(), CheckpointRefusal> {
    let okay = kind(event) == expected;
    if okay { Ok(()) } else { Err(failure(&cut.session.path, event)) }
}

fn kind(event: &file::Event) -> Kind {
    match event {
        file::Event::Closed { .. } => Kind::Closed,
        file::Event::Written { .. } => Kind::Written,
        file::Event::Synced { .. } => Kind::Synced,
        file::Event::Stored { .. } => Kind::Stored,
        file::Event::Renamed { .. } => Kind::Renamed,
        file::Event::Linked { .. } => Kind::Linked,
        file::Event::Made { .. } => Kind::Made,
        file::Event::Failed { .. } => Kind::Failed,
        file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Refused { .. }
        | file::Event::Opened { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Read { .. }
        | file::Event::Cut { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. }
        | file::Event::Cancelled { .. } => Kind::Other,
    }
}
fn error(event: &file::Event) -> Option<Error> {
    match event {
        file::Event::Failed { error, .. } => Some(*error),
        file::Event::Linked { .. }
        | file::Event::Closed { .. }
        | file::Event::Written { .. }
        | file::Event::Synced { .. }
        | file::Event::Stored { .. }
        | file::Event::Renamed { .. }
        | file::Event::Made { .. }
        | file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Refused { .. }
        | file::Event::Opened { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Read { .. }
        | file::Event::Cut { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. }
        | file::Event::Cancelled { .. } => None,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LinkWay {
    Linked,
    Copy,
    Missing,
}
fn split(path: &[u8]) -> (Box<[u8]>, Box<[u8]>) {
    let mut at = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            at = index;
        }
    }
    (
        Box::from(path.get(..at).expect("parent prefix")),
        Box::from(path.get(at.saturating_add(1)..).expect("leaf suffix")),
    )
}

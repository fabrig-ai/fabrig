//! Walk the committed file prefixes and turns-log extents before planning (protocol/store/opening-rehearsal.md, 1).
use crate::{Arrived, Below, Cursor, Event, OpenRefusal, Owner, Phase, Store, Undecoded, codec, keys, open, pages};
use alloc::boxed::Box;
use skein_codec::value::Value;
use skein_io::file;
use skein_lib::{Queue, Reader, Token, Writer};

pub(crate) const OWNER: Token = Token::new(0x5457);

#[derive(Clone, Copy, Debug)]
struct Type {
    family: &'static str,
    number: u16,
}

#[derive(Debug)]
pub(super) struct Walk {
    decoder: crate::registry::rehearsal::Decoder,
    value: Value,
    records: u64,
    body: Option<Type>,
    turn: Type,
    after: Option<Cursor>,
    task: u64,
    end: u64,
    at: u64,
    stage: Stage,
}
#[derive(Debug)]
enum Stage {
    Next,
    Opening,
    Length { file: Token, bytes: Writer },
    Frame { file: Token, bytes: Writer },
    Closing,
    Log { log: u64, waiting: bool },
}
#[derive(Debug)]
struct Unread {
    family: &'static str,
    version: u16,
}

impl Walk {
    pub(super) fn new(store: &Store, decoder: crate::registry::rehearsal::Decoder, records: u64) -> Option<Self> {
        let mut body = None;
        let mut turn = None;
        for set in store.owner_sets.sets {
            for family in set.families {
                if let Some(frame) = family.frames {
                    let ty = Type { family: family.name, number: frame.number };
                    match set.owner {
                        Owner::Jig => body = Some(ty),
                        Owner::Smith => turn = Some(ty),
                        Owner::Skein | Owner::Application => unreachable!("admitted frames slots"),
                    }
                }
            }
        }
        Some(Self {
            decoder,
            value: Value::with_room(store.limits.store.page.bytes),
            records,
            body,
            turn: turn?,
            after: None,
            task: 0,
            end: 0,
            at: 0,
            stage: Stage::Next,
        })
    }

    pub(super) fn seed(&mut self, store: &Store) -> Result<(), OpenRefusal> {
        self.decoder.seed_frames(store)
    }

    fn bad_frame(&self) -> Unread {
        let family = match self.body {
            Some(body) => body.family,
            None => self.turn.family,
        };
        Unread { family, version: 0 }
    }

    fn decode(&mut self, store: &Store, bytes: Box<[u8]>) -> Result<(), Unread> {
        let turn = match self.body {
            Some(body) => {
                match self.decoder.transcript(body.family, body.number, self.turn.family, &mut self.value, bytes) {
                    Ok(turn) => turn,
                    Err(version) => return Err(Unread { family: body.family, version }),
                }
            }
            None => codec::core::read_turn(&bytes, &store.codec_limits).or(Err(self.bad_frame()))?.transcript,
        };
        if let Err(version) = self.decoder.decode(self.turn.family, self.turn.number, &mut self.value, turn) {
            return Err(Unread { family: self.turn.family, version });
        }
        self.records = self.records.checked_add(1).expect("bounded disk counts fit in u64");
        drop(self.value.take());
        Ok(())
    }
}

pub(crate) fn ready(store: &Store) -> bool {
    let Some(state) = &store.rehearsal else { return false };
    let Some(walk) = &state.frames else { return false };
    match walk.stage {
        Stage::Next | Stage::Log { waiting: false, .. } => true,
        Stage::Opening
        | Stage::Length { .. }
        | Stage::Frame { .. }
        | Stage::Closing
        | Stage::Log { waiting: true, .. } => false,
    }
}

pub(crate) fn fire(store: &mut Store, above: &mut Queue<Event>, below: &mut Below) {
    if !ready(store) {
        return;
    }
    let mut walk = super::state(store).frames.take().expect("ready walk");
    let owner = super::state(store).owner;
    match walk.stage {
        Stage::Next => match next(store, &mut walk, below) {
            Ok(true) => {
                super::decoded(store, walk.records, above);
                open::open(store, owner, above, below);
            }
            Ok(false) => super::state(store).frames = Some(walk),
            Err(()) => open::refuse(store, owner, OpenRefusal::Failed, above),
        },
        Stage::Log { log, waiting: false } => {
            let at = walk.at;
            walk.stage = Stage::Log { log, waiting: true };
            super::state(store).frames = Some(walk);
            let mut events = Queue::uncapped();
            skein_kv::down(
                &mut store.stores.store,
                &crate::memory_env(store.limits.store),
                skein_kv::Request::ReadLog {
                    owner: OWNER,
                    log,
                    at,
                    direction: skein_kv::Direction::Forward,
                    max: store.limits.store.page.bytes,
                    rows: 1,
                },
                &mut events,
                &mut below.store,
            );
            crate::directory::events_up(store, false, &mut events, above, below);
        }
        Stage::Opening
        | Stage::Length { .. }
        | Stage::Frame { .. }
        | Stage::Closing
        | Stage::Log { waiting: true, .. } => unreachable!("only ready steps fire"),
    }
}

fn next(store: &mut Store, walk: &mut Walk, below: &mut Below) -> Result<bool, ()> {
    let ranges = [
        skein_kv::Range { start: Box::from([0, 1]), end: Some(Box::from([0, 2])) },
        skein_kv::Range { start: Box::from([0, 11]), end: Some(Box::from([0, 12])) },
    ];
    let page = pages::reader::walk(store, &ranges, walk.after.as_ref(), 1).or(Err(()))?;
    let Some(row) = page.rows.into_iter().next() else { return Ok(true) };
    walk.after = Some(row.cursor);
    walk.at = 0;
    match keys::decode_own(&row.row.key).ok_or(())? {
        keys::Own::Transcript { task } => {
            walk.task = task;
            walk.end = codec::component::Transcript::decode(&row.row.value).or(Err(()))?.length;
            if walk.end != 0 {
                let path = path(task);
                let source = super::source(store);
                below.transcripts.push(file::Request::OpenReadNoFollow { owner: OWNER, root: source.fd, name: path });
                walk.stage = Stage::Opening;
            }
        }
        keys::Own::TurnLog { task } => {
            walk.task = task;
            let log = crate::logs::turns(task).ok_or(())?;
            walk.end = store.stores.store.log_end(log);
            if walk.end != 0 {
                walk.stage = Stage::Log { log, waiting: false };
            }
        }
        keys::Own::Format
        | keys::Own::Floor
        | keys::Own::Wires
        | keys::Own::Attempt { .. }
        | keys::Own::Opening(_)
        | keys::Own::Layout { .. }
        | keys::Own::Text(_)
        | keys::Own::Job { .. }
        | keys::Own::History { .. }
        | keys::Own::LogAttempt { .. } => return Err(()),
    }
    Ok(false)
}

fn path(task: u64) -> Box<[u8]> {
    let name = crate::transcripts::path(task);
    let directory = crate::generations::TRANSCRIPTS;
    let capacity = directory.len().checked_add(1).expect("fixed path").checked_add(name.len()).expect("fixed path");
    let mut path = Writer::new(capacity);
    path.put(directory).expect("fixed path");
    path.put(b"/").expect("fixed path");
    path.put(&name).expect("fixed path");
    path.finish()
}

pub(crate) fn file(store: &mut Store, arrived: Arrived, above: &mut Queue<Event>, below: &mut Below) {
    let Some(mut walk) = super::state(store).frames.take() else { return };
    let stage = core::mem::replace(&mut walk.stage, Stage::Next);
    match advance(store, &mut walk, stage, arrived.event, below) {
        Ok(()) => super::state(store).frames = Some(walk),
        Err(error) => unread(store, &walk, error, false, above),
    }
}

fn advance(store: &Store, walk: &mut Walk, stage: Stage, event: file::Event, below: &mut Below) -> Result<(), Unread> {
    match stage {
        Stage::Opening => {
            let file::Event::Opened { file, len, .. } = event else { return Err(walk.bad_frame()) };
            if len < walk.end || walk.end < 4 {
                return Err(walk.bad_frame());
            }
            start_length(walk, file, below);
        }
        Stage::Length { file, mut bytes } => {
            if !append(walk, file, event, &mut bytes, below)? {
                walk.stage = Stage::Length { file, bytes };
                return Ok(());
            }
            let prefix = bytes.finish();
            let length = Reader::new(&prefix).u32().ok_or(walk.bad_frame())?;
            if length < crate::transcripts::frame::WRAPPER_BYTES
                || length > store.limits.store.page.bytes
                || u64::from(length) > walk.end.checked_sub(walk.at).ok_or(walk.bad_frame())?
            {
                return Err(walk.bad_frame());
            }
            let mut bytes = Writer::new(skein_lib::usize_of(length));
            bytes.put(&prefix).or(Err(walk.bad_frame()))?;
            demand(
                file,
                walk.at.checked_add(4).ok_or(walk.bad_frame())?,
                length.checked_sub(4).ok_or(walk.bad_frame())?,
                below,
            );
            walk.stage = Stage::Frame { file, bytes };
        }
        Stage::Frame { file, mut bytes } => {
            if !append(walk, file, event, &mut bytes, below)? {
                walk.stage = Stage::Frame { file, bytes };
                return Ok(());
            }
            let raw = bytes.finish();
            let body = crate::transcripts::frame::body(&raw).or(Err(walk.bad_frame()))?;
            walk.decode(store, Box::from(body))?;
            walk.at = walk.at.checked_add(skein_lib::len_u64(raw.len())).ok_or(walk.bad_frame())?;
            if walk.at == walk.end {
                below.transcripts.push(file::Request::Close { owner: OWNER, file });
                walk.stage = Stage::Closing;
            } else if walk.end.checked_sub(walk.at).ok_or(walk.bad_frame())? >= 4 {
                start_length(walk, file, below);
            } else {
                return Err(walk.bad_frame());
            }
        }
        Stage::Closing => {
            if !matches_closed(event) {
                return Err(walk.bad_frame());
            }
        }
        Stage::Next | Stage::Log { .. } => return Err(walk.bad_frame()),
    }
    Ok(())
}

fn start_length(walk: &mut Walk, file: Token, below: &mut Below) {
    demand(file, walk.at, 4, below);
    walk.stage = Stage::Length { file, bytes: Writer::new(4) };
}
fn demand(file: Token, offset: u64, max: u32, below: &mut Below) {
    below.transcripts.push(file::Request::ReadAt { owner: OWNER, file, offset, max });
}
fn append(walk: &Walk, file: Token, event: file::Event, bytes: &mut Writer, below: &mut Below) -> Result<bool, Unread> {
    let file::Event::Read { bytes: read, .. } = event else { return Err(walk.bad_frame()) };
    if read.is_empty() || bytes.put(&read).is_err() {
        return Err(walk.bad_frame());
    }
    if bytes.room() == 0 {
        return Ok(true);
    }
    demand(
        file,
        walk.at.checked_add(skein_lib::len_u64(bytes.written())).ok_or(walk.bad_frame())?,
        u32::try_from(bytes.room()).or(Err(walk.bad_frame()))?,
        below,
    );
    Ok(false)
}
#[expect(clippy::match_like_matches_macro, reason = "step code uses if let for a single match")]
fn matches_closed(event: file::Event) -> bool {
    if let file::Event::Closed { .. } = event { true } else { false }
}

pub(crate) fn log(store: &mut Store, records: Option<Box<[skein_kv::LogRecord]>>, above: &mut Queue<Event>) {
    let Some(mut walk) = super::state(store).frames.take() else { return };
    match log_record(store, &mut walk, records) {
        Ok(()) => super::state(store).frames = Some(walk),
        Err(error) => unread(store, &walk, error, true, above),
    }
}
fn log_record(store: &Store, walk: &mut Walk, records: Option<Box<[skein_kv::LogRecord]>>) -> Result<(), Unread> {
    let Stage::Log { log, waiting: true } = walk.stage else { return Err(walk.bad_frame()) };
    let mut records = records.ok_or(walk.bad_frame())?.into_iter();
    let record = records.next().ok_or(walk.bad_frame())?;
    if records.next().is_some() || record.at != walk.at || record.end <= record.at || record.end > walk.end {
        return Err(walk.bad_frame());
    }
    walk.decode(store, record.record.ok_or(walk.bad_frame())?)?;
    walk.at = record.end;
    walk.stage = if walk.at == walk.end { Stage::Next } else { Stage::Log { log, waiting: false } };
    Ok(())
}

#[expect(clippy::disallowed_methods, reason = "the file and log path builders return fixed ASCII names")]
fn unread(store: &mut Store, walk: &Walk, error: Unread, log: bool, above: &mut Queue<Event>) {
    let name = if log {
        skein_kv::log_path(crate::logs::turns(walk.task).expect("checked turns log"))
    } else {
        path(walk.task)
    };
    let transcript = Box::from(core::str::from_utf8(&name).expect("ascii name"));
    store.phase = Phase::Stopped;
    above.push(Event::Undecodable {
        owner: super::state(store).owner,
        family: Box::from(error.family),
        version: u32::from(error.version),
        at: Undecoded::Frame { transcript, offset: walk.at },
    });
}

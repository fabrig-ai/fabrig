//! Read-only inner-version counts before jig's settling pages (protocol/store/jobs.md, 5.4).
use crate::{JobCount, Store, codec, keys, logs, open, pages};
use alloc::boxed::Box;
use skein_io::file;
use skein_lib::{Reader, Token, Writer};

pub(crate) const OWNER: Token = Token::new(0x5457);
// Leading length, fixed metadata (including Some(read)), payload length and inner version.
pub(crate) const HEAD: u32 = 58;

#[derive(Debug)]
enum Stage {
    Rows,
    Opening { end: u64 },
    Length { file: Token, end: u64, at: u64, bytes: Writer },
    Head { file: Token, end: u64, at: u64, length: u32, bytes: Writer },
    Closing { file: Token },
    Log { log: u64, end: u64, at: u64, span: Option<(u64, u32)> },
    Done,
}
#[derive(Debug)]
pub(crate) struct Census {
    after: Option<Box<[u8]>>,
    logs: bool,
    stage: Stage,
    six: u64,
    seven: u64,
    other: u64,
    unread: u64,
}
pub(crate) enum Next {
    File(file::Request),
    Log(skein_kv::Request),
    Done([JobCount; 4]),
}
impl Census {
    pub(crate) fn new() -> Self {
        Self { after: None, logs: false, stage: Stage::Rows, six: 0, seven: 0, other: 0, unread: 0 }
    }
    fn count(&mut self, version: Option<u16>) {
        let count = match version {
            Some(6) => &mut self.six,
            Some(7) => &mut self.seven,
            Some(0..=5 | 8..=u16::MAX) => &mut self.other,
            None => &mut self.unread,
        };
        *count = count.saturating_add(1);
    }
    pub(crate) fn next(&mut self, store: &mut Store) -> Result<Next, ()> {
        let mut bound = 4_u64;
        for space in store.stores.store.census().keyspaces() {
            bound = bound.saturating_add(space.rows);
        }
        for _ in 0..bound {
            match core::mem::replace(&mut self.stage, Stage::Done) {
                Stage::Rows => {
                    if let Some(request) = self.row(store)? {
                        return Ok(request);
                    }
                }
                Stage::Length { file, end, at, bytes } => {
                    if end.checked_sub(at).ok_or(())? < 4 {
                        self.count(None);
                        self.stage = Stage::Closing { file };
                        continue;
                    }
                    if bytes.room() != 0 {
                        let request = read(file, at, &bytes)?;
                        self.stage = Stage::Length { file, end, at, bytes };
                        return Ok(Next::File(request));
                    }
                    let length = Reader::new(&bytes.finish()).u32().ok_or(())?;
                    if length < super::frame::WRAPPER_BYTES || u64::from(length) > end.checked_sub(at).ok_or(())? {
                        self.count(None);
                        self.stage = Stage::Closing { file };
                    } else {
                        let max = HEAD.min(length.checked_sub(8).ok_or(())?);
                        let mut head = Writer::new(skein_lib::usize_of(max));
                        head.put(&length.to_be_bytes()).or(Err(()))?;
                        self.stage = Stage::Head { file, end, at, length, bytes: head };
                    }
                }
                Stage::Head { file, end, at, length, bytes } => {
                    if bytes.room() != 0 {
                        let request = read(file, at, &bytes)?;
                        self.stage = Stage::Head { file, end, at, length, bytes };
                        return Ok(Next::File(request));
                    }
                    let head = bytes.finish();
                    let body = head.get(4..).ok_or(())?;
                    self.count(version(
                        body,
                        length.checked_sub(super::frame::WRAPPER_BYTES).ok_or(())?,
                        &store.codec_limits,
                    ));
                    self.advance_file(file, end, at, length)?;
                }
                Stage::Closing { file } => {
                    self.stage = Stage::Closing { file };
                    return Ok(Next::File(file::Request::Close { owner: OWNER, file }));
                }
                Stage::Log { log, end, at, span } => {
                    if at == end {
                        self.stage = Stage::Rows;
                        continue;
                    }
                    let (position, direction, max) = match span {
                        // A turn is larger than this probe; its answer locates
                        // one span and gives the room for the whole record.
                        None => (at, skein_kv::Direction::Forward, 1),
                        // Read back from the probed end. KV may look behind at a record
                        // already visited, never ahead at an unread following record.
                        Some((next, length)) => (next, skein_kv::Direction::Backward, length),
                    };
                    self.stage = Stage::Log { log, end, at, span };
                    return Ok(Next::Log(skein_kv::Request::ReadLog {
                        owner: OWNER,
                        log,
                        at: position,
                        direction,
                        max,
                    }));
                }
                Stage::Done => {
                    return Ok(Next::Done([
                        JobCount { name: Box::from("turns-6"), count: self.six },
                        JobCount { name: Box::from("turns-7"), count: self.seven },
                        JobCount { name: Box::from("turns-other"), count: self.other },
                        JobCount { name: Box::from("turns-unread"), count: self.unread },
                    ]));
                }
                Stage::Opening { .. } => return Err(()),
            }
        }
        Err(())
    }
    fn row(&mut self, store: &mut Store) -> Result<Option<Next>, ()> {
        let tag = if self.logs { 11 } else { 1 };
        let start = match &self.after {
            Some(key) => pages::past(key).ok_or(())?,
            None => Box::new([0, tag]),
        };
        let range = skein_kv::Range { start, end: Some(Box::new([0, tag.checked_add(1).ok_or(())?])) };
        let page = pages::load(store, range).or(Err(()))?;
        let Some(row) = page.rows.into_iter().next() else {
            self.after = None;
            self.stage = if self.logs { Stage::Done } else { Stage::Rows };
            self.logs = true;
            return Ok(None);
        };
        self.after = Some(row.key.clone());
        self.stage = Stage::Rows;
        match keys::decode_own(&row.key) {
            Some(keys::Own::Transcript { task }) => {
                let end = match codec::component::Transcript::decode(&row.value) {
                    Ok(row) => row.length,
                    Err(_) => {
                        self.count(None);
                        return Ok(None);
                    }
                };
                if end == 0 {
                    return Ok(None);
                }
                self.stage = Stage::Opening { end };
                // Serving opens its transcripts root after the jobs. Rehearsal
                // shares the source's files, not the scratch generation's map.
                let root = if store.rehearsal.is_some() {
                    crate::rehearsal::source(store)
                } else {
                    store.directory.root.ok_or(())?
                };
                let name = skein_lib::bytes::concat(&[crate::generations::TRANSCRIPTS, b"/", &super::path(task)]);
                Ok(Some(Next::File(file::Request::OpenReadNoFollow { owner: OWNER, root: root.fd, name })))
            }
            Some(keys::Own::TurnLog { task }) => {
                let Some(log) = logs::turns(task) else {
                    self.count(None);
                    return Ok(None);
                };
                let extent = open::get(store, OWNER, logs::extent_key(log)).or(Err(()))?;
                let end = match extent {
                    Some(bytes) => match logs::extent(&bytes) {
                        Some(end) => end,
                        None => {
                            self.count(None);
                            return Ok(None);
                        }
                    },
                    None => 0,
                };
                self.stage = Stage::Log { log, end, at: 0, span: None };
                Ok(None)
            }
            Some(
                keys::Own::Format
                | keys::Own::Attempt { .. }
                | keys::Own::Layouts
                | keys::Own::Wires
                | keys::Own::Opening(_)
                | keys::Own::Job { .. }
                | keys::Own::Layout { .. }
                | keys::Own::Text(_)
                | keys::Own::History { .. }
                | keys::Own::LogAttempt { .. }
                | keys::Own::Floor,
            )
            | None => {
                self.count(None);
                Ok(None)
            }
        }
    }
    fn advance_file(&mut self, file: Token, end: u64, at: u64, length: u32) -> Result<(), ()> {
        let at = at.checked_add(u64::from(length)).ok_or(())?;
        self.stage =
            if at == end { Stage::Closing { file } } else { Stage::Length { file, end, at, bytes: Writer::new(4) } };
        Ok(())
    }
    pub(crate) fn file(&mut self, event: file::Event) -> Result<(), ()> {
        if event.owner() != OWNER {
            return Err(());
        }
        let event = arrival(event);
        match core::mem::replace(&mut self.stage, Stage::Rows) {
            Stage::Opening { end } => match event {
                Arrival::Opened(file) => {
                    self.stage = Stage::Length { file, end, at: 0, bytes: Writer::new(4) };
                }
                // Places keep no frame count. An inaccessible file contributes
                // one unread, as does a tail whose next span cannot be located.
                Arrival::Read(_) | Arrival::Closed | Arrival::Unread => self.count(None),
            },
            Stage::Length { file, end, at, mut bytes } => {
                if append(event, &mut bytes).is_err() {
                    self.count(None);
                    self.stage = Stage::Closing { file };
                } else {
                    self.stage = Stage::Length { file, end, at, bytes };
                }
            }
            Stage::Head { file, end, at, length, mut bytes } => {
                if append(event, &mut bytes).is_err() {
                    self.count(None);
                    self.advance_file(file, end, at, length)?;
                } else {
                    self.stage = Stage::Head { file, end, at, length, bytes };
                }
            }
            Stage::Closing { .. } => match event {
                Arrival::Closed => {}
                Arrival::Opened(_) | Arrival::Read(_) | Arrival::Unread => return Err(()),
            },
            Stage::Rows | Stage::Log { .. } | Stage::Done => return Err(()),
        }
        Ok(())
    }
    pub(crate) fn records(
        &mut self,
        records: Option<Box<[skein_kv::LogRecord]>>,
        limits: &codec::View,
    ) -> Result<(), ()> {
        let (log, end, at, span) = match self.stage {
            Stage::Log { log, end, at, span } => (log, end, at, span),
            Stage::Rows
            | Stage::Opening { .. }
            | Stage::Length { .. }
            | Stage::Head { .. }
            | Stage::Closing { .. }
            | Stage::Done => return Err(()),
        };
        let record = match records {
            Some(records) => records.into_iter().next(),
            None => None,
        };
        if let Some(record) = record
            && record.at == at
            && record.end > at
            && record.end <= end
        {
            if let Some(body) = record.record {
                self.count(version(&body, record.length, limits));
                self.stage = Stage::Log { log, end, at: record.end, span: None };
            } else if span.is_none() && record.length > 0 {
                self.stage = Stage::Log { log, end, at, span: Some((record.end, record.length)) };
            } else {
                self.count(None);
                self.stage = Stage::Log { log, end, at: record.end, span: None };
            }
        } else {
            self.count(None);
            self.stage = match span {
                Some((next, _)) => Stage::Log { log, end, at: next, span: None },
                None => Stage::Rows,
            };
        }
        Ok(())
    }
}
enum Arrival {
    Opened(Token),
    Read(Box<[u8]>),
    Closed,
    Unread,
}
fn arrival(event: file::Event) -> Arrival {
    match event {
        file::Event::Opened { file, .. } => Arrival::Opened(file),
        file::Event::Read { bytes, .. } => Arrival::Read(bytes),
        file::Event::Closed { .. } => Arrival::Closed,
        file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Stated { .. }
        | file::Event::Written { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Linked { .. }
        | file::Event::Renamed { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. }
        | file::Event::Failed { .. }
        | file::Event::Cancelled { .. } => Arrival::Unread,
    }
}
fn append(event: Arrival, bytes: &mut Writer) -> Result<(), ()> {
    match event {
        Arrival::Read(got) if !got.is_empty() => bytes.put(&got).or(Err(())),
        Arrival::Opened(_) | Arrival::Read(_) | Arrival::Closed | Arrival::Unread => Err(()),
    }
}
fn read(file: Token, at: u64, bytes: &Writer) -> Result<file::Request, ()> {
    let offset = at.checked_add(u64::try_from(bytes.written()).or(Err(()))?).ok_or(())?;
    Ok(file::Request::ReadAt { owner: OWNER, file, offset, max: u32::try_from(bytes.room()).or(Err(()))? })
}
fn version(body: &[u8], length: u32, limits: &codec::View) -> Option<u16> {
    let mut reader = Reader::new(body);
    reader.skip(31)?; // Version, opens_window, task, attempt, turn and spent.
    match reader.u8()? {
        0 => {}
        1 => {
            reader.u64()?;
        }
        2..=u8::MAX => return None,
    }
    reader.u64()?; // at
    let fields = body.len().checked_sub(skein_lib::usize_of(reader.remaining()))?;
    let size = reader.u32()?;
    if size < 2
        || size > limits.core.turn_record_transcript
        || u32::try_from(fields).ok()?.checked_add(4)?.checked_add(size)? != length
    {
        return None;
    }
    // Let the body's codec validate its version and fixed fields, without
    // handing it the transcript or invoking either frame patch machine.
    let mut metadata = Writer::new(fields.checked_add(4)?);
    metadata.put(body.get(..fields)?).ok()?;
    metadata.put(&0_u32.to_be_bytes()).ok()?;
    codec::core::read_turn(&metadata.finish(), limits).ok()?;
    skein_codec::version_of(reader.bytes(2)?).ok()
}

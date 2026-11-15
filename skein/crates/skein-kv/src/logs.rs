//! Payload frames, admission and bounded traversal (skein/docs/design/data/kv.md, 6).

#![expect(clippy::disallowed_types, reason = "frames and admitted appends are bounded by record and queued_bytes; memory logs keep committed frames")]

use alloc::{boxed::Box, vec::Vec};
use skein_lib::{List, Map, Writer, usize_of};
use crate::{Direction, Event, Large, Limits, LogRecord, LogRule, Op, Refusal};
use crate::{crc::crc32c, frame::{u32_at, u64_at}, map::KvMap};

/// An opaque frame made by `Store::frame` (data/kv.md, 6.1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LogFrame { pub(crate) bytes: Box<[u8]> }
impl LogFrame {
    #[must_use]
    #[expect(clippy::len_without_is_empty, reason = "a log frame always has its 17-byte framing")]
    pub fn len(&self) -> u64 { skein_lib::len_u64(self.bytes.len()) }
}

pub(crate) fn frame(record: &[u8], limit: u32) -> Result<LogFrame, Refusal> {
    let Ok(length) = u32::try_from(record.len()) else { return Err(Refusal::TooLarge(Large::Record)); };
    if length > limit { return Err(Refusal::TooLarge(Large::Record)); }
    let bound = skein_deflate::bound(record.len()).ok_or(Refusal::TooLarge(Large::Record))?;
    let mut encoded = skein_lib::bytes::zeroed(bound);
    let compressed = skein_deflate::encode(record, &mut encoded).expect("deflate's bound holds the encoding");
    let (kind, body) = if compressed < record.len() {
        (1, encoded.get(..compressed).expect("encoded prefix fits its bound"))
    } else { (0, record) };
    let total = body.len().checked_add(17).expect("record plus framing fits usize");
    let Ok(total_u32) = u32::try_from(total) else { return Err(Refusal::TooLarge(Large::Record)); };
    let mut out = Writer::new(total.checked_sub(8).expect("frame has checksum and tail"));
    out.put(&total_u32.to_be_bytes()).expect("sized frame");
    out.put(&[kind]).expect("sized frame");
    out.put(&length.to_be_bytes()).expect("sized frame");
    out.put(body).expect("sized frame");
    let prefix = out.finish();
    let mut out = Writer::new(total);
    out.put(&prefix).expect("sized frame");
    out.put(&crc32c(&prefix).to_be_bytes()).expect("sized frame");
    out.put(&total_u32.to_be_bytes()).expect("sized frame");
    Ok(LogFrame { bytes: out.finish() })
}

pub(crate) fn decode(bytes: &[u8], limit: u32) -> Option<Box<[u8]>> {
    let total = bytes.len();
    let tail = total.checked_sub(4)?;
    let crc = total.checked_sub(8)?;
    if total < 17 || usize_of(u32_at(bytes, 0)?) != total || usize_of(u32_at(bytes, tail)?) != total
        || u32_at(bytes, crc)? != crc32c(bytes.get(..crc)?) { return None; }
    let length = u32_at(bytes, 5)?;
    if length > limit { return None; }
    let body = bytes.get(9..crc)?;
    match bytes.get(4)? {
        0 => if body.len() == usize_of(length) { Some(Box::from(body)) } else { None },
        1 => {
            let mut out = List::with_capacity(length);
            skein_deflate::decode(body, &mut out)?;
            if out.len() != length { return None; }
            Some(out.into_boxed())
        }
        2..=u8::MAX => None,
    }
}

#[derive(Debug)]
pub(crate) struct Append { pub(crate) log: u64, pub(crate) at: u64, pub(crate) frame: LogFrame }
#[derive(Debug)]
pub(crate) struct Prepared {
    pub(crate) ops: Box<[Op]>,
    pub(crate) appends: Box<[Append]>,
    pub(crate) drops: Box<[u64]>,
    pub(crate) bytes: u64,
}
#[derive(Debug)]
struct End { at: u64, number: u64 }
#[derive(Debug)]
pub(crate) struct Logs {
    pub(crate) rule: Option<LogRule>,
    record: u32,
    ends: Map<u64, End>,
    memory: Map<u64, Vec<u8>>,
}
impl Logs {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self { rule: None, record: limits.record, ends: Map::uncapped(), memory: Map::uncapped() }
    }
    pub(crate) fn frame(&self, record: &[u8]) -> Result<LogFrame, Refusal> { frame(record, self.record) }
    pub(crate) fn key(&self, log: u64) -> Option<Box<[u8]>> {
        let rule = self.rule.as_ref()?;
        let mut out = Writer::new(rule.prefix.len().checked_add(8).expect("prefix plus name fits"));
        out.put(&rule.prefix).expect("sized extent key");
        out.put(&log.to_be_bytes()).expect("sized extent key");
        Some(out.finish())
    }
    pub(crate) fn extent(&self, map: &KvMap, log: u64) -> u64 {
        let Some(key) = self.key(log) else { return 0; };
        let Some(bytes) = map.get(&key) else { return 0; };
        u64_at(bytes, 0).unwrap_or(0)
    }
    pub(crate) fn exists(&self, map: &KvMap, log: u64) -> bool {
        let Some(key) = self.key(log) else { return false; };
        map.get(&key).is_some()
    }
    pub(crate) fn end(&self, map: &KvMap, log: u64) -> u64 {
        match self.ends.get(&log) { Some(end) => end.at, None => self.extent(map, log) }
    }
    pub(crate) fn prepare(&self, map: &KvMap, ops: Box<[Op]>, limits: &Limits) -> Result<Prepared, Refusal> {
        if ops.len() > usize_of(limits.ops) { return Err(Refusal::TooLarge(Large::Ops)); }
        let mut ends = Map::<u64, u64>::uncapped();
        let mut drops = Vec::new();
        let mut appends = Vec::new();
        let mut rows = Vec::new();
        let mut bytes = 0_u64;
        for op in ops {
            match op {
                Op::Put { key, value } => {
                    if self.reserved(&key) { return Err(Refusal::Logs); }
                    rows.push(Op::Put { key, value });
                }
                Op::Erase { key } => {
                    if self.reserved(&key) { return Err(Refusal::Logs); }
                    rows.push(Op::Erase { key });
                }
                Op::Append { log, frame } => {
                    if self.rule.is_none() || drops.contains(&log) { return Err(Refusal::Logs); }
                    if match u32_at(&frame.bytes, 5) { Some(length) => length > limits.record, None => true } {
                        return Err(Refusal::TooLarge(Large::Record));
                    }
                    let at = match ends.get(&log) { Some(end) => *end, None => self.end(map, log) };
                    let end = at.checked_add(frame.len()).ok_or(Refusal::TooLarge(Large::Log))?;
                    if end > limits.log { return Err(Refusal::TooLarge(Large::Log)); }
                    assert!(ends.insert(log, end).is_ok(), "ends bounded by admitted ops");
                    bytes = bytes.checked_add(frame.len()).ok_or(Refusal::Busy)?;
                    appends.push(Append { log, at, frame });
                }
                Op::Drop { log } => {
                    if self.rule.is_none() || ends.contains_key(&log) { return Err(Refusal::Logs); }
                    if !drops.contains(&log) { drops.push(log); }
                }
            }
        }
        for (log, end) in &ends {
            rows.push(Op::Put { key: self.key(*log).expect("append has a rule"), value: Box::from(end.to_be_bytes()) });
        }
        for &log in &drops { rows.push(Op::Erase { key: self.key(log).expect("drop has a rule") }); }
        Ok(Prepared { ops: rows.into_boxed_slice(), appends: appends.into_boxed_slice(), drops: drops.into_boxed_slice(), bytes })
    }
    fn reserved(&self, key: &[u8]) -> bool {
        match &self.rule { Some(rule) => key.starts_with(&rule.prefix), None => false }
    }
    pub(crate) fn reserve(&mut self, prepared: &Prepared, number: u64) {
        for append in &prepared.appends {
            let at = append.at.checked_add(append.frame.len()).expect("admitted end fits");
            assert!(self.ends.insert(append.log, End { at, number }).is_ok(), "ends bounded by queued ops");
        }
        for &log in &prepared.drops {
            assert!(self.ends.insert(log, End { at: 0, number }).is_ok(), "ends bounded by queued ops");
        }
    }
    pub(crate) fn applied(&mut self, appends: &[Append], drops: &[u64], number: u64) {
        for append in appends { self.forget(append.log, number); }
        for &log in drops { self.forget(log, number); }
    }
    fn forget(&mut self, log: u64, number: u64) {
        if match self.ends.get(&log) { Some(end) => end.number <= number, None => false } { let _old = self.ends.remove(&log); }
    }
    pub(crate) fn clear(&mut self) { self.ends = Map::uncapped(); self.memory = Map::uncapped(); }
    pub(crate) fn apply_memory(&mut self, appends: &[Append], drops: &[u64]) {
        for append in appends {
            if !self.memory.contains_key(&append.log) {
                assert!(self.memory.insert(append.log, Vec::new()).is_ok(), "memory logs grow with commits");
            }
            let bytes = self.memory.get_mut(&append.log).expect("inserted log");
            bytes.truncate(usize::try_from(append.at).expect("memory extent fits usize"));
            bytes.extend_from_slice(&append.frame.bytes);
        }
        for log in drops { let _old = self.memory.remove(log); }
    }
    pub(crate) fn read_memory(&self, mut read: Read, above: &mut skein_lib::Queue<Event>) {
        let bytes = self.memory.get(&read.log);
        for _ in 0..read.extent.saturating_add(1) {
            if read.finished() { above.push(read.answer()); return; }
            let frame = match bytes {
                Some(bytes) => match read.bounds(bytes) { Some((at, end)) => bytes.get(at..end), None => None },
                None => None,
            };
            let Some(frame) = frame else { above.push(read.unread()); return; };
            if !read.accept(frame, self.record) { above.push(read.unread()); return; }
        }
    }
}

/// A read holds the durable extent captured at admission, not a projected end.
#[derive(Debug)]
pub(crate) struct Read {
    pub(crate) owner: skein_lib::Token,
    pub(crate) log: u64,
    pub(crate) at: u64,
    pub(crate) direction: Direction,
    pub(crate) max: u32,
    pub(crate) extent: u64,
    pub(crate) records: Vec<LogRecord>,
    stopped: bool,
}
impl Read {
    pub(crate) fn new(owner: skein_lib::Token, log: u64, at: u64, direction: Direction, max: u32, extent: u64) -> Self {
        Self { owner, log, at, direction, max, extent, records: Vec::new(), stopped: false }
    }
    pub(crate) fn finished(&self) -> bool {
        self.stopped || self.extent == 0 || match self.direction {
            Direction::Forward => self.at >= self.extent,
            Direction::Backward => self.at == 0,
        }
    }
    pub(crate) fn bounds(&self, bytes: &[u8]) -> Option<(usize, usize)> {
        let at = usize::try_from(self.at).ok()?;
        match self.direction {
            Direction::Forward => Some((at, at.checked_add(usize_of(u32_at(bytes, at)?))?)),
            Direction::Backward => Some((at.checked_sub(usize_of(u32_at(bytes, at.checked_sub(4)?)?))?, at)),
        }
    }
    pub(crate) fn span(&self, length: u32) -> Option<(u64, u64)> {
        if length < 17 { return None; }
        let (at, end) = match self.direction {
            Direction::Forward => (self.at, self.at.checked_add(u64::from(length))?),
            Direction::Backward => (self.at.checked_sub(u64::from(length))?, self.at),
        };
        if end > self.extent { None } else { Some((at, end)) }
    }
    pub(crate) fn accept(&mut self, bytes: &[u8], limit: u32) -> bool {
        let Some(length) = u32_at(bytes, 0) else { return false; };
        let Some((at, end)) = self.span(length) else { return false; };
        let Some(record) = decode(bytes, limit) else { return false; };
        let length = u32::try_from(record.len()).expect("decoded record within limit");
        if length > self.max && !self.records.is_empty() { self.stopped = true; return true; }
        let record = if length > self.max { self.stopped = true; None } else { Some(record) };
        self.max = self.max.saturating_sub(length);
        self.records.push(LogRecord { at, end, length, record });
        self.at = match self.direction { Direction::Forward => end, Direction::Backward => at };
        true
    }
    pub(crate) fn answer(self) -> Event {
        let next = match self.direction {
            Direction::Forward => if self.at >= self.extent { None } else { Some(self.at) },
            Direction::Backward => if self.at == 0 { None } else { Some(self.at) },
        };
        Event::LogRead { owner: self.owner, records: self.records.into_boxed_slice(), next }
    }
    pub(crate) fn unread(&self) -> Event { Event::LogUnread { owner: self.owner, at: self.at } }
}

pub(crate) fn hex(number: u64) -> Box<[u8]> {
    let mut bytes = [b'0'; 16];
    let mut number = number;
    for byte in bytes.iter_mut().rev() {
        let digit = u8::try_from(number & 15).expect("hex digit fits");
        *byte = *b"0123456789abcdef".get(usize::from(digit)).expect("hex digit");
        number >>= 4_u32;
    }
    Box::from(bytes)
}
pub(crate) fn directory(log: u64) -> Box<[u8]> {
    let mut out = Writer::new(21);
    out.put(b"logs/").expect("sized directory");
    out.put(&hex(log >> 12)).expect("sized directory");
    out.finish()
}
pub(crate) fn path(log: u64) -> Box<[u8]> {
    let mut out = Writer::new(38);
    out.put(&directory(log)).expect("sized path");
    out.put(b"/").expect("sized path");
    out.put(&hex(log)).expect("sized path");
    out.finish()
}
pub(crate) fn number(bytes: &[u8]) -> Option<u64> {
    if bytes.len() != 16 { return None; }
    let mut number = 0_u64;
    for &byte in bytes {
        let digit = match byte { b'0'..=b'9' => byte.checked_sub(b'0')?, b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10)?, _ => return None };
        number = number.checked_mul(16)?.checked_add(u64::from(digit))?;
    }
    Some(number)
}

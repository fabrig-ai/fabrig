//! Progress through a decoded record's parts. A resumed walk skips every
//! finished slot; it never runs a completed pass or decodes the record again.
use super::{Halt, Need, envelope};
use crate::layouts::{Cause, Nested, Peer, Unconverted, Value};
use crate::v22 as wire;
use alloc::boxed::Box;
use skein_codec::{Reason, layout::Mode};
use skein_lib::{Buffer, List, Reader, Secret, bytes, usize_of};

#[derive(Debug)]
pub(super) struct Walked {
    done: u32,
    window: Option<Window>,
    kept: Option<Kept>,
}

impl Walked {
    pub(super) const fn new() -> Self {
        Self { done: 0, window: None, kept: None }
    }
}

#[derive(Debug)]
struct Window {
    offset: u32,
    out: Buffer,
}

#[derive(Debug)]
struct Kept {
    parts: List<wire::KeptPart>,
    next: u32,
    ready: List<wire::KeptPart>,
}

pub(super) struct Walk<'a> {
    peer: &'a mut Peer,
    mode: Mode,
    walked: &'a mut Walked,
    answer: Option<Result<Box<[u8]>, Unconverted>>,
    visited: u32,
}

impl<'a> Walk<'a> {
    pub(super) fn new(
        peer: &'a mut Peer,
        mode: Mode,
        walked: &'a mut Walked,
        answer: Option<Result<Box<[u8]>, Unconverted>>,
    ) -> Self {
        Self { peer, mode, walked, answer, visited: 0 }
    }

    // An empty or unchanged slot still counts: resume at the same field even
    // if a preceding pass changed the size of its value.
    fn enter(&mut self) -> bool {
        self.visited = self.visited.checked_add(1).expect("bounded fields");
        self.visited > self.walked.done
    }

    fn done(&mut self) {
        self.walked.done = self.visited;
    }

    fn one(&mut self, part: Nested, value: &[u8], bound: u32) -> Result<Option<Box<[u8]>>, Halt> {
        if !self.peer.converts_part(self.mode, part) {
            return Ok(None);
        }
        let site = Value::Part(part);
        if self.peer.written_at(self.mode, site).is_some() {
            return match self.answer.take() {
                Some(Ok(bytes)) => Ok(Some(bytes)),
                Some(Err(problem)) => Err(Halt::Bad(problem)),
                None => Err(Halt::Ask(Need { site, bytes: Secret::new(bytes::copy_of(value)), bound })),
            };
        }
        let mut out = Buffer::with_capacity(bound);
        let result = match self.mode {
            Mode::Keep => self.peer.down(part, value, &mut out),
            Mode::Read => self.peer.up(part, value, &mut out),
        };
        match result {
            Ok(()) => Ok(Some(out.to_boxed())),
            Err(problem) => Err(Halt::Bad(problem)),
        }
    }

    pub(super) fn part(&mut self, part: Nested, value: &mut Box<[u8]>, bound: u32) -> Result<(), Halt> {
        if self.enter() {
            if let Some(converted) = self.one(part, value, bound)? {
                *value = converted;
            }
            self.done();
        }
        Ok(())
    }

    pub(super) fn grant(&mut self, grant: &mut wire::Grant, bound: u32) -> Result<(), Halt> {
        if self.enter() {
            match &mut grant.value {
                wire::GrantValue::Application(value) => {
                    let part = Nested::Carried(crate::Carried::Grant);
                    if let Some(converted) = self.one(part, value.secret.expose(), bound)? {
                        value.secret = Secret::new(converted);
                    }
                }
                wire::GrantValue::Credential(_) | wire::GrantValue::Program(_) => {}
            }
            self.done();
        }
        Ok(())
    }

    pub(super) fn grants(&mut self, grants: &mut List<wire::Grant>, bound: u32) -> Result<(), Halt> {
        let skip = self.walked.done.saturating_sub(self.visited).min(grants.len());
        self.visited = self.visited.checked_add(skip).expect("bounded fields");
        for grant in grants.iter_mut().skip(usize_of(skip)) {
            self.grant(grant, bound)?;
        }
        Ok(())
    }

    pub(super) fn window(&mut self, value: &mut Box<[u8]>, bound: u32) -> Result<(), Halt> {
        if !self.enter() {
            return Ok(());
        }
        if self.peer.converts_part(self.mode, Nested::Turn) && !value.is_empty() {
            let mut window = match self.walked.window.take() {
                Some(window) => window,
                None => Window { offset: 0, out: Buffer::with_capacity(bound) },
            };
            let mut reader = Reader::new(value);
            reader.skip(window.offset).expect("resume at a counted turn");
            let length = u32::try_from(value.len()).expect("bounded transcript");
            let turn_bound = bound.saturating_sub(4);
            // Reuse the turn buffer across a derived window's turns.
            let written = self.peer.written_at(self.mode, Value::Part(Nested::Turn)).is_some();
            let mut turn = Buffer::with_capacity(if written { 0 } else { turn_bound });
            for _ in 0..=reader.remaining().saturating_div(4) {
                if reader.is_empty() {
                    break;
                }
                let Some(size) = reader.u32() else { return Err(bad_envelope("transcript", Cause::Malformed)) };
                let Some(body) = reader.bytes(size) else { return Err(bad_envelope("transcript", Cause::Malformed)) };
                if written {
                    match self.one(Nested::Turn, body, turn_bound) {
                        Ok(Some(bytes)) => put_turn(&mut window.out, &bytes)?,
                        Ok(None) => unreachable!("the turn has a written pass"),
                        Err(halt) => {
                            self.walked.window = Some(window);
                            return Err(halt);
                        }
                    }
                } else {
                    if let Err(problem) = self.peer.down(Nested::Turn, body, &mut turn) {
                        return Err(Halt::Bad(problem));
                    }
                    put_turn(&mut window.out, turn.as_slice())?;
                }
                window.offset = length.checked_sub(reader.remaining()).expect("inside bounded transcript");
            }
            *value = window.out.to_boxed();
        }
        self.done();
        Ok(())
    }

    pub(super) fn kept(&mut self, value: &mut Box<[u8]>, limits: &wire::Limits) -> Result<(), Halt> {
        if !self.enter() {
            return Ok(());
        }
        if self.peer.converts_part(self.mode, Nested::Turn) && !value.is_empty() {
            let mut pending = match self.walked.kept.take() {
                Some(kept) => kept,
                None => {
                    let Ok(parts) = crate::parts::decode(value, limits) else {
                        return Err(bad_envelope("parts", Cause::Malformed));
                    };
                    Kept { next: 0, ready: List::with_capacity(parts.len()), parts }
                }
            };
            let bound = limits.kept_part_body.min(wire::CEILINGS.kept_part_body);
            let written = self.peer.written_at(self.mode, Value::Part(Nested::Turn)).is_some();
            let mut out = Buffer::with_capacity(if written { 0 } else { bound });
            for index in pending.next..pending.parts.len() {
                let item = pending.parts.get_mut(index).expect("counted kept parts");
                let result = if written {
                    self.one(Nested::Turn, &item.body, bound)
                } else {
                    match self.peer.down(Nested::Turn, &item.body, &mut out) {
                        Ok(()) => Ok(Some(out.to_boxed())),
                        Err(problem) => Err(Halt::Bad(problem)),
                    }
                };
                match result {
                    Ok(Some(bytes)) => {
                        item.body = Box::new([]);
                        let kept = wire::KeptPart { body: bytes, ..item.clone() };
                        pending.ready.push(kept).expect("kept parts can only shrink in number");
                    }
                    Ok(None) => unreachable!("the part has a pass"),
                    Err(Halt::Bad(_)) => {} // A kept part's refusal drops only that part.
                    Err(halt @ Halt::Ask(_)) => {
                        pending.next = index;
                        self.walked.kept = Some(pending);
                        return Err(halt);
                    }
                }
            }
            *value = match crate::parts::encode(pending.ready.as_slice(), limits) {
                Ok(bytes) => bytes,
                Err(problem) => {
                    let cause = if problem.reason == Reason::Bound { Cause::Large } else { Cause::Malformed };
                    return Err(bad_envelope("parts", cause));
                }
            };
        }
        self.done();
        Ok(())
    }
}

fn put_turn(out: &mut Buffer, bytes: &[u8]) -> Result<(), Halt> {
    let size = u32::try_from(bytes.len()).expect("turn conversion is bounded");
    if out.put(&size.to_be_bytes()).is_err() || out.put(bytes).is_err() {
        return Err(bad_envelope("transcript", Cause::Large));
    }
    Ok(())
}

fn bad_envelope(field: &str, cause: Cause) -> Halt {
    Halt::Bad(envelope("Assignment", field, cause))
}

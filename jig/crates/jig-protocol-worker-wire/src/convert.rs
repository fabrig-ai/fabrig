//! Whole frames between the engine's and worker's layouts (jig/docs/design/protocol/channel/layouts.md, 4 and 6).
use crate::layouts::{Cause, Peer, Sides, Unconverted, Value};
use crate::{KINDS, Kind, sizes, v22};
use alloc::boxed::Box;
use skein_channel::{Direction, Frame, frame_writer};
use skein_codec::Reason;
use skein_codec::layout::{Misconverted, Mode, Refusal, Side};
use skein_lib::{Buffer, Secret, usize_of};
use v22::Record;
use walk::{Walk, Walked};

mod down;
mod up;
mod walk;

/// A value lent to its owner, whose converter writes at most `bound` bytes.
pub struct Converting<'a> {
    pub wire: &'a str,
    pub mode: Mode,
    pub from: Side<'a>,
    pub to: Side<'a>,
    pub value: &'a [u8],
    pub bound: u32,
}

impl core::fmt::Debug for Converting<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Converting")
            .field("wire", &self.wire)
            .field("mode", &self.mode)
            .field("bound", &self.bound)
            .finish()
    }
}

#[derive(Debug)]
pub enum Advance {
    Frame(Frame),
    Refused(Unconverted),
    Waiting(Box<Waiting>),
}

/// The rest of a frame, suspended at one written pass.
pub struct Waiting {
    resume: Resume,
    need: Need,
}

#[expect(clippy::missing_fields_in_debug, reason = "suspended frames can carry secrets")]
impl core::fmt::Debug for Waiting {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Waiting").field("mode", &self.mode()).field("site", &self.need.site).finish()
    }
}

enum Resume {
    Keep { kind: Kind, body: down::Body, walked: Walked },
    Read { source: u16, kind: Kind, record: v22::FromWorker, walked: Walked },
    KeepFrame { source: u16, target: u16 },
    ReadFrame { source: u16, kind: Kind },
}

struct Need {
    site: Value,
    bytes: Secret,
    bound: u32,
}

enum Halt {
    Ask(Need),
    Bad(Unconverted),
}

impl Waiting {
    fn mode(&self) -> Mode {
        match self.resume {
            Resume::Keep { .. } | Resume::KeepFrame { .. } => Mode::Keep,
            Resume::Read { .. } | Resume::ReadFrame { .. } => Mode::Read,
        }
    }

    #[must_use]
    pub fn converting<'a>(&'a self, peer: &'a Peer) -> Converting<'a> {
        let mode = self.mode();
        let Sides { wire, from, to } = peer.written_at(mode, self.need.site).expect("waiting for a written pass");
        Converting { wire, mode, from, to, value: self.need.bytes.expose(), bound: self.need.bound }
    }

    pub fn answered(self, peer: &mut Peer, answer: Result<Box<[u8]>, Refusal>, limits: &v22::Limits) -> Advance {
        let mode = self.mode();
        let answer = match answer {
            Ok(bytes) if bytes.len() <= usize_of(self.need.bound) => Ok(bytes),
            Ok(_) => Err(peer.written_problem(mode, self.need.site, Cause::Large)),
            Err(refusal) => Err(peer.written_refusal(mode, self.need.site, refusal)),
        };
        match self.resume {
            Resume::Keep { kind, body, walked } => keep_parts(peer, kind, body, walked, Some(answer), limits),
            Resume::Read { source, kind, record, walked } => {
                read_parts(peer, source, kind, record, walked, Some(answer), limits, None)
            }
            Resume::KeepFrame { source, target } => match answer {
                Ok(bytes) => match written_body(peer, mode, source, target, &bytes) {
                    Ok(body) => frame_result(frame_from(peer, mode, source, target, body)),
                    Err(problem) => Advance::Refused(problem),
                },
                Err(problem) => Advance::Refused(problem),
            },
            Resume::ReadFrame { source, kind } => match answer {
                Ok(bytes) => match written_body(peer, mode, source, kind.number(), &bytes) {
                    Ok(body) => read_record(peer, source, kind, body, limits),
                    Err(problem) => Advance::Refused(problem),
                },
                Err(problem) => Advance::Refused(problem),
            },
        }
    }

    fn required(&self, peer: &Peer) -> Unconverted {
        peer.written_problem(self.mode(), self.need.site, Cause::WrittenRequired)
    }
}

/// Synchronous conversion for callers that have no written converter route.
pub fn send(peer: &mut Peer, frame: Frame, limits: &v22::Limits) -> Result<Frame, Unconverted> {
    let result = start_send(peer, frame, limits);
    immediate(peer, result)
}

/// Synchronous conversion for callers that have no written converter route.
pub fn receive(peer: &mut Peer, kind: u16, body: &[u8], limits: &v22::Limits) -> Result<Frame, Unconverted> {
    let result = start_receive(peer, kind, body, limits);
    immediate(peer, result)
}

fn immediate(peer: &Peer, result: Advance) -> Result<Frame, Unconverted> {
    match result {
        Advance::Frame(frame) => Ok(frame),
        Advance::Refused(problem) => Err(problem),
        Advance::Waiting(waiting) => Err(waiting.required(peer)),
    }
}

/// Keeps the parts, then the frame. A refused kept part is dropped; a refused window turn is not.
pub fn start_send(peer: &mut Peer, frame: Frame, limits: &v22::Limits) -> Advance {
    if peer.same() {
        return Advance::Frame(frame);
    }
    let number = frame.kind();
    let Some(kind) = outgoing(number) else {
        return Advance::Refused(peer.frame_problem(Mode::Keep, number, Misconverted::Tag));
    };
    let bytes = frame.bytes().get(8..).expect("a frame owns its header");
    match down::Body::decode(peer, kind, bytes, limits) {
        Ok(Some(body)) => keep_parts(peer, kind, body, Walked::new(), None, limits),
        Ok(None) => keep_frame(peer, kind, bytes, limits),
        Err(problem) => Advance::Refused(problem),
    }
}

fn keep_parts(
    peer: &mut Peer,
    kind: Kind,
    mut body: down::Body,
    mut walked: Walked,
    answer: Option<Result<Box<[u8]>, Unconverted>>,
    limits: &v22::Limits,
) -> Advance {
    let mut walk = Walk::new(peer, Mode::Keep, &mut walked, answer);
    match body.parts(&mut walk, limits) {
        Ok(()) => match body.encode(limits) {
            Ok(bytes) => keep_frame(peer, kind, &bytes, limits),
            Err(error) => Advance::Refused(codec_error(peer, Mode::Keep, kind.number(), error)),
        },
        Err(Halt::Ask(need)) => waiting(Resume::Keep { kind, body, walked }, need),
        Err(Halt::Bad(problem)) => Advance::Refused(problem),
    }
}

fn keep_frame(peer: &mut Peer, kind: Kind, body: &[u8], limits: &v22::Limits) -> Advance {
    let source = kind.number();
    let room = match peer.send_room(kind, limits) {
        Ok(room) => room,
        Err(problem) => return Advance::Refused(problem),
    };
    if peer.written_at(Mode::Keep, Value::Frame).is_some() {
        let target = peer.sends(kind).expect("a kind with room at the worker's layout");
        return tagged_wait(peer, Mode::Keep, source, body, room, Resume::KeepFrame { source, target });
    }
    let mut out = Buffer::with_capacity(room);
    match peer.send(kind, body, &mut out) {
        Ok(target) => frame_result(frame_from(peer, Mode::Keep, source, target, out.as_slice())),
        Err(problem) => Advance::Refused(problem),
    }
}

/// Reads the frame, then the parts it carries.
pub fn start_receive(peer: &mut Peer, source: u16, body: &[u8], limits: &v22::Limits) -> Advance {
    if peer.same() {
        return frame_result(frame_from(peer, Mode::Read, source, source, body));
    }
    let kind = match peer.reads(source) {
        Ok(kind) => kind,
        Err(problem) => return Advance::Refused(problem),
    };
    let Some(room) = sizes::body(limits, kind) else {
        return Advance::Refused(peer.frame_problem(Mode::Read, source, Misconverted::Room));
    };
    if peer.written_at(Mode::Read, Value::Frame).is_some() {
        return tagged_wait(peer, Mode::Read, source, body, room, Resume::ReadFrame { source, kind });
    }
    let mut out = Buffer::with_capacity(room);
    match peer.receive(source, body, &mut out) {
        Ok(kind) => read_record(peer, source, kind, out.as_slice(), limits),
        Err(problem) => Advance::Refused(problem),
    }
}

fn read_record(peer: &mut Peer, source: u16, kind: Kind, bytes: &[u8], limits: &v22::Limits) -> Advance {
    match up::decode(peer, source, kind, bytes, limits) {
        Ok(record) => read_parts(peer, source, kind, record, Walked::new(), None, limits, Some(bytes)),
        Err(problem) => Advance::Refused(problem),
    }
}

#[expect(clippy::too_many_arguments, reason = "the original body is only borrowed on the initial pass")]
fn read_parts(
    peer: &mut Peer,
    source: u16,
    kind: Kind,
    mut record: v22::FromWorker,
    mut walked: Walked,
    answer: Option<Result<Box<[u8]>, Unconverted>>,
    limits: &v22::Limits,
    original: Option<&[u8]>,
) -> Advance {
    let mut walk = Walk::new(peer, Mode::Read, &mut walked, answer);
    let encoded = match up::parts(&mut walk, &mut record, limits) {
        Ok(false) => {
            return frame_result(frame_from(
                peer,
                Mode::Read,
                source,
                kind.number(),
                original.expect("a body without parts never waits"),
            ));
        }
        Ok(true) => match record.encode(limits) {
            Ok(bytes) => bytes,
            Err(error) => return Advance::Refused(codec_error(peer, Mode::Read, source, error)),
        },
        Err(Halt::Ask(need)) => return waiting(Resume::Read { source, kind, record, walked }, need),
        Err(Halt::Bad(problem)) => return Advance::Refused(problem),
    };
    frame_result(frame_from(
        peer,
        Mode::Read,
        source,
        kind.number(),
        encoded.get(1..).expect("the codec writes its tag"),
    ))
}

fn waiting(resume: Resume, need: Need) -> Advance {
    Advance::Waiting(Box::new(Waiting { resume, need }))
}

fn frame_result(result: Result<Frame, Unconverted>) -> Advance {
    match result {
        Ok(frame) => Advance::Frame(frame),
        Err(problem) => Advance::Refused(problem),
    }
}

fn tagged_wait(peer: &Peer, mode: Mode, kind: u16, body: &[u8], room: u32, resume: Resume) -> Advance {
    let Some(tag) = tag(mode, kind) else {
        return Advance::Refused(peer.frame_problem(mode, kind, Misconverted::Tag));
    };
    let Some(bound) = room.checked_add(1) else {
        return Advance::Refused(peer.frame_problem(mode, kind, Misconverted::Room));
    };
    let size = body.len().checked_add(1).expect("the body and its tag fit");
    let mut bytes = skein_lib::Writer::new(size);
    bytes.put(&[tag]).expect("room for the tag");
    bytes.put(body).expect("room for the body");
    waiting(resume, Need { site: Value::Frame, bytes: Secret::new(bytes.finish()), bound })
}

fn written_body<'a>(
    peer: &Peer,
    mode: Mode,
    source: u16,
    target: u16,
    bytes: &'a [u8],
) -> Result<&'a [u8], Unconverted> {
    if let Some((found, body)) = bytes.split_first()
        && Some(*found) == tag(mode, target)
    {
        return Ok(body);
    }
    Err(peer.frame_problem(mode, source, Misconverted::Tag))
}

fn tag(mode: Mode, kind: u16) -> Option<u8> {
    let base = match mode {
        Mode::Read => 0x0100,
        Mode::Keep => 0x0180,
    };
    u8::try_from(kind.checked_sub(base)?).ok()
}

/// The engine kind corresponding to an admitted worker kind, for diagnostics.
#[must_use]
pub fn received_kind(peer: &Peer, kind: u16) -> Option<Kind> {
    peer.reads(kind).ok()
}

fn outgoing(number: u16) -> Option<Kind> {
    for rule in KINDS {
        if rule.kind == number && rule.direction == Direction::FromResponder {
            return Some(rule.body);
        }
    }
    None
}

fn frame_from(peer: &Peer, mode: Mode, source: u16, kind: u16, body: &[u8]) -> Result<Frame, Unconverted> {
    let Ok(room) = u32::try_from(body.len()) else {
        return Err(peer.frame_problem(mode, source, Misconverted::Room));
    };
    let Ok(mut writer) = frame_writer(kind, room) else {
        return Err(peer.frame_problem(mode, source, Misconverted::Room));
    };
    writer.put(body).expect("the body length was measured");
    Ok(writer.finish().expect("the measured body was written"))
}

fn codec_error(peer: &Peer, mode: Mode, source: u16, problem: v22::Problem) -> Unconverted {
    let error =
        if mode == Mode::Read && problem.reason == Reason::Bound { Misconverted::Room } else { Misconverted::Tag };
    peer.frame_problem(mode, source, error)
}

fn envelope(declaration: &str, field: &str, cause: Cause) -> Unconverted {
    Unconverted {
        wire: Box::from(crate::wire::FAMILY),
        declaration: Box::from(declaration),
        part: Box::from(field),
        cause,
    }
}

//! Whole frames between the engine's and worker's layouts (jig/docs/design/protocol/channel/contracts.md, 2).
use crate::layouts::{Cause, Nested, Peer, Unconverted};
use crate::{KINDS, Kind, sizes, v22};
use alloc::boxed::Box;
use skein_channel::{Direction, Frame, frame_writer};
use skein_codec::Reason;
use skein_codec::layout::{Misconverted, Mode};
use skein_lib::Buffer;

mod down;
mod up;

/// Keeps the carried parts, then the record, at the worker's layout.
/// A refused body of a kept part is dropped; a refused window turn is not.
pub fn send(peer: &mut Peer, frame: Frame, limits: &v22::Limits) -> Result<Frame, Unconverted> {
    if peer.same() {
        return Ok(frame);
    }
    let number = frame.kind();
    let Some(kind) = outgoing(number) else {
        return Err(peer.frame_problem(Mode::Keep, number, Misconverted::Tag));
    };
    let body = frame.bytes().get(8..).expect("a frame owns its header");
    let converted = down::parts(peer, kind, body, limits)?;
    let body = match &converted {
        Some(bytes) => bytes.as_ref(),
        None => body,
    };
    let room = peer.send_room(kind, limits)?;
    let mut out = Buffer::with_capacity(room);
    let worker_kind = peer.send(kind, body, &mut out)?;
    frame_from(peer, Mode::Keep, number, worker_kind, out.as_slice())
}

/// Reads the record into the engine's layout, then the parts it carries.
/// With no layout difference, only the returned frame's ownership costs an allocation.
pub fn receive(peer: &mut Peer, kind: u16, body: &[u8], limits: &v22::Limits) -> Result<Frame, Unconverted> {
    if peer.same() {
        return frame_from(peer, Mode::Read, kind, kind, body);
    }
    let engine_kind = peer.reads(kind)?;
    let Some(room) = sizes::body(limits, engine_kind) else {
        return Err(peer.frame_problem(Mode::Read, kind, Misconverted::Room));
    };
    let mut out = Buffer::with_capacity(room);
    let engine_kind = peer.receive(kind, body, &mut out)?;
    up::frame(peer, kind, engine_kind, out.as_slice(), limits)
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

fn part(peer: &mut Peer, mode: Mode, nested: Nested, bytes: &mut Box<[u8]>, bound: u32) -> Result<(), Unconverted> {
    if let Some(converted) = convert_part(peer, mode, nested, bytes, bound)? {
        *bytes = converted;
    }
    Ok(())
}

fn convert_part(
    peer: &mut Peer,
    mode: Mode,
    nested: Nested,
    bytes: &[u8],
    bound: u32,
) -> Result<Option<Box<[u8]>>, Unconverted> {
    if !peer.converts_part(mode, nested) {
        return Ok(None);
    }
    let mut out = Buffer::with_capacity(bound);
    match mode {
        Mode::Keep => peer.down(nested, bytes, &mut out)?,
        Mode::Read => peer.up(nested, bytes, &mut out)?,
    }
    Ok(Some(out.to_boxed()))
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

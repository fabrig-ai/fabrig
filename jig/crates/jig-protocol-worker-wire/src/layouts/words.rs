//! Opening refusals (jig/docs/design/protocol/channel/wires.md, 3).
use super::Cause;
use alloc::boxed::Box;
use skein_codec::layout::{Mode, Why};
use skein_lib::{Writer, bytes};

const REFUSAL: usize = 256;

pub(super) fn wire(name: &str, rest: &[&[u8]]) -> Box<[u8]> {
    join_after(&[b"wire ", name.as_bytes()], rest, REFUSAL)
}

pub(super) fn no_plan(name: &str, value: &str, mode: Mode, why: Why, decl: &[u8], part: Option<&[u8]>) -> Box<[u8]> {
    let direction = match mode {
        Mode::Read => b"from the worker".as_slice(),
        Mode::Keep => b"to the worker",
    };
    let why = match why {
        Why::Root => b"no such root".as_slice(),
        Why::NoDefault => b"no default",
        Why::Retyped => b"retyped",
        Why::Reshaped => b"reshaped",
        Why::VariantDropped => b"variant dropped",
        Why::Payload => b"payload changed",
        Why::Held => b"held on one side",
        Why::Pieces => b"pieces",
        Why::Deep => b"too deep",
    };
    let (dot, part) = match part {
        Some(part) => (b".".as_slice(), part),
        None => (b"".as_slice(), b"".as_slice()),
    };
    wire(name, &[b" ", value.as_bytes(), b": no plan ", direction, b": ", why, b" at ", decl, dot, part])
}

pub(super) fn hex(layout: &[u8; 32]) -> Box<[u8]> {
    let mut out = Writer::new(16);
    for byte in layout.iter().take(8) {
        out.put(&[bytes::hex_digit(byte >> 4), bytes::hex_digit(byte & 15)]).expect("eight hex pairs");
    }
    out.finish()
}

pub(super) fn kept(wire: &str, declaration: &str, part: &str, cause: Cause) -> Box<[u8]> {
    value(b"the worker's layout cannot keep ", wire, declaration, part, cause)
}

pub(super) fn value(prefix: &[u8], wire: &str, declaration: &str, part: &str, cause: Cause) -> Box<[u8]> {
    let dot = if part.is_empty() { b"".as_slice() } else { b"." };
    join(
        &[prefix, wire.as_bytes(), b" ", declaration.as_bytes(), dot, part.as_bytes(), b": ", cause.words()],
        usize::MAX,
    )
}

pub(super) fn join(parts: &[&[u8]], cap: usize) -> Box<[u8]> {
    join_after(&[], parts, cap)
}

fn join_after(prefix: &[&[u8]], parts: &[&[u8]], cap: usize) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in prefix.iter().chain(parts) {
        length = length.saturating_add(part.len()).min(cap);
    }
    let mut out = Writer::new(length);
    let mut left = length;
    for part in prefix.iter().chain(parts) {
        let size = left.min(part.len());
        out.put(part.get(..size).expect("no more than the part")).expect("measured diagnostic");
        left = left.checked_sub(size).expect("no more than the remaining room");
    }
    out.finish()
}

//! Wire agreement before the domain hears anything (protocol/channel/wires.md, 3).
use crate::Stated;
use alloc::boxed::Box;
use jig_protocol_worker_wire::{Agreed, Spoken};
use skein_lib::{Decimal, List, Writer, bytes};

fn highest(wire: &Stated) -> Option<u16> {
    wire.lowest.checked_add(u16::try_from(wire.layouts.len().checked_sub(1)?).ok()?)
}

pub(crate) fn agree(engine: &List<Spoken>, worker: &List<Stated>) -> Result<List<Agreed>, Box<[u8]>> {
    for (index, wire) in worker.iter().enumerate() {
        for earlier in worker.iter().take(index) {
            if wire.name.as_bytes() == earlier.name.as_bytes() {
                return Err(words(&[b"wire ", wire.name.as_bytes(), b": stated twice"]));
            }
        }
        if highest(wire).is_none() {
            return Err(words(&[b"wire ", wire.name.as_bytes(), b": no such range"]));
        }
    }
    for wire in engine {
        let mut stated = None;
        for candidate in worker {
            if candidate.name.as_bytes() == wire.name.as_bytes() {
                stated = Some(candidate);
            }
        }
        let Some(stated) = stated else {
            return Err(words(&[b"wire ", wire.name.as_bytes(), b": the worker states none"]));
        };
        let low = wire.lowest.max(stated.lowest);
        let high = wire.highest().min(highest(stated).expect("validated range"));
        if low > high {
            return Err(words(&[
                b"wire ",
                wire.name.as_bytes(),
                b": the engine speaks ",
                Decimal::of(u64::from(wire.lowest)).as_bytes(),
                b"..",
                Decimal::of(u64::from(wire.highest())).as_bytes(),
                b", the worker ",
                Decimal::of(u64::from(stated.lowest)).as_bytes(),
                b"..",
                Decimal::of(u64::from(highest(stated).expect("validated range"))).as_bytes(),
            ]));
        }
        for version in low..=high {
            let ours = wire.layout(version).expect("common version");
            let theirs = *stated
                .layouts
                .get(u32::from(version.checked_sub(stated.lowest).expect("common version")))
                .expect("common layout");
            if ours != theirs {
                return Err(words(&[
                    b"wire ",
                    wire.name.as_bytes(),
                    b" version ",
                    Decimal::of(u64::from(version)).as_bytes(),
                    b": the engine's layout ",
                    &hex(&ours),
                    b", the worker's ",
                    &hex(&theirs),
                ]));
            }
        }
    }
    let mut agreed = List::with_capacity(engine.len());
    for stated in worker {
        for wire in engine {
            if stated.name.as_bytes() == wire.name.as_bytes() {
                agreed
                    .push(Agreed {
                        name: wire.name,
                        version: wire.highest().min(highest(stated).expect("validated range")),
                    })
                    .expect("one agreement per engine wire");
            }
        }
    }
    Ok(agreed)
}

fn hex(layout: &[u8; 32]) -> Box<[u8]> {
    let mut out = Writer::new(16);
    for byte in layout.iter().take(8) {
        out.put(&[bytes::hex_digit(byte >> 4), bytes::hex_digit(byte & 15)]).expect("eight hex pairs");
    }
    out.finish()
}

fn words(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded diagnostic");
    }
    let mut words = Writer::new(length);
    for part in parts {
        words.put(part).expect("measured diagnostic");
    }
    words.finish()
}

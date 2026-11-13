//! The host tools' contract and its fingerprint (protocol/wires.md, 3.1 and 6).

use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits};
use skein_lib::{Map, Writer, sha256};
use smith_domain_run::HostEffect;

use crate::calls::{SPECS, Spec};

/// The engine's host-tool family.
pub const FAMILY: &str = "temper-host-tools";
/// The current host-tool contract's version.
pub const VERSION: u16 = 1;

/// SHA-256 of the rendered contract, including every declaration in `SPECS`.
#[must_use]
pub fn layout() -> [u8; 32] {
    sha256::digest(&render(&SPECS))
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Order {
    name: Box<[u8]>,
    position: usize,
}

/// One line per declaration, ordered by name with ties in declaration order.
pub(crate) fn render(specs: &[Spec]) -> Box<[u8]> {
    let mut lines = Map::with_capacity(u32::try_from(specs.len()).expect("fixed declaration count"));
    let mut length = 0_usize;
    let limits = Limits { depth: 1, length: u32::MAX };
    for (position, spec) in specs.iter().enumerate() {
        let mut measure = Encoder::measure(&limits);
        declaration(spec, &mut measure);
        let len = measure.measured().expect("declared names and schemas are UTF-8 and fit u32");
        let mut write = Encoder::write(len, &limits);
        declaration(spec, &mut write);
        let line = write.finish();
        length = length.checked_add(line.len()).expect("fixed contract fits usize");
        length = length.checked_add(1).expect("fixed contract's newline fits usize");
        lines.insert(Order { name: spec.name.into(), position }, line).expect("one slot per declaration");
    }
    let mut out = Writer::new(length);
    for (_, line) in &lines {
        out.put(line).expect("measured contract length");
        out.put(b"\n").expect("measured line ending");
    }
    out.finish()
}

fn declaration(spec: &Spec, out: &mut Encoder) {
    out.object_start();
    out.key(b"name");
    out.string(spec.name);
    out.key(b"effect");
    out.string(match spec.effect {
        HostEffect::Read => b"read",
        HostEffect::Write => b"write",
    });
    out.key(b"input");
    out.string(spec.schema);
    out.object_end();
}

//! The component's own rows that carry its values (protocol/store/records.md, 3).
pub(crate) mod openings;

use crate::component_v3 as wire;
use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};

/// Encode the shell's validated start wires (protocol/store/records.md 3).
#[must_use]
pub(crate) fn wires(wires: &[crate::Wire]) -> Box<[u8]> {
    let mut values = List::with_capacity(wire::CEILINGS.wires_wires);
    for value in wires {
        let value = wire::Wire::new(
            &wire::CEILINGS,
            wire::WireParts { name: Box::from(value.name.as_bytes()), version: value.version },
        )
        .expect("Store::wires validated each name");
        values.push(value).expect("Store::wires validated the count");
    }
    let row = wire::Wires::new(&wire::CEILINGS, wire::WiresParts { wires: values }).expect("validated wires");
    let mut writer = Writer::new(skein_lib::length::usize_of(row.measure()));
    row.encode(&mut writer).expect("measured wires row");
    writer.finish()
}

/// The floor row written at creation or by the seal (protocol/store/records.md, 3).
pub(crate) fn floor(build: &str) -> Box<[u8]> {
    let row = wire::Floor::new(
        &wire::CEILINGS,
        wire::FloorParts { number: crate::FLOOR, build: Box::from(build.as_bytes()) },
    )
    .expect("Store::build_name bounded the name");
    let mut writer = Writer::new(skein_lib::length::usize_of(row.measure()));
    row.encode(&mut writer).expect("measured floor row");
    writer.finish()
}

/// Decode the last start's optional wires for rehearsal (protocol/store/opening.md 5).
#[expect(clippy::disallowed_methods, reason = "the protocol codec validates stored wire names as UTF-8")]
pub(crate) fn kept_wires(bytes: &[u8]) -> Option<Box<[crate::KeptWire]>> {
    let row = crate::component_v3::Wires::decode(&crate::component_v3::CEILINGS, &mut Reader::new(bytes)).ok()?;
    let mut wires = List::with_capacity(crate::component_v3::CEILINGS.wires_wires);
    for wire in row.wires() {
        let name = core::str::from_utf8(wire.name()).ok()?;
        wires
            .push(crate::KeptWire { name: Box::from(name), version: wire.version() })
            .expect("decoded wire count fits");
    }
    Some(wires.into_boxed())
}

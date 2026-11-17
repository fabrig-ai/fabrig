//! Workers' authenticated openings (protocol/store/records.md 3 and protocol/store/opening.md 5).
use crate::{OpenedTerm, OpenedWire, WorkerOpening, openings_v1 as wire};
use alloc::boxed::Box;
use skein_lib::{List, Reader, Wall, Writer};

pub(crate) fn encode(opening: WorkerOpening) -> Option<Box<[u8]>> {
    let limits = &wire::CEILINGS;
    let mut wires = List::with_capacity(u32::try_from(opening.wires.len()).ok()?);
    for value in opening.wires {
        let mut layouts = List::with_capacity(u32::try_from(value.layouts.len()).ok()?);
        for layout in value.layouts {
            layouts.push(layout).ok()?;
        }
        wires
            .push(
                wire::OpenedWire::new(
                    limits,
                    wire::OpenedWireParts { name: Box::from(value.name.as_bytes()), lowest: value.lowest, layouts },
                )
                .ok()?,
            )
            .ok()?;
    }
    let mut terms = List::with_capacity(u32::try_from(opening.terms.len()).ok()?);
    for value in opening.terms {
        terms
            .push(
                wire::OpenedTerm::new(limits, wire::OpenedTermParts { kind: value.kind, largest: value.largest })
                    .ok()?,
            )
            .ok()?;
    }
    let row = wire::WorkerOpening::new(
        limits,
        wire::WorkerOpeningParts {
            worker: Box::from(opening.worker.as_bytes()),
            wires,
            output: opening.output,
            terms,
            at: opening.at.as_nanos(),
        },
    )
    .ok()?;
    let mut writer = Writer::new(skein_lib::length::usize_of(row.measure()));
    row.encode(&mut writer).expect("measured opening row");
    Some(writer.finish())
}

#[expect(clippy::disallowed_methods, reason = "the protocol codec validates stored opening names as UTF-8")]
pub(crate) fn decode(bytes: &[u8]) -> Option<WorkerOpening> {
    let row = wire::WorkerOpening::decode(&wire::CEILINGS, &mut Reader::new(bytes)).ok()?.into_parts();
    let mut wires = List::with_capacity(row.wires.len());
    for value in row.wires.into_boxed() {
        let value = value.into_parts();
        wires
            .push(OpenedWire {
                name: Box::from(core::str::from_utf8(&value.name).ok()?),
                lowest: value.lowest,
                layouts: value.layouts.into_boxed(),
            })
            .ok()?;
    }
    let mut terms = List::with_capacity(row.terms.len());
    for value in row.terms.into_boxed() {
        terms.push(OpenedTerm { kind: value.kind(), largest: value.largest() }).ok()?;
    }
    Some(WorkerOpening {
        worker: Box::from(core::str::from_utf8(&row.worker).ok()?),
        wires: wires.into_boxed(),
        output: row.output,
        terms: terms.into_boxed(),
        at: Wall::from_nanos(row.at),
    })
}

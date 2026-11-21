//! Workers' authenticated openings (protocol/store/records.md 3 and protocol/store/opening.md 5).
use crate::{WorkerOpening, generated::openings as maps, openings_v1 as wire};
use alloc::boxed::Box;
use skein_codec::Record as _;
use skein_lib::Reader;

pub(crate) fn encode(opening: WorkerOpening) -> Option<Box<[u8]>> {
    maps::worker_opening_to_record(&opening, &()).ok()?.encode(&wire::CEILINGS).ok()
}

pub(crate) fn decode(bytes: &[u8]) -> Option<WorkerOpening> {
    let row = wire::WorkerOpening::decode(&wire::CEILINGS, &mut Reader::new(bytes)).ok()?;
    maps::worker_opening_to_domain(row, &()).ok()
}

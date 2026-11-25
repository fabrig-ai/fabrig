//! Workers' authenticated openings (protocol/store/records.md 3 and protocol/store/opening-rehearsal.md 1).
use crate::{WorkerOpening, generated::openings as maps, openings_v2 as wire};
use alloc::boxed::Box;
use skein_codec::Record as _;
use skein_lib::Reader;

pub(crate) fn encode(opening: WorkerOpening) -> Option<Box<[u8]>> {
    maps::worker_opening_to_record(&opening).ok()?.encode(&wire::CEILINGS).ok()
}

pub(crate) fn decode(bytes: &[u8]) -> Option<WorkerOpening> {
    let row = wire::WorkerOpening::decode(&wire::CEILINGS, &mut Reader::new(bytes)).ok()?;
    maps::worker_opening_to_domain(row).ok()
}

/// Version 1 is recognized by its head alone; its body has no layout text.
pub(crate) fn before_layouts(bytes: &[u8]) -> bool {
    crate::registry::head(bytes) == Some(1)
}

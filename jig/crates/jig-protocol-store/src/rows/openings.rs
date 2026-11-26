//! Workers' authenticated openings (protocol/store/records.md 3 and protocol/store/opening-rehearsal.md 1).
use crate::{WorkerOpening, generated::openings as maps, openings_v3 as wire};
use alloc::boxed::Box;
use jig_domain_link::JobKind;
use skein_codec::Record as _;
use skein_lib::Reader;

pub(crate) fn encode(opening: WorkerOpening) -> Option<Box<[u8]>> {
    maps::worker_opening_to_record(&opening).ok()?.encode(&wire::CEILINGS).ok()
}

pub(crate) fn decode(bytes: &[u8]) -> Option<WorkerOpening> {
    let row = wire::WorkerOpening::decode(&wire::CEILINGS, &mut Reader::new(bytes)).ok()?;
    maps::worker_opening_to_domain(row).ok()
}

/// Earlier openings are recognized by their heads alone.
pub(crate) enum Earlier {
    BeforeLayouts,
    BeforeJobs,
}

pub(crate) fn earlier(bytes: &[u8]) -> Option<Earlier> {
    match crate::registry::head(bytes) {
        Some(1) => Some(Earlier::BeforeLayouts),
        Some(2) => Some(Earlier::BeforeJobs),
        Some(_) | None => None,
    }
}

#[expect(clippy::trivially_copy_pass_by_ref, reason = "the generated caller borrows the domain field")]
pub(crate) fn job_kind_to_record(kind: &JobKind) -> u32 {
    kind.0
}

pub(crate) fn job_kind_to_domain(kind: u32) -> JobKind {
    JobKind(kind)
}

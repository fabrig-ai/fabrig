//! Notes rows and their bounded resource patterns (protocol/store/records.md, 1).
use crate::generated::notes as maps;
use crate::notes_v1::{self as wire, Record};
use alloc::boxed::Box;
use jig_domain_notes as domain;
use skein_lib::{List, Reader};

/// Translate the notes child's bounds, refusing a schema ceiling.
#[must_use]
pub fn limits(domain: &domain::Limits) -> Option<wire::Limits> {
    let limits = wire::Limits {
        segment_bytes: domain.pattern_bytes,
        pattern_segments: domain.pattern_bytes,
        entry_description: domain.description_bytes,
        line_description: domain.description_bytes,
        entry_body: domain.body_bytes,
        entry_references: domain.references,
    };
    wire::check_limits(&limits).ok()?;
    Some(limits)
}

/// Encode one versioned notes row within the configured bounds.
pub fn encode(row: &domain::Record, limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    wire::Row { value: maps::value_to_record(row, limits)? }.encode(limits)
}

/// Decode one notes row, refusing another version or trailing bytes.
pub fn decode(bytes: &[u8], limits: &wire::Limits) -> Result<domain::Record, wire::Problem> {
    let wire::Row { value } = wire::Row::decode(limits, &mut Reader::new(bytes))?;
    maps::value_to_domain(value, limits)
}

/// Patterns share one aggregate byte bound across their segments and tail.
pub(crate) fn pattern_to_record(
    value: &domain::Pattern,
    limits: &wire::Limits,
) -> Result<wire::Pattern, wire::Problem> {
    let domain::Pattern { segments, last } = value;
    let mut bytes = match last {
        domain::Last::Exact(bytes) | domain::Last::Open(bytes) => bytes.len(),
    };
    let mut members = List::with_capacity(limits.pattern_segments);
    for segment in segments {
        bytes = bytes.saturating_add(segment.len());
        let segment = wire::Segment { bytes: segment.clone() };
        if members.push(segment).is_err() {
            return Err(pattern_problem());
        }
    }
    if bytes > skein_lib::length::usize_of(limits.segment_bytes) {
        return Err(pattern_problem());
    }
    Ok(wire::Pattern { segments: members, last: maps::last_to_record(last, limits)? })
}

pub(crate) fn pattern_to_domain(value: wire::Pattern, limits: &wire::Limits) -> Result<domain::Pattern, wire::Problem> {
    let wire::Pattern { segments, last } = value;
    let last = maps::last_to_domain(last, limits)?;
    let mut bytes = match &last {
        domain::Last::Exact(bytes) | domain::Last::Open(bytes) => bytes.len(),
    };
    let mut members = List::with_capacity(segments.len());
    for segment in segments.into_boxed() {
        let wire::Segment { bytes: segment } = segment;
        bytes = bytes.saturating_add(segment.len());
        members.push(segment).expect("same number of segments");
    }
    if bytes > skein_lib::length::usize_of(limits.segment_bytes) {
        return Err(pattern_problem());
    }
    Ok(domain::Pattern { segments: members.into_boxed(), last })
}

fn pattern_problem() -> wire::Problem {
    wire::Problem { path: wire::paths::PATTERN_SEGMENTS, reason: skein_codec::Reason::Bound }
}

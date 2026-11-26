use super::{Bounds, Undecodable, w};
use alloc::boxed::Box;
use skein_codec::Record;
use skein_forge::api;
use skein_lib::{List, Reader, Writer};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_repository(value: api::Repository) -> w::Repository {
    crate::generated::store::repository_to_record(&value)
}
pub(super) fn decode_repository(value: w::Repository) -> api::Repository {
    crate::generated::store::repository_to_domain(value)
}
pub(super) fn encode_piece(values: &[Box<[u8]>]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size.checked_add(value.len()).expect("bounded piece").checked_add(8).expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = w::AnswerName { name: value.clone() };
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece");
    }
    writer.finish()
}
fn encode_segments(values: &[Box<[u8]>]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size.checked_add(value.len()).expect("bounded piece").checked_add(8).expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = w::Segment { name: value.clone() };
        let encoded = item.encode(&w::CEILINGS).expect(super::FITS);
        skein_codec::piece::write(&mut writer, &encoded).expect("measured piece");
    }
    writer.finish()
}
fn decode_segments(bytes: &[u8], bounds: &Bounds) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut segments = List::with_capacity(bounds.forge().name_bytes);
    let mut size = 0_u32;
    for _ in 0..=bounds.forge().name_bytes {
        let item = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))?;
        let Some(segment) = item else {
            return Ok(segments.into_boxed());
        };
        let segment =
            w::Segment::decode(bounds.limits(), &mut Reader::new(&segment)).or(Err(Undecodable::Malformed))?.name;
        size = size
            .checked_add(
                u32::try_from(segment.len())
                    .or(Err(Undecodable::Malformed))?
                    .checked_add(1)
                    .ok_or(Undecodable::Malformed)?,
            )
            .ok_or(Undecodable::Malformed)?;
        if size > bounds.forge().name_bytes {
            return Err(Undecodable::Malformed);
        }
        segments.push(segment).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}

pub(super) fn encode_name(value: &d::Name) -> w::Name {
    crate::generated::store::name_to_record(value)
}
pub(super) fn decode_name(value: w::Name, bounds: &Bounds) -> Result<d::Name, Undecodable> {
    crate::generated::store::name_to_domain(value, &super::Context::bounded(bounds)).or(Err(Undecodable::Malformed))
}
pub(super) fn encode_protection(value: &d::Protection) -> w::RepositoryProtection {
    crate::generated::store::repository_protection_to_record(value)
}
pub(super) fn decode_contexts(bytes: &[u8], bounds: &Bounds) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    decode_names(bytes, bounds, bounds.forge().client.rows)
}
// Stored names keep their encoding; Branches is byte-bounded (skein's forge/vocabulary.md, 1.7).
pub(super) fn decode_branches(bytes: &[u8], bounds: &Bounds) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    let width = u32::try_from(skein_forge::measure::STRING_BYTES).expect("string width fits");
    let capacity = bounds.forge().client.answer_bytes.checked_div(width).expect("nonzero string width");
    let names = decode_names(bytes, bounds, capacity)?;
    let mut measured = 0_u64;
    for name in &names {
        measured = measured
            .checked_add(u64::from(width))
            .ok_or(Undecodable::Malformed)?
            .checked_add(skein_lib::len_u64(name.len()))
            .ok_or(Undecodable::Malformed)?;
        if measured > u64::from(bounds.forge().client.answer_bytes) {
            return Err(Undecodable::Malformed);
        }
    }
    Ok(names)
}
fn decode_names(bytes: &[u8], bounds: &Bounds, capacity: u32) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    let mut reader = Reader::new(bytes);
    let mut contexts = List::with_capacity(capacity);
    for _ in 0..=capacity {
        let item = skein_codec::piece::split(&mut reader).or(Err(Undecodable::Malformed))?;
        let Some(item) = item else {
            return Ok(contexts.into_boxed());
        };
        let item =
            w::AnswerName::decode(bounds.limits(), &mut Reader::new(&item)).or(Err(Undecodable::Malformed))?.name;
        contexts.push(item).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
pub(super) fn decode_protection(value: w::RepositoryProtection, bounds: &Bounds) -> Result<d::Protection, Undecodable> {
    crate::generated::store::repository_protection_to_domain(value, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))
}

pub(super) fn encode_role(value: d::Role) -> w::Role {
    crate::generated::store::role_to_record(&value)
}
pub(super) fn decode_role(value: w::Role) -> d::Role {
    crate::generated::store::role_to_domain(value)
}

pub(super) fn encode_kinds(value: &d::Kinds) -> w::Kinds {
    crate::generated::store::kinds_to_record(value)
}
pub(super) fn decode_kinds(value: w::Kinds) -> d::Kinds {
    crate::generated::store::kinds_to_domain(value)
}

pub(super) fn encode_settings(value: &api::Settings) -> w::Settings {
    crate::generated::store::settings_to_record(value)
}
pub(super) fn decode_settings(value: w::Settings) -> api::Settings {
    crate::generated::store::settings_to_domain(value)
}

pub(super) fn encode_adopted(value: &d::Repository) -> w::Adopted {
    let d::Repository {
        project,
        contexts,
        home,
        provider,
        host,
        owner,
        name,
        prefix,
        role,
        ci,
        checks,
        kinds,
        protection,
        settings,
        workflows,
        landing,
        refusals,
    } = value;
    w::Adopted {
        project: *project,
        contexts: encode_checks(contexts),
        home: *home,
        provider: encode_repository(*provider),
        host: host.clone(),
        owner: owner.clone(),
        name: name.clone(),
        prefix: prefix.clone(),
        role: encode_role(*role),
        ci: *ci,
        checks: encode_checks(checks),
        kinds: encode_kinds(kinds),
        protection: encode_protection(protection),
        settings: encode_settings(settings),
        workflows: encode_workflows(workflows),
        landing: landing.clone(),
        refusals: encode_train_refusals(refusals),
    }
}
pub(super) fn decode_adopted(value: w::Adopted, bounds: &Bounds) -> Result<d::Repository, Undecodable> {
    let w::Adopted {
        project,
        contexts,
        home,
        provider,
        host,
        owner,
        name,
        prefix,
        role,
        ci,
        checks,
        kinds,
        protection,
        settings,
        workflows,
        landing,
        refusals,
    } = value;
    let settings = decode_settings(settings);
    let landing = if landing.is_empty() { settings.default_branch.clone() } else { landing };
    Ok(d::Repository {
        project,
        contexts: contexts.into_boxed(),
        home,
        provider: decode_repository(provider),
        host,
        owner,
        name,
        prefix,
        role: decode_role(role),
        ci,
        checks: checks.into_boxed(),
        kinds: decode_kinds(kinds),
        protection: decode_protection(protection, bounds)?,
        settings,
        workflows: workflows.into_boxed(),
        landing,
        refusals: decode_train_refusals(refusals)?,
    })
}

fn encode_train_refusals(values: &[d::TrainRefusal]) -> List<w::TrainRefusal> {
    let mut refusals = List::with_capacity(u32::try_from(values.len()).expect("admitted train count"));
    for refusal in values {
        let cause = match &refusal.cause {
            d::FormingRefusal::Landings => w::FormingRefusal::Landings,
            d::FormingRefusal::Jig { number, why, retry } => {
                w::FormingRefusal::Jig(w::RefusedByJig { number: *number, why: why.clone(), retry: retry.as_nanos() })
            }
        };
        refusals
            .push(w::TrainRefusal { branch: refusal.branch.clone(), since: refusal.since.as_nanos(), cause })
            .expect("same count");
    }
    refusals
}

fn decode_train_refusals(values: List<w::TrainRefusal>) -> Result<Box<[d::TrainRefusal]>, Undecodable> {
    let mut refusals: List<d::TrainRefusal> = List::with_capacity(values.len());
    for w::TrainRefusal { branch, since, cause } in &values {
        if branch.is_empty() {
            return Err(Undecodable::Malformed);
        }
        if let Some(last) = refusals.last()
            && last.branch >= *branch
        {
            return Err(Undecodable::Malformed);
        }
        let cause = match cause {
            w::FormingRefusal::Landings => d::FormingRefusal::Landings,
            w::FormingRefusal::Jig(w::RefusedByJig { number, why, retry }) => {
                d::FormingRefusal::Jig { number: *number, why: why.clone(), retry: skein_lib::Wall::from_nanos(*retry) }
            }
        };
        refusals
            .push(d::TrainRefusal { branch: branch.clone(), since: skein_lib::Wall::from_nanos(*since), cause })
            .expect("same count");
    }
    Ok(refusals.into_boxed())
}

pub(super) fn encode_hold(value: &d::Hold) -> w::Hold {
    crate::generated::store::hold_to_record(value).expect(super::FITS)
}
pub(super) fn decode_hold(value: w::Hold, bounds: &Bounds) -> Result<d::Hold, Undecodable> {
    crate::generated::store::hold_to_domain(value, &super::Context::bounded(bounds)).or(Err(Undecodable::Malformed))
}

fn encode_checks(values: &[u32]) -> List<u32> {
    let mut checks = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for check in values {
        checks.push(*check).expect("same count");
    }
    checks
}

fn encode_workflows(values: &[Box<[u8]>]) -> List<Box<[u8]>> {
    let mut names = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        names.push(value.clone()).expect("same count");
    }
    names
}

/// Owner-shaped `Segments` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn segments_to_record(value: &[Box<[u8]>]) -> w::Segments {
    w::Segments { segments: encode_segments(value) }
}
pub fn segments_to_domain(value: w::Segments, context: &super::Context) -> Result<Box<[Box<[u8]>]>, w::Problem> {
    decode_segments(&value.segments, context.bounds())
        .or(Err(w::paths::SEGMENTS_SEGMENTS.problem(skein_codec::Reason::Malformed)))
}

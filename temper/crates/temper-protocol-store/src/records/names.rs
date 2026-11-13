use super::{Bounds, FITS, Undecodable, w};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{List, Reader, Wall, Writer};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_repository(value: api::Repository) -> w::Repository {
    let api::Repository { forge, repository } = value;
    w::Repository::new(&w::CEILINGS, w::RepositoryParts { forge, repository }).expect(FITS)
}
pub(super) fn decode_repository(value: w::Repository) -> api::Repository {
    let w::RepositoryParts { forge, repository } = value.into_parts();
    api::Repository { forge, repository }
}
pub(super) fn encode_piece(values: &[Box<[u8]>]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size.checked_add(value.len()).expect("bounded piece").checked_add(8).expect("bounded piece");
    }
    let mut writer = Writer::new(size);
    for value in values {
        let item = w::AnswerName::new(&w::CEILINGS, w::AnswerNameParts { name: value.clone() }).expect(FITS);
        let mut encoded = Writer::new(usize::try_from(item.measure()).expect("name length fits"));
        item.encode(&mut encoded).expect("measured name room");
        skein_codec::piece::write(&mut writer, &encoded.finish()).expect("measured piece");
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
        let item = w::Segment::new(&w::CEILINGS, w::SegmentParts { name: value.clone() }).expect(FITS);
        let mut encoded = Writer::new(usize::try_from(item.measure()).expect("segment length fits"));
        item.encode(&mut encoded).expect("measured segment room");
        skein_codec::piece::write(&mut writer, &encoded.finish()).expect("measured piece");
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
        let segment = w::Segment::decode(bounds.limits(), &mut Reader::new(&segment))
            .or(Err(Undecodable::Malformed))?
            .into_parts()
            .name;
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
pub(super) fn encode_what(value: &d::What) -> w::What {
    match value {
        d::What::Repository => w::What::Repository,
        d::What::Branch(segments) => w::What::Branch(
            w::Segments::new(&w::CEILINGS, w::SegmentsParts { segments: encode_segments(segments) }).expect(FITS),
        ),
        d::What::Pull(number) => {
            w::What::Pull(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::What::Issue(number) => {
            w::What::Issue(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
    }
}
pub(super) fn decode_what(value: w::What, bounds: &Bounds) -> Result<d::What, Undecodable> {
    Ok(match value {
        w::What::Repository => d::What::Repository,
        w::What::Branch(value) => d::What::Branch(decode_segments(value.segments(), bounds)?),
        w::What::Pull(value) => d::What::Pull(value.number()),
        w::What::Issue(value) => d::What::Issue(value.number()),
    })
}
pub(super) fn encode_name(value: &d::Name) -> w::Name {
    let d::Name { forge, repository, what } = value;
    w::Name::new(&w::CEILINGS, w::NameParts { forge: *forge, repository: *repository, what: encode_what(what) })
        .expect(FITS)
}
pub(super) fn decode_name(value: w::Name, bounds: &Bounds) -> Result<d::Name, Undecodable> {
    let w::NameParts { forge, repository, what } = value.into_parts();
    Ok(d::Name { forge, repository, what: decode_what(what, bounds)? })
}
pub(super) fn encode_protection(value: &d::Protection) -> w::RepositoryProtection {
    match value {
        d::Protection::Unknown => w::RepositoryProtection::Unknown,
        d::Protection::Absent => w::RepositoryProtection::Absent,
        d::Protection::Rule(value) => w::RepositoryProtection::Rule(super::answers::encode_protection(value)),
    }
}
pub(super) fn decode_contexts(bytes: &[u8], bounds: &Bounds) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    decode_names(bytes, bounds, bounds.forge().client.rows)
}
// Stored names keep their encoding; Branches is byte-bounded (skein's forge/README.md, 3.7).
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
        let item = w::AnswerName::decode(bounds.limits(), &mut Reader::new(&item))
            .or(Err(Undecodable::Malformed))?
            .into_parts()
            .name;
        contexts.push(item).or(Err(Undecodable::Malformed))?;
    }
    Err(Undecodable::Malformed)
}
pub(super) fn decode_protection(value: w::RepositoryProtection, bounds: &Bounds) -> Result<d::Protection, Undecodable> {
    Ok(match value {
        w::RepositoryProtection::Unknown => d::Protection::Unknown,
        w::RepositoryProtection::Absent => d::Protection::Absent,
        w::RepositoryProtection::Rule(value) => d::Protection::Rule(super::answers::decode_protection(value, bounds)?),
    })
}

pub(super) fn encode_role(value: d::Role) -> w::Role {
    match value {
        d::Role::Owned => w::Role::Owned,
        d::Role::Adopted => w::Role::Adopted,
        d::Role::Fork => w::Role::Fork,
        d::Role::Context => w::Role::Context,
    }
}
pub(super) fn decode_role(value: w::Role) -> d::Role {
    match value {
        w::Role::Owned => d::Role::Owned,
        w::Role::Adopted => d::Role::Adopted,
        w::Role::Fork => d::Role::Fork,
        w::Role::Context => d::Role::Context,
    }
}

pub(super) fn encode_kinds(value: &d::Kinds) -> w::Kinds {
    let d::Kinds { read, push, open, land, review, status, comment, issue, branch } = value;
    w::Kinds::new(
        &w::CEILINGS,
        w::KindsParts { skein_bools: [*read, *push, *open, *land, *review, *status, *comment, *issue, *branch] },
    )
    .expect(FITS)
}
pub(super) fn decode_kinds(value: w::Kinds) -> d::Kinds {
    let w::KindsParts { skein_bools: [read, push, open, land, review, status, comment, issue, branch] } =
        value.into_parts();
    d::Kinds { read, push, open, land, review, status, comment, issue, branch }
}

pub(super) fn encode_settings(value: &api::Settings) -> w::Settings {
    let api::Settings { default_branch, merge, squash, rebase } = value;
    w::Settings::new(
        &w::CEILINGS,
        w::SettingsParts { default_branch: default_branch.clone(), merge: *merge, squash: *squash, rebase: *rebase },
    )
    .expect(FITS)
}
pub(super) fn decode_settings(value: w::Settings) -> api::Settings {
    let w::SettingsParts { default_branch, merge, squash, rebase } = value.into_parts();
    api::Settings { default_branch, merge, squash, rebase }
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
    } = value;
    w::Adopted::new(
        &w::CEILINGS,
        w::AdoptedParts {
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
        },
    )
    .expect(FITS)
}
pub(super) fn decode_adopted(value: w::Adopted, bounds: &Bounds) -> Result<d::Repository, Undecodable> {
    let w::AdoptedParts {
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
    } = value.into_parts();
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
        settings: decode_settings(settings),
        workflows: workflows.into_boxed(),
    })
}

pub(super) fn encode_hold(value: &d::Hold) -> w::Hold {
    let d::Hold { name, task, writer, design_writers, drift } = value;
    let mut designs = List::with_capacity(u32::try_from(design_writers.len()).expect("admitted design count"));
    for design in design_writers {
        designs
            .push(
                w::DesignWriter::new(
                    &w::CEILINGS,
                    w::DesignWriterParts { task: design.task, attempt: design.attempt, rereading: design.rereading },
                )
                .expect(FITS),
            )
            .expect("same design count");
    }
    w::Hold::new(
        &w::CEILINGS,
        w::HoldParts {
            name: encode_name(name),
            task: *task,
            design_writers: designs,
            writer: match writer {
                Some(v) => Some(encode_writer(*v)),
                None => None,
            },
            drift: match drift {
                Some(v) => Some(encode_drift(v)),
                None => None,
            },
        },
    )
    .expect(FITS)
}
pub(super) fn decode_hold(value: w::Hold, bounds: &Bounds) -> Result<d::Hold, Undecodable> {
    let w::HoldParts { name, task, writer, design_writers, drift } = value.into_parts();
    let mut designs = List::with_capacity(design_writers.len());
    for design in design_writers.into_boxed() {
        let w::DesignWriterParts { task, attempt, rereading } = design.into_parts();
        designs.push(d::DesignWriter { task, attempt, rereading }).expect("same design count");
    }
    Ok(d::Hold {
        name: decode_name(name, bounds)?,
        task,
        design_writers: designs.into_boxed(),
        writer: match writer {
            Some(v) => Some(decode_writer(v)),
            None => None,
        },
        drift: match drift {
            Some(v) => Some(decode_drift(v)),
            None => None,
        },
    })
}

fn encode_writer(value: d::Writer) -> w::Writer {
    match value {
        d::Writer::Run { task, attempt } => {
            w::Writer::Run(w::RunWriter::new(&w::CEILINGS, w::RunWriterParts { task, attempt }).expect(FITS))
        }
        d::Writer::Lost { task, attempt } => {
            w::Writer::Lost(w::RunWriter::new(&w::CEILINGS, w::RunWriterParts { task, attempt }).expect(FITS))
        }
        d::Writer::Entry(number) => {
            w::Writer::Entry(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS))
        }
    }
}
fn decode_writer(value: w::Writer) -> d::Writer {
    match value {
        w::Writer::Run(value) => {
            let w::RunWriterParts { task, attempt } = value.into_parts();
            d::Writer::Run { task, attempt }
        }
        w::Writer::Lost(value) => {
            let w::RunWriterParts { task, attempt } = value.into_parts();
            d::Writer::Lost { task, attempt }
        }
        w::Writer::Entry(value) => d::Writer::Entry(value.number()),
    }
}
fn encode_drift(value: &d::Drift) -> w::Drift {
    let d::Drift { at, change } = value;
    let change = match change {
        d::DriftChange::Branch { expected, observed } => w::DriftChange::Branch(
            w::BranchDrift::new(&w::CEILINGS, w::BranchDriftParts { expected: *expected, observed: *observed })
                .expect(FITS),
        ),
        d::DriftChange::PullClosed { number } => {
            w::DriftChange::PullClosed(w::Number::new(&w::CEILINGS, w::NumberParts { number: *number }).expect(FITS))
        }
        d::DriftChange::Retargeted { number, before, after } => w::DriftChange::Retargeted(
            w::Retargeted::new(
                &w::CEILINGS,
                w::RetargetedParts { number: *number, before: before.clone(), after: after.clone() },
            )
            .expect(FITS),
        ),
    };
    w::Drift::new(&w::CEILINGS, w::DriftParts { at: at.as_nanos(), change }).expect(FITS)
}
fn decode_drift(value: w::Drift) -> d::Drift {
    let w::DriftParts { at, change } = value.into_parts();
    let change = match change {
        w::DriftChange::Branch(value) => {
            let w::BranchDriftParts { expected, observed } = value.into_parts();
            d::DriftChange::Branch { expected, observed }
        }
        w::DriftChange::PullClosed(value) => d::DriftChange::PullClosed { number: value.number() },
        w::DriftChange::Retargeted(value) => {
            let w::RetargetedParts { number, before, after } = value.into_parts();
            d::DriftChange::Retargeted { number, before, after }
        }
    };
    d::Drift { at: Wall::from_nanos(at), change }
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

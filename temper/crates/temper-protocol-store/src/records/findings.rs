//! Findings (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use crate::Undecodable;
use alloc::boxed::Box;
use skein_lib::Wall;
use temper_engine_domain_forge_vocabulary as d;
fn encode_holder(value: d::Holder) -> w::FindingHolder {
    crate::generated::store::finding_holder_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_holder(value: w::FindingHolder) -> d::Holder {
    crate::generated::store::finding_holder_to_domain(value, &super::Context::UNBOUNDED).expect(FITS)
}

fn encode_findingkind(value: d::FindingKind) -> w::FindingKind {
    crate::generated::store::finding_kind_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_findingkind(value: w::FindingKind) -> d::FindingKind {
    crate::generated::store::finding_kind_to_domain(value, &super::Context::UNBOUNDED).expect(FITS)
}
fn encode_severity(value: d::Severity) -> w::Severity {
    crate::generated::store::severity_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_severity(value: w::Severity) -> d::Severity {
    crate::generated::store::severity_to_domain(value, &super::Context::UNBOUNDED).expect(FITS)
}
fn encode_findingstate(value: d::FindingState) -> w::FindingState {
    crate::generated::store::finding_state_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_findingstate(value: w::FindingState) -> d::FindingState {
    crate::generated::store::finding_state_to_domain(value, &super::Context::UNBOUNDED).expect(FITS)
}
fn encode_filer(value: d::Filer) -> w::Filer {
    crate::generated::store::filer_to_record(&value, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_filer(value: w::Filer) -> d::Filer {
    crate::generated::store::filer_to_domain(value, &super::Context::UNBOUNDED).expect(FITS)
}
fn encode_signature(signature: &d::FindingSignature) -> w::FindingSignature {
    let mut ids = skein_lib::List::with_capacity(64);
    for id in &signature.ids {
        ids.push(id.clone()).expect(FITS);
    }
    w::FindingSignature {
        repository: super::names::encode_repository(signature.repository),
        source: signature.source.clone(),
        ids,
        first: signature.first.clone(),
        many: signature.many,
    }
}
fn decode_signature(signature: w::FindingSignature) -> Result<d::FindingSignature, Undecodable> {
    let w::FindingSignature { repository, source, ids, first, many } = signature;
    let signature = d::FindingSignature {
        repository: super::names::decode_repository(repository),
        source,
        ids: ids.into_boxed(),
        first,
        many,
    };
    if !signature.fits() {
        return Err(Undecodable::Malformed);
    }
    Ok(signature)
}
pub(super) fn encode(row: &d::Finding) -> w::Finding {
    w::Finding {
        project: row.project,
        number: row.number,
        kind: encode_findingkind(row.kind),
        severity: encode_severity(row.severity),
        area: row.area.clone(),
        state: encode_findingstate(row.state),
        title: row.title.clone(),
        body: row.body.clone(),
        filer: encode_filer(row.filer),
        at: row.at.as_nanos(),
        why: row.why.clone(),
        hander: match row.hander {
            Some(value) => Some(encode_filer(value)),
            None => None,
        },
        goal: row.goal,
        reason: row.reason.clone(),
        signature: match &row.signature {
            Some(signature) => Some(encode_signature(signature)),
            None => None,
        },
        seen: row.seen,
        blame: match &row.blame {
            Some(blame) => Some(encode_blame(blame)),
            None => None,
        },
        repair: match &row.repair {
            Some(repair) => Some(encode_repair(repair)),
            None => None,
        },
        cleared: match row.cleared {
            Some(cleared) => Some(w::FindingCleared { run: cleared.run, at: cleared.at.as_nanos() }),
            None => None,
        },
    }
}
pub(super) fn decode(row: w::Finding) -> Result<d::Finding, Undecodable> {
    let w::Finding {
        project,
        number,
        kind,
        severity,
        area,
        state,
        title,
        body,
        filer,
        at,
        why,
        hander,
        goal,
        reason,
        signature,
        seen,
        blame,
        cleared,
        repair,
    } = row;
    if number == 0 || !d::settings::words_fit(&title, 200) || !d::findings::body_fits(&body) {
        return Err(Undecodable::Malformed);
    }
    if let Some(words) = &why
        && !d::settings::words_fit(words, 200)
    {
        return Err(Undecodable::Malformed);
    }
    if let Some(words) = &reason
        && !d::findings::reason_fits(words)
    {
        return Err(Undecodable::Malformed);
    }
    if why.is_some() != hander.is_some() {
        // Goal judging returns a landed red with a why but no hander. A
        // retry keeps that why while routing the finding to the new goal.
        let returned = match kind {
            w::FindingKind::Red => why.is_some() && blame.is_some() && repair.is_some(),
            w::FindingKind::Qa | w::FindingKind::Flake | w::FindingKind::Review | w::FindingKind::Feedback => false,
        };
        if !returned {
            return Err(Undecodable::Malformed);
        }
    }
    let signature = match signature {
        Some(signature) => Some(decode_signature(signature)?),
        None => None,
    };
    if seen == Some(0) {
        return Err(Undecodable::Malformed);
    }
    Ok(d::Finding {
        project,
        number,
        kind: decode_findingkind(kind),
        severity: decode_severity(severity),
        area,
        state: decode_findingstate(state),
        title,
        body,
        filer: decode_filer(filer),
        at: Wall::from_nanos(at),
        why,
        hander: match hander {
            Some(value) => Some(decode_filer(value)),
            None => None,
        },
        goal,
        reason,
        signature,
        seen,
        blame: match blame {
            Some(blame) => Some(decode_blame(blame)),
            None => None,
        },
        repair: match repair {
            Some(repair) => Some(decode_repair(repair)),
            None => None,
        },
        cleared: match cleared {
            Some(cleared) => Some(d::FindingCleared { run: cleared.run, at: Wall::from_nanos(cleared.at) }),
            None => None,
        },
    })
}

fn encode_blame(blame: &d::FindingBlame) -> w::FindingBlame {
    crate::generated::store::finding_blame_to_record(blame, &super::Context::UNBOUNDED).expect(FITS)
}
fn decode_blame(blame: w::FindingBlame) -> d::FindingBlame {
    crate::generated::store::finding_blame_to_domain(blame, &super::Context::UNBOUNDED).expect(FITS)
}

fn encode_repair(repair: &d::FindingRepair) -> w::FindingRepair {
    let mut failing = skein_lib::List::with_capacity(d::FindingSignature::IDS);
    for id in &repair.failing {
        failing.push(id.clone()).expect(FITS);
    }
    let judging = match repair.judging {
        Some(d::RepairJudging::Landing(commit)) => Some(w::RepairJudging::Landing(w::RepairLanding { commit })),
        Some(d::RepairJudging::After(goal)) => Some(w::RepairJudging::After(w::RepairAfter { goal })),
        Some(d::RepairJudging::Release) => Some(w::RepairJudging::Release),
        None => None,
    };
    w::FindingRepair {
        goal: repair.goal,
        made: repair.made,
        attempts: repair.attempts,
        cancelled: repair.cancelled,
        failing,
        until: match repair.until {
            Some(until) => Some(until.as_nanos()),
            None => None,
        },
        judging,
    }
}
fn decode_repair(repair: w::FindingRepair) -> d::FindingRepair {
    let judging = match repair.judging {
        Some(w::RepairJudging::Landing(landing)) => Some(d::RepairJudging::Landing(landing.commit)),
        Some(w::RepairJudging::After(after)) => Some(d::RepairJudging::After(after.goal)),
        Some(w::RepairJudging::Release) => Some(d::RepairJudging::Release),
        None => None,
    };
    d::FindingRepair {
        goal: repair.goal,
        made: repair.made,
        attempts: repair.attempts,
        cancelled: repair.cancelled,
        failing: repair.failing.into_boxed(),
        until: match repair.until {
            Some(until) => Some(Wall::from_nanos(until)),
            None => None,
        },
        judging,
    }
}

/// Waiting filings preserve their words and blame without passing through a Finding.
pub(super) fn encode_filings(filings: &[d::VerdictFiling]) -> skein_lib::List<w::VerdictFiling> {
    let mut rows = skein_lib::List::with_capacity(8);
    for filing in filings {
        rows.push(crate::generated::store::verdict_filing_to_record(filing, &super::Context::UNBOUNDED).expect(FITS))
            .expect(FITS);
    }
    rows
}
pub(super) fn decode_filings(
    filings: skein_lib::List<w::VerdictFiling>,
) -> Result<Box<[d::VerdictFiling]>, Undecodable> {
    let mut rows = skein_lib::List::with_capacity(8);
    for filing in filings.into_boxed() {
        rows.push(
            crate::generated::store::verdict_filing_to_domain(filing, &super::Context::UNBOUNDED)
                .or(Err(Undecodable::Malformed))?,
        )
        .expect(FITS);
    }
    Ok(rows.into_boxed())
}

/// Owner-shaped `FindingSignature` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn finding_signature_to_record(
    value: &d::FindingSignature,
    _context: &super::Context,
) -> Result<w::FindingSignature, w::Problem> {
    Ok(encode_signature(value))
}
pub fn finding_signature_to_domain(
    value: w::FindingSignature,
    _context: &super::Context,
) -> Result<d::FindingSignature, w::Problem> {
    decode_signature(value).or(Err(w::paths::FINDING_SIGNATURE_REPOSITORY.problem(skein_codec::Reason::Malformed)))
}

/// Owner-shaped `FindingAreas` (skein/docs/design/data/codec/maps.md, 3.3).
#[expect(clippy::borrowed_box, reason = "maps pairs take the domain type by reference")]
pub fn finding_areas_to_record(
    value: &Box<[Box<[u8]>]>,
    _context: &super::Context,
) -> Result<w::FindingAreas, w::Problem> {
    Ok({
        let mut names = skein_lib::List::with_capacity(16);
        for name in value {
            names.push(name.clone()).expect(FITS);
        }
        w::FindingAreas { names }
    })
}
pub fn finding_areas_to_domain(
    value: w::FindingAreas,
    _context: &super::Context,
) -> Result<Box<[Box<[u8]>]>, w::Problem> {
    Ok::<_, w::Problem>(value.names.into_boxed())
        .or(Err(w::paths::FINDING_AREAS_NAMES.problem(skein_codec::Reason::Malformed)))
}

/// Owner-shaped `FindingResult` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn finding_result_to_record(
    value: &Result<d::FindingDone, d::FindingRefusal>,
    _context: &super::Context,
) -> Result<w::FindingResult, w::Problem> {
    Ok(match value {
        Ok(d::FindingDone::Filed(holder)) => w::FindingResult::Filed(w::FindingHeld { holder: encode_holder(*holder) }),
        Ok(d::FindingDone::Handed(holder)) => {
            w::FindingResult::Handed(w::FindingHeld { holder: encode_holder(*holder) })
        }
        Ok(d::FindingDone::Routed(number)) => w::FindingResult::Routed(w::Number { number: *number }),
        Ok(d::FindingDone::Backlog) => w::FindingResult::Backlog,
        Ok(d::FindingDone::Dropped) => w::FindingResult::Dropped,
        Ok(d::FindingDone::Repaired(number)) => w::FindingResult::Repaired(w::Number { number: *number }),
        Ok(d::FindingDone::Blamed) => w::FindingResult::Blamed,
        Err(why) => w::FindingResult::Refused(w::FindingRefused {
            why: crate::generated::store::finding_refusal_to_record(why, &super::Context::UNBOUNDED).expect(FITS),
        }),
    })
}
pub fn finding_result_to_domain(
    value: w::FindingResult,
    _context: &super::Context,
) -> Result<Result<d::FindingDone, d::FindingRefusal>, w::Problem> {
    Ok::<_, w::Problem>(match value {
        w::FindingResult::Filed(value) => Ok(d::FindingDone::Filed(decode_holder(value.holder))),
        w::FindingResult::Handed(value) => Ok(d::FindingDone::Handed(decode_holder(value.holder))),
        w::FindingResult::Routed(value) => Ok(d::FindingDone::Routed(value.number)),
        w::FindingResult::Backlog => Ok(d::FindingDone::Backlog),
        w::FindingResult::Dropped => Ok(d::FindingDone::Dropped),
        w::FindingResult::Repaired(value) => Ok(d::FindingDone::Repaired(value.number)),
        w::FindingResult::Blamed => Ok(d::FindingDone::Blamed),
        w::FindingResult::Refused(value) => {
            Err(crate::generated::store::finding_refusal_to_domain(value.why, &super::Context::UNBOUNDED).expect(FITS))
        }
    })
    .or(Err(w::paths::FINDING_RESULT_TAG.problem(skein_codec::Reason::Malformed)))
}

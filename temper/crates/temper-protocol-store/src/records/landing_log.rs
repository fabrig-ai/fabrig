//! Family 30's bounded translations (temper/docs/design/protocol/store.md, 2).
use super::{FITS, tiers, w};
use crate::Undecodable;
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode(row: &d::LandingLog) -> w::LandingLog {
    crate::generated::store::landing_log_to_record(row).expect(FITS)
}

pub(super) fn decode(row: w::LandingLog, tier_bound: u32) -> Result<d::LandingLog, Undecodable> {
    let context = super::Context { logged_landing_tiers: tier_bound, ..super::Context::UNBOUNDED };
    crate::generated::store::landing_log_to_domain(row, &context).or(Err(Undecodable::Malformed))
}

fn encode_tiers(tiers: &[d::LoggedTier]) -> List<w::LoggedTier> {
    let mut result = List::with_capacity(u32::try_from(tiers.len()).expect(FITS));
    for tier in tiers {
        result.push(crate::generated::store::logged_tier_to_record(tier)).expect(FITS);
    }
    result
}

fn decode_tiers(tiers: List<w::LoggedTier>) -> Result<Box<[d::LoggedTier]>, Undecodable> {
    let mut result = List::with_capacity(tiers.len());
    for tier in tiers.into_boxed() {
        result.push(crate::generated::store::logged_tier_to_domain(tier).or(Err(Undecodable::Malformed))?).expect(FITS);
    }
    Ok(result.into_boxed())
}

pub(super) fn encode_verdict(verdict: &d::LoggedVerdict) -> w::LoggedVerdict {
    match verdict {
        d::LoggedVerdict::Verifying => w::LoggedVerdict::Verifying,
        d::LoggedVerdict::Cleared { run } => w::LoggedVerdict::Cleared(w::LoggedCleared { run: *run }),
        d::LoggedVerdict::Red { run, failing, tests, many } => w::LoggedVerdict::Red(w::LoggedRed {
            run: *run,
            failing: *failing,
            tests: match many {
                Some(many) => tiers::encode_tests(&many.tests),
                None => tiers::encode_tests(tests),
            },
            many: match many {
                Some(many) => Some(w::LoggedMany { more: many.more, cut: many.cut }),
                None => None,
            },
        }),
    }
}

pub(super) fn decode_verdict(verdict: w::LoggedVerdict) -> Result<d::LoggedVerdict, Undecodable> {
    Ok(match verdict {
        w::LoggedVerdict::Verifying => d::LoggedVerdict::Verifying,
        w::LoggedVerdict::Cleared(row) => d::LoggedVerdict::Cleared { run: row.run },
        w::LoggedVerdict::Red(row) => {
            let w::LoggedRed { run, failing, tests, many } = row;
            let tests = tiers::decode_tests(tests)?;
            match many {
                Some(many) => {
                    let w::LoggedMany { more, cut } = many;
                    d::LoggedVerdict::Red {
                        run,
                        failing,
                        tests: Box::new([]),
                        many: Some(temper_engine_domain_forge_tiers::ManyTests { tests, more, cut }),
                    }
                }
                None => d::LoggedVerdict::Red { run, failing, tests, many: None },
            }
        }
    })
}

pub(super) fn encode_landing(landing: &d::LoggedLanding) -> w::LoggedLanding {
    let mut changes = List::with_capacity(u32::try_from(landing.changes.len()).expect(FITS));
    for change in &landing.changes {
        changes.push(crate::generated::store::logged_change_to_record(change).expect(FITS)).expect(FITS);
    }
    w::LoggedLanding {
        commit: landing.commit,
        branch: landing.branch.clone(),
        at: landing.at.as_nanos(),
        changes,
        tiers: encode_tiers(&landing.tiers),
    }
}
pub(super) fn decode_landing(landing: w::LoggedLanding, tier_bound: u32) -> Result<d::LoggedLanding, Undecodable> {
    let w::LoggedLanding { commit, branch, at, changes, tiers } = landing;
    let mut members = List::with_capacity(changes.len());
    for change in changes.into_boxed() {
        members.push(crate::generated::store::logged_change_to_domain(change)).expect(FITS);
    }
    let tiers = decode_tiers(tiers)?;
    if tiers.len() > skein_lib::usize_of(tier_bound) {
        // Only logs written before ids were kept need the schema's wider
        // tier ceiling; their entries are retired with settings at restore.
        for tier in &tiers {
            match &tier.verdict {
                d::LoggedVerdict::Red { tests, many, .. } => {
                    if !tests.is_empty() || many.is_some() {
                        return Err(Undecodable::Malformed);
                    }
                }
                d::LoggedVerdict::Verifying | d::LoggedVerdict::Cleared { .. } => {}
            }
        }
    }

    Ok(d::LoggedLanding { tiers, commit, branch, at: Wall::from_nanos(at), changes: members.into_boxed() })
}

#[must_use]
pub fn logged_verdict_to_record(value: &d::LoggedVerdict) -> w::LoggedVerdict {
    encode_verdict(value)
}

pub fn logged_verdict_to_domain(value: w::LoggedVerdict) -> Result<d::LoggedVerdict, w::Problem> {
    decode_verdict(value).or(Err(w::paths::LOGGED_VERDICT_TAG.problem(skein_codec::Reason::Malformed)))
}

#[must_use]
pub fn logged_landing_to_record(value: &d::LoggedLanding) -> w::LoggedLanding {
    encode_landing(value)
}

pub fn logged_landing_to_domain(
    value: w::LoggedLanding,
    context: &super::Context,
) -> Result<d::LoggedLanding, w::Problem> {
    decode_landing(value, context.logged_landing_tiers)
        .or(Err(w::paths::LOGGED_LANDING_TIERS.problem(skein_codec::Reason::Malformed)))
}

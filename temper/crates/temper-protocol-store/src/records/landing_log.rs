//! Family 30's bounded translations (temper/docs/design/protocol/store.md, 2).
use super::{FITS, names, tiers, w};
use crate::Undecodable;
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode(row: &d::LandingLog) -> w::LandingLog {
    let mut landings = List::with_capacity(u32::try_from(row.landings.len()).expect(FITS));
    for landing in &row.landings {
        let mut changes = List::with_capacity(u32::try_from(landing.changes.len()).expect(FITS));
        for change in &landing.changes {
            changes
                .push(w::LoggedChange {
                    task: change.task,
                    pull: change.pull,
                    title: change.title.clone(),
                    signed: encode_signed(&change.signed),
                    goal: change.goal,
                    milestone: change.milestone,
                })
                .expect(FITS);
        }
        landings
            .push(w::LoggedLanding {
                commit: landing.commit,
                branch: landing.branch.clone(),
                at: landing.at.as_nanos(),
                changes,
                tiers: encode_tiers(&landing.tiers),
            })
            .expect(FITS);
    }
    w::LandingLog { repository: names::encode_repository(row.repository), landings }
}

pub(super) fn decode(row: w::LandingLog, tier_bound: u32) -> Result<d::LandingLog, Undecodable> {
    let w::LandingLog { repository, landings } = row;
    let mut decoded = List::with_capacity(landings.len());
    for landing in landings.into_boxed() {
        let w::LoggedLanding { commit, branch, at, changes, tiers } = landing;
        let mut members = List::with_capacity(changes.len());
        for change in changes.into_boxed() {
            let w::LoggedChange { task, pull, title, signed, goal, milestone } = change;
            members
                .push(d::LoggedChange { goal, milestone, task, pull, title, signed: decode_signed(signed) })
                .expect(FITS);
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
        decoded
            .push(d::LoggedLanding { tiers, commit, branch, at: Wall::from_nanos(at), changes: members.into_boxed() })
            .expect(FITS);
    }
    Ok(d::LandingLog { repository: names::decode_repository(repository), landings: decoded.into_boxed() })
}

pub(super) fn encode_signed(signed: &[d::SignOff]) -> List<w::SignOff> {
    let mut result = List::with_capacity(u32::try_from(d::SIGN_OFFS).expect(FITS));
    for sign in signed {
        result.push(w::SignOff { person: sign.person, head: sign.head, at: sign.at.as_nanos() }).expect(FITS);
    }
    result
}

pub(super) fn decode_signed(signed: List<w::SignOff>) -> Box<[d::SignOff]> {
    let mut result = List::with_capacity(signed.len());
    for sign in signed.into_boxed() {
        let w::SignOff { person, head, at } = sign;
        result.push(d::SignOff { person, head, at: Wall::from_nanos(at) }).expect(FITS);
    }
    result.into_boxed()
}

fn encode_tiers(tiers: &[d::LoggedTier]) -> List<w::LoggedTier> {
    let mut result = List::with_capacity(u32::try_from(tiers.len()).expect(FITS));
    for tier in tiers {
        let verdict = match &tier.verdict {
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
        };
        result.push(w::LoggedTier { tier: tier.tier.clone(), verdict }).expect(FITS);
    }
    result
}

fn decode_tiers(tiers: List<w::LoggedTier>) -> Result<Box<[d::LoggedTier]>, Undecodable> {
    let mut result = List::with_capacity(tiers.len());
    for tier in tiers.into_boxed() {
        let w::LoggedTier { tier, verdict } = tier;
        let verdict = match verdict {
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
        };
        result.push(d::LoggedTier { tier, verdict }).expect(FITS);
    }
    Ok(result.into_boxed())
}

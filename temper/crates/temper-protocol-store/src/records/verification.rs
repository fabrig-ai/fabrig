//! Family 29's bounded translations (temper/docs/design/protocol/store.md, 2).
use super::{FITS, names, w};
use crate::Undecodable;
use skein_lib::List;
use temper_engine_domain_forge_tiers::{Blame, Failures};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_failures(value: &Failures) -> w::Failures {
    match value {
        Failures::Many => w::Failures::Many,
        Failures::Tests(tests) => {
            let mut encoded = List::with_capacity(u32::try_from(tests.len()).expect(FITS));
            for test in tests {
                encoded.push(w::FailingTest { test_id: test.clone() }).expect(FITS);
            }
            w::Failures::Tests(w::FailedTests { tests: encoded })
        }
    }
}

pub(super) fn encode_blame(value: &Blame) -> w::Blame {
    match value {
        Blame::Cleared => w::Blame::Cleared,
        Blame::Owns(failures) => w::Blame::Owns(w::BlamedFailures { failures: encode_failures(failures) }),
        Blame::Suspect(failures) => w::Blame::Suspect(w::BlamedFailures { failures: encode_failures(failures) }),
    }
}

pub(super) fn encode(row: &d::Verification) -> w::Verification {
    let mut changes = List::with_capacity(u32::try_from(row.changes.len()).expect(FITS));
    for task in &row.changes {
        changes.push(*task).expect(FITS);
    }
    let mut tiers = List::with_capacity(u32::try_from(row.tiers.len()).expect(FITS));
    for tier in &row.tiers {
        let verdict = match &tier.verdict {
            Some(verdict) => Some(w::TierVerdict { run: verdict.run, blame: encode_blame(&verdict.blame) }),
            None => None,
        };
        tiers
            .push(w::TierVerification {
                tier: tier.tier.clone(),
                verdict,
                repaired: match tier.repaired {
                    Some(repair) => Some(w::TierRepair { run: repair.run, tip: repair.tip, by: repair.by }),
                    None => None,
                },
            })
            .expect(FITS);
    }
    w::Verification { repository: names::encode_repository(row.repository), commit: row.commit, changes, tiers }
}

pub(super) fn decode_failures(value: w::Failures) -> Result<Failures, Undecodable> {
    Ok(match value {
        w::Failures::Many => Failures::Many,
        w::Failures::Tests(tests) => {
            let mut decoded = List::with_capacity(tests.tests.len());
            for test in tests.tests.into_boxed() {
                let test = test.test_id;
                if let Some(previous) = decoded.as_slice().last()
                    && previous >= &test
                {
                    return Err(Undecodable::Malformed);
                }
                decoded.push(test).expect(FITS);
            }
            Failures::Tests(decoded.into_boxed())
        }
    })
}

pub(super) fn decode_blame(value: w::Blame) -> Result<Blame, Undecodable> {
    Ok(match value {
        w::Blame::Cleared => Blame::Cleared,
        w::Blame::Owns(value) => Blame::Owns(decode_failures(value.failures)?),
        w::Blame::Suspect(value) => Blame::Suspect(decode_failures(value.failures)?),
    })
}

pub(super) fn decode(row: w::Verification) -> Result<d::Verification, Undecodable> {
    let w::Verification { repository, commit, changes, tiers } = row;
    let mut decoded = List::with_capacity(tiers.len());
    for tier in tiers.into_boxed() {
        let w::TierVerification { tier, verdict, repaired } = tier;
        if tier.is_empty() {
            return Err(Undecodable::Malformed);
        }
        for byte in &tier {
            if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_' {
                return Err(Undecodable::Malformed);
            }
        }
        for previous in &decoded {
            let previous: &d::TierVerification = previous;
            if previous.tier == tier {
                return Err(Undecodable::Malformed);
            }
        }
        let verdict = match verdict {
            Some(verdict) => {
                let w::TierVerdict { run, blame } = verdict;
                Some(d::TierVerdict { run, blame: decode_blame(blame)? })
            }
            None => None,
        };
        let repaired = match repaired {
            Some(repair) => Some(d::TierRepair { run: repair.run, tip: repair.tip, by: repair.by }),
            None => None,
        };
        decoded.push(d::TierVerification { tier, verdict, repaired }).expect(FITS);
    }
    Ok(d::Verification {
        repository: names::decode_repository(repository),
        commit,
        changes: changes.into_boxed(),
        tiers: decoded.into_boxed(),
    })
}

//! Family 29's bounded translations (temper/docs/design/protocol/store.md, 2).
use super::{FITS, names, w};
use crate::Undecodable;
use skein_lib::List;
use temper_engine_domain_forge_tiers::Failures;
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

pub(super) fn encode(row: &d::Verification) -> w::Verification {
    let mut changes = List::with_capacity(u32::try_from(row.changes.len()).expect(FITS));
    for task in &row.changes {
        changes.push(*task).expect(FITS);
    }
    let mut tiers = List::with_capacity(u32::try_from(row.tiers.len()).expect(FITS));
    for tier in &row.tiers {
        tiers.push(crate::generated::store::tier_verification_to_record(tier)).expect(FITS);
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

pub(super) fn decode(row: w::Verification) -> Result<d::Verification, Undecodable> {
    let w::Verification { repository, commit, changes, tiers } = row;
    let mut decoded = List::with_capacity(tiers.len());
    for tier in tiers.into_boxed() {
        let tier = crate::generated::store::tier_verification_to_domain(tier).or(Err(Undecodable::Malformed))?;

        if tier.tier.is_empty() {
            return Err(Undecodable::Malformed);
        }
        for byte in &tier.tier {
            if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_' {
                return Err(Undecodable::Malformed);
            }
        }
        for previous in &decoded {
            let previous: &d::TierVerification = previous;
            if previous.tier == tier.tier {
                return Err(Undecodable::Malformed);
            }
        }
        decoded.push(tier).expect(FITS);
    }
    Ok(d::Verification {
        repository: names::decode_repository(repository),
        commit,
        changes: changes.into_boxed(),
        tiers: decoded.into_boxed(),
    })
}

//! Report reads and selected parts (temper/docs/design/domain/after-landing.md, 2, 4 and 5.2).
use super::{Box, Commit, Domain, Env, Limits, List, TierRow, api, branch, configured, joined, p};

pub(super) fn part_number(part: &p::Part) -> u16 {
    match part {
        p::Part::Whole => 0,
        p::Part::Rerun { .. } => 1,
        p::Part::Probe { landing, .. } => {
            u16::try_from(*landing).expect("bounded range").checked_add(2).expect("probe part")
        }
    }
}
pub(super) fn dispatch_operation(part: &p::Part) -> u8 {
    match part {
        p::Part::Whole => 1,
        p::Part::Rerun { .. } => 2,
        p::Part::Probe { .. } => 4,
    }
}
pub(super) fn probe_branch(row: &TierRow, landing: u32) -> Box<[u8]> {
    let run = row.record.run.as_ref().expect("probe run");
    joined(&[&branch(&row.tier, run.number), b".", skein_lib::Decimal::of(u64::from(landing)).as_bytes()])
}
pub(super) fn part_branch(row: &TierRow) -> Box<[u8]> {
    let run = row.record.run.as_ref().expect("live run");
    match &run.part {
        p::Part::Whole | p::Part::Rerun { .. } => branch(&row.tier, run.number),
        p::Part::Probe { landing, .. } => probe_branch(row, *landing),
    }
}
pub(super) fn candidate(run: &p::Run) -> (Commit, p::Key, Option<&[p::TestId]>) {
    match &run.part {
        p::Part::Whole => (run.tip, run.key, None),
        p::Part::Rerun { failed } => (run.tip, run.key, Some(failed)),
        p::Part::Probe { landing, tests, .. } => {
            let landing = run.landings.get(usize::try_from(*landing).expect("range index")).expect("probe landing");
            let tests = match tests {
                p::Selection::All => None,
                p::Selection::Tests(tests) => Some(tests.as_ref()),
            };
            (landing.commit, landing.key, tests)
        }
    }
}
pub(super) fn part_inputs(commit: Commit, key: p::Key, tests: Option<&[p::TestId]>) -> Box<[api::Input]> {
    let base = crate::landing::inputs(commit, key);
    let Some(tests) = tests else { return base };
    let mut inputs = List::with_capacity(3);
    for input in base {
        inputs.push(input).expect("two landing inputs");
    }
    let mut length = 0_usize;
    for test in tests {
        length = length.checked_add(test.len()).expect("bounded ids").checked_add(1).expect("newline");
    }
    let mut value = skein_lib::Writer::new(length);
    for test in tests {
        value.put(test).expect("measured ids");
        value.put(b"\n").expect("newline per id");
    }
    inputs
        .push(api::Input { name: Box::from(b"temper_tests".as_slice()), value: value.finish() })
        .expect("selection input");
    inputs.into_boxed()
}
pub(super) fn report_read(d: &Domain, row: &TierRow, env: &Env<Limits>) -> Option<api::Read> {
    let report = row.flight.as_ref()?.report.as_ref()?;
    let tier = configured(d, row)?;
    let limits = &env.limits.tiers;
    // Lines allow CRLF; XML allows a six-byte entity per id byte, the case
    // attributes and failure tag, declaration and suite wrapper. Larger reports
    // are cut by the read and judged Many, including reports with extra logs.
    let bytes = match tier.format {
        p::Format::Lines => {
            u64::from(limits.test_id_bytes).checked_add(2)?.checked_mul(u64::from(limits.failing_tests))?
        }
        p::Format::Junit => u64::from(limits.test_id_bytes)
            .checked_mul(6)?
            .checked_add(64)?
            .checked_mul(u64::from(limits.failing_tests))?
            .checked_add(128)?,
    };
    Some(api::Read::Artifact {
        run: report.id,
        name: tier.artifact,
        max_bytes: u32::try_from(bytes.min(u64::from(env.limits.client.answer_bytes))).ok()?.max(1),
    })
}
pub(super) fn reported(
    d: &Domain,
    env: &Env<Limits>,
    row: &TierRow,
    answer: Result<api::Answer, api::Error>,
) -> p::Outcome {
    match answer {
        Ok(api::Answer::File { truncated: true, .. }) => p::Outcome::Failed(p::Failures::Many),
        Ok(api::Answer::File { bytes, .. }) => p::failed(Some(p::report(
            configured(d, row).expect("configured tier").format,
            &bytes,
            &p::Limits { failing_tests: env.limits.tiers.failing_tests, test_id_bytes: env.limits.tiers.test_id_bytes },
        ))),
        Ok(_) | Err(_) => p::failed(None),
    }
}

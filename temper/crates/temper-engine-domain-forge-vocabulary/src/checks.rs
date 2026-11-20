//! Receipt checks for a landing repair (temper/docs/design/domain/forge/change/repairs.md, A landing's repair).
use crate::{LandingFailure, LoggedTier, LoggedVerdict};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::List;
use temper_engine_domain_forge_tiers::{ManyTests, TestId};

/// Overlay the receipt's gate jobs, reserving rows before the forge's checks.
pub fn append_gate_checks(statuses: &mut Box<[api::Status]>, failure: LandingFailure, rows: u32) {
    if failure.jobs.is_empty() {
        return;
    }
    let bound = usize::try_from(rows).expect("row bound fits usize");
    let gate_rows = failure.jobs.len().min(bound);
    let forge_rows = statuses.len().min(bound.saturating_sub(gate_rows));
    let cut = forge_rows < statuses.len() || gate_rows < failure.jobs.len();
    let mut combined = List::with_capacity(rows);
    for status in statuses.iter().take(forge_rows) {
        combined.push(status.clone()).expect("forge rows leave room for gate jobs");
    }
    for job in failure.jobs.iter().take(gate_rows) {
        let mut context = List::uncapped();
        for byte in b"temper/gate/".iter().chain(job.name.iter()) {
            context.push(*byte).expect("context is internal receipt text");
        }
        combined
            .push(api::Status {
                at: skein_lib::Time::ZERO,
                context: context.into_boxed(),
                check: api::Check::Failed,
                description: if cut {
                    Box::from(b"Row bound: gate jobs take precedence; remaining checks omitted.".as_slice())
                } else {
                    Box::new([])
                },
                url: failure.run.url.clone(),
                job: Some(api::JobAttempt {
                    head: failure.prefix,
                    run: failure.run.id,
                    job: job.job,
                    attempt: job.attempt,
                }),
            })
            .expect("gate jobs have reserved rows");
    }
    *statuses = combined.into_boxed();
}

/// Overlay logged red tiers and a repair's gate jobs, in their priority order
/// (temper/docs/design/domain/after-landing.md, 9). Gate words and pins stay the receipt's.
pub fn append_tier_checks(
    statuses: &mut Box<[api::Status]>,
    tiers: &[LoggedTier],
    gate: Option<LandingFailure>,
    rows: u32,
    op_bytes: u32,
    answer_bytes: u32,
) {
    let mut red_count = 0_u32;
    for tier in tiers {
        match &tier.verdict {
            LoggedVerdict::Red { .. } => red_count = red_count.checked_add(1).expect("bounded logged tiers"),
            LoggedVerdict::Verifying | LoggedVerdict::Cleared { .. } => {}
        }
    }
    let gate_count = match &gate {
        Some(gate) => u32::try_from(gate.jobs.len()).expect("bounded gate jobs").min(rows),
        None => 0,
    };
    let count = red_count.min(rows.saturating_sub(gate_count));
    if let Some(gate) = gate {
        append_gate_checks(statuses, gate, rows.checked_sub(count).expect("reserved tier rows"));
    }
    if count == 0 {
        return;
    }
    let gate_start = statuses.len().checked_sub(skein_lib::usize_of(gate_count)).expect("gate rows at the end");
    let mut reds = List::with_capacity(count);
    for tier in tiers {
        if reds.len() == count {
            break;
        }
        match &tier.verdict {
            LoggedVerdict::Red { run, failing, tests, many } => {
                let mut context = List::uncapped();
                append(&mut context, b"temper/verdict/");
                append(&mut context, &tier.tier);
                let description = tier_description(&tier.tier, *run, *failing, tests, many.as_ref(), 0);
                reds.push(api::Status {
                    at: skein_lib::Time::ZERO,
                    context: context.into_boxed(),
                    check: api::Check::Failed,
                    description: crate::text::fit(&description, skein_lib::usize_of(op_bytes), b""),
                    url: Box::new([]),
                    job: None,
                })
                .expect("reserved red tiers");
            }
            LoggedVerdict::Verifying | LoggedVerdict::Cleared { .. } => {}
        }
    }
    let mut fixed = 0_usize;
    for gate in statuses.iter().skip(gate_start) {
        fixed = fixed.saturating_add(status_text_bytes(gate));
    }
    let mut fitting = 0_u32;
    for red in &reds {
        let next = fixed.saturating_add(status_text_bytes(red));
        if next > skein_lib::usize_of(answer_bytes) {
            break;
        }
        fixed = next;
        fitting = fitting.checked_add(1).expect("bounded red rows");
    }
    let count = fitting;
    let mut forge_count = 0_usize;
    let forge_bound = skein_lib::usize_of(rows.saturating_sub(gate_count).saturating_sub(count)).min(gate_start);
    for status in statuses.iter().take(forge_bound) {
        let next = fixed.saturating_add(status_text_bytes(status));
        if next > skein_lib::usize_of(answer_bytes) {
            break;
        }
        fixed = next;
        forge_count = forge_count.checked_add(1).expect("bounded forge rows");
    }
    let mut combined = List::with_capacity(rows);
    for status in statuses.iter().take(forge_count).chain(statuses.iter().skip(gate_start)) {
        combined.push(status.clone()).expect("reserved gate and forge rows");
    }
    let mut room = skein_lib::usize_of(answer_bytes).saturating_sub(fixed);
    let mut left = count;
    let mut red_rows = reds.into_boxed().into_iter().take(skein_lib::usize_of(count));
    for tier in tiers {
        let LoggedVerdict::Red { run, failing, tests, many } = &tier.verdict else { continue };
        let Some(mut status) = red_rows.next() else { break };
        let share = room.checked_div(skein_lib::usize_of(left)).expect("positive red rows");
        let bound = status.description.len().saturating_add(share).min(skein_lib::usize_of(op_bytes));
        let description = tier_description(&tier.tier, *run, *failing, tests, many.as_ref(), bound);
        let grown = description.len().saturating_sub(status.description.len());
        room = room.saturating_sub(grown);
        status.description = crate::text::fit(&description, bound, b"");
        combined.push(status).expect("reserved tier rows");
        left = left.checked_sub(1).expect("one red row consumed");
    }
    *statuses = combined.into_boxed();
}

// Mirrors the unchanged smith status_text framing: bytes, not quoted or escaped.
fn status_text_bytes(status: &api::Status) -> usize {
    let check = match status.check {
        api::Check::Pending => b"pending".len(),
        api::Check::Passed => b"passed".len(),
        api::Check::Failed => b"failed".len(),
    };
    let job = match &status.job {
        Some(job) => b"\nread_ci what=job commit  run  job  attempt "
            .len()
            .saturating_add(64)
            .saturating_add(skein_lib::Decimal::of(job.run).as_bytes().len())
            .saturating_add(skein_lib::Decimal::of(job.job).as_bytes().len())
            .saturating_add(skein_lib::Decimal::of(u64::from(job.attempt)).as_bytes().len()),
        None => b"\nNo job attempt resolved from this check's link.".len(),
    };
    status
        .context
        .len()
        .saturating_add(b":   \n".len())
        .saturating_add(check)
        .saturating_add(status.description.len())
        .saturating_add(status.url.len())
        .saturating_add(job)
}

fn tier_description(
    tier: &[u8],
    run: u64,
    failing: Option<u32>,
    tests: &[TestId],
    many: Option<&ManyTests>,
    bound: usize,
) -> Box<[u8]> {
    let mut prefix = List::uncapped();
    append(&mut prefix, tier);
    append(&mut prefix, b" run ");
    append(&mut prefix, skein_lib::Decimal::of(run).as_bytes());
    append(&mut prefix, b" found this landing red: ");
    let ids = match failing {
        Some(_) => tests,
        None => {
            append(&mut prefix, b"many tests");
            let Some(many) = many else {
                append(&mut prefix, b".");
                return prefix.into_boxed();
            };
            if many.tests.is_empty() {
                append(&mut prefix, b".");
                return prefix.into_boxed();
            }
            append(&mut prefix, b"; its report names ");
            &many.tests
        }
    };
    let mut kept = 0_usize;
    let mut ids_bytes = 0_usize;
    // The final id can remove the remainder, so examine every prefix.
    for (at, id) in ids.iter().enumerate() {
        let take = at.checked_add(1).expect("bounded names");
        ids_bytes = ids_bytes.saturating_add(id.len());
        if at != 0 {
            ids_bytes = ids_bytes.saturating_add(2);
        }
        let remainder = description_remainder(failing, many, ids.len(), take);
        let rest = match remainder {
            Some(rest) => rest.len().saturating_add(2),
            None => 0,
        };
        let length = skein_lib::usize_of(prefix.len()).saturating_add(ids_bytes).saturating_add(rest).saturating_add(1);
        if length <= bound {
            kept = take;
        }
    }
    named_description(&prefix, ids, kept, description_remainder(failing, many, ids.len(), kept))
}

fn description_remainder(
    failing: Option<u32>,
    many: Option<&ManyTests>,
    count: usize,
    kept: usize,
) -> Option<Box<[u8]>> {
    match failing {
        Some(count) => named_remainder(count, kept),
        None => {
            let many = many.expect("a many description names its report");
            let left = count.checked_sub(kept).expect("prefix of names");
            crate::red::many_remainder(many, u32::try_from(left).expect("bounded names"))
        }
    }
}

fn named_remainder(count: u32, kept: usize) -> Option<Box<[u8]>> {
    let left = count.saturating_sub(u32::try_from(kept).expect("bounded logged names"));
    if left == 0 {
        return None;
    }
    let mut out = List::uncapped();
    append(&mut out, b"and ");
    append(&mut out, skein_lib::Decimal::of(u64::from(left)).as_bytes());
    append(&mut out, b" more");
    Some(out.into_boxed())
}

fn named_description(prefix: &List<u8>, tests: &[TestId], kept: usize, remainder: Option<Box<[u8]>>) -> Box<[u8]> {
    let mut out = List::uncapped();
    append(&mut out, prefix.as_slice());
    for (at, test) in tests.iter().take(kept).enumerate() {
        if at != 0 {
            append(&mut out, b", ");
        }
        append(&mut out, test);
    }
    if let Some(remainder) = remainder {
        if kept != 0 {
            append(&mut out, b", ");
        }
        append(&mut out, &remainder);
    }
    append(&mut out, b".");
    out.into_boxed()
}

fn append(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("internal check text");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red(tests: Box<[TestId]>, failing: Option<u32>, many: Option<ManyTests>) -> LoggedTier {
        LoggedTier { tier: b"main".as_slice().into(), verdict: LoggedVerdict::Red { run: 7, failing, tests, many } }
    }

    #[test]
    fn logged_red_checks_keep_whole_ids_and_remainders() {
        let tests: Box<[TestId]> = Box::new([b"suite::a".as_slice().into(), b"suite::b".as_slice().into()]);
        let names = ManyTests {
            tests: Box::new([b"suite::t00".as_slice().into(), b"suite::t01".as_slice().into()]),
            more: 21,
            cut: false,
        };
        for (entry, bound, expected) in [
            (red(tests.clone(), Some(2), None), 1024, "main run 7 found this landing red: suite::a, suite::b."),
            (red(tests.clone(), Some(70), None), 60, "main run 7 found this landing red: suite::a, and 69 more."),
            (red(Box::new([]), None, None), 1024, "main run 7 found this landing red: many tests."),
            (
                red(Box::new([]), None, Some(names.clone())),
                1024,
                "main run 7 found this landing red: many tests; its report names suite::t00, suite::t01, and 21 more.",
            ),
            (
                red(Box::new([]), None, Some(names.clone())),
                88,
                "main run 7 found this landing red: many tests; its report names suite::t00, and 22 more.",
            ),
            (
                red(Box::new([]), None, Some(ManyTests { cut: true, ..names })),
                1024,
                "main run 7 found this landing red: many tests; its report names suite::t00, suite::t01, and 21 more, its report cut at the read's bound.",
            ),
        ] {
            let mut statuses: Box<[api::Status]> = Box::new([]);
            append_tier_checks(&mut statuses, &[entry], None, 8, bound, 65_536);
            assert_eq!(statuses.len(), 1);
            let row = &statuses[0];
            assert_eq!(row.context.as_ref(), b"temper/verdict/main");
            assert_eq!(row.check, api::Check::Failed);
            assert_eq!(row.description.as_ref(), expected.as_bytes());
            assert!(row.description.len() <= skein_lib::usize_of(bound));
            assert!(row.url.is_empty());
            assert_eq!(row.job, None);
        }
        let mut names = List::with_capacity(crate::LOGGED_TESTS);
        for at in 0..crate::LOGGED_TESTS {
            names.push(Box::from(format!("suite::t{at:02}").as_bytes())).unwrap();
        }
        let mut statuses: Box<[api::Status]> = Box::new([]);
        append_tier_checks(&mut statuses, &[red(names.into_boxed(), Some(70), None)], None, 8, 4096, 65_536);
        assert!(statuses[0].description.ends_with(b"suite::t63, and 6 more."));
    }

    #[test]
    fn tier_checks_give_way_to_gate_jobs_and_displace_forge_rows_within_the_answer() {
        let forge = api::Status {
            at: skein_lib::Time::ZERO,
            context: b"own".as_slice().into(),
            check: api::Check::Passed,
            description: Box::new([]),
            url: Box::new([]),
            job: None,
        };
        let gate = LandingFailure {
            report: crate::ReceiptReport::Unset,
            gate: b"gate".as_slice().into(),
            run: crate::ReceiptRun { id: 11, number: 3, url: b"run-url".as_slice().into() },
            prefix: [7; 32],
            base: None,
            jobs: Box::new([crate::ReceiptJob { job: 17, attempt: 2, name: b"test".as_slice().into() }]),
        };
        let red = red(Box::new([Box::new([b'x'; 512]), Box::new([b'y'; 512])]), Some(2), None);
        let mut second = red.clone();
        second.tier = b"fuzzy".as_slice().into();
        for (rows, expected) in [
            (4, &[b"own".as_slice(), b"temper/gate/test", b"temper/verdict/main", b"temper/verdict/fuzzy"][..]),
            (3, &[b"temper/gate/test".as_slice(), b"temper/verdict/main", b"temper/verdict/fuzzy"][..]),
            (2, &[b"temper/gate/test".as_slice(), b"temper/verdict/main"][..]),
            (1, &[b"temper/gate/test".as_slice()][..]),
        ] {
            let mut statuses: Box<[api::Status]> = Box::new([forge.clone(), forge.clone()]);
            append_tier_checks(&mut statuses, &[red.clone(), second.clone()], Some(gate.clone()), rows, 8192, 65_536);
            assert_eq!(statuses.len(), expected.len());
            for (row, context) in statuses.iter().zip(expected) {
                assert_eq!(row.context.as_ref(), *context);
            }
            let mut found = false;
            for row in &statuses {
                if row.job.is_some() {
                    found = true;
                    assert_eq!(row.job, Some(api::JobAttempt { head: [7; 32], run: 11, job: 17, attempt: 2 }));
                    assert_eq!(
                        row.description.as_ref(),
                        b"Row bound: gate jobs take precedence; remaining checks omitted."
                    );
                }
            }
            assert!(found);
        }
        let mut statuses: Box<[api::Status]> = Box::new([forge]);
        append_tier_checks(&mut statuses, &[red, second], None, 8, 8192, 512);
        let mut bytes = 0_usize;
        for status in &statuses {
            bytes += status_text_bytes(status);
        }
        assert!(bytes <= 512);
        assert!(statuses[1].description.ends_with(b"and 2 more."));
        assert!(statuses[2].description.ends_with(b"and 2 more."));
    }

    #[test]
    fn gate_jobs_reserve_rows_and_report_displaced_forge_checks() {
        let failure = LandingFailure {
            report: crate::ReceiptReport::Unset,
            gate: b"gate".as_slice().into(),
            run: crate::ReceiptRun { id: 11, number: 3, url: b"https://forge.example/run/3".as_slice().into() },
            prefix: [7; 32],
            base: None,
            jobs: Box::new([crate::ReceiptJob { job: 17, attempt: 2, name: b"test".as_slice().into() }]),
        };
        let first = api::Status {
            at: skein_lib::Time::ZERO,
            context: b"build".as_slice().into(),
            check: api::Check::Passed,
            description: Box::new([]),
            url: Box::new([]),
            job: None,
        };
        let second = api::Status { context: b"fmt".as_slice().into(), ..first.clone() };
        for (rows, kept) in [(3, 2), (2, 1), (1, 0)] {
            let mut statuses: Box<[api::Status]> = Box::new([first.clone(), second.clone()]);
            append_gate_checks(&mut statuses, failure.clone(), rows);
            assert_eq!(statuses.len(), kept + 1);
            let (gate, own) = statuses.split_last().expect("gate job");
            assert_eq!(own, &([first.clone(), second.clone()])[..kept]);
            assert_eq!(gate.context.as_ref(), b"temper/gate/test");
            assert_eq!(gate.check, api::Check::Failed);
            assert_eq!(gate.url, failure.run.url);
            assert_eq!(gate.job, Some(api::JobAttempt { head: failure.prefix, run: 11, job: 17, attempt: 2 }));
            assert_eq!(gate.description.is_empty(), kept == 2);
            if kept < 2 {
                assert_eq!(
                    gate.description.as_ref(),
                    b"Row bound: gate jobs take precedence; remaining checks omitted."
                );
            }
        }
    }
}

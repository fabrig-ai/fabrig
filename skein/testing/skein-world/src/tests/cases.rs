use core::cell::RefCell;

use crate::faults::FaultLedger;
use crate::sweep::Sweep;
use crate::sweep::cases::{Case, Test, cases_in, dispatch_hook, input_in};
use crate::test_env::Mode;

const TEST: Test = Test::new("p", "m", "n", None);
const PART: Test = Test::new("p", "m", "n", Some("a"));
const CAMPAIGN: Mode = Mode::Campaign(0x0123_4567_89ab_cdef);
const FRESH: [u64; 2] = [0x399b_5d25_0b13_cc00, 0xe903_d5ea_5f24_c5ac];

struct Run {
    seeds: Vec<u64>,
    lines: Vec<String>,
    count: Option<u64>,
    result: Result<(), Case<&'static str>>,
}

fn judge<S: IntoIterator<Item = u64>>(mode: Mode, test: Test, seeds: S, fail_at: Option<u64>) -> Run {
    let mut seen = Vec::new();
    let mut lines = Vec::new();
    let mut count = None;
    let result = cases_in(
        mode,
        test,
        seeds,
        |seed| {
            seen.push(seed);
            if fail_at == Some(seed) { Err("boom") } else { Ok(()) }
        },
        |line| lines.push(line.to_string()),
        |ran| count = Some(ran),
    );
    Run { seeds: seen, lines, count, result }
}

#[test]
fn test_identity_uses_the_calling_module_and_optional_part() {
    assert_eq!(TEST.to_string(), "p::m::n");
    assert_eq!(PART.to_string(), "p::m::n/a");
    assert_eq!(crate::this_test!("n").to_string(), "skein-world::skein_world::tests::cases::n");
    assert_eq!(crate::this_test!("n", "a").to_string(), "skein-world::skein_world::tests::cases::n/a");
}

#[test]
fn gate_runs_given_seeds_in_order_without_a_summary() {
    assert!(Mode::Gate.gate(), "gate configuration for the given seeds");
    let run = judge(Mode::Gate, TEST, [5, 3, 9], None);
    run.result.expect("all gate cases pass");
    assert_eq!(run.seeds, [5, 3, 9]);
    assert_eq!(run.count, Some(3));
    assert!(run.lines.is_empty(), "gate success writes no lines");
    let empty = judge(Mode::Gate, TEST, [], None);
    empty.result.expect("an empty sweep passes");
    assert_eq!(empty.count, None);
}

#[test]
fn campaign_seeds_depend_only_on_campaign_identity_and_index() {
    assert!(!CAMPAIGN.gate(), "campaign configuration for fresh seeds");
    let first = judge(CAMPAIGN, TEST, [5, 3], None);
    first.result.expect("both campaign cases pass");
    assert_eq!(first.seeds, FRESH);
    assert_eq!(first.lines, ["fuzzy cases: p::m::n 2"]);
    assert_eq!(first.count, Some(2));
    let part = judge(CAMPAIGN, PART, [5], None);
    part.result.expect("the part's case passes");
    assert_eq!(part.seeds, [0xb782_189f_7cd9_e98b]);
    let other = judge(CAMPAIGN, Test::new("p", "m", "other", None), [5, 3], None);
    other.result.expect("the other test's cases pass");
    assert_ne!(other.seeds, FRESH);
    assert_eq!(judge(CAMPAIGN, TEST, [88, 99], None).seeds, FRESH);
    assert_eq!(judge(Mode::Campaign(0), TEST, [5], None).seeds, [0xad17_4649_fc7f_5b07]);
}

#[test]
fn replay_runs_once_even_without_gate_seeds() {
    let replay = Mode::Replay(42);
    assert!(!replay.gate(), "replay uses the campaign's configuration");
    for seeds in [0..64, 0..0] {
        let run = judge(replay, TEST, seeds, None);
        run.result.expect("the replay passes");
        assert_eq!(run.seeds, [42]);
        assert_eq!(run.lines, ["fuzzy cases: p::m::n 1"]);
        assert_eq!(run.count, Some(1));
    }
}

const INPUT_TEST: Test = Test::new("pkg", "suite::fuzzy::walk", "walks", None);
const INPUT_SEED: u64 = 0xab;
const INPUT_REPLAY: Mode = Mode::Replay(INPUT_SEED);
const INPUT_DIGEST: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12,
    0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];

fn record_input(mode: Mode, digest: [u8; 32], lines: &mut Vec<String>) {
    input_in(|| mode, digest, |line| lines.push(line.to_string()));
}

fn in_input_case(mode: Mode, test: Test, mut check: impl FnMut()) {
    cases_in(
        mode,
        test,
        [INPUT_SEED],
        |_| {
            check();
            Ok::<(), &'static str>(())
        },
        |_| {},
        |_| {},
    )
    .expect("the input check passes");
}

#[test]
fn replay_reports_input_with_case_identity_and_hex_digest() {
    for (test, digest, expected) in [
        (
            INPUT_TEST,
            INPUT_DIGEST,
            "fuzzy input: pkg::suite::fuzzy::walk::walks seed 00000000000000ab 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        ),
        (
            Test::new("pkg", "suite::fuzzy::walk", "walks", Some("part")),
            [0xff; 32],
            "fuzzy input: pkg::suite::fuzzy::walk::walks/part seed 00000000000000ab ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        ),
    ] {
        let mut lines = Vec::new();
        in_input_case(INPUT_REPLAY, test, || record_input(INPUT_REPLAY, digest, &mut lines));
        assert_eq!(lines, [expected]);
    }
}

#[test]
fn gate_and_campaign_never_report_input() {
    for mode in [Mode::Gate, CAMPAIGN] {
        let mut lines = Vec::new();
        record_input(mode, INPUT_DIGEST, &mut lines);
        in_input_case(mode, INPUT_TEST, || record_input(mode, INPUT_DIGEST, &mut lines));
        record_input(mode, INPUT_DIGEST, &mut lines);
        assert!(lines.is_empty(), "unexpected input in {mode:?}: {lines:?}");
    }
}

#[test]
fn replay_never_reports_input_outside_a_check() {
    let mut lines = Vec::new();
    record_input(INPUT_REPLAY, INPUT_DIGEST, &mut lines);
    in_input_case(INPUT_REPLAY, INPUT_TEST, || {});
    record_input(INPUT_REPLAY, INPUT_DIGEST, &mut lines);
    assert!(lines.is_empty(), "unexpected input outside a replay check: {lines:?}");
}

#[test]
fn a_finding_stops_the_sweep_and_names_the_case() {
    for (mode, seed, seen, line, summary) in [
        (Mode::Gate, 3, &[5, 3][..], "fuzzy case: p::m::n seed 0000000000000003: boom", None),
        (
            CAMPAIGN,
            FRESH[0],
            &FRESH[..1],
            "fuzzy case: p::m::n seed 399b5d250b13cc00: boom",
            Some("fuzzy cases: p::m::n 1"),
        ),
        (
            Mode::Replay(42),
            42,
            &[42][..],
            "fuzzy case: p::m::n seed 000000000000002a: boom",
            Some("fuzzy cases: p::m::n 1"),
        ),
    ] {
        let run = judge(mode, TEST, [5, 3, 9], Some(seed));
        assert_eq!(run.seeds, seen);
        assert_eq!(run.count, Some(u64::try_from(seen.len()).expect("fixture count fits u64")));
        let case = run.result.expect_err("the check found a failure");
        assert_eq!((case.test, case.seed, case.finding), (TEST, seed, "boom"));
        assert_eq!(case.to_string(), line);
        assert!(
            run.lines.iter().map(String::as_str).eq(core::iter::once(line).chain(summary)),
            "unexpected case output: {}",
            run.lines.join("\n")
        );
    }
}

#[test]
fn folding_ledgers_counts_only_the_cases_in_each_mode() {
    const GATE_CASES: u64 = 3;
    const REPLAY_CASES: u64 = 1;
    let ledger = FaultLedger::new();
    for (mode, expected) in [(Mode::Gate, GATE_CASES), (CAMPAIGN, GATE_CASES), (Mode::Replay(42), REPLAY_CASES)] {
        let mut sweep = Sweep::new("folding");
        let counts = RefCell::new(Vec::new());
        let count = |n| counts.borrow_mut().push(n);
        cases_in(
            mode,
            TEST,
            0..GATE_CASES,
            |_| {
                sweep.ran_in(&ledger, count);
                Ok::<_, &str>(())
            },
            |_| {},
            count,
        )
        .expect("all cases pass");
        assert_eq!(counts.into_inner(), [expected], "{mode:?}");
    }
}

#[test]
fn folding_ledgers_counts_each_call_before_and_after_a_case() {
    const ONE_FOLD: u64 = 1;
    let ledger = FaultLedger::new();
    for result in [Ok(()), Err("boom")] {
        let mut sweep = Sweep::new("folding");
        let mut counts = Vec::new();
        sweep.ran_in(&ledger, |n| counts.push(n));
        let _case = cases_in(Mode::Gate, TEST, [3], |_| result, |_| {}, |_| {});
        sweep.ran_in(&ledger, |n| counts.push(n));
        assert_eq!(counts, [ONE_FOLD, ONE_FOLD], "check returned {result:?}");
    }
}

#[test]
fn hook_names_only_the_active_case_and_then_chains() {
    let lines = RefCell::new(Vec::new());
    let dispatch = || {
        dispatch_hook(|line| lines.borrow_mut().push(line.to_string()), || lines.borrow_mut().push("previous".into()));
    };
    for result in [Ok(()), Err("boom")] {
        let after = cases_in(
            Mode::Gate,
            TEST,
            [3],
            |_| {
                dispatch();
                result
            },
            |_| {},
            |_| {},
        );
        assert_eq!(after.map_err(|case| case.finding), result);
        dispatch();
        assert_eq!(lines.take(), ["fuzzy case: p::m::n seed 0000000000000003", "previous", "previous"]);
    }
}

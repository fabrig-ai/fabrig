use core::cell::RefCell;

use crate::sweep::cases::{Case, Test, cases_in, dispatch_hook};
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

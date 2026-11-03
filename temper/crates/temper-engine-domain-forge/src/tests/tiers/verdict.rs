//! Verdict presentation from retained policy facts (domain/after-landing.md, 5.3 and 10).
use super::*;

fn verdict(blames: Box<[p::Blame]>, failing: p::Failures) -> p::Verdict {
    p::Verdict { number: 3, tip: [9; 32], key: [9; 32], failing, landings: blames, flakes: Box::new([]) }
}

#[test]
fn singular_verdicts_name_one_test_and_one_landing() {
    let failures = p::Failures::Tests(Box::new([Box::from(b"no-break".as_slice())]));
    let red = verdict(Box::new([p::Blame::Owns(failures.clone())]), failures);
    assert_eq!(verdict_summary(b"slow", &red).as_ref(), b"Tier slow run 3: red, 1 test failing, blamed on 1 landing");
    let green = verdict(Box::new([p::Blame::Cleared]), p::Failures::Tests(Box::new([])));
    assert_eq!(verdict_summary(b"slow", &green).as_ref(), b"Tier slow run 3: green, cleared 1 landing");
}

#[test]
fn inherited_failures_blame_no_landing_and_many_stays_explicit() {
    let failures = p::Failures::Tests(Box::new([Box::from(b"inherited".as_slice())]));
    let red = verdict(Box::new([p::Blame::Cleared]), failures);
    assert_eq!(
        verdict_summary(b"slow", &red).as_ref(),
        b"Tier slow run 3: red, 1 test failing, blamed on 0 landings; 1 inherited test"
    );
    let many = verdict(Box::new([p::Blame::Suspect(p::Failures::Many)]), p::Failures::Many);
    assert_eq!(
        verdict_summary(b"slow", &many).as_ref(),
        b"Tier slow run 3: red, many tests failing, blamed on 1 landing"
    );
}

#[test]
fn quotes_and_line_breaks_keep_an_id_in_one_attribution_line() {
    assert_eq!(report_id(b"a\nb\r\\\"").as_ref(), b"\"a\\nb\\r\\\\\\\"\"");
}

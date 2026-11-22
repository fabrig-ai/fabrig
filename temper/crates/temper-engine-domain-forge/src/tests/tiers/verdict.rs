//! Verdict presentation from retained policy facts (domain/after-landing.md, 5.3, and domain/after-landing-storage.md, 1).
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
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "literal report expectations use ordinary test containers and text"
)]
fn a_many_verdict_names_its_report_after_its_attributions() {
    let red = verdict(Box::new([p::Blame::Owns(p::Failures::Many)]), p::Failures::Many);
    let landings =
        [crate::TierPostedLanding { commit: [9; 32], blame: p::Blame::Owns(p::Failures::Many), change: Some(8) }];
    let many = p::ManyTests {
        tests: (0_u8..16).map(|at| format!("suite::t{at:02}").into_bytes().into_boxed_slice()).collect(),
        more: 21,
        cut: false,
    };
    let ids = (0_u8..16).map(|at| format!("\"suite::t{at:02}\"")).collect::<Vec<_>>().join(", ");
    let without = posted_words(b"main", &red, &landings, None);
    let expected = format!("{}\nIts report names: {ids} and 21 more.", core::str::from_utf8(&without).unwrap());
    assert_eq!(posted_words(b"main", &red, &landings, Some(&many)).as_ref(), expected.as_bytes());
    let escaped = p::ManyTests { tests: Box::new([b"a\nb\r\\\"".as_slice().into()]), more: 0, cut: true };
    assert!(
        posted_words(b"main", &red, &landings, Some(&escaped))
            .ends_with(b"\nIts report names: \"a\\nb\\r\\\\\\\"\" and more, its report cut at the read's bound.")
    );
    let empty = p::ManyTests { tests: Box::new([]), more: 21, cut: true };
    assert_eq!(posted_words(b"main", &red, &landings, Some(&empty)), without);
}

#[test]
fn quotes_and_line_breaks_keep_an_id_in_one_attribution_line() {
    assert_eq!(report_id(b"a\nb\r\\\"").as_ref(), b"\"a\\nb\\r\\\\\\\"\"");
}

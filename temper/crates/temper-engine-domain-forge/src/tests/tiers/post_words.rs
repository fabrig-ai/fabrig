//! Section 9's literal projections, bounded as the forge client bounds them.
#![expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "ordinary projection table rows use test containers")]
use super::*;
use CommentKind::{Verdict, Repaired};

fn tests(ids: &[&[u8]]) -> p::Failures {
    p::Failures::Tests(ids.iter().map(|id| Box::from(*id)).collect())
}
#[test]
fn status_words() {
    let owned = p::Blame::Owns(tests(&[b"x"]));
    let rows = [
        (tests(&[]), vec![], false, api::Check::Passed, "run 3 passed"),
        (tests(&[]), vec![], true, api::Check::Passed, "run 3 passed by receipt"),
        (tests(&[b"x", b"y"]), vec![owned.clone(), p::Blame::Suspect(tests(&[b"x"]))], false, api::Check::Failed,
            "run 3 failed: 1 blamed, 1 inherited"),
        (p::Failures::Many, vec![p::Blame::Owns(p::Failures::Many)], false, api::Check::Failed,
            "run 3 failed: many blamed, 0 inherited"),
        (p::Failures::Many, vec![owned], false, api::Check::Failed,
            "run 3 failed: 1 blamed, many inherited"),
    ];
    for (failing, landings, received, check, words) in rows {
        let verdict = p::Verdict { number: 3, tip: [9; 32], key: [8; 32], failing,
            landings: landings.into_boxed_slice(), flakes: Box::new([]) };
        assert_eq!(status(&verdict, received), (check, Box::from(words.as_bytes())));
    }
    let long = format!("{}éfin", "a".repeat(139));
    assert_eq!(description(long.as_bytes()).as_ref(), "a".repeat(139).as_bytes());
    assert_eq!(description("é".repeat(70).as_bytes()).len(), 140);
}

#[test]
fn comment_words() {
    let owns = p::Blame::Owns(tests(&[b"x", b"y"]));
    let rows = [
        (p::Blame::Cleared, Verdict, "https://forge/run", usize::MAX, "Cleared by slow run 3.\n\nRun: https://forge/run"),
        (owns.clone(), Verdict, "", usize::MAX, "Red: slow run 3 blames this landing for x, y."),
        (p::Blame::Owns(p::Failures::Many), Verdict, "", usize::MAX, "Red: slow run 3 blames this landing for many tests."),
        (p::Blame::Suspect(tests(&[b"x"])), Verdict, "", usize::MAX, "Suspected: slow run 3 suspects this landing of x."),
        (p::Blame::Cleared, Repaired { by: Some(19) }, "", usize::MAX, "Repaired by #19: slow run 3 passed the failures blamed on this landing."),
        (p::Blame::Cleared, Repaired { by: None }, "", usize::MAX, "Repaired: slow run 3 passed the failures blamed on this landing."),
        (owns, Verdict, "", 50, "Red: slow run 3 blames this landing for x, y."),
        (p::Blame::Owns(tests(&[b"first-test-is-too-long", b"second"])), Verdict, "", 51,
            "Red: slow run 3 blames this landing for and 2 more."),
        (p::Blame::Owns(tests(&[b"x", b"a-long-test-id-that-will-not-fit", b"y"])), Verdict, "", 60,
            "Red: slow run 3 blames this landing for x, and 2 more."),
    ];
    for (blame, repair, link, room, expected) in rows {
        let body = comment(b"slow", 3, &blame, repair, link.as_bytes(), room);
        assert_eq!(body.as_ref(), expected.as_bytes());
        assert!(body.len() <= room);
    }
}

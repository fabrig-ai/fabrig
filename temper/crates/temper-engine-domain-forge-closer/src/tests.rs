//! Goal report composition and byte bounds (temper/docs/design/domain/forge.md, 11.2).
#![expect(clippy::disallowed_macros, reason = "test fixtures use formatted expected words and vectors")]
#![expect(clippy::disallowed_methods, reason = "tests inspect UTF-8 and construct fixture lists")]

use crate::{
    LOG_START_BYTES, Landed, Landing, Report, SELECTION_LINE_BYTES, SELECTION_PREFIX, Selection, Unlanded, Why,
    first_line, left_section, selection_line, words,
};
use alloc::boxed::Box;
use skein_lib::Rng;
use skein_lib::bytes::{concat, utf8_prefix};

fn empty_report() -> Report {
    Report {
        goal: 7,
        title: Box::from(b"TITLE".as_slice()),
        landed: Box::new([]),
        unlanded: Box::new([]),
        landing: Landing::NoBranch,
        selections: Box::new([]),
    }
}

fn landed(task: u64, title: &[u8], pull: u64, left: Option<&[u8]>) -> Landed {
    Landed {
        task,
        title: Box::from(title),
        pull,
        branch: Box::from(b"temper/1/g7".as_slice()),
        left: left.map(Box::from),
    }
}

#[test]
fn a_report_lists_landed_and_unlanded_changes_the_landing_and_ci_selection() {
    let mut report = empty_report();
    report.title = Box::from(b"Add OAuth login".as_slice());
    report.landed = Box::new([
        landed(9, b"Parser", 41, left_section(b"Done.\nLeft: two items\n- a flaky test in x\n## Checks\nok")),
        landed(12, b"Lexer", 42, None),
    ]);
    report.unlanded = Box::new([
        Unlanded { task: 10, title: Box::from(b"Docs".as_slice()), why: Why::Cancelled },
        Unlanded {
            task: 11,
            title: Box::from(b"Retry".as_slice()),
            why: Why::Failed(Box::from(first_line(b"build broke\nmore"))),
        },
        Unlanded {
            task: 13,
            title: Box::from(b"Noop".as_slice()),
            why: Why::NoChange(Box::from(b"Already there.".as_slice())),
        },
    ]);
    report.landing = Landing::Landed {
        pull: 50,
        base: Box::from(b"main".as_slice()),
        commit: Box::from(b"0123abcd".as_slice()),
        repairs: Box::new([20, 21]),
        resolutions: Box::new([22]),
    };
    report.selections = Box::new([
        Selection { pull: 50, line: selection_line(b"selection: full\nrest").map(Box::from) },
        Selection { pull: 51, line: None },
    ]);
    assert_eq!(words(&report, 8192).as_ref(), b"Goal T7: Add OAuth login\nLanded:\n- T9 Parser: pull request #41 into temper/1/g7\n  Left: two items\n  - a flaky test in x\n- T12 Lexer: pull request #42 into temper/1/g7\n  Left: none given\nEnded without landing:\n- T10 Docs: cancelled\n- T11 Retry: failed: build broke\n- T13 Noop: no change: Already there.\nThe landing: pull request #50, merged into main as 0123abcd; repairs T20, T21; resolutions T22.\nCI selection:\n- pull request #50: selection: full\n- pull request #51: unknown\n");
}

#[test]
fn empty_parts_are_left_out() {
    let mut report = empty_report();
    assert_eq!(words(&report, 8192).as_ref(), b"Goal T7: TITLE\nThe landing: none, no goal branch.\n");
    for (landing, expected) in [
        (Landing::Held, "none, its base already held the goal's branch"),
        (Landing::Ended { pull: Some(50), task: 30, why: Why::Cancelled }, "none, pull request #50 ended cancelled"),
        (Landing::Ended { pull: None, task: 30, why: Why::Cancelled }, "none, T30 ended cancelled"),
        (
            Landing::Ended { pull: None, task: 30, why: Why::Failed(Box::from(b"bad".as_slice())) },
            "none, T30 ended failed: bad",
        ),
        (
            Landing::Ended { pull: None, task: 30, why: Why::NoChange(Box::from(b"done".as_slice())) },
            "none, T30 ended no change: done",
        ),
        (
            Landing::Landed {
                pull: 50,
                base: Box::from(b"main".as_slice()),
                commit: Box::from(b"abc".as_slice()),
                repairs: Box::new([]),
                resolutions: Box::new([]),
            },
            "pull request #50, merged into main as abc",
        ),
    ] {
        report.landing = landing;
        assert_eq!(words(&report, 8192).as_ref(), format!("Goal T7: TITLE\nThe landing: {expected}.\n").as_bytes());
    }
}

#[test]
fn a_left_section_runs_from_its_heading_to_the_next_heading() {
    for (input, expected) in [
        (b"Done.\n## Left\n- a\n- b\n## Checks\nok".as_slice(), Some(b"## Left\n- a\n- b".as_slice())),
        (b"x\n**Left:** none\n", Some(b"**Left:** none".as_slice())),
        (b"Left", Some(b"Left".as_slice())),
        (b"  *# Left: x\ncontinued\n", Some(b"  *# Left: x\ncontinued".as_slice())),
        (b"Leftover: x", None),
        (b"no leftovers\n", None),
    ] {
        assert_eq!(left_section(input), expected, "{input:?}");
    }
}

#[test]
fn a_selection_line_is_the_logs_first_line() {
    assert_eq!(LOG_START_BYTES, 4096);
    assert_eq!(SELECTION_PREFIX, b"selection: ");
    assert_eq!(SELECTION_LINE_BYTES, 256);
    assert_eq!(selection_line(b"selection: docs\nrest"), Some(b"selection: docs".as_slice()));
    assert_eq!(selection_line(b"selection: docs\r\nrest"), Some(b"selection: docs".as_slice()));
    assert_eq!(selection_line(b"other\nselection: full"), None);
    assert_eq!(selection_line(b""), None);
    assert_eq!(first_line(b"no newline"), b"no newline");
    assert_eq!(first_line(b"\r\nrest"), b"");
    let line = concat(&[SELECTION_PREFIX, "é".repeat(144).as_bytes(), b"x"]);
    assert_eq!(line.len(), 300);
    let selection = selection_line(&line).expect("selection prefix");
    assert_eq!(selection.len(), 255);
    assert_eq!(selection, utf8_prefix(&line, 256));
}

fn with_lefts() -> Report {
    let mut report = empty_report();
    let left = format!("## Left\n{}", "é".repeat(100));
    report.landed =
        Box::new([landed(9, b"Parser", 41, Some(left.as_bytes())), landed(12, b"Lexer", 42, Some(left.as_bytes()))]);
    report
}

#[test]
fn left_sections_are_cut_first_from_the_last_change_back() {
    let report = with_lefts();
    let full = words(&report, 8192);
    let section = concat(&[b"  ## Left\n  ", "é".repeat(100).as_bytes(), b"\n"]);
    let replacement = format!("  Left: [{} bytes cut]\n", section.len());
    let savings = section.len() - replacement.len();
    let bound = u32::try_from(full.len() - savings).expect("fixture bound");
    let text = core::str::from_utf8(&full).expect("UTF-8 fixture");
    let last = text.rfind(core::str::from_utf8(&section).expect("UTF-8 section")).expect("last section");
    let expected = concat(&[&full[..last], replacement.as_bytes(), &full[last + section.len()..]]);
    assert_eq!(words(&report, bound), expected);
    let both = text.replace(core::str::from_utf8(&section).expect("UTF-8 section"), &replacement);
    assert_eq!(words(&report, bound - 1).as_ref(), both.as_bytes());
    assert_eq!(words(&report, u32::try_from(full.len()).expect("fixture bound")), full);
}

#[test]
fn a_report_past_its_bound_keeps_its_first_bytes_and_says_what_it_cut() {
    let report = with_lefts();
    let full = words(&report, 8192);
    let section = concat(&[b"  ## Left\n  ", "é".repeat(100).as_bytes(), b"\n"]);
    let replacement = format!("  Left: [{} bytes cut]\n", section.len());
    let reduced = core::str::from_utf8(&full)
        .expect("UTF-8 fixture")
        .replace(core::str::from_utf8(&section).expect("UTF-8 section"), &replacement);
    let bound = 96_usize;
    let expected = (0..=bound)
        .rev()
        .find_map(|limit| {
            let kept = utf8_prefix(reduced.as_bytes(), limit);
            let dropped = section.len() * 2 + reduced.len() - kept.len();
            let marker = format!("\n[{dropped} bytes cut]\n");
            (kept.len() + marker.len() <= bound).then(|| concat(&[kept, marker.as_bytes()]))
        })
        .expect("marker fits");
    assert_eq!(words(&report, 96), expected);
    let mut report = empty_report();
    report.title = Box::from("é".repeat(100).as_bytes());
    let full = words(&report, 8192);
    for bound in 0..=u32::try_from(full.len()).expect("fixture length") {
        let actual = words(&report, bound);
        assert!(actual.len() <= usize::try_from(bound).expect("fixture bound"));
        core::str::from_utf8(&actual).expect("cut words remain UTF-8");
        if bound < 16 {
            assert!(actual.is_empty());
        }
    }
}

#[test]
fn no_report_passes_its_bound() {
    const SEED: u64 = 0x72e3_a0cd_19b4_056f;
    let mut rng = Rng::new(SEED);
    for case in 0_u32..256 {
        let mut report = empty_report();
        let title_count = usize::try_from(rng.below(300)).expect("fixture count");
        report.title = Box::from("Aé界🦀".repeat(title_count).as_bytes());
        let mut landed_changes = skein_lib::List::uncapped();
        for task in 0..rng.below(12) {
            let count = usize::try_from(rng.below(300)).expect("fixture count");
            let left = concat(&[b"Left: ", "é界🦀".repeat(count).as_bytes()]);
            let section = if rng.below(3) == 0 { None } else { Some(left.as_ref()) };
            landed_changes.push(landed(task, &report.title, task, section)).expect("fixture list");
        }
        report.landed = landed_changes.into_boxed();
        let mut unlanded = skein_lib::List::uncapped();
        for task in 0..rng.below(8) {
            unlanded
                .push(Unlanded { task, title: report.title.clone(), why: Why::Failed(report.title.clone()) })
                .expect("fixture list");
        }
        report.unlanded = unlanded.into_boxed();
        let bound = u32::try_from(rng.between(64, 8192)).expect("fixture bound");
        let actual = words(&report, bound);
        assert!(
            actual.len() <= usize::try_from(bound).expect("fixture bound"),
            "seed {SEED:#x}, case {case}, bound {bound}"
        );
        assert!(core::str::from_utf8(&actual).is_ok(), "seed {SEED:#x}, case {case}");
        assert!(actual.starts_with(b"Goal T"), "seed {SEED:#x}, case {case}, bound {bound}");
    }
}

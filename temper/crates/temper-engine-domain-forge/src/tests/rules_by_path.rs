//! Rules cover paths at their judged head (domain/forge/change/gates.md, 2).
use super::*;
use temper_protocol_reaches::{Classes, PathSet};

#[test]
fn touched_counts_matches_missing_sets_and_incomplete_reads() {
    let classes = Classes {
        items: Box::new([]),
        sets: Box::new([
            PathSet {
                number: 1,
                name: b"protocol".as_slice().into(),
                patterns: Box::new([b"src/protocol.*".as_slice().into(), b"include/vsr.h".as_slice().into()]),
            },
            PathSet { number: 2, name: b"docs".as_slice().into(), patterns: Box::new([b"docs/**".as_slice().into()]) },
        ]),
    };
    for (path, whole, sets) in [
        (b"src/protocol.c".as_slice(), true, 0b11_1101),
        (b"README.md".as_slice(), true, 0b11_1100),
        (b"README.md".as_slice(), false, 0b11_1111),
    ] {
        assert_eq!(touched(&classes, &[path.into()], whole), Touched { sets, whole });
    }
}

#[test]
fn covers_requires_the_branch_and_any_named_touched_set() {
    let repository = adopted();
    for (sets, branch, touched, expected) in [
        ([].as_slice(), b"main".as_slice(), Touched { sets: 0, whole: true }, true),
        ([1].as_slice(), b"main".as_slice(), Touched { sets: 1, whole: true }, true),
        ([1].as_slice(), b"main".as_slice(), Touched { sets: 2, whole: true }, false),
        ([1].as_slice(), b"main".as_slice(), Touched::UNKNOWN, true),
        ([1].as_slice(), b"other".as_slice(), Touched::UNKNOWN, false),
    ] {
        let rule = crate::settings::LandingRule {
            sets: sets.into(),
            pattern: GatePattern {
                segments: Box::new([b"branch".as_slice().into()]),
                last: GateLast::Exact(b"main".as_slice().into()),
            },
            enforce: true,
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([]),
        };
        assert_eq!(covers(&rule, &repository, branch, touched), expected, "{rule:?}, {branch:?}, {touched:?}");
    }
}

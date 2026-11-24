//! Fault vocabulary value contracts (`temper/docs/design/testing/restarts.md`, section 10;
//! `temper/docs/design/testing/workloads.md`, section 2.2).

use std::collections::BTreeSet;

use skein_world::faults::Kind;

use crate::restarts::*;

// Section 2 names sixteen constants, apart from the twelve worker pairs.
const NAMED: [(Kind, &str); 16] = [
    (ENGINE_CRASH, "engine crash"),
    (SETTLED_STOP, "settled stop"),
    (STOP_BEFORE_MARK, "stop before the mark"),
    (STOP_BEFORE_OPENING, "stop before the opening"),
    (STOP_AFTER_OPENING, "stop after the opening"),
    (SECOND_SIGNAL, "second signal"),
    (FORGE_DOWN_UNTIL, "forge down until a moment"),
    (FORGE_DOWN_FOR_GOOD, "forge down for good"),
    (ENTRIES_HELD_BY_ENDED, "write entries held by ended owners"),
    (BRANCH_THE_FORGE_LACKS, "held branch the forge lacks"),
    (FIND_KEEPS_FAILING, "uncertain write's find failing"),
    (READ_ALWAYS_LIMITED, "read limited at every attempt"),
    (ENGINE_RESTART_MID_TURN, "engine restarted mid-turn"),
    (ENGINE_CRASH_MID_TURN, "engine crashed mid-turn"),
    (UNIT_SIGNALLED, "unit signalled mid-turn"),
    (OLDER_LAYOUT, "older layout"),
];

#[test]
fn every_kind_and_stream_part_belongs_to_restarts() {
    assert_eq!(FAMILY, "restarts");
    assert_eq!(PART, "restarts");
    for kind in NAMED.iter().map(|(kind, _)| *kind).chain(WORKER) {
        assert_eq!(kind.family, FAMILY, "{}", kind.name);
    }
}

#[test]
fn named_kinds_keep_their_literal_names() {
    for (kind, name) in NAMED {
        assert_eq!(kind.name, name);
    }
}

#[test]
fn all_twenty_eight_kind_names_are_distinct() {
    // Section 2's sixteen named constants and twelve worker pairs.
    const KIND_COUNT: usize = 28;
    let mut names = BTreeSet::new();
    for kind in NAMED.iter().map(|(kind, _)| *kind).chain(WORKER) {
        assert!(names.insert(kind.name), "duplicate kind: {}", kind.name);
    }
    assert_eq!(names.len(), KIND_COUNT);
}

#[test]
fn worker_pairs_have_twelve_distinct_descriptive_names() {
    // Three ways times four points, as section 2 fixes them.
    const WORKER_COUNT: usize = 12;
    let mut kinds = BTreeSet::new();
    for (way, way_name) in [(Way::Drained, "drained"), (Way::Signalled, "signalled"), (Way::Crashed, "crashed")] {
        for (point, point_name) in [
            (Point::LlmCall, "an LLM call"),
            (Point::ToolCall, "a tool call"),
            (Point::DeliveryCheck, "a delivery check"),
            (Point::Save, "a save"),
        ] {
            let kind = worker(way, point);
            assert_eq!(kind.family, "restarts");
            assert_eq!(kind.name, format!("worker {way_name} in {point_name}"));
            assert!(kinds.insert(kind), "duplicate worker kind: {kind:?}");
        }
    }
    assert_eq!(kinds.len(), WORKER_COUNT);
}

#[test]
fn world_groups_keep_only_their_kinds_in_order() {
    let groups: [(&str, &[Kind], &[Kind]); 5] = [
        ("engine", &ENGINE, &[ENGINE_CRASH, SETTLED_STOP, STOP_BEFORE_MARK, STOP_AFTER_OPENING, SECOND_SIGNAL]),
        (
            "forge",
            &FORGE,
            &[
                ENGINE_CRASH,
                FORGE_DOWN_UNTIL,
                FORGE_DOWN_FOR_GOOD,
                ENTRIES_HELD_BY_ENDED,
                BRANCH_THE_FORGE_LACKS,
                FIND_KEEPS_FAILING,
                READ_ALWAYS_LIMITED,
            ],
        ),
        (
            "worker",
            &WORKER,
            &[
                worker(Way::Drained, Point::LlmCall),
                worker(Way::Drained, Point::ToolCall),
                worker(Way::Drained, Point::DeliveryCheck),
                worker(Way::Drained, Point::Save),
                worker(Way::Signalled, Point::LlmCall),
                worker(Way::Signalled, Point::ToolCall),
                worker(Way::Signalled, Point::DeliveryCheck),
                worker(Way::Signalled, Point::Save),
                worker(Way::Crashed, Point::LlmCall),
                worker(Way::Crashed, Point::ToolCall),
                worker(Way::Crashed, Point::DeliveryCheck),
                worker(Way::Crashed, Point::Save),
            ],
        ),
        ("mid-turn", &MID_TURN, &[ENGINE_RESTART_MID_TURN, ENGINE_CRASH_MID_TURN]),
        ("simulated", &SIMULATED, &[UNIT_SIGNALLED, OLDER_LAYOUT]),
    ];
    for (world, kinds, expected) in groups {
        assert_eq!(kinds, expected, "{world}");
    }
}

mod forge_and_stale {
    use std::collections::BTreeSet;

    use skein_world::faults::Kind;

    use crate::{forge::*, stale::*};

    const NAMED: [(Kind, &str, &str); 18] = [
        (NO_ROOM, "busy", "no_room"),
        (UNAVAILABLE, "busy", "unavailable"),
        (TIMED_OUT, "busy", "timed_out"),
        (LANDING, "busy", "landing"),
        (LATE, "busy", "late"),
        (LIMITED, "busy", "limited"),
        (HEAD_BEHIND, "lagging", "head_behind"),
        (BASE_BEHIND, "lagging", "base_behind"),
        (MERGE_LATE, "lagging", "merge_late"),
        (PULL_FILES, "bounds", "pull_files"),
        (COMPARISON, "bounds", "comparison"),
        (JOB_LOG, "bounds", "job_log"),
        (CLOSE_REFUSED, "refusing", "close_refused"),
        (CLOSED_BY_ANOTHER, "refusing", "closed_by_another"),
        (MERGED_BY_ANOTHER, "refusing", "merged_by_another"),
        (REOPENED, "stale", "reopened_by_another"),
        (SQUASHED, "stale", "squashed_dependency"),
        (MOVED_BASE, "stale", "moved_base"),
    ];

    #[test]
    fn moved_kinds_keep_their_literal_values() {
        assert_eq!(FAMILY, "stale");
        for (kind, family, name) in NAMED {
            assert_eq!(kind, Kind { family, name });
        }
    }

    #[test]
    fn moved_kind_names_are_unique_within_each_family() {
        let mut names = BTreeSet::new();
        for (kind, _, _) in NAMED {
            assert!(names.insert((kind.family, kind.name)), "duplicate kind: {kind:?}");
        }
    }

    #[test]
    fn sweep_groups_keep_their_members_in_order() {
        let groups: [(&str, &[Kind], &[Kind]); 5] = [
            ("busy", &BUSY, &[NO_ROOM, UNAVAILABLE, TIMED_OUT, LANDING, LATE, LIMITED]),
            ("lagging", &LAGGING, &[HEAD_BEHIND, BASE_BEHIND, MERGE_LATE]),
            ("bounds", &PAST_BOUNDS, &[PULL_FILES, COMPARISON, JOB_LOG]),
            ("refusing", &REFUSING, &[CLOSE_REFUSED, CLOSED_BY_ANOTHER, MERGED_BY_ANOTHER]),
            (
                "stale",
                &STALE,
                &[HEAD_BEHIND, MERGE_LATE, REOPENED, SQUASHED, MOVED_BASE, CLOSED_BY_ANOTHER, MERGED_BY_ANOTHER],
            ),
        ];
        for (sweep, kinds, expected) in groups {
            assert_eq!(kinds, expected, "{sweep}");
        }
    }
}

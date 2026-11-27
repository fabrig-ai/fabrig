//! Focused checks of the cell matrix and the pairing, without a world.

use std::collections::BTreeSet;

use skein_lib::Duration;

use super::*;

const JIG_COLUMNS: [(Kind, Column); 7] = [
    (jig::ANSWERED_WHILE_PARKED, Column::Resumes),
    (jig::ANSWERED_TO_ITS_SUCCESSOR, Column::Resumes),
    (jig::PARKS_ON_NOTHING, Column::Resumes),
    (jig::WATCHES_A_TASK_THAT_CAN_ANSWER, Column::Resumes),
    (jig::ANSWERED_AFTER_ITS_END, Column::Ends),
    (jig::COORDINATOR_ENDS_WHILE_A_CHILD_WAITS, Column::Ends),
    (jig::STUCK_IN_A_CIRCLE, Column::Ends),
];

#[test]
fn the_matrix_partitions_all_pairs_of_park_kinds() {
    // The three parked-answer states by the seven outcomes; the remaining pairs are impossible.
    const CELL_COUNT: usize = 21;
    const NEVER_COUNT: usize = 27;
    assert_eq!(CELLS.len(), CELL_COUNT);
    assert_eq!(NEVER.len(), NEVER_COUNT);
    for smith in smith::KINDS {
        let needs_only_a_park = [smith::WITHHELD_WAIT, smith::JOB_RUNNING, smith::HOST_CALL_ANSWERED].contains(&smith);
        for jig in JIG_COLUMNS.map(|(kind, _)| kind).into_iter().chain([jig::PARKS_WITH_A_QUESTION_OPEN]) {
            let can_pair = needs_only_a_park && jig != jig::PARKS_WITH_A_QUESTION_OPEN;
            let cells = CELLS.iter().filter(|cell| cell.smith == smith && cell.jig == jig).count();
            let never = NEVER.iter().filter(|pair| pair.smith == smith && pair.jig == jig).count();
            assert_eq!((cells, never), (usize::from(can_pair), usize::from(!can_pair)), "{smith:?}, {jig:?}");
        }
    }
}

#[test]
fn cell_names_join_the_halves_without_duplicates() {
    assert_eq!(FAMILY, "parks against wakes");
    let mut names = BTreeSet::new();
    for cell in CELLS {
        assert_eq!(cell.kind.family, FAMILY);
        assert_eq!(cell.kind.name, format!("{}, {}", cell.smith.name, cell.jig.name));
        assert!(names.insert(cell.kind.name), "duplicate cell name: {:?}", cell.kind.name);
    }
}

#[test]
fn cell_columns_follow_the_jig_outcome() {
    for (jig, column) in JIG_COLUMNS {
        for cell in CELLS.iter().filter(|cell| cell.jig == jig) {
            assert_eq!(cell.column, column, "{}", cell.kind.name);
        }
    }
}

#[test]
fn lookup_returns_the_cell_for_the_park_outcome() {
    for (smith, jig, kind, column) in [
        (smith::JOB_RUNNING, jig::PARKS_ON_NOTHING, JOB_RUNNING_PARKS_ON_NOTHING, Column::Resumes),
        (smith::WITHHELD_WAIT, jig::STUCK_IN_A_CIRCLE, WITHHELD_WAIT_STUCK_IN_A_CIRCLE, Column::Ends),
    ] {
        assert_eq!(cell(smith, jig), Ok(&Cell { smith, jig, kind, column }));
    }
    assert_eq!(JOB_RUNNING_PARKS_ON_NOTHING.name, "a job running, a run parks with nothing to await");
}

#[test]
fn lookup_explains_why_a_pair_cannot_occur() {
    for (smith, jig, explanation) in [
        (smith::CUT_TURN, jig::ANSWERED_WHILE_PARKED, "a parked answer tells its turn whole"),
        (smith::JOB_RUNNING, jig::PARKS_WITH_A_QUESTION_OPEN, "pairs under the kind of its outcome"),
    ] {
        let reason = cell(smith, jig).expect_err("no cell for an impossible pair");
        assert!(reason.contains(explanation), "{reason:?} does not explain {explanation:?}");
    }
}

const PARK: Park = Park { task: 1, attempt: 1, number: 1 };
const SMITH_HALF: Kind = smith::JOB_RUNNING;
const JIG_HALF: Kind = jig::PARKS_ON_NOTHING;

struct Observations {
    pairing: Pairing,
    ledger: FaultLedger,
}

impl Observations {
    fn new() -> Observations {
        Observations { pairing: Pairing::new(), ledger: FaultLedger::new() }
    }

    fn fell(&mut self, park: Park, kind: Kind, at: Time) -> Result<Option<&'static Cell>, String> {
        self.pairing.fell(park, kind, at, &mut self.ledger)
    }
}

#[test]
fn opposite_halves_do_not_pair_across_park_keys() {
    let other_keys = [
        Park { task: 2, ..PARK },
        Park { attempt: 2, number: 2, ..PARK },
        Park { attempt: 2, ..PARK },
        Park { number: 2, ..PARK },
    ];
    for other in other_keys {
        let mut observed = Observations::new();
        observed.fell(PARK, SMITH_HALF, Time::ZERO).expect("smith half accepted");
        observed.fell(other, JIG_HALF, Time::ZERO).expect("jig half accepted");
        assert!(observed.ledger.entries().is_empty(), "{PARK:?} cannot pair with {other:?}");
    }
}

#[test]
fn one_park_injects_its_cell_once_in_either_order() {
    let paired_at = Time::ZERO.checked_add(Duration::from_secs(1)).expect("test time fits");
    let expected =
        Cell { smith: SMITH_HALF, jig: JIG_HALF, kind: JOB_RUNNING_PARKS_ON_NOTHING, column: Column::Resumes };
    for (first, second) in [(SMITH_HALF, JIG_HALF), (JIG_HALF, SMITH_HALF)] {
        let mut observed = Observations::new();
        let first_report = observed.fell(PARK, first, Time::ZERO).expect("first half accepted");
        let repeated_first = observed.fell(PARK, first, Time::ZERO).expect("first half accepted again");
        let second_report = observed.fell(PARK, second, paired_at).expect("second half accepted");
        let repeats = [first, second].map(|kind| observed.fell(PARK, kind, paired_at).expect("known half accepted"));
        assert_eq!([first_report, repeated_first, repeats[0], repeats[1]], [None; 4]);
        assert_eq!(second_report, Some(&expected));
        let entries: Vec<_> =
            observed.ledger.entries().iter().map(|entry| (entry.kind, entry.from, &entry.struck)).collect();
        let struck = Struck { request: Some("task 1".into()), ..Struck::default() };
        assert_eq!(entries, [(JOB_RUNNING_PARKS_ON_NOTHING, paired_at, &struck)]);
    }
}

fn assert_finding(finding: &str, kinds: &[Kind], reason: &str) {
    assert!(finding.contains(&format!("{PARK:?}")), "{finding:?} does not name the park");
    for kind in kinds {
        assert!(finding.contains(kind.name), "{finding:?} does not name {kind:?}");
    }
    assert!(finding.contains(reason), "{finding:?} does not explain {reason:?}");
}

#[test]
fn impossible_pair_findings_name_the_park_and_halves() {
    let mut observed = Observations::new();
    let cut = smith::CUT_TURN;
    observed.fell(PARK, cut, Time::ZERO).expect("cut half accepted");
    let finding = observed.fell(PARK, JIG_HALF, Time::ZERO).expect_err("a cut cannot pair with a park");
    assert_finding(&finding, &[cut, JIG_HALF], "leaving no cut turn");
    assert!(observed.ledger.entries().is_empty());
}

#[test]
fn conflicting_half_findings_name_the_park_and_kinds() {
    for (first, different) in [(SMITH_HALF, smith::WITHHELD_WAIT), (JIG_HALF, jig::STUCK_IN_A_CIRCLE)] {
        let mut observed = Observations::new();
        observed.fell(PARK, first, Time::ZERO).expect("first half accepted");
        let finding = observed.fell(PARK, different, Time::ZERO).expect_err("different kind for the same half");
        assert_finding(&finding, &[first, different], "conflicting park kinds");
        assert!(observed.ledger.entries().is_empty());
    }
}

#[test]
fn foreign_kind_findings_name_the_park_and_kind() {
    let mut observed = Observations::new();
    let kind = JOB_RUNNING_PARKS_ON_NOTHING;
    let finding = observed.fell(PARK, kind, Time::ZERO).expect_err("a cell's kind is not a half");
    assert_finding(&finding, &[kind], "neither park family");
    assert!(observed.ledger.entries().is_empty());
}

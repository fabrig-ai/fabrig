use super::*;
use skein_lib::{List, Wall};
pub fn chip_from_rows(rows: &[w::Fact]) -> w::Chip {
    jig_client_test_records::chip_from_rows(rows)
}
pub fn phases() -> [(w::Chip, c::Chip); 1] {
    [phase_owner()]
}

pub fn phase_owner() -> (w::Chip, c::Chip) {
    let lim = limits().wire;
    let rows = [
        chip_root_fact(),
        w::Fact::ChipPhase(
            w::ChipPhaseRow::new(
                &lim,
                w::ChipPhaseRowParts {
                    parent: Some(0),
                    value: w::ChipPhaseOwnerFields::new(
                        &lim,
                        w::ChipPhaseOwnerFieldsParts {
                            phase: w::OwnerChipPhase::Waiting(
                                w::OwnerChipPhaseWaiting::new(
                                    &lim,
                                    w::OwnerChipPhaseWaitingParts {
                                        dependencies: List::with_capacity(0),
                                        more: false,
                                        holds: false,
                                    },
                                )
                                .unwrap(),
                            ),
                            details: 2,
                        },
                    )
                    .unwrap(),
                },
            )
            .unwrap(),
        ),
        w::Fact::OwnerChipDetails(Box::new(wire_owner_chip_details_row())),
    ];
    let record = chip_from_rows(&rows);
    let mut expected = chip();
    expected.details = Box::new(c::ChipPhase {
        phase: c::OwnerChipPhase::Waiting { dependencies: List::with_capacity(0), more: false, holds: false },
        details: Box::new(owner_chip_details()),
    });
    (record, expected)
}
pub fn chip_root_fact() -> w::Fact {
    jig_client_test_records::chip_root_fact()
}

pub fn wire_owner_chip_details_row() -> w::OwnerChipDetailsRow {
    jig_client_test_records::wire_owner_chip_details_row()
}

pub fn owner_chip_details() -> c::OwnerChipDetails {
    c::OwnerChipDetails {
        dependencies: List::with_capacity(0),
        project: 7,
        executor: c::OwnerExecutor::Agent { charter: 7 },
        phase_since: Wall::from_nanos(7),
        run_since: None,
        attempt: None,
        retry: None,
        wake: c::OwnerWakePolicy {
            words: c::WakeRule::Never,
            notices: c::WakeRule::Never,
            news: c::WakeRule::Never,
            results: c::ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        person_wait: None,
        hold: None,
        budget: c::OwnerBudget {
            numbers: c::OwnerNumbers { budget: 7, spent: 7, spent_below: 7, reserved: 7 },
            left: 7,
            funder: c::Funder::Task(7),
            deadline: None,
        },
        activity: 7,
        resources: List::with_capacity(0),
        resources_cut: false,
        resources_observed: false,
        procedure: None,
        live_delegates: 7,
        closing_connectors: None,
        ending: None,
        ended_at: None,
        observation: None,
    }
}

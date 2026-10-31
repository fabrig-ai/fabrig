use jig_client::contract::*;
use skein_lib::{List, Wall};

pub(in crate::tests) fn details() -> ChipDetails {
    ChipDetails {
        dependencies: List::with_capacity(0),
        project: u32::MAX,
        executor: Executor::Person(PersonAddress::Role(u32::MAX)),
        phase_since: Wall::from_nanos(10),
        run_since: None,
        attempt: None,
        retry: None,
        wake: WakePolicy {
            words: WakeRule::Never,
            notices: WakeRule::Never,
            news: WakeRule::Never,
            results: ResultsWake::Never,
            questions: false,
            answers: false,
            timers: false,
        },
        person_wait: None,
        hold: None,
        budget: Budget {
            numbers: Numbers { budget: 9, spent: 1, spent_below: 2, reserved: 3 },
            left: 4,
            funder: Funder::Period { project: u32::MAX, period: u64::MAX },
            deadline: None,
        },
        activity: u64::MAX,
        resources: List::with_capacity(0),
        resources_cut: false,
        resources_observed: false,
        procedure: None,
        live_delegates: u32::MAX,
        closing_connectors: None,
        ending: None,
        ended_at: None,
        observation: None,
    }
}

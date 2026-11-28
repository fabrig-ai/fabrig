//! Builder protocol fault sweeps (hearth/docs/design/testing/README.md, 4.5).

use hearth_builder_world::{fixtures, protocol};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};
use std::{collections::BTreeSet, convert::Infallible};

#[test]
fn the_protocol_fault_draw_reduces_the_full_case_seed() {
    assert_eq!(fixtures::swept_fault(u64::MAX).0.name, "normal"); // (2^64 - 1) % 5 == 0
    assert_eq!(fixtures::swept_fault(1_u64 << 32).0.name, "killed"); // 2^32 % 5 == 1
}

#[test]
fn random_cuts_kills_and_vanished_files_settle_and_each_fault_falls() {
    let mut fell = gate().then(BTreeSet::new);
    cases(
        this_test!("random_cuts_kills_and_vanished_files_settle_and_each_fault_falls"),
        fixtures::FUZZY_PROTOCOL_SEEDS,
        |seed| -> Result<(), Infallible> {
            let (case, scenario) = fixtures::swept_fault(seed);
            let run = protocol::run(&scenario, seed);
            assert_eq!(run.fault_fell, case.fell, "seed {seed}, {}: the drawn fault fell", case.name);
            if let Some(fell) = &mut fell {
                fell.insert(case.name);
            }
            assert_eq!(run.spawned, run.reaped, "seed {seed}: every child reaped");
            assert_eq!(run.opened_pipes, run.ended_pipes, "seed {seed}: every output read to its end");
            assert_eq!(run.code, case.code, "seed {seed}, {}: terminal outcome", case.name);
            Ok(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
    if let Some(fell) = fell {
        assert_eq!(
            fell.into_iter().collect::<Vec<_>>(),
            ["absent", "cache gone", "killed", "normal", "not executable"],
            "the sweep tried every fault and normal pipes"
        );
    }
}

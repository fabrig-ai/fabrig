//! Builder simulation sweeps and fixed restart cuts (hearth/docs/design/testing/README.md, 4.5).

use hearth_builder_world::{fixtures, simulated};
use skein_io::kernel::Exit;
use skein_world::{Cut, sweep::cases, this_test};
use std::convert::Infallible;

#[test]
fn random_seeds_under_chaos_settle_with_the_calm_runs_spec() {
    let calm = fixtures::simulated_thin();
    cases(
        this_test!("random_seeds_under_chaos_settle_with_the_calm_runs_spec"),
        0..fixtures::FUZZY_SEEDS,
        |seed| -> Result<(), Infallible> {
            let outcome = simulated::run(&fixtures::thin(), seed, simulated::chaos());
            assert_eq!(
                outcome.machine.contents(fixtures::OUTPUT),
                calm.machine.contents(fixtures::OUTPUT),
                "seed {seed}"
            );
            assert_eq!(fixtures::client_exits(&outcome), [Exit::Code(0)], "seed {seed}");
            Ok(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn a_builder_killed_before_each_submission_restarts_cold_and_settles() {
    for before in 0..fixtures::CUTS {
        let mut world = simulated::for_cut(&fixtures::thin(), fixtures::SIMULATION_SEED);
        world.cut(fixtures::BUILDER_SLOT, before, Cut::Kill);
        let outcome = world.run();
        assert!(
            outcome
                .machine
                .contents(fixtures::OUTPUT)
                .expect("output append")
                .ends_with(fixtures::THIN_TEXT.as_bytes()),
            "cut before submission {before}: cold restart prints the complete spec"
        );
    }
    let outcome = fixtures::simulated_thin();
    let submissions = fixtures::client_submissions(&outcome);
    assert_eq!(submissions, fixtures::CUTS, "every calm submission is a cut point");
}

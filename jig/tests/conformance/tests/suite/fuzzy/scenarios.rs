use std::convert::Infallible;

use jig_conformance::{
    Cut,
    scenarios::{ALL, Scenario, run},
};
use jig_conformance_world::{Testing, replay};
use skein_world::{
    sweep::{Case, cases},
    this_test,
};

#[test]
fn every_shared_story_survives_every_commit_and_seeded_random_cuts() -> Result<(), Case<Infallible>> {
    macro_rules! part {
        ($story:literal) => {
            this_test!("every_shared_story_survives_every_commit_and_seeded_random_cuts", $story)
        };
    }

    let mut random = skein_lib::Rng::new(31);
    for scenario in ALL {
        let test = match scenario {
            Scenario::LateCopy => part!("LateCopy"),
            Scenario::OtherHand => part!("OtherHand"),
            Scenario::UncertainRestarts => part!("UncertainRestarts"),
            Scenario::ChangedJudge => part!("ChangedJudge"),
            Scenario::ShrinkingPool => part!("ShrinkingPool"),
            Scenario::NarrowingPolicy => part!("NarrowingPolicy"),
            Scenario::StandingTask => part!("StandingTask"),
            Scenario::RunBudget => part!("RunBudget"),
            Scenario::RacingDecisions => part!("RacingDecisions"),
            Scenario::ConversationEndsWhileRead => part!("ConversationEndsWhileRead"),
            Scenario::QueryWatchedWhileTasksEnd => part!("QueryWatchedWhileTasksEnd"),
            Scenario::Compound => part!("Compound"),
            Scenario::Grace => part!("Grace"),
        };
        cases(test, [random.below(1000)], |seed| {
            let baseline = replay::fresh_cuts(scenario, None, seed, None, move |cut, world| {
                assert!(world.outcome().crashes.contains(&cut), "{scenario:?}, cut {cut}, seed {seed}");
            })
            .unwrap_or_else(|error| panic!("{scenario:?}, seed {seed}: {error:?}"));
            assert_eq!(baseline.forked, (1..=baseline.commits).collect::<Vec<_>>(), "every original numbered cut");
            assert_eq!(baseline.checked, baseline.commits, "each original commit checked by its child");
            let cut = Cut::Random { seed };
            run::<Testing>(scenario, cut, seed)
                .unwrap_or_else(|error| panic!("{scenario:?}, {cut:?}: {error:?}"))
                .expect("bound kinds");
            Ok(())
        })?;
    }
    Ok(())
}

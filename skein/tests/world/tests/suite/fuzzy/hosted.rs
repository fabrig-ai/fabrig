//! Hosted process interleavings, replay and ownership (skein/docs/design/testing/sweeps.md, 4.2–4.5).

use std::convert::Infallible;

use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use skein_world_tests::hosted::{Story, check_story, story};

#[test]
fn every_hosted_story_settles_replays_and_frees_its_own_heap_over_seeds() -> Result<(), Case<Infallible>> {
    cases(this_test!("every_hosted_story_settles_replays_and_frees_its_own_heap_over_seeds"), 0..100, |seed| {
        for scenario in
            [Story::Exchange, Story::Kill, Story::EarlyExit, Story::Interrupt, Story::Terminate, Story::Memory]
        {
            let first = story(seed, scenario);
            let replay = story(seed, scenario);
            check_story(scenario, &first);
            check_story(scenario, &replay);
            assert_eq!(first.trace, replay.trace, "seed {seed} replays its hosted records");
            assert_eq!(first.peaks(), replay.peaks(), "seed {seed} replays its metered heaps");
        }
        Ok(())
    })
}

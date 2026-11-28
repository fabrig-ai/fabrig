//! Each case runs the shared lock story and its replay (skein/docs/design/testing/sweeps.md, 4.2).

use skein_lib::Rng;
use skein_oauth_accounts_world::shared::{self, Story};
use skein_world::Memory;
use skein_world::sweep::{Case, cases};
use std::convert::Infallible;

const SEEDS: u64 = 64;

#[test]
fn seeded_lock_answers_settle_with_one_unlock_each_and_replay() -> Result<(), Case<Infallible>> {
    let stories = [
        Story::Busy,
        Story::Slow,
        Story::Newer,
        Story::NoRecord,
        Story::Subject,
        Story::Cancelled,
        Story::Removed,
        Story::Aborted,
    ];
    let count = skein_lib::len_u64(stories.len());
    cases(skein_world::this_test!("seeded_lock_answers_settle_with_one_unlock_each_and_replay"), 0..SEEDS, |seed| {
        let story = stories[usize::try_from(Rng::new(seed).below(count)).expect("story index")];
        let first = shared::run(seed, story, Memory::Unchecked);
        let second = shared::run(seed, story, Memory::Unchecked);
        assert_eq!(first.trace, second.trace, "{story:?} at seed {seed}");
        assert_eq!(shared::facts(&first.procs), shared::facts(&second.procs));
        Ok(())
    })
}

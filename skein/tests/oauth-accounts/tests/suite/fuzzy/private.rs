//! Each case runs the private keeper story and its replay (skein/docs/design/testing/sweeps.md, 4.2).

use skein_lib::Rng;
use skein_oauth_accounts_world::private::{self, Story};
use skein_world::sweep::{Case, cases};
use std::convert::Infallible;

#[test]
fn seeded_private_file_failures_and_keeper_settlement_replay() -> Result<(), Case<Infallible>> {
    let stories = [
        Story::Refresh,
        Story::KeepFailed,
        Story::KeepStalled,
        Story::CloseKeeping,
        Story::AbortKeeping,
        Story::Missing,
        Story::Unreadable,
        Story::NoIssuedId,
        Story::FileOwner,
        Story::FileMode,
        Story::FileLink,
        Story::FileLinks,
        Story::RootMode,
        Story::Load,
        Story::LoadFailed,
        Story::LoadStalled,
        Story::Fixed,
        Story::Conflict,
        Story::TooLarge,
        Story::CloseLoading,
        Story::AbortLoading,
    ];
    let count = skein_lib::len_u64(stories.len());
    cases(skein_world::this_test!("seeded_private_file_failures_and_keeper_settlement_replay"), 0..32, |seed| {
        let mut random = Rng::new(seed);
        let story = stories[usize::try_from(random.below(count)).expect("story index")];
        let mut first = private::run(seed, story);
        let mut second = private::run(seed, story);
        assert_eq!(first.trace, second.trace);
        assert_eq!(private::facts(&first.procs), private::facts(&second.procs));
        first.machine.finish();
        second.machine.finish();
        Ok(())
    })
}

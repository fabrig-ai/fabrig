//! Each case checks a world under stream cuts (skein/docs/design/testing/sweeps.md, 4.2 and 4.5).

use skein_lib::Rng;
use skein_oauth_accounts_world::world::{self, Story};
use skein_world::Memory;
use skein_world::sweep::{Case, cases};
use std::convert::Infallible;

#[test]
fn seeded_stream_cuts_rejections_and_closes_keep_owner_and_socket_contracts() -> Result<(), Case<Infallible>> {
    let stories = [
        Story::Rotate,
        Story::Retry,
        Story::Omit,
        Story::NotKept,
        Story::Rejected,
        Story::Repeated,
        Story::Released,
        Story::CloseConnecting,
        Story::CloseSending,
        Story::CloseKeeping,
        Story::Abort,
    ];
    cases(
        skein_world::this_test!("seeded_stream_cuts_rejections_and_closes_keep_owner_and_socket_contracts"),
        0..64,
        |seed| {
            let mut random = Rng::new(seed);
            let story = stories[usize::try_from(random.below(skein_lib::len_u64(stories.len()))).expect("story index")];
            drop(world::run(seed, story, true, Memory::Unchecked));
            Ok(())
        },
    )
}

use skein_lib::Rng;
use skein_oauth_accounts_world::world::{self, Story};
use skein_world::Memory;

#[test]
fn seeded_stream_cuts_rejections_and_closes_keep_owner_and_socket_contracts() {
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
    for seed in 0..64_u64 {
        let mut random = Rng::new(seed);
        let story = stories[usize::try_from(random.below(skein_lib::len_u64(stories.len()))).expect("story index")];
        drop(world::run(seed, story, true, Memory::Unchecked));
    }
}

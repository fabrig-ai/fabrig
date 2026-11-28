//! Each case draws a sign-in story and replays it (skein/docs/design/testing/sweeps.md, 4.2 and 4.5).

use skein_lib::len_u64;
use skein_oauth_accounts_world::sign_in::{self, Story};
use skein_world::Memory;
use skein_world::sweep::{Case, cases};
use std::convert::Infallible;

const STORIES: &[Story] = &[
    Story::WrongPath,
    Story::WrongHost,
    Story::LongHead,
    Story::CloseWaiting,
    Story::Cancel,
    Story::Abort,
    Story::Confidential,
    Story::Return(sign_in::Return::Back),
    Story::Return(sign_in::Return::Replace),
    Story::Return(sign_in::Return::WrongPath),
    Story::Return(sign_in::Return::After),
    Story::Paste,
    Story::PasteThenListener,
    Story::ListenerThenPaste,
    Story::PasteTwice,
    Story::PasteWrong,
    Story::StateRefusals,
    Story::ListenerWrongState,
    Story::PasteExpired,
    Story::PasteCancelled,
    Story::PasteClose,
    Story::Public,
    Story::Profile(true),
    Story::Profile(false),
    Story::Restart,
];

fn drawn(seed: u64) -> Story {
    let index = seed % len_u64(STORIES.len());
    STORIES[usize::try_from(index).expect("story index")]
}

#[test]
fn sign_in_story_draw_accepts_full_width_seeds() {
    assert_eq!(drawn(u64::MAX), Story::PasteWrong);
}

#[test]
fn browser_arrival_wrong_redirects_and_close_replay_under_short_operations() -> Result<(), Case<Infallible>> {
    cases(
        skein_world::this_test!("browser_arrival_wrong_redirects_and_close_replay_under_short_operations"),
        0..32,
        |seed| {
            let story = drawn(seed);
            let first = sign_in::run(seed, story, true, false, Memory::Unchecked);
            let second = sign_in::run(seed, story, true, false, Memory::Unchecked);
            sign_in::assert_replay(&first, &second);
            Ok(())
        },
    )
}

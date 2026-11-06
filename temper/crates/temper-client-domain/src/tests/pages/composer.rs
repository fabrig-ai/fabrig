use super::fixtures;
use crate::pages::composer::{held, over};

pub(super) const BOUND: u32 = 1024;
pub(super) const HOLD: u32 = 2048;
// Typed bytes, retained bytes, characters beyond long_text, characters over Me's bound.
pub(super) const PAST: &[(u32, u32, u64, u64)] = &[(1025, 1025, 0, 1), (2049, 2048, 1, 1025)];
pub(super) fn limits() -> crate::Limits {
    let mut limits = fixtures::limits();
    limits.client.domain.long_text = HOLD;
    limits
}
pub(super) fn words(length: u32) -> Box<[u8]> {
    let mut words = skein_lib::bytes::zeroed(usize::try_from(length).unwrap());
    words.fill(b'a');
    words
}
pub(super) fn deliver_me(f: &mut fixtures::chat::Fixture) {
    let mut me = fixtures::chats::me();
    me.words_bytes = BOUND;
    for event in [
        jig_client::Event::Watch {
            handle: crate::pages::me_handle(),
            subject: jig_client::Watchable::Jig(jig_client::contract::Subject::Me),
        },
        jig_client::Event::Up {
            up: jig_client::Up::Watched {
                watch: f.me,
                event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(me))),
            },
        },
    ] {
        f.client.step_unlinked(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, event, &mut skein_lib::Queue::uncapped());
    }
}
pub(super) fn root(limits: &crate::Limits) -> crate::Domain {
    let mut root = crate::unlinked(limits, fixtures::SEED);
    let watch = fixtures::start(&mut root, skein_lib::List::with_capacity(0));
    let mut me = fixtures::chats::me();
    me.words_bytes = BOUND;
    crate::step(
        &mut root,
        &fixtures::env(limits),
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
        &mut skein_lib::Queue::uncapped(),
    );
    root
}

#[test]
fn held_keeps_what_fits_and_counts_what_doesnt() {
    for (typed, hold, kept, beyond) in [
        ("abc", 3, "abc", 0),
        ("abcd", 3, "abc", 1),
        ("aé🦀z", 4, "aé", 2),
        ("éz", 1, "", 2),
        ("é", 0, "", 1),
        ("", 0, "", 0),
    ] {
        let (words, count) = held(typed.as_bytes().into(), hold);
        assert_eq!((&*words, count), (kept.as_bytes(), beyond), "{typed:?} at {hold}");
    }
    let (words, beyond) = held(b"short".as_slice().into(), 8);
    assert_eq!((&*words, beyond), (b"short".as_slice(), 0));
}

#[test]
fn over_counts_characters_past_the_bound() {
    for (words, beyond, bound, expected) in [
        ("abc", 0, 3, 0),
        ("abcd", 0, 3, 1),
        ("aé🦀z", 0, 2, 3),
        ("aé🦀z", 0, 3, 2),
        ("aé🦀z", 0, 6, 2),
        ("aé🦀z", 0, 7, 1),
        ("aé", 2, 2, 3),
        ("aé", 2, 99, 2),
        ("", 1, 0, 1),
        ("", 0, 0, 0),
    ] {
        assert_eq!(over(words.as_bytes(), beyond, bound), expected, "{words:?} at {bound}");
    }
}

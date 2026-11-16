//! Advertised host capabilities and the reason a hello was refused.
use super::{Builder, HelloRefusalLine, Hole, Key, Owner, P, Words, region};
use crate::words::fragments::hello::{duration, field_number, refusal, text};
pub(super) fn hello(b: &mut Builder, words: &Words, value: &HelloRefusalLine, position: u64) {
    region(
        b,
        words,
        P::Host,
        &[Hole::Number(value.channel)],
        Some(Key { owner: Owner::Jig.opaque(), kind: 97, a: position, b: value.channel }),
    );
    field_number(b, words, P::Host, value.channel);
    field_number(b, words, P::Slots, u64::from(value.slots));
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(value.at))]);
    duration(b, words, P::StopBound, value.stop_bound);
    refusal(b, words, value.why);
    b.close();
}

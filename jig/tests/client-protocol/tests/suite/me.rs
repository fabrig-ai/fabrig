//! The signed-in party's bounds through the engine and native bytes (contract.md, 5.1).
use jig_client::{Asker, Handle, Latest, Watchable, contract};
use jig_client_protocol_world::{client::Story, fixtures};

#[test]
fn me_says_the_engines_words_bound() {
    // Distinct from the profile's people.words of 64.
    const WORDS_BYTES: u32 = 37;
    const ME: Handle = Handle { asker: Asker::JIG, number: 1 };
    let mut limits = fixtures::owner_limits();
    limits.core.people.words = WORDS_BYTES;
    let mut story =
        Story::reading_chat_with(fixtures::SEED, Box::from(fixtures::TURN_BYTES), fixtures::domain_limits(), limits);
    story.watch(ME, Watchable::Jig(contract::Subject::Me));
    let Some(Latest::Jig(contract::Delivery::Me(me))) = story.client.document(ME).expect("watched me").latest else {
        panic!("signed-in me")
    };
    assert_eq!(me.words_bytes, WORDS_BYTES);
}

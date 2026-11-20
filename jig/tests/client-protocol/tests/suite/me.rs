//! The signed-in party's bounds through the engine and native bytes (client/domain/contract/documents.md, 1).
use jig_client::{Asker, Handle, Latest, Watchable, contract};
use jig_client_protocol_world::{client::Story, fixtures};

#[test]
fn me_says_the_engines_words_bound() {
    // Distinct from the profile's people.words of 64.
    const WORDS_BYTES: u32 = 37;
    const ME: Handle = Handle { asker: Asker::JIG, number: 1 };
    let mut limits = fixtures::owner_limits();
    limits.core.people.words = WORDS_BYTES;
    let mut story = Story::reading_chat_with(fixtures::SEED, fixtures::turn_bytes(), fixtures::domain_limits(), limits);
    story.watch(ME, Watchable::Jig(contract::Subject::Me));
    let Some(Latest::Jig(contract::Delivery::Me(me))) = story.client.document(ME).expect("watched me").latest else {
        panic!("signed-in me")
    };
    assert_eq!(me.words_bytes, WORDS_BYTES);
}

#[test]
fn the_stream_brings_me_recovers_it_and_refuses_an_unserved_jig_version() {
    use jig_client::{Link, State};
    use skein_lib::Duration;
    const ME: Handle = Handle { asker: Asker::JIG, number: 1 };
    let mut story = Story::live(fixtures::SEED);
    story.watch(ME, Watchable::Jig(contract::Subject::Me));
    let party = story.client.link().party.expect("hello confirmed the party");
    let Some(Latest::Jig(contract::Delivery::Me(me))) = story.client.document(ME).expect("shared me").latest else {
        panic!("hello carried me");
    };
    assert_eq!(me.party.number, party);
    assert_eq!(story.client.document(ME).expect("shared me").state, State::Live);
    story.cut_stream();
    assert!(story.run_until(|s| s.client.link().state == Link::Offline, Duration::from_secs(1)));
    assert_eq!(story.client.document(ME).expect("held me").state, State::Behind);
    assert!(story.run_until(|s| s.client.link().state == Link::Live, Duration::from_secs(1)));
    assert_eq!(story.client.link().party, Some(party));
    assert_eq!(story.client.document(ME).expect("recovered me").state, State::Live);
    story.serve_jig(jig_client_protocol_wire::VERSION.checked_add(1).expect("next version"));
    assert!(story.run_until(|s| s.client.link().state == Link::OutOfDate, Duration::from_secs(1)));
    assert!(!story.client.link().unreadable);
    assert_eq!(story.client.document(ME).expect("ended me").state, State::Ended);
}

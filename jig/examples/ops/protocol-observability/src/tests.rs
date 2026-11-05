use crate::{alert_words, health_words, lead, observability, triage};
use alloc::boxed::Box;
use jig_protocol_tools::Limits;

// Two delegates must reach the application's single-delegate check, not the decoder's bound.
const LIMITS: Limits = Limits { input_bytes: 4096, words: 128, batch: 2, depth: 16 };
const MEMBER: &[u8] = br#"{"executor":{"kind":"agent","charter":1},"spec":{"words":"check"},"contract":{"kind":"report","words":100},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":100},"notes":0}}"#;

#[expect(clippy::disallowed_methods, reason = "test fixtures assemble JSON batches from one delegate envelope")]
fn batch(members: &[&[u8]]) -> Box<[u8]> {
    let members = members.join(b",".as_slice());
    [b"{\"batch\":[".as_slice(), &members, b"]}"].concat().into_boxed_slice()
}

#[test]
fn alert_words_name_the_alert_and_its_severity() {
    let alert = observability::Alert {
        number: 7,
        service: observability::Service::new(Box::from(b"production".as_slice()), Box::from(b"checkout".as_slice())),
        severity: 3,
    };
    assert_eq!(alert_words(&alert).as_ref(), b"alert 7 severity 3");
}

#[test]
fn health_words_describe_both_health_states() {
    for (healthy, expected) in [(true, b"healthy".as_slice()), (false, b"unhealthy".as_slice())] {
        assert_eq!(health_words(healthy).as_ref(), expected);
    }
}

#[test]
fn triage_rejects_undecodable_or_non_singleton_batches() {
    for input in [Box::from(b"undecodable".as_slice()), batch(&[MEMBER, MEMBER]), batch(&[])] {
        assert!(triage(&input, &[1, 2], &LIMITS).is_none(), "{input:?}");
    }
}

#[test]
fn triage_appends_the_explicit_alerts_to_its_words() {
    let delegates = triage(&batch(&[MEMBER]), &[1, 2], &LIMITS).expect("one triage delegate");
    let [delegate] = delegates.as_ref() else { panic!("one triage delegate") };
    assert_eq!(delegate.spec.words.as_ref(), b"check\nAlerts: 1 2");
}

#[test]
fn lead_rejects_undecodable_or_non_singleton_batches() {
    for input in [Box::from(b"undecodable".as_slice()), batch(&[MEMBER, MEMBER]), batch(&[])] {
        assert!(lead(&input, &LIMITS).is_none(), "{input:?}");
    }
}

#[test]
fn lead_keeps_its_delegate_words_unchanged() {
    let delegates = lead(&batch(&[MEMBER]), &LIMITS).expect("one lead delegate");
    let [delegate] = delegates.as_ref() else { panic!("one lead delegate") };
    assert_eq!(delegate.spec.words.as_ref(), b"check");
}

use super::*;
use crate::tests::fixtures::ConnectionHarness;

#[test]
fn queued_answers_precede_transport_endings() {
    let mut h = ConnectionHarness::new();
    h.forgejo.heard.push(Heard::Done(Body::Empty));
    h.connection.failed(super::super::Failure::Seen);
    assert!(h.forgejo.ready(&h.connection));
    assert_eq!(h.forgejo.heard(&mut h.connection), Some(Heard::Done(Body::Empty)));
    assert_eq!(h.forgejo.heard(&mut h.connection), Some(Heard::Failed(super::super::Failure::Seen)));
    assert_eq!(h.forgejo.heard(&mut h.connection), None, "one terminal");
    assert!(!h.forgejo.ready(&h.connection));
}

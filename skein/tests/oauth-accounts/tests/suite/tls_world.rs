use skein_oauth_accounts_world::world;

#[test]
fn a_graceful_tls_close_reaches_the_issuer() {
    let close = world::tls_close(83, true);
    assert!(close.peer_closed, "the issuer reads close_notify");
    assert_eq!(close.aborted, None, "the graceful close needs no abort");
}

#[test]
fn a_tls_close_without_room_keeps_its_deadline_and_aborts() {
    let close = world::tls_close(83, false);
    let deadline = close.started.expect("the keeper starts the close").saturating_add(world::limits().io.close_timeout);
    assert_eq!(close.deadline, Some(deadline), "the component holds the link's close deadline");
    assert_eq!(close.aborted, Some(deadline), "the component fires the link at its bound");
    assert!(!close.peer_closed, "withheld room keeps close_notify off the wire");
}

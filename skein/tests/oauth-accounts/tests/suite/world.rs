use skein_oauth_accounts_world::world::{self, Story};

#[test]
fn an_access_only_record_expiring_is_handed_in_again_and_lent() {
    let (facts, _) = world::protocol(101, Story::HandedIn);
    assert_eq!(
        facts,
        vec![world::Fact::Granted(0), world::Fact::Expiring, world::Fact::Granted(1), world::Fact::Closed]
    );
}

#[test]
fn refresh_keeping_rejection_release_and_teardown_cross_real_http_streams() {
    for story in [
        Story::Lead,
        Story::Rotate,
        Story::Omit,
        Story::NotKept,
        Story::Rejected,
        Story::Repeated,
        Story::Released,
        Story::CloseConnecting,
        Story::CloseSending,
        Story::CloseKeeping,
        Story::Abort,
    ] {
        let (facts, _) = world::protocol(83, story);
        assert_eq!(facts.last(), Some(&world::Fact::Closed));
        match story {
            Story::Abort | Story::Repeated => assert!(facts.iter().any(|fact| matches!(fact, world::Fact::Failed(_)))),
            Story::Released | Story::NotKept => assert!(facts.contains(&world::Fact::Granted(0))),
            Story::Retry
            | Story::HandedIn
            | Story::SignInAgain
            | Story::Lead
            | Story::Rotate
            | Story::Omit
            | Story::Rejected
            | Story::CloseConnecting
            | Story::CloseSending
            | Story::CloseKeeping => assert!(facts.contains(&world::Fact::Granted(1))),
        }
    }
}

#[test]
fn stream_fragments_and_owner_terminals_replay() {
    assert_eq!(world::protocol(97, Story::Rotate), world::protocol(97, Story::Rotate));
}

#[test]
fn a_tick_waits_until_the_previous_oauth_output_is_dispatched() {
    assert_eq!(world::delayed_oauth_output(), ["held OAuth output", "dispatched HTTP request", "Tick aborted socket"]);
}

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

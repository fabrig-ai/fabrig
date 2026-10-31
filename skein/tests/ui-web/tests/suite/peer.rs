//! The lending peer leaves io work queued until the kernel edge has room.
use skein_lib::{Queue, Time, Wall};
use skein_ui_web_tests::harness_peer::Peer;
use skein_world::Host;

#[test]
fn a_lent_peer_waits_for_submission_room() {
    let mut peer = Peer::new();
    let short = skein_io::MAX_OUT_DOWN.submissions.checked_sub(1).expect("down reserves submission room");
    *peer.submissions() = Queue::with_capacity(short);
    peer.iterate(Time::ZERO, Wall::EPOCH);
    assert!(peer.submissions().is_empty(), "Listen stays queued without down's submission room");
    assert!(peer.work_pending(Time::ZERO), "the held Listen keeps the peer runnable");

    *peer.submissions() = Queue::with_capacity(skein_io::MAX_OUT_DOWN.submissions);
    peer.iterate(Time::ZERO, Wall::EPOCH);
    assert_eq!(peer.submissions().len(), 1, "the held Listen submits once room is available");
}

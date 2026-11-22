use skein_fake_person_browser::testing::{Peer, ready_and_drained};
use std::net::TcpListener;

#[test]
fn a_lent_peer_becomes_ready_after_claiming_its_adopted_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve listener");
    let reserved = listener.local_addr().expect("reserved address");
    let (address, listens) = ready_and_drained(Peer::new(Some(listener)));
    assert_eq!(address, reserved);
    assert_eq!(listens, 0, "adoption does not submit a kernel Listen");
}

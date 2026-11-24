use super::*;
use crate::bounds::PENDING_EXCHANGES;

#[test]
fn workload_draw_is_independent_of_party_draws() {
    let streams = Streams::new(0);
    let before = draw(&streams);
    let mut parties = streams.rng(Stream::Parties, "party-0");
    for _ in 0..17 {
        parties.next_u64();
    }
    assert_eq!(draw(&streams), before);
    assert_eq!(before.parties.len(), 3);
    assert_eq!(before.parties[0].delays, [1, 4, 1, 3]);
}

#[test]
fn each_partys_acts_use_its_own_part() {
    let streams = Streams::new(1);
    let workload = draw(&streams);
    let first = &workload.parties[0];
    let other = &workload.parties[1];
    let before = acts(&streams, first);
    let mut other_draws = streams.rng(Stream::Parties, &other.name);
    for _ in 0..17 {
        other_draws.next_u64();
        assert_eq!(acts(&streams, first), before);
    }
    assert_eq!(
        (&before, acts(&streams, other)),
        (&Acts { resend_at: 3, park_first: false }, Acts { resend_at: 2, park_first: true })
    );
    assert!(before.resend_at > 0 && before.resend_at <= first.delays[0]);
}

#[test]
fn pending_kind_names_the_inspected_table() {
    let mut peer = Peer::signed_in(&fixtures::settings(), fixtures::FIRST_SIGN_IN);
    peer.post_signed(b"/jig/v1/request", keyed([1; 16], b"hello".as_slice().into()));
    let inspection = peer.inspection();
    assert_eq!(inspection.tables().len(), 1);
    assert_eq!(PENDING_EXCHANGES.family, "the engine's own bounds");
    assert_eq!(PENDING_EXCHANGES.name, inspection.tables()[0].name);
}

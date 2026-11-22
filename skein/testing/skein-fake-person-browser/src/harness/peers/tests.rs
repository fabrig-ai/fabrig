use super::fixture::Peer;
use super::{FIRST, Peers};
use crate::harness::Lent;
use skein_io::kernel::{Complete, Op};
use skein_lib::Token;

#[test]
fn a_lent_peer_is_named_before_control_and_only_after_the_name_changes() {
    let output = std::cell::RefCell::new(Vec::new());
    let mut last = None;
    for (name, expected) in [
        (b"neighbor".as_slice(), b"lent peer neighbor\n".as_slice()),
        (b"neighbor".as_slice(), b"lent peer neighbor\n".as_slice()),
        (b"other".as_slice(), b"lent peer neighbor\nlent peer other\n".as_slice()),
    ] {
        let result = super::call_with_report(
            &mut last,
            name,
            || {
                assert_eq!(output.borrow().as_slice(), expected, "the name precedes control");
                Err::<(), _>("peer's value")
            },
            |line| {
                use std::io::Write;
                writeln!(output.borrow_mut(), "{line}").expect("write the test's report");
            },
        );
        assert_eq!(result, Err("peer's value"), "the peer's return value is unchanged");
    }
}

#[test]
fn a_lent_peers_startup_error_names_the_peer() {
    let mut peer = Peer::new(None);
    let mut lent = [Lent { name: b"neighbor", peer: &mut peer }];
    let mut peers = Peers::new(&mut lent);
    let complete = Complete {
        op: Token::new(7),
        kind: Op::Socket { family: skein_io::kernel::Family::Ipv4 },
        result: Err(skein_io::kernel::Error::Refused),
    };
    assert_eq!(peers.completion(FIRST, &complete), Some("lent peer neighbor startup: Refused".into()));
}

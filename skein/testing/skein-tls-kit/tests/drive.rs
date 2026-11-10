//! The prompt wire preserves fills across a wrapped byte queue
//! (skein/docs/design/testing/README.md, 6).

use skein_lib::stream::{Down, Read, Up};
use skein_tls_kit::{drive::Wire, pki, server::Server};
use std::collections::VecDeque;

#[test]
fn a_fill_copies_both_queue_slices_and_leaves_the_rest() {
    let mut wire = Wire::new(Server::new(pki::Server::plain().config()));
    wire.bytes = VecDeque::with_capacity(8);
    wire.bytes.extend(0..8);
    drop(wire.bytes.drain(..6));
    wire.bytes.extend(8..12);
    assert_eq!(wire.bytes.as_slices(), ([6, 7].as_slice(), [8, 9, 10, 11].as_slice()));

    wire.take(Down::Demand { read: Read::Fill(4), room: 0 });
    assert_eq!(wire.answer(), Some(Up::Bytes(Box::from([6, 7, 8, 9]))));
    assert_eq!(wire.demand, None);
    assert_eq!(wire.bytes, VecDeque::from([10, 11]));

    wire.take(Down::Demand { read: Read::Fill(3), room: 0 });
    assert_eq!(wire.answer(), None, "a fill waits for all its bytes");
    assert_eq!(wire.demand, Some((Read::Fill(3), 0)));
    assert_eq!(wire.bytes, VecDeque::from([10, 11]));
    wire.bytes.push_back(12);
    assert_eq!(wire.answer(), Some(Up::Bytes(Box::from([10, 11, 12]))));
    assert!(wire.bytes.is_empty());
}

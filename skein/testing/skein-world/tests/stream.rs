//! Empty writes preserve the seeded stream schedule
//! (skein/docs/design/testing/README.md, 6).

#[cfg(test)]
mod tests {
    use skein_lib::stream::{Down, Read};
    use skein_world::stream::Wire;

    #[test]
    fn empty_writes_preserve_the_fragment_and_room_schedule() {
        let mut empty = Wire::new(7);
        let mut plain = Wire::new(7);
        let payload = b"the peer's bytes still arrive in the same seeded fragments";
        empty.write(payload);
        plain.write(payload);
        empty.take(Down::Demand { read: Read::Fill(1), room: 0 });
        plain.take(Down::Demand { read: Read::Fill(1), room: 0 });
        let mut received = 0_usize;
        let mut room = false;
        for _ in 0_usize..1_000 {
            empty.write(&[]);
            let answer = plain.answer();
            assert_eq!(empty.answer(), answer, "empty writes preserve each delivery turn");
            assert_eq!(empty.trace, plain.trace, "fragments and delays stay identical");
            if answer.is_some() {
                if received == payload.len() {
                    room = true;
                    break;
                }
                received = received.checked_add(1).expect("payload length fits");
                let read = if received == payload.len() { Read::Nothing } else { Read::Fill(1) };
                let grant = if received == payload.len() { 7 } else { 0 };
                empty.take(Down::Demand { read, room: grant });
                plain.take(Down::Demand { read, room: grant });
            }
        }
        assert_eq!(received, payload.len());
        assert!(room, "the same room grant follows the bytes");
    }
}

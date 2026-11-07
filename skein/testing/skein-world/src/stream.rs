//! A seeded in-memory stream for protocol worlds
//! (testing-strategy.md, sections 2.5 and 6). It keeps demands, granted room, buffered
//! peer bytes and the bytes the client sent; it knows no connection state.

use alloc::collections::VecDeque;

use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Intake, Rng};

/// One stream endpoint, driven by a protocol world and its peer.
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "worlds inspect outside stream facts; demands and scheduling remain private"
)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "peer EOF, delivered end, sent finish and scheduling idleness are independent facts"
)]
pub struct Wire {
    pub received: Vec<u8>,
    pub eof: bool,
    pub ended: bool,
    pub finished: bool,
    pub trace: Vec<String>,
    input: Intake,
    pending: VecDeque<u8>,
    demand: Option<(Read, u32)>,
    granted: Option<u32>,
    rng: Rng,
    idle: bool,
}

impl Wire {
    /// A bounded receive intake and seeded byte delivery schedule.
    #[must_use]
    pub fn new(seed: u64) -> Wire {
        Wire {
            received: Vec::new(),
            eof: false,
            ended: false,
            finished: false,
            trace: Vec::new(),
            input: Intake::with_capacity(32768),
            pending: VecDeque::new(),
            demand: None,
            granted: None,
            rng: Rng::new(seed),
            idle: true,
        }
    }

    /// Put the peer's bytes on the wire for delivery in seeded fragments.
    pub fn write(&mut self, bytes: &[u8]) {
        self.pending.extend(bytes);
        if !bytes.is_empty() {
            self.idle = false;
        }
    }

    /// Whether another answer attempt can progress without new peer bytes or a demand.
    /// A successful answer leaves one probe pending; an unmet read settles after that
    /// probe, while seeded byte and room delays keep work pending.
    #[must_use]
    pub fn work_pending(&self) -> bool {
        !self.idle
            || !self.pending.is_empty()
            || match self.demand {
                Some((read, _)) => read != Read::Nothing && self.eof && !self.ended,
                None => false,
            }
    }

    /// Check one client demand, send, withdrawal or finish.
    ///
    /// # Panics
    /// On a broken stream contract.
    pub fn take(&mut self, down: Down) {
        self.trace.push(format!("down {down:?}"));
        match down {
            Down::Demand { read: Read::Nothing, room: 0 } => {
                assert!(self.demand.take().is_some(), "only an outstanding demand is withdrawn");
                self.idle = false;
            }
            Down::Demand { read, room } => {
                assert!(self.demand.is_none(), "one demand at a time");
                if room > 0 {
                    assert!(!self.finished, "no room after Finish");
                    self.granted = None;
                }
                self.demand = Some((read, room));
                self.idle = false;
            }
            Down::Send(bytes) => {
                let granted = self.granted.take().expect("Send has granted room");
                assert!(bytes.len() <= usize::try_from(granted).expect("u32 fits"), "Send fits granted room");
                self.received.extend_from_slice(&bytes);
            }
            Down::Finish => {
                assert!(!self.finished, "one Finish");
                self.finished = true;
            }
        }
    }

    /// Deliver bytes and answer a demand, with seeded room and byte delays.
    ///
    /// # Panics
    /// If the scripted peer exceeds the bounded response intake.
    pub fn answer(&mut self) -> Option<Up> {
        if !self.pending.is_empty() && self.rng.chance(700) {
            let count = usize::try_from(self.rng.between(1, 31)).expect("small fragment").min(self.pending.len());
            let bytes: Vec<u8> = self.pending.drain(..count).collect();
            self.trace.push(format!("fragment {bytes:?}"));
            self.input.append(&bytes).expect("bounded scripted response intake");
        }
        let Some((read, room)) = self.demand else {
            self.idle = true;
            return None;
        };
        let answer = match self.input.meet(read) {
            Some(bytes) => {
                self.demand = None;
                Some(Up::Bytes(bytes))
            }
            None if room > 0 && self.rng.chance(700) => {
                self.demand = None;
                self.granted = Some(room);
                Some(Up::Room)
            }
            None if read != Read::Nothing && self.eof && self.pending.is_empty() && !self.ended => {
                self.ended = true;
                Some(Up::End)
            }
            None => {
                if room > 0 {
                    self.trace.push("room delayed".to_owned());
                }
                None
            }
        };
        if let Some(answer) = &answer {
            self.trace.push(format!("up {answer:?}"));
        }
        self.idle = answer.is_none() && room == 0;
        answer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_work_keeps_delayed_room_bytes_and_end_but_settles_an_unmet_read() {
        let mut wire = Wire::new(7);
        assert!(!wire.work_pending());
        wire.take(Down::Demand { read: Read::Fill(4), room: 0 });
        assert!(wire.work_pending());
        assert_eq!(wire.answer(), None);
        assert!(!wire.work_pending(), "an unmet read needs peer input");
        wire.write(b"ab");
        assert!(wire.work_pending());
        for _ in 0_u32..128 {
            if !wire.work_pending() {
                break;
            }
            assert_eq!(wire.answer(), None);
        }
        assert!(!wire.work_pending(), "a partial fill settles with buffered bytes");
        wire.write(b"cd");
        let mut delivered = false;
        for _ in 0_u32..128 {
            if !wire.work_pending() {
                break;
            }
            if let Some(up) = wire.answer() {
                assert_eq!(up, Up::Bytes(Box::from(*b"abcd")));
                delivered = true;
            }
        }
        assert!(delivered);
        assert!(!wire.work_pending());

        let mut wire = Wire::new(0);
        wire.take(Down::Demand { read: Read::Nothing, room: 4 });
        assert_eq!(wire.answer(), None, "this seed delays its first room grant");
        assert!(wire.work_pending());
        let mut room = false;
        for _ in 0_u32..128 {
            if !wire.work_pending() {
                break;
            }
            if let Some(up) = wire.answer() {
                assert_eq!(up, Up::Room);
                room = true;
            }
        }
        assert!(room, "a seeded room delay cannot settle the wire");
        assert!(!wire.work_pending());
        wire.take(Down::Send(Box::from(*b"sent")));
        assert_eq!(wire.received, b"sent");
        wire.take(Down::Demand { read: Read::Fill(1), room: 0 });
        assert_eq!(wire.answer(), None);
        assert!(!wire.work_pending());
        wire.eof = true;
        assert!(wire.work_pending(), "EOF wakes an unmet read");
        assert_eq!(wire.answer(), Some(Up::End));
        assert_eq!(wire.answer(), None);
        assert!(!wire.work_pending());
    }
}

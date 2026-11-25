//! In-memory native stream, with seeded cuts in both directions.
use skein_channel::{Control, Lower, LowerEvent, decode_control, parse_header};
use skein_fake_channel::{Observed, ScriptedPeer};
use skein_lib::{Intake, Rng, Token, stream};
use std::collections::VecDeque;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteRoom {
    Free,
    Withheld,
    One,
    Held,
}
pub(crate) struct Net {
    pub peer: ScriptedPeer,
    intake: Intake,
    read: stream::Read,
    bytes: VecDeque<u8>,
    right: Option<Token>,
    pub write_room: WriteRoom,
    pub cap: Option<u32>,
    asked: u32,
    pub ended: bool,
    pub gone: bool,
    pub taken: bool,
    pub accept: Option<Observed>,
    pub refused: Option<(u16, Box<[u8]>)>,
    events: VecDeque<LowerEvent>,
}
impl Net {
    pub fn new(peer: ScriptedPeer, chunk: u32) -> Self {
        Self {
            peer,
            intake: Intake::with_capacity(chunk.max(8)),
            read: stream::Read::Nothing,
            bytes: VecDeque::new(),
            right: None,
            write_room: WriteRoom::Free,
            cap: None,
            asked: 0,
            ended: false,
            gone: false,
            taken: false,
            accept: None,
            refused: None,
            events: VecDeque::new(),
        }
    }
    pub fn lower(&mut self, lower: Lower, rng: &mut Rng) {
        match lower {
            Lower::Read(stream::Down::Demand { read, room }) => {
                assert_eq!(room, 0);
                self.read = read;
            }
            Lower::Read(stream::Down::Finish) => {}
            Lower::Read(stream::Down::Send(_)) | Lower::FinishWrite => panic!("one stream, independent output"),
            Lower::Write(write) => match write {
                stream::OutputDown::Room { right, bytes } => {
                    self.asked = bytes;
                    assert!(self.right.replace(right).is_none());
                }
                stream::OutputDown::Cancel { right } => {
                    if self.write_room != WriteRoom::Held && self.right.take() == Some(right) {
                        self.events.push_back(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Cancelled,
                        }));
                    }
                }
                stream::OutputDown::Release { .. } => {}
                stream::OutputDown::Send { bytes, .. } => {
                    let header = parse_header(&bytes[..8]).expect("a complete sent frame");
                    if header.kind < 256 {
                        let control = decode_control(header.kind, &bytes[8..], &super::fixtures::channel_limits())
                            .expect("control");
                        match control {
                            Control::Accept { .. } => {
                                self.accept = Some(Observed { kind: header.kind, body: bytes[8..].into() });
                            }
                            Control::Refuse { reason, text } => self.refused = Some((reason, text)),
                            _ => {}
                        }
                    }
                    let mut at = 0;
                    while at < bytes.len() {
                        let cut = usize::try_from(rng.between(1, 17)).expect("small cut").min(bytes.len() - at);
                        self.peer.feed(&bytes[at..at + cut]).expect("worker accepts engine bytes");
                        at += cut;
                    }
                }
            },
        }
    }
    pub fn buffered_input(&self) -> bool {
        !self.bytes.is_empty() || !self.intake.is_empty()
    }
    pub fn tick(&mut self, rng: &mut Rng) -> Option<LowerEvent> {
        if self.gone || !self.taken {
            return None;
        }
        if let Some(event) = self.events.pop_front() {
            return Some(event);
        }
        if self.cap.is_none_or(|cap| self.asked <= cap)
            && !matches!(self.write_room, WriteRoom::Withheld | WriteRoom::Held)
            && let Some(right) = self.right.take()
        {
            if self.write_room == WriteRoom::One {
                self.write_room = WriteRoom::Withheld;
            }
            return Some(LowerEvent::Write(stream::OutputUp::Settled {
                right,
                outcome: stream::OutputOutcome::Granted,
            }));
        }
        if self.bytes.is_empty()
            && let Some(bytes) = self.peer.pop_output()
        {
            self.bytes = VecDeque::from(bytes.into_vec());
        }
        // Take the same seeded cuts before meeting the read. Nothing observes
        // the intake between cuts, so copy their combined prefix once.
        let available = self.bytes.len().min(skein_lib::length::usize_of(self.intake.room()));
        let mut arrived = 0;
        while arrived < available {
            let cut = usize::try_from(rng.between(1, 17)).expect("small cut");
            arrived += cut.min(available - arrived);
        }
        if arrived != 0 {
            let (front, back) = self.bytes.as_slices();
            let first = front.len().min(arrived);
            self.intake.append(&front[..first]).expect("room checked");
            let rest = arrived - first;
            if rest != 0 {
                self.intake.append(&back[..rest]).expect("room checked");
            }
            drop(self.bytes.drain(..arrived));
        }
        if let Some(bytes) = self.intake.meet(self.read) {
            self.read = stream::Read::Nothing;
            return Some(LowerEvent::Read(stream::Up::Bytes(bytes)));
        }
        if self.ended && self.read != stream::Read::Nothing {
            self.ended = false;
            return Some(LowerEvent::Read(stream::Up::End));
        }
        None
    }
    pub fn inject(&mut self, bytes: &[u8]) {
        self.bytes.extend(bytes);
    }
}

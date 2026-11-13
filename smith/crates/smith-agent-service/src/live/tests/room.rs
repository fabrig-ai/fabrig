//! Real channel fixture with explicit transport credit and decoded live records.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "deterministic real-channel fixture uses ordinary Rust: programming-model.md, 10.2"
)]

use super::*;
use alloc::collections::VecDeque;
use skein_channel::{Control, Direction, Lower, LowerEvent, StreamMode, Term};
use skein_lib::{Queue, Reader, stream};

// One piece may use the frame credit beyond the authoritative and fact reserves.
const CHANNEL_FRAMES: u32 = 16;
const QUEUE: u32 = 128;

pub(super) struct Room {
    pub takes: bool,
    pub remaining: u32,
    pub lost: u64,
    pub sent: List<Sent>,
    pub channel: channel::Component,
    pub limits: channel::Limits,
    pub above: Queue<channel::Heard>,
    pub below: Queue<Lower>,
    pub incoming: VecDeque<u8>,
    pub next: u64,
}

impl Room {
    pub fn new(takes: bool) -> Self {
        let mut limits = smith_agent_profile_fixtures::small().channel;
        limits.channel.output_frames = CHANNEL_FRAMES;
        limits.channel.output_bytes = u32::MAX;
        let mut endpoints = List::with_capacity(1);
        endpoints.push(smith_protocol_translate::Endpoint { name: Box::default(), number: 0, dialect: 0 }).unwrap();
        let channel =
            channel::Component::new(&limits, StreamMode::Two, smith_protocol_translate::Endpoints::new(endpoints))
                .unwrap();
        let mut room = Self {
            takes,
            remaining: u32::MAX,
            lost: 0,
            sent: List::with_capacity(super::super::HELD),
            channel,
            limits,
            above: Queue::with_capacity(QUEUE),
            below: Queue::with_capacity(QUEUE),
            incoming: VecDeque::new(),
            next: 1,
        };
        room.open();
        room
    }

    #[expect(clippy::match_like_matches_macro, reason = "the enclosing crate bans the matches macro")]
    fn open(&mut self) {
        let schema = channel::schema(&self.limits).unwrap();
        let open = skein_channel::control_frame(
            &Control::Open {
                magic: schema.magic,
                lowest: smith_channel::kind::VERSION,
                highest: smith_channel::kind::VERSION,
                features: 0,
                credential: Box::default(),
            },
            &self.limits.channel,
        )
        .unwrap();
        let mut entries = List::with_capacity(self.limits.channel.kinds);
        for kind in &schema.version(smith_channel::kind::VERSION).unwrap().kinds {
            if kind.direction == Direction::FromResponder {
                entries.push(Term { kind: kind.kind, largest: kind.largest }).unwrap();
            }
        }
        let terms = skein_channel::control_frame(&Control::Terms { entries }, &self.limits.channel).unwrap();
        let start = smith_channel::Start::new(
            &self.limits.bodies,
            smith_channel::StartParts {
                identity: None,
                activation: 1,
                charter: Box::from(
                    include_bytes!("../../../../smith-charter/golden/v5/record_charter_smallest.bin").as_slice(),
                ),
                workspace: None,
                transcript: smith_protocol_translate::transcript_piece(&[]).unwrap(),
                answered: List::with_capacity(0),
                grants: List::with_capacity(0),
                messages: List::with_capacity(0),
                window: smith_channel::Window::new(
                    &self.limits.bodies,
                    smith_channel::WindowParts { turns: 1, bytes: u64::MAX },
                )
                .unwrap(),
                live: Some(ask(true, false, true)),
            },
        )
        .unwrap();
        let start = smith_channel::frame(smith_channel::Body::Start(start)).unwrap();
        for frame in [open, terms, start] {
            self.incoming.extend(frame.bytes());
        }
        self.channel.fire(&mut self.above, &mut self.below);
        self.transport();
        assert!(
            self.above.iter().any(|event| match event {
                channel::Heard::Start { .. } => true,
                _ => false,
            }),
            "real channel accepted Start: {:?}",
            self.above
        );
        self.above.clear();
        self.channel.send_admitted(Token::new(0), &mut self.above, &mut self.below).unwrap();
        self.transport();
    }

    #[expect(clippy::match_like_matches_macro, reason = "the enclosing crate bans the matches macro")]
    pub fn act(&mut self, live: &mut Live, action: Action<'_>, now: Time) {
        if self.takes && self.remaining == 1 {
            // Fill authoritative output until exactly one best-effort frame fits.
            for _ in 0..CHANNEL_FRAMES.checked_sub(self.limits.fact_reserve_frames).unwrap().checked_sub(2).unwrap() {
                self.channel.send_waiting(None, Token::new(self.next), &mut self.above, &mut self.below).unwrap();
                self.next = self.next.checked_add(1).unwrap();
            }
        }
        let before = self.sent.len();
        let mut edge = channel::edge_room();
        if !self.takes || self.remaining == 0 {
            edge.below = u32::MAX;
        }
        let ending = match action {
            Action::End { .. } => true,
            _ => false,
        };
        let mut connection = crate::service::link::HostLink {
            channel: &mut self.channel,
            above: &mut self.above,
            below: &mut self.below,
            next: &mut self.next,
            failed: false,
            room: edge,
            ending,
        };
        live.act(action, now, &mut connection);
        assert!(!connection.failed, "live frame encoded");
        self.transport();
        self.remaining = self.remaining.saturating_sub(self.sent.len().checked_sub(before).unwrap());
        self.lost = self.channel.lost_live_bytes();
    }

    fn transport(&mut self) {
        loop {
            self.channel.fire(&mut self.above, &mut self.below);
            if self.below.is_empty() {
                break;
            }
            while let Some(lower) = self.below.pop() {
                match lower {
                    Lower::Read(stream::Down::Demand { read: stream::Read::Fill(count), .. })
                        if self.incoming.len() >= usize::try_from(count).unwrap() =>
                    {
                        let bytes = self.incoming.drain(..usize::try_from(count).unwrap()).collect();
                        self.channel.from_below(
                            LowerEvent::Read(stream::Up::Bytes(bytes)),
                            &mut self.above,
                            &mut self.below,
                        );
                    }
                    Lower::Write(stream::OutputDown::Room { right, .. }) => {
                        self.channel.from_below(
                            LowerEvent::Write(stream::OutputUp::Settled {
                                right,
                                outcome: stream::OutputOutcome::Granted,
                            }),
                            &mut self.above,
                            &mut self.below,
                        );
                    }
                    Lower::Write(stream::OutputDown::Send { bytes, .. }) => {
                        let header = skein_channel::parse_header(&bytes[..8]).unwrap();
                        if header.kind == smith_channel::kind::LIVE {
                            let live =
                                smith_channel::Live::decode(&smith_channel::CEILINGS, &mut Reader::new(&bytes[8..]))
                                    .unwrap()
                                    .into_parts();
                            self.sent
                                .push(Sent {
                                    turn: live.turn,
                                    retry: live.retry,
                                    offset: live.offset,
                                    bytes: live.bytes,
                                    call: live.call,
                                })
                                .unwrap();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn payloads(&self) -> List<&[u8]> {
        let mut pieces = List::with_capacity(self.sent.len());
        for piece in &self.sent {
            pieces.push(piece.bytes.as_ref()).unwrap();
        }
        pieces
    }

    pub fn offsets(&self) -> List<(u64, usize)> {
        let mut offsets = List::with_capacity(self.sent.len());
        for piece in &self.sent {
            offsets.push((piece.offset, piece.bytes.len())).unwrap();
        }
        offsets
    }
}

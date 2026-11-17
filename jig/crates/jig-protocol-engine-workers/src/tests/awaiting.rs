//! Openings wait for the initial settings table (domain/engine/workers.md, 4).
use super::fixtures;
use crate::{Component, Event, Loss, Request, Worker};
use alloc::boxed::Box;
use jig_protocol_worker_wire as wire;
use skein_channel::{Lower, LowerEvent};
use skein_lib::{Intake, Queue, Time, Token, Writer, stream};

const CONNECTION: Token = Token::new(1);
const LOCAL: &[u8] = b"local";

struct Opening {
    component: Component,
    peer: skein_fake_channel::ScriptedPeer,
    input: Intake,
    read: stream::Read,
    out: Queue<Request>,
    hello: Option<u32>,
    refused: Option<u16>,
    ended: Option<Loss>,
}
impl Opening {
    fn new(in_process: bool) -> Self {
        let schema = crate::limits::schema(&fixtures::LIMITS).expect("schema");
        let cap = wire::sizes::worker_cap(&fixtures::LIMITS.sizes).expect("worker output");
        let peer_limits = skein_channel::Limits {
            output_bytes: cap.cap.bytes,
            output_frames: cap.cap.frames,
            ..crate::limits::channel(&fixtures::LIMITS).expect("channel limits")
        };
        let mut peer = skein_fake_channel::ScriptedPeer::new(
            schema,
            skein_channel::Role::Initiator,
            peer_limits,
            wire::VERSION,
            fixtures::credential(LOCAL, fixtures::SECRET_A, fixtures::APPLICATION_VERSION),
        )
        .expect("worker peer");
        let hello = fixtures::hello_record();
        let mut body = Writer::new(skein_lib::length::usize_of(hello.measure()));
        hello.encode(&mut body).expect("measured hello");
        let mut frame = skein_channel::frame_writer(wire::Kind::Hello.number(), hello.measure()).expect("hello frame");
        frame.put(&body.finish()).expect("measured hello frame");
        peer.play(frame.finish().expect("whole hello"));
        let mut opening = Self {
            component: Component::new(
                &fixtures::LIMITS,
                Box::new([]),
                wire::spoken(),
                wire::Application::opaque(fixtures::application()),
            )
            .expect("component before its table"),
            peer,
            input: Intake::with_capacity(fixtures::LIMITS.room),
            read: stream::Read::Nothing,
            out: Queue::uncapped(),
            hello: None,
            refused: None,
            ended: None,
        };
        opening.component.down(Time::ZERO, Event::Accepted { connection: CONNECTION, in_process }, &mut opening.out);
        opening.pump(Time::ZERO);
        opening
    }
    fn pump(&mut self, now: Time) {
        for _ in 0_u16..1000 {
            let mut progress = false;
            while let Some(request) = self.out.pop() {
                progress = true;
                match request {
                    Request::Lower { lower: Lower::Write(stream::OutputDown::Room { right, .. }), .. } => {
                        self.component.down(
                            now,
                            Event::Lower {
                                connection: CONNECTION,
                                event: LowerEvent::Write(stream::OutputUp::Settled {
                                    right,
                                    outcome: stream::OutputOutcome::Granted,
                                }),
                            },
                            &mut self.out,
                        );
                    }
                    Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                        self.peer.feed(&bytes).expect("channel bytes");
                    }
                    Request::Lower { lower: Lower::Read(stream::Down::Demand { read, .. }), .. } => self.read = read,
                    Request::Hello { worker, .. } => self.hello = Some(worker),
                    Request::Refused { reason, .. } => self.refused = Some(reason),
                    Request::Ended { why, .. } => self.ended = Some(why),
                    Request::Close { .. } | Request::Abort { .. } => {
                        self.component.down(now, Event::Gone { connection: CONNECTION }, &mut self.out);
                    }
                    Request::Told { .. }
                    | Request::Take { .. }
                    | Request::TurnAway { .. }
                    | Request::Opened { .. }
                    | Request::FiguresTaken { .. }
                    | Request::Lost { .. }
                    | Request::Up { .. }
                    | Request::Unsent { .. }
                    | Request::Closed
                    | Request::Lower { .. } => {}
                }
            }
            if let Some(bytes) = self.peer.pop_output() {
                self.input.append(&bytes).expect("channel input room");
                progress = true;
            }
            if let Some(bytes) = self.input.meet(self.read) {
                self.read = stream::Read::Nothing;
                self.component.down(
                    now,
                    Event::Lower { connection: CONNECTION, event: LowerEvent::Read(stream::Up::Bytes(bytes)) },
                    &mut self.out,
                );
                progress = true;
            }
            if !progress {
                return;
            }
        }
        panic!("the opening settles");
    }
}

#[test]
fn an_opening_waits_for_the_table_with_its_original_deadline_and_listener_rule() {
    for (in_process, configured, expire) in
        [(true, true, false), (false, true, false), (true, false, false), (true, true, true)]
    {
        let mut opening = Opening::new(in_process);
        assert_eq!(opening.refused, None, "no opening is judged before the table");
        assert_eq!(opening.hello, None, "no opening is admitted before the table");
        let deadline = Time::ZERO.saturating_add(fixtures::LIMITS.handshake);
        assert_eq!(opening.component.next_deadline(), Some(deadline), "the opening keeps its handshake bound");
        let now = if expire { deadline } else { Time::ZERO };
        let workers: Box<[Worker]> = if configured {
            Box::from([Worker { number: 1, name: LOCAL.into(), places: 4, digest: None, withdrawn: false }])
        } else {
            Box::new([])
        };
        opening.component.down(now, Event::Workers(workers), &mut opening.out);
        opening.pump(now);
        if expire {
            assert_eq!(opening.hello, None, "the table cannot extend the opening's deadline");
            assert_eq!(opening.component.next_deadline(), Some(deadline));
            opening.component.fire(now, &mut opening.out);
            opening.pump(now);
            assert_eq!(opening.ended, Some(Loss::Deadline));
            assert_eq!(opening.hello, None, "a late table cannot revive an expired opening");
        } else if in_process && configured {
            assert_eq!(opening.hello, Some(0), "the waiting local worker opens when its table arrives");
        } else {
            assert_eq!(
                opening.refused,
                Some(wire::Reason::NotAdmitted.code()),
                "an empty table and a network listener do not admit a digest-less worker"
            );
            assert_eq!(opening.hello, None);
        }
    }
}

//! Concrete engine/worker channel peer shared by service steps and the protocol world.
use crate as fixtures;
use crate::worker_settings;
use jig_domain_host as hub;
use jig_protocol_engine_workers as engine_link;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_io as io;
use skein_lib::{Intake, List, Queue, Time, Token, Wall, stream};

// Two openings, three retained turns and one final answer.
const LINK_RECORDS: u32 = 6;

#[derive(Debug, PartialEq, Eq)]
pub enum LinkRecord {
    Hello,
    Lost,
    Ended,
    Turn(u32),
    Answer(u32),
}

pub struct LinkPeer {
    pub now: Time,
    pub received: usize,
    pub sent: usize,
    pub engine_pings: u32,
    pub worker_pings: u32,
    room: u32,
    pub records: List<LinkRecord>,
    pub component: engine_link::Component,
    pub out: Queue<engine_link::Request>,
    pub engine_bytes: Intake,
    pub worker_bytes: Intake,
    pub engine_read: Option<stream::Read>,
    pub worker_read: Option<stream::Read>,
    pub owner: Token,
}
impl LinkPeer {
    #[must_use]
    pub fn with_settings(settings: &crate::settings::Worker, agent: &crate::settings::Agent) -> Self {
        Self::with_limits(settings, agent, &fixtures::worker_limits())
    }
    #[must_use]
    pub fn with_limits(
        settings: &crate::settings::Worker,
        agent: &crate::settings::Agent,
        own: &worker_settings::Limits,
    ) -> Self {
        let prepared = worker_settings::prepare(settings, agent, own)
            .expect("channel fixture values and demands fit their bounds");
        let limits = engine_link::Limits {
            room: settings.jig.limits.as_ref().expect("checked worker limits").io.largest_room(),
            sizes: prepared.link.sizes,
            names: 1,
            workers: 1,
            opening: 1,
            chunk: prepared.link.chunk,
            handshake: prepared.link.handshake,
            hello: prepared.link.handshake,
            ping: prepared.link.ping,
            silence: prepared.link.silence,
            stall: prepared.link.stall,
        };
        Self::with_engine_limits(settings, agent, own, &limits, 65536, u32::MAX)
    }
    /// Stock composition with the engine's own sizes and a stream that grants no room past its cap.
    #[must_use]
    pub fn with_engine_limits(
        settings: &crate::settings::Worker,
        agent: &crate::settings::Agent,
        own: &worker_settings::Limits,
        limits: &engine_link::Limits,
        pipe: u32,
        room: u32,
    ) -> Self {
        let prepared = worker_settings::prepare(settings, agent, own)
            .expect("channel fixture values and demands fit their bounds");
        Self {
            now: fixtures::NOW,
            received: 0,
            sent: 0,
            engine_pings: 0,
            worker_pings: 0,
            room,
            records: List::with_capacity(LINK_RECORDS),
            component: engine_link::Component::new(
                limits,
                Box::new([engine_link::Worker {
                    number: 1,
                    places: 1,
                    withdrawn: false,
                    name: settings.jig.name.clone(),
                    digest: Some(skein_lib::sha256::digest(
                        fixtures::worker_files()
                            .contents(b"worker-secret")
                            .expect("channel fixture values and demands fit their bounds"),
                    )),
                }]),
                jig_protocol_worker_wire::spoken(),
                {
                    let mut wires = List::with_capacity(1);
                    wires
                        .push(temper_protocol_worker_wire::spoken())
                        .expect("channel fixture values and demands fit their bounds");
                    jig_protocol_worker_wire::Application::opaque(wires)
                },
            )
            .expect("channel fixture values and demands fit their bounds"),
            out: Queue::with_capacity(64),
            engine_bytes: Intake::with_capacity(pipe),
            worker_bytes: Intake::with_capacity(pipe),
            engine_read: None,
            worker_read: None,
            owner: Token::new(prepared.prefix),
        }
    }
    pub fn open(&mut self, worker: &mut crate::Worker) {
        worker.iterate(self.now, Wall::EPOCH);
        worker.up(self.now, io::Event::Connecting { owner: self.owner, socket: Token::new(44) });
        worker.up(self.now, io::Event::Connected { owner: self.owner });
        self.component.down(
            self.now,
            engine_link::Event::Accepted { connection: Token::new(1), in_process: false },
            &mut self.out,
        );
        for _ in 0_u16..1000 {
            if let Some(engine_link::Request::Hello { .. }) = self.turn(worker) {
                return;
            }
        }
        panic!("worker opens and says hello");
    }
    pub fn answer(&mut self, worker: &mut crate::Worker) -> (hub::LinkUp, Box<[Part]>) {
        for _ in 0_u16..1000 {
            if let Some(engine_link::Request::Up { up: up @ hub::LinkUp::Answer { .. }, parts, .. }) = self.turn(worker)
            {
                return (up, parts);
            }
        }
        panic!("worker sends its answer");
    }
    pub fn turn(&mut self, worker: &mut crate::Worker) -> Option<engine_link::Request> {
        worker.iterate(self.now, Wall::EPOCH);
        self.exchange(worker)
    }
    // Exchange demanded bytes while leaving root entrances queued for inspection.
    #[expect(clippy::too_many_lines, reason = "the shared peer exhaustively routes both channel ends' requests")]
    pub fn exchange(&mut self, worker: &mut crate::Worker) -> Option<engine_link::Request> {
        for _ in 0..worker.io().len() {
            let request = worker.io().pop().expect("queued IO");
            if !stream_request(&request, &[self.owner, Token::new(44)]) {
                worker.io().push(request);
                continue;
            }
            match request {
                io::Request::Output { down: stream::OutputDown::Room { right, .. }, .. } => worker.up(
                    self.now,
                    io::Event::Output {
                        owner: self.owner,
                        up: stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted },
                    },
                ),
                io::Request::Output { down: stream::OutputDown::Send { bytes, .. }, .. } => {
                    if skein_channel::parse_header(
                        bytes.get(..8).expect("channel fixture values and demands fit their bounds"),
                    )
                    .expect("channel fixture values and demands fit their bounds")
                    .kind
                        == 5
                    {
                        self.worker_pings = self
                            .worker_pings
                            .checked_add(1)
                            .expect("channel fixture values and demands fit their bounds");
                    }
                    self.received = self.received.checked_add(bytes.len()).expect("received link bytes fit usize");
                    self.engine_bytes.append(&bytes).expect("channel fixture values and demands fit their bounds");
                }
                io::Request::Stream { down: stream::Down::Demand { read, .. }, .. } => self.worker_read = Some(read),
                other @ (io::Request::Listen { .. }
                | io::Request::Connect { .. }
                | io::Request::Bind { .. }
                | io::Request::Reject { .. }
                | io::Request::Stream { .. }
                | io::Request::Output { .. }
                | io::Request::Spawn { .. }
                | io::Request::SpawnAlone { .. }
                | io::Request::Signal { .. }
                | io::Request::Usage { .. }
                | io::Request::Window { .. }
                | io::Request::Close { .. }
                | io::Request::CloseAfterFlush { .. }
                | io::Request::Abort { .. }) => panic!("link request: {other:?}"),
            }
        }
        while let Some(request) = self.out.pop() {
            match request {
                engine_link::Request::Lower {
                    lower: Lower::Write(stream::OutputDown::Room { right, bytes }), ..
                } => {
                    if bytes <= self.room {
                        self.lower(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Granted,
                        }));
                    }
                }
                engine_link::Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                    if skein_channel::parse_header(
                        bytes.get(..8).expect("channel fixture values and demands fit their bounds"),
                    )
                    .expect("channel fixture values and demands fit their bounds")
                    .kind
                        == 5
                    {
                        self.engine_pings = self
                            .engine_pings
                            .checked_add(1)
                            .expect("channel fixture values and demands fit their bounds");
                    }
                    self.sent = self
                        .sent
                        .checked_add(bytes.len())
                        .expect("channel fixture values and demands fit their bounds");
                    self.worker_bytes.append(&bytes).expect("channel fixture values and demands fit their bounds");
                }
                engine_link::Request::Lower { lower: Lower::Read(stream::Down::Demand { read, .. }), .. } => {
                    self.engine_read = Some(read);
                }
                engine_link::Request::Opened { .. }
                | engine_link::Request::Refused { .. }
                | engine_link::Request::Take { .. }
                | engine_link::Request::Close { .. }
                | engine_link::Request::Abort { .. } => {}
                request @ (engine_link::Request::Unsent { .. } | engine_link::Request::Unread { .. }) => {
                    return Some(request);
                }
                request @ (engine_link::Request::Hello { .. } | engine_link::Request::Up { .. }) => {
                    if let engine_link::Request::Hello { .. } = &request {
                        self.records.push(LinkRecord::Hello).expect("bounded link records");
                    } else if let engine_link::Request::Up { up: hub::LinkUp::Turn { turn, .. }, .. } = &request {
                        self.records.push(LinkRecord::Turn(turn.turn)).expect("bounded link records");
                    } else if let engine_link::Request::Up { up: hub::LinkUp::Answer { answer, .. }, .. } = &request {
                        self.records.push(LinkRecord::Answer(answer.turns)).expect("bounded link records");
                    }
                    return Some(request);
                }
                request @ engine_link::Request::Lost { .. } => {
                    self.records.push(LinkRecord::Lost).expect("bounded link records");
                    return Some(request);
                }
                request @ engine_link::Request::Ended { .. } => {
                    self.records.push(LinkRecord::Ended).expect("bounded link records");
                    return Some(request);
                }
                other @ (engine_link::Request::TurnAway { .. }
                | engine_link::Request::Lower { .. }
                | engine_link::Request::FiguresTaken { .. }
                | engine_link::Request::Unpaused { .. }
                | engine_link::Request::Told { .. }
                | engine_link::Request::Closed) => panic!("engine request: {other:?}"),
            }
        }
        if let Some(bytes) = demanded_bytes(&mut self.engine_read, &mut self.engine_bytes) {
            self.lower(LowerEvent::Read(stream::Up::Bytes(bytes)));
        }
        if let Some(bytes) = demanded_bytes(&mut self.worker_read, &mut self.worker_bytes) {
            worker.up(self.now, io::Event::Stream { owner: self.owner, up: stream::Up::Bytes(bytes) });
        }
        None
    }
    pub fn lower(&mut self, event: LowerEvent) {
        self.component.down(self.now, engine_link::Event::Lower { connection: Token::new(1), event }, &mut self.out);
    }
}

// Both concrete peers use the same stream contract: a delivery spends a real
// demand, and everything else stays in the intake below the component.
pub fn demanded_bytes(read: &mut Option<stream::Read>, intake: &mut Intake) -> Option<Box<[u8]>> {
    let bytes = intake.meet((*read)?)?;
    *read = None;
    Some(bytes)
}

#[must_use]
pub fn stream_request(request: &io::Request, streams: &[Token]) -> bool {
    if let io::Request::Output { stream, .. } | io::Request::Stream { stream, .. } = request {
        streams.contains(stream)
    } else if let io::Request::Close { entity } = request {
        streams.contains(entity)
    } else {
        false
    }
}

// Offer the assignment to the component, then recover smith's opaque owner
// while leaving its IO Spawn queued for the peer.

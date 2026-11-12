//! Concrete worker peers shared by worker, order and admission tests.
use super::super::fixtures;
use crate::worker_settings;
use jig_domain_host as hub;
use jig_domain_link::Grant;
use jig_protocol_engine_workers as engine_link;
use jig_protocol_worker_link as link;
use jig_protocol_worker_wire::Part;
use skein_channel::{Lower, LowerEvent};
use skein_io as io;
use skein_lib::{Intake, List, Queue, Time, Token, Wall, stream};
use temper_worker_domain as root;

// Two openings, three retained turns and one final answer.
const LINK_RECORDS: u32 = 6;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LinkRecord {
    Hello,
    Lost,
    Ended,
    Turn(u32),
    Answer(u32),
}

pub(crate) struct LinkPeer {
    pub(crate) now: Time,
    pub(crate) received: usize,
    pub(crate) sent: usize,
    pub(crate) engine_pings: u32,
    pub(crate) worker_pings: u32,
    room: u32,
    pub(crate) records: List<LinkRecord>,
    pub(crate) component: engine_link::Component,
    pub(crate) out: Queue<engine_link::Request>,
    pub(crate) engine_bytes: Intake,
    pub(crate) worker_bytes: Intake,
    pub(crate) engine_read: Option<stream::Read>,
    pub(crate) worker_read: Option<stream::Read>,
    pub(crate) owner: Token,
}
impl LinkPeer {
    pub(crate) fn with_settings(settings: &crate::settings::Worker, agent: &crate::settings::Agent) -> Self {
        Self::with_limits(settings, agent, &fixtures::worker_limits())
    }
    pub(crate) fn with_limits(
        settings: &crate::settings::Worker,
        agent: &crate::settings::Agent,
        own: &worker_settings::Limits,
    ) -> Self {
        let prepared = worker_settings::prepare(settings, agent, own).unwrap();
        let limits = engine_link::Limits {
            room: settings.jig.limits.io.largest_room(),
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
    pub(crate) fn with_engine_limits(
        settings: &crate::settings::Worker,
        agent: &crate::settings::Agent,
        own: &worker_settings::Limits,
        limits: &engine_link::Limits,
        pipe: u32,
        room: u32,
    ) -> Self {
        let prepared = worker_settings::prepare(settings, agent, own).unwrap();
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
                    name: settings.jig.name.clone(),
                    digest: io::digest::digest(fixtures::worker_files().contents(b"worker-secret").unwrap()),
                }]),
                temper_protocol_worker_wire::PAYLOAD_VERSION,
            )
            .unwrap(),
            out: Queue::with_capacity(64),
            engine_bytes: Intake::with_capacity(pipe),
            worker_bytes: Intake::with_capacity(pipe),
            engine_read: None,
            worker_read: None,
            owner: Token::new(prepared.prefix),
        }
    }
    pub(crate) fn open(&mut self, worker: &mut crate::Worker) {
        worker.iterate(self.now, Wall::EPOCH);
        worker.up(self.now, io::Event::Connecting { owner: self.owner, socket: Token::new(44) });
        worker.up(self.now, io::Event::Connected { owner: self.owner });
        self.component.down(self.now, engine_link::Event::Accepted { connection: Token::new(1) }, &mut self.out);
        for _ in 0_u16..1000 {
            if let Some(engine_link::Request::Hello { .. }) = self.turn(worker) {
                return;
            }
        }
        panic!("worker opens and says hello");
    }
    pub(crate) fn answer(&mut self, worker: &mut crate::Worker) -> (hub::LinkUp, Box<[Part]>) {
        for _ in 0_u16..1000 {
            if let Some(engine_link::Request::Up { up: up @ hub::LinkUp::Answer { .. }, parts, .. }) = self.turn(worker)
            {
                return (up, parts);
            }
        }
        panic!("worker sends its answer");
    }
    pub(crate) fn turn(&mut self, worker: &mut crate::Worker) -> Option<engine_link::Request> {
        worker.iterate(self.now, Wall::EPOCH);
        self.exchange(worker)
    }
    // Exchange demanded bytes while leaving root entrances queued for inspection.
    pub(crate) fn exchange(&mut self, worker: &mut crate::Worker) -> Option<engine_link::Request> {
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
                    if skein_channel::parse_header(bytes.get(..8).unwrap()).unwrap().kind == 5 {
                        self.worker_pings = self.worker_pings.checked_add(1).unwrap();
                    }
                    self.received = self.received.checked_add(bytes.len()).expect("received link bytes fit usize");
                    self.engine_bytes.append(&bytes).unwrap();
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
                    if skein_channel::parse_header(bytes.get(..8).unwrap()).unwrap().kind == 5 {
                        self.engine_pings = self.engine_pings.checked_add(1).unwrap();
                    }
                    self.sent = self.sent.checked_add(bytes.len()).unwrap();
                    self.worker_bytes.append(&bytes).unwrap();
                }
                engine_link::Request::Lower { lower: Lower::Read(stream::Down::Demand { read, .. }), .. } => {
                    self.engine_read = Some(read);
                }
                engine_link::Request::Take { .. }
                | engine_link::Request::Close { .. }
                | engine_link::Request::Abort { .. } => {}
                request @ engine_link::Request::Unsent { .. } => return Some(request),
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
    pub(crate) fn lower(&mut self, event: LowerEvent) {
        self.component.down(self.now, engine_link::Event::Lower { connection: Token::new(1), event }, &mut self.out);
    }
}

// Both concrete peers use the same stream contract: a delivery spends a real
// demand, and everything else stays in the intake below the component.
pub(crate) fn demanded_bytes(read: &mut Option<stream::Read>, intake: &mut Intake) -> Option<Box<[u8]>> {
    let bytes = intake.meet((*read)?)?;
    *read = None;
    Some(bytes)
}

pub(crate) const AGENT_CHILD: Token = Token::new(101);
pub(crate) const AGENT_INPUT: Token = Token::new(102);
pub(crate) const AGENT_OUTPUT: Token = Token::new(103);
pub(crate) const AGENT_ERROR: Token = Token::new(104);
pub(crate) const ADMITTED: u16 = 0x0106;
pub(crate) const FRAME_HEADER_BYTES: u32 = 8; // skein's channel frame header.
pub(crate) const PEER_FRAMES: u32 = 6; // Accept, Terms, Admitted, Live, Turn, Answer.
pub(crate) const PEER_STEPS: u16 = 1000; // The existing LinkPeer's finite settling bound.

// Private agent peer for the Worker's Process IO, without a root assignment.
// The ledger measures frame writes and demanded byte deliveries; it
// neither supplies process events nor stands in for Worker::process_read.
pub(crate) struct AgentPeer {
    pub(crate) owner: Token,
    pub(crate) now: Time,
    pub(crate) component: smith_protocol_channel::Component,
    pub(crate) events: Queue<smith_protocol_channel::Heard>,
    pub(crate) out: Queue<Lower>,
    pub(crate) agent_bytes: Intake,
    pub(crate) worker_bytes: Intake,
    pub(crate) agent_read: Option<stream::Read>,
    pub(crate) worker_read: Option<stream::Read>,
    pub(crate) writes: List<(u16, u64)>,
    pub(crate) written: u64,
    pub(crate) consumed: u64,
    pub(crate) stdout_demands: u64,
    pub(crate) finished: bool,
    pub(crate) ends: u32,
    pub(crate) input_finishes: u32,
    pub(crate) output_closes: u32,
}
impl AgentPeer {
    pub(crate) fn new(agent: &crate::settings::Agent, owner: Token) -> Self {
        let host = agent.limits;
        // The host's output bound prices initiator frames. The new responder
        // needs its own directional frame bound; Worker keeps its existing one.
        let mut channel = host.channel;
        for version in &smith_protocol_channel::schema_for_start(&host.bodies, host.start_bytes).unwrap().versions {
            for kind in &version.kinds {
                if kind.direction == skein_channel::Direction::FromResponder {
                    channel.output_bytes =
                        channel.output_bytes.max(kind.largest.checked_add(FRAME_HEADER_BYTES).unwrap());
                }
            }
        }
        let limits = smith_protocol_channel::Limits {
            bodies: host.bodies,
            start_bytes: host.start_bytes,
            charter: host.charter,
            transcript: host.transcript,
            channel,
            endpoints: host.endpoints,
            calls: host.calls,
            resume_turns: agent.host_limits.turns,
            search_hits: host.transcript.found_hits,
            turns: agent.host_limits.turns,
            fact_reserve_frames: 1,
            fact_reserve_bytes: 128,
            grants: agent.host_limits.accounts,
        };
        Self {
            owner,
            now: fixtures::NOW,
            component: smith_protocol_channel::Component::new(
                &limits,
                skein_channel::StreamMode::Two,
                agent.endpoints.clone(),
            )
            .unwrap(),
            events: Queue::uncapped(),
            out: Queue::uncapped(),
            agent_bytes: Intake::with_capacity(host.channel.output_bytes),
            worker_bytes: Intake::with_capacity(channel.output_bytes),
            agent_read: None,
            worker_read: None,
            writes: List::with_capacity(PEER_FRAMES),
            written: 0,
            consumed: 0,
            stdout_demands: 0,
            finished: false,
            ends: 0,
            input_finishes: 0,
            output_closes: 0,
        }
    }

    /// Receive a real assignment's Start without admitting the run.
    pub(crate) fn pending(
        worker: &mut crate::Worker,
        agent: &crate::settings::Agent,
        assignment: link::Request,
    ) -> (Self, Box<smith_protocol_channel::DecodedStart>) {
        let client = assigned_agent_with(worker, agent, assignment);
        worker.iterate(fixtures::NOW, Wall::EPOCH);
        let owner = agent_spawn(worker);
        let mut peer = Self::new(agent, client);
        peer.attach(worker, owner);
        let mut notices = Queue::uncapped();
        while let Some(event) = peer.events.pop() {
            if let smith_protocol_channel::Heard::Start { start } = event {
                return (peer, start);
            }
            notices.push(event);
        }
        panic!("assigned agent receives Start: {notices:?}");
    }

    /// End the fake channel, then return its process exit and reap terminals.
    pub(crate) fn end_process(&mut self, worker: &mut crate::Worker, detail: &[u8]) {
        self.finished = true;
        self.settle(worker);
        for event in [
            smith_host_domain::process::Event::Exited {
                owner: self.owner,
                exit: Some(smith_host_domain::Exit::Code(1)),
            },
            smith_host_domain::process::Event::Reaped { owner: self.owner, detail: detail.into() },
        ] {
            worker.events().push(root::Event::Process(Box::new(event)));
        }
    }

    fn attach(&mut self, worker: &mut crate::Worker, owner: Token) {
        worker.up(
            fixtures::NOW,
            io::Event::Spawned { owner, child: AGENT_CHILD, pipes: Box::new([AGENT_INPUT, AGENT_OUTPUT, AGENT_ERROR]) },
        );
        self.settle(worker);
    }

    pub(crate) fn lower(&mut self, event: LowerEvent) {
        self.component.from_below(event, &mut self.events, &mut self.out);
    }
    pub(crate) fn consumed_frames(&self) -> Box<[u16]> {
        let mut kinds = List::with_capacity(PEER_FRAMES);
        for &(kind, end) in &self.writes {
            if kind >= ADMITTED && end <= self.consumed {
                kinds.push(kind).unwrap();
            }
        }
        kinds.into_boxed()
    }
    pub(crate) fn worker_request(&mut self, worker: &mut crate::Worker, request: io::Request) {
        match request {
            io::Request::Output { stream, down: stream::OutputDown::Room { right, .. } } => {
                assert_eq!(stream, AGENT_INPUT);
                worker.up(
                    self.now,
                    io::Event::Output {
                        owner: AGENT_INPUT,
                        up: stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted },
                    },
                );
            }
            io::Request::Output { stream, down: stream::OutputDown::Send { bytes, .. } } => {
                assert_eq!(stream, AGENT_INPUT);
                self.agent_bytes.append(&bytes).unwrap();
            }
            io::Request::Stream { stream, down: stream::Down::Demand { read, room: 0 } } => {
                if stream == AGENT_OUTPUT {
                    if read == stream::Read::Nothing {
                        self.worker_read = None;
                    } else {
                        assert!(self.worker_read.is_none(), "one outstanding stdout demand");
                        self.stdout_demands = self.stdout_demands.checked_add(1).unwrap();
                        self.worker_read = Some(read);
                    }
                } else {
                    assert_eq!(stream, AGENT_ERROR, "only stderr is independent of the channel");
                }
            }
            io::Request::Stream { stream, down: stream::Down::Finish } => {
                assert_eq!(stream, AGENT_INPUT);
                self.input_finishes = self.input_finishes.checked_add(1).unwrap();
            }
            io::Request::Close { entity } => {
                assert_eq!(entity, AGENT_OUTPUT);
                self.output_closes = self.output_closes.checked_add(1).unwrap();
            }
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
            | io::Request::Abort { .. }) => panic!("agent IO: {other:?}"),
        }
    }
    pub(crate) fn settle(&mut self, worker: &mut crate::Worker) {
        self.pump(worker, None);
    }

    fn pump(&mut self, worker: &mut crate::Worker, frames: Option<usize>) {
        for _ in 0..PEER_STEPS {
            worker.iterate(self.now, Wall::EPOCH);
            self.component.fire(&mut self.events, &mut self.out);
            let mut progressed = false;
            for _ in 0..worker.io().len() {
                let request = worker.io().pop().expect("queued IO");
                if !stream_request(&request, &[AGENT_INPUT, AGENT_OUTPUT, AGENT_ERROR]) {
                    worker.io().push(request);
                    continue;
                }
                progressed = true;
                self.worker_request(worker, request);
            }
            while let Some(request) = self.out.pop() {
                progressed = true;
                match request {
                    Lower::Write(stream::OutputDown::Room { right, .. }) => {
                        self.lower(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Granted,
                        }));
                    }
                    Lower::Write(stream::OutputDown::Send { bytes, .. }) => {
                        let header = skein_channel::parse_header(
                            bytes.get(..usize::try_from(FRAME_HEADER_BYTES).unwrap()).unwrap(),
                        )
                        .unwrap();
                        assert_eq!(
                            u32::try_from(bytes.len()).unwrap(),
                            FRAME_HEADER_BYTES.checked_add(header.body_len).unwrap(),
                            "one whole frame per channel write"
                        );
                        self.written = self.written.checked_add(skein_lib::len_u64(bytes.len())).unwrap();
                        self.writes.push((header.kind, self.written)).unwrap();
                        self.worker_bytes.append(&bytes).unwrap();
                    }
                    Lower::Read(stream::Down::Demand { read, room: 0 }) => self.agent_read = Some(read),
                    Lower::FinishWrite => self.finished = true,
                    other @ (Lower::Read(_)
                    | Lower::Write(stream::OutputDown::Cancel { .. } | stream::OutputDown::Release { .. })) => {
                        panic!("agent lower: {other:?}")
                    }
                }
            }
            if let Some(bytes) = demanded_bytes(&mut self.agent_read, &mut self.agent_bytes) {
                progressed = true;
                self.lower(LowerEvent::Read(stream::Up::Bytes(bytes)));
            }
            if let Some(bytes) = demanded_bytes(&mut self.worker_read, &mut self.worker_bytes) {
                progressed = true;
                self.consumed = self.consumed.checked_add(skein_lib::len_u64(bytes.len())).unwrap();
                worker.up(self.now, io::Event::Stream { owner: AGENT_OUTPUT, up: stream::Up::Bytes(bytes) });
            } else if self.worker_read.is_some() && self.worker_bytes.is_empty() && self.finished && self.ends == 0 {
                progressed = true;
                self.ends = self.ends.checked_add(1).unwrap();
                worker.up(self.now, io::Event::Stream { owner: AGENT_OUTPUT, up: stream::Up::End });
            }
            if let Some(count) = frames
                && self.consumed_frames().len() >= count
            {
                return;
            }
            if !progressed {
                return;
            }
        }
        panic!("agent channel settles within the existing peer bound");
    }
}

/// Encode a concrete saved assistant turn, with the supplied text.
pub(crate) fn text_turn(agent: &crate::settings::Agent, number: u32, text: &[u8]) -> Box<[u8]> {
    use smith_transcript::v7 as transcript;
    let limits = &agent.limits.transcript;
    let mut blocks = List::with_capacity(1);
    blocks
        .push(transcript::Block::Text(
            transcript::Text::new(limits, transcript::TextParts { text: text.into(), replay: None }).unwrap(),
        ))
        .unwrap();
    let mut messages = List::with_capacity(1);
    messages
        .push(
            transcript::Message::new(limits, transcript::MessageParts { role: transcript::Role::Assistant, blocks })
                .unwrap(),
        )
        .unwrap();
    let turn = transcript::Turn::new(
        limits,
        transcript::TurnParts {
            system: None,
            kind: transcript::TurnKind::Ordinary,
            opening: turn_opening(limits, number),
            endpoint: b"fake".as_slice().into(),
            model: b"small".as_slice().into(),
            effort: None,
            dialect: fixtures::DIALECT,
            place: number,
            usage: transcript::Usage::new(
                limits,
                transcript::UsageParts {
                    input: None,
                    output: None,
                    cache_read: None,
                    cache_write: None,
                    reasoning: None,
                },
            )
            .unwrap(),
            spent: 0,
            jobs: 0,
            messages,
        },
    )
    .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(turn.measure()).unwrap());
    turn.encode(&mut writer).unwrap();
    writer.finish()
}

fn agent_spawn(worker: &mut crate::Worker) -> Token {
    for _ in 0..worker.io().len() {
        let request = worker.io().pop().expect("queued IO");
        if let io::Request::Spawn { owner, .. } = request {
            return owner;
        }
        worker.io().push(request);
    }
    panic!("assigned process spawn");
}

fn turn_opening(limits: &smith_transcript::v7::Limits, number: u32) -> Option<smith_transcript::v7::Opening> {
    use smith_transcript::v7 as t;
    if number != 1 {
        return None;
    }
    Some(
        t::Opening::new(
            limits,
            t::OpeningParts {
                affinity: t::Affinity::new(limits, t::AffinityParts { key: [0; 16], thread: 0 }).unwrap(),
                account: None,
                backend: t::Backend::Direct,
            },
        )
        .unwrap(),
    )
}

// Each peer leaves the other peer's IO queued. This lets the same concrete
// byte drivers share one Worker without granting the other's output credit.
fn stream_request(request: &io::Request, streams: &[Token]) -> bool {
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

fn assigned_agent_with(
    worker: &mut crate::Worker,
    _agent: &crate::settings::Agent,
    assignment: link::Request,
) -> Token {
    let link::Request::Down { down, .. } = assignment else { panic!("assignment") };
    worker.offer(root::Event::Link(down));
    for _ in 0..worker.io().len() {
        let request = worker.io().pop().unwrap();
        if let io::Request::Spawn { owner, .. } = &request {
            let owner = *owner;
            worker.io().push(request);
            return worker.process_owner(owner).expect("spawn belongs to the worker process");
        }
        worker.io().push(request);
    }
    panic!("the assigned chat spawns smith");
}

pub(crate) fn socket_stand_with(
    settings: &crate::settings::Worker,
    agent: &crate::settings::Agent,
) -> fixtures::WorkerStand {
    socket_stand_with_limits(settings, agent, &fixtures::worker_limits())
}

pub(crate) fn socket_stand_with_limits(
    settings: &crate::settings::Worker,
    agent: &crate::settings::Agent,
    own: &worker_settings::Limits,
) -> fixtures::WorkerStand {
    let mut stand = fixtures::WorkerStand::new();
    stand.worker =
        crate::Worker::new(settings, agent, own, &fixtures::worker_files(), fixtures::worker_runtime(), 7).unwrap();
    for _ in 0_u16..1000 {
        if !stand.turn() {
            return stand;
        }
    }
    panic!("socket worker starts");
}

use jig_domain_core::{self as core, settings};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, host_tool::host_tool, protocol, store::Store};

/// QA's approximate saved byte count, in eight turns below the 65,536-byte turn bound.
pub(crate) const ASKING_HISTORY_BYTES: usize = 470_054;
const HISTORY_TURNS: u32 = 8;
const SAVED_TURNS: u32 = 16; // Eight saved turns and subsequent scripted replies.
const HISTORY_TEXT_BYTES: usize = 60_000;
/// Stand-in question for the walk's unavailable description wording.
pub(crate) const DESCRIPTION: &[u8] = b"Describe how our work runs after accepting the proposal.";
const CHANNEL: Token = Token::new(7);

/// A stock Ask temper chat on the real engine and ordered fake store.
pub(crate) struct AskingChat {
    pub(crate) driver: Driver,
}

impl AskingChat {
    /// Start stock settings, a signed-in owner, an account and a worker.
    pub(crate) fn new() -> Self {
        let setup = fixtures::engine();
        let mut limits = crate::limits::engine_candidate(&setup.jig.limits, &setup.limits);
        let mut core = jig_protocol_config::core(&setup.jig, [31; 16], 91);
        core.kinds = crate::stock::kinds(&limits, 1);
        core.workers_in_process = false;
        core.settings.resume_bytes = 1_048_576;
        core.settings.tail_bytes = 1024;
        core.hub_turns = HISTORY_TURNS;
        for owner in &mut core.owners {
            owner.identity = temper_engine_domain_forge::user_key(1, 7_u64);
        }
        let period_budget = core.defaults.rules.period_spend;
        limits.people.amendment_bytes = 2048;
        limits.tasks.authority_bytes = 256;
        limits.tasks.authority_grants = 16;
        limits.tasks.authority_segments = 8;
        limits.journal.transcript_bytes = 1_048_576;
        limits.loads.transcript_bytes = 1_048_576;
        limits.loads.bytes = 2_097_152;
        limits.loads.reply_bytes = 2_097_152;
        limits.loads.read_bytes = 2_097_152;
        let config = engine::Config {
            host_part: None,
            kinds_stock: setup.kinds_stock,
            ways_stock: Box::default(),
            period_span: crate::stock::PERIOD_SPAN,
            core,
            landings: setup.landings,
            forge: skein_forge_client::Config { namespace: Box::new([31; 16]), writers: Box::new([]) },
            identities: Box::new([]),
            machine: None,
            forges: Box::new([]),
            under_box: false,
            playground: None,
        };
        let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
        driver.period_sent = true;
        driver.settle();
        driver.sign_in();
        driver.settle();
        let mut world = Self { driver };
        world.account();
        world.driver.send(engine::Event::Party(engine::PartyInput::Period {
            project: 1,
            period: 1,
            budget: period_budget,
            next: None,
        }));
        world.driver.settle();
        world
    }

    fn account(&mut self) {
        self.driver.send(engine::Event::Party(engine::PartyInput::Key {
            reply_to: skein_lib::ReplyTo::new(Token::new(10)),
            sign_in: self.driver.session(),
            key: [10; 16],
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Deployment,
                change: people::AccountChange::AddKey { provider: 1, join: None, digest: [10; 32] },
            }),
            carried: core::Carried { number: 1_u64 << 63, digest: [10; 32] },
        }));
        self.driver.settle();
        let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) =
            protocol::hello(CHANNEL, Box::new([]))
        else {
            panic!("hello")
        };
        let mut hello = hello;
        hello.bounds = fixtures::worker_agent().bounds;
        hello.bounds.unacknowledged = HISTORY_TURNS;
        self.driver.send(engine::Event::Worker(engine::WorkerInput::Link {
            channel: CHANNEL,
            link: engine::Link::Hello { worker: 0, hello },
        }));
        self.driver.settle();
    }

    /// Submit a keyed owner's request and settle its durable deliveries.
    pub(crate) fn ask(&mut self, key: u8, ask: people::Ask) {
        self.driver.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(Token::new(u64::from(key))),
            sign_in: self.driver.session(),
            key: [key; 16],
            ask,
        }));
        self.driver.settle();
    }

    /// Create a standing Ask temper chat with stock authority.
    pub(crate) fn chat(&mut self, key: u8) -> engine::Assignment {
        let mut ask = self.driver.chat_ask(1, DESCRIPTION);
        let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat") };
        batch[0].executor = people::TaskExecutor::Agent { charter: 11 };
        batch[0].contract = people::Contract::Report { words: 32 };
        batch[0].budget = people::Budget::AsAllowed { cap: Some(128) };
        batch[0].label = 1;
        batch[0].flags.ended_by = people::EndedBy::Requester;
        self.ask(key, ask);
        self.driver.last_assignment()
    }

    /// Script document reads and a whole settings proposal, then accept with running work staying.
    #[expect(
        clippy::disallowed_methods,
        clippy::disallowed_macros,
        reason = "the scripted model formats whole JSON settings and reads its durable proposal"
    )]
    pub(crate) fn accept(&mut self, run: &engine::Assignment) {
        let (_, error) = host_tool(&mut self.driver, run, 1, b"see", br#"{"address":"settings"}"#);
        assert!(!error);
        let mut value = self.read_chat_charter(run);
        value["instructions"] = serde_json::json!("Push early and report the result.");
        let input = serde_json::json!({"against":1_u32,"reason":"Explain running work",
            "items":[{"setting":{"kind":"charter","number":1_u32},"set":value}]});
        let (body, error) = host_tool(&mut self.driver, run, 100, b"propose_settings", input.to_string().as_bytes());
        assert!(!error, "proposal: {:?}", ::core::str::from_utf8(&body));
        let proposal = self
            .driver
            .store
            .rows
            .values()
            .find_map(|row| {
                if let engine::Record::Core(core::Record::Core(core::CoreRecord::Settings(
                    settings::Record::Proposal { proposal, .. },
                ))) = row
                {
                    Some(u64::from(*proposal))
                } else {
                    None
                }
            })
            .expect("scripted proposal retained");
        self.ask(
            21,
            people::Ask::DecideProposal {
                project: people::DEPLOYMENT,
                proposer: people::Proposer::Task(run.task),
                proposal,
                decision: people::ProposalDecision::Accept,
                running: Some(people::Running::Stays),
            },
        );
    }

    #[expect(
        clippy::disallowed_types,
        clippy::disallowed_methods,
        clippy::disallowed_macros,
        reason = "the scripted model formats continuation addresses and joins paginated UTF-8 settings into JSON"
    )]
    fn read_chat_charter(&mut self, run: &engine::Assignment) -> serde_json::Value {
        let mut from = 0;
        let mut address = String::from("settings charter 1");
        let mut text = Vec::new();
        for completion in 2..100 {
            let input = serde_json::json!({"address":address});
            let (body, error) = host_tool(&mut self.driver, run, completion, b"see", input.to_string().as_bytes());
            assert!(!error, "settings page");
            let page = ::core::str::from_utf8(&body).unwrap();
            let end = page.find("more: ").or_else(|| page.find("see: ")).unwrap();
            if let Some(next) = page.lines().find_map(|line| line.strip_prefix("more: ")) {
                let offset: usize = next.rsplit_once('@').unwrap().1.parse().unwrap();
                text.extend_from_slice(page.as_bytes().get(..offset.checked_sub(from).unwrap()).unwrap());
                from = offset;
                address = format!("more {next}");
            } else {
                text.extend_from_slice(page.as_bytes().get(..end).unwrap());
                let text = ::core::str::from_utf8(&text).unwrap();
                let (_, object) = text.split_once('\n').unwrap();
                return serde_json::from_str(object.trim()).unwrap();
            }
        }
        panic!("the scripted charter read reaches its end");
    }

    /// Commit valid history at the walk's size, park, then send a stand-in description question.
    pub(crate) fn resume(&mut self, run: &engine::Assignment) -> engine::Assignment {
        let history = asking_history();
        let read = match self.driver.task(run.task).unwrap().inbox.last() {
            Some(message) => Some(message.number),
            None => None,
        };
        for (index, body) in history.iter().enumerate() {
            let number = u32::try_from(index).unwrap().checked_add(1).unwrap();
            self.driver.send(protocol::turn(CHANNEL, run.task, run.attempt, number, 0, read, body));
            self.driver.settle();
        }
        self.driver.answer(CHANNEL, run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
        self.driver.settle();
        self.ask(22, people::Ask::Say { project: 1, task: run.task, words: DESCRIPTION.into() });
        self.driver.last_assignment()
    }

    /// Feed the concrete worker's terminal through the engine's startup translation.
    pub(crate) fn answer(&mut self, answer: AskingAnswer) {
        if let Some(turn) = answer.turn {
            self.driver.send(protocol::up(CHANNEL, turn, engine::Parts::none()));
            self.driver.settle();
        }
        self.driver.send(protocol::up(CHANNEL, answer.terminal, engine::Parts::none()));
        self.driver.settle();
    }

    /// Advance only to the task's retry deadline, preserving the configured allowance.
    pub(crate) fn retry(&mut self) -> engine::Assignment {
        let due = self.driver.root.next_deadline(&self.driver.env).expect("retry deadline");
        self.driver.env.now = due;
        engine::fire(&mut self.driver.root, &self.driver.env);
        self.driver.settle();
        self.driver.last_assignment()
    }
}

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "the saved-history fixture sizes eight concrete turns to the walk's byte count"
)]
fn asking_history() -> Box<[Box<[u8]>]> {
    let agent = fixtures::worker_agent();
    let mut turns = Vec::new();
    let mut remaining = ASKING_HISTORY_BYTES;
    for number in 1..=HISTORY_TURNS {
        let overhead = text_turn(&agent, number, b"").len();
        let bytes = if number == HISTORY_TURNS { remaining.checked_sub(overhead).unwrap() } else { HISTORY_TEXT_BYTES };
        let text = ::core::iter::repeat_n(b'x', bytes).collect::<Vec<_>>();
        let turn = text_turn(&agent, number, &text);
        remaining = remaining.checked_sub(turn.len()).unwrap();
        turns.push(turn);
    }
    assert_eq!(remaining, 0, "the saved stream has the walk's approximate size");
    turns.into_boxed_slice()
}

/// The fake agent's behavior after receiving a concrete Start.
pub(crate) enum ResumeAttempt {
    /// End the channel without admission.
    Fault,
    /// Identify unusable history without taking pending words.
    Transcript,
    /// Admit, answer the pending question and park.
    Answer,
}

/// Concrete worker records, including the answer turn before its terminal.
pub(crate) struct AskingAnswer {
    /// The attempt's terminal, decoded from the worker wire.
    pub(crate) terminal: hub::LinkUp,
    /// The description turn, decoded from the worker wire before its terminal.
    pub(crate) turn: Option<hub::LinkUp>,
}

/// Start through the concrete worker/channel path with the engine's retained history.
pub(crate) fn asking_attempt(run: engine::Assignment, behavior: ResumeAttempt) -> AskingAnswer {
    let (settings, agent) = asking_worker_settings();
    let mut stand = socket_stand_with(&settings, &agent);
    asking_grants(&mut stand.worker, &run.grants);
    let mut link = LinkPeer::with_settings(&settings, &agent);
    link.open(&mut stand.worker);
    let encoding = fixtures::encoding(&fixtures::engine());
    let engine_link::Event::Down { down, parts, .. } = crate::assignment::down(
        run.clone(),
        &fixtures::forge_table(&fixtures::forges()),
        &encoding,
        &crate::values::Values::new(2, 4096),
        CHANNEL,
        fixtures::NOW,
    )
    .expect("Ask temper assignment encodes") else {
        panic!("assignment")
    };
    let (mut peer, start) = AgentPeer::pending(&mut stand.worker, &agent, link::Request::Down { down, parts });
    assert_eq!(start.transcript.is_some(), !run.transcript.is_empty(), "startup retains the history choice");
    assert!(question_reaches(&start), "the pending question or fresh brief reaches Start");
    let turn = match behavior {
        ResumeAttempt::Fault => {
            peer.end_process(&mut stand.worker, b"resume channel ended before admission");
            None
        }
        ResumeAttempt::Transcript => {
            transcript_refusal(&mut peer, &mut stand.worker);
            None
        }
        ResumeAttempt::Answer => {
            Some(scripted_description(&mut stand.worker, &mut link, &mut peer, &agent, &start, &run))
        }
    };
    let (terminal, _) = link.answer(&mut stand.worker);
    AskingAnswer { terminal, turn }
}

fn question_reaches(start: &smith_protocol_channel::DecodedStart) -> bool {
    for message in &start.messages {
        if skein_lib::bytes::find(&message.text, DESCRIPTION).is_some() {
            return true;
        }
    }
    for section in &start.charter.brief.sections {
        if skein_lib::bytes::find(&section.text, DESCRIPTION).is_some() {
            return true;
        }
    }
    false
}

fn asking_worker_settings() -> (crate::settings::Worker, crate::settings::Agent) {
    let mut settings = fixtures::worker_settings();
    settings.jig.engine = jig_protocol_config::Address::Addr(io::Addr::Ip(([127, 0, 0, 1], 7100).into()));
    settings.jig.limits.hub.transcript_bytes = 2_097_152;
    settings.jig.limits.hub.turns = SAVED_TURNS;
    settings.jig.limits.hub.turn_queue_bytes = 1_048_576;
    let mut agent = fixtures::worker_agent();
    agent.host_limits.transcript_bytes = 2_097_152;
    agent.host_limits.turns = SAVED_TURNS;
    agent.limits.bodies.start_transcript = 1_048_576;
    agent.limits.channel.chunk = 4096;
    agent.limits.channel.output_bytes = smith_host_protocol::output_bytes(&agent.limits).unwrap();
    (settings, agent)
}

fn asking_grants(worker: &mut crate::Worker, grants: &[Grant]) {
    for grant in grants {
        worker.link(
            fixtures::NOW,
            link::Request::Value {
                account: grant.account,
                generation: grant.generation,
                expires: Time::from_nanos(u64::MAX),
                value: link::GrantValue::Credential {
                    scheme: link::Scheme::Key,
                    token: skein_lib::Secret::new(b"fake-model-key".as_slice().into()),
                },
            },
        );
    }
}

fn scripted_description(
    worker: &mut crate::Worker,
    link: &mut LinkPeer,
    peer: &mut AgentPeer,
    agent: &crate::settings::Agent,
    start: &smith_protocol_channel::DecodedStart,
    run: &engine::Assignment,
) -> hub::LinkUp {
    peer.component.send_admitted(peer.owner, &mut peer.events, &mut peer.out).unwrap();
    peer.settle(worker);
    let read = match start.messages.last() {
        Some(message) => Some(message.name),
        None => None,
    };
    let sequence = description_sequence(start);
    let body = text_turn(agent, sequence, b"Accepted settings apply to new work; running work stays on its revision.");
    let turn =
        smith_protocol_channel::decode_turn(&body, sequence, &agent.limits.transcript, &agent.endpoints).unwrap();
    peer.component
        .send_turn(
            1,
            read,
            smith_domain_run::Spend::ZERO,
            start.transcript.is_none(),
            &turn,
            peer.owner,
            &mut peer.events,
            &mut peer.out,
        )
        .unwrap();
    peer.settle(worker);
    let answer_turn = kept_description(worker, link, peer, run);
    peer.component
        .send_answer(
            fixtures::NOW,
            smith_domain_run::Answer::Parked { spent: smith_domain_run::Spend::ZERO, turns: 1 },
            read,
            peer.owner,
            &mut peer.events,
            &mut peer.out,
        )
        .unwrap();
    peer.settle(worker);
    peer.end_process(worker, b"");
    answer_turn
}

fn description_sequence(start: &smith_protocol_channel::DecodedStart) -> u32 {
    match &start.transcript {
        Some(history) => match history.turns.last() {
            Some(turn) => turn.sequence.checked_add(1).unwrap(),
            None => 1,
        },
        None => 1,
    }
}

fn kept_description(
    worker: &mut crate::Worker,
    link: &mut LinkPeer,
    peer: &mut AgentPeer,
    run: &engine::Assignment,
) -> hub::LinkUp {
    let mut answer_turn = None;
    for _ in 0..PEER_STEPS {
        peer.settle(worker);
        if let Some(engine_link::Request::Up { up: up @ hub::LinkUp::Turn { .. }, .. }) = link.turn(worker) {
            answer_turn = Some(up);
            worker.link(
                fixtures::NOW,
                link::Request::Down {
                    down: hub::LinkDown::TurnKept {
                        run: Token::new(run.task),
                        attempt: Token::new(run.attempt),
                        turn: 1,
                    },
                    parts: Box::new([]),
                },
            );
            break;
        }
    }
    peer.settle(worker);
    answer_turn.expect("the description answer crosses the worker wire")
}

fn transcript_refusal(peer: &mut AgentPeer, worker: &mut crate::Worker) {
    peer.component.send_admitted(peer.owner, &mut peer.events, &mut peer.out).unwrap();
    peer.settle(worker);
    peer.component
        .send_answer(
            fixtures::NOW,
            smith_domain_run::Answer::Failed {
                failure: smith_domain_run::RunFailure::Transcript(smith_domain_run::TranscriptRefusal::Malformed),
                spent: smith_domain_run::Spend::ZERO,
                turns: 0,
            },
            None,
            peer.owner,
            &mut peer.events,
            &mut peer.out,
        )
        .unwrap();
    peer.settle(worker);
    peer.end_process(worker, b"transcript refused");
}

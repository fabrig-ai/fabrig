//! Admission under pressure, through the real worker and its concrete peers.
use super::fixtures;
use super::fixtures::worker_peers::{
    AGENT_CHILD, AGENT_ERROR, AGENT_INPUT, AGENT_OUTPUT, AGENT_OWNER, AgentPeer, LinkPeer, LinkRecord, PEER_STEPS,
    cancellation_turn, demanded_bytes, socket_stand,
};
use jig_domain_host as hub;
use jig_protocol_worker_link as link;
use skein_channel::{Lower, LowerEvent};
use skein_io as io;
use skein_lib::{Duration, Time, Token, Wall, stream};
use temper_worker_domain as root;

// Two queued IO requests reach the mark; one leaves a single entry's credit.
const WATERMARK: u32 = 2;
const BACKLOG_ENTITY: Token = Token::new(901);
const NEXT_RUN: Token = Token::new(902);
const PROGRESS_AFTER: Duration = Duration::from_secs(1);
// The automatic start can report Opened and Start before the peer takes them.
const OPENING_EVENTS: u32 = 2;
// The pass-by scenario sends one turn and then holds one new assignment.
const SENT_TURNS: u32 = 1;
const HELD_ASSIGNMENTS: u32 = 1;

fn assignment() -> link::Request {
    let link::Request::Down { down: hub::LinkDown::Assign { mut assignment }, parts } = fixtures::chat_assignment()
    else {
        panic!("chat assignment")
    };
    assignment.assignment.run = NEXT_RUN;
    link::Request::Down { down: hub::LinkDown::Assign { assignment }, parts }
}

fn load(worker: &mut crate::Worker, entries: u32) {
    worker.env.limits.watermark = WATERMARK;
    for _ in 0..entries {
        worker.io().push(io::Request::Close { entity: BACKLOG_ENTITY });
    }
}

fn waiting_assignment() -> (fixtures::WorkerStand, LinkPeer) {
    let mut stand = fixtures::started_worker();
    let mut engine = LinkPeer::new();
    engine.open(&mut stand.worker);
    load(&mut stand.worker, WATERMARK);
    stand.worker.link(fixtures::NOW, assignment());
    (stand, engine)
}

impl AgentPeer {
    /// Attach the same concrete agent peer to a spawn requested by the real
    /// root, letting the root drive its start and reads.
    fn assigned_for_admission(worker: &mut crate::Worker) -> Self {
        let link::Request::Down { down: hub::LinkDown::Assign { mut assignment }, parts } = fixtures::chat_assignment()
        else {
            panic!("chat assignment")
        };
        assignment.assignment.live.text = true;
        worker.link(fixtures::NOW, link::Request::Down { down: hub::LinkDown::Assign { assignment }, parts });
        worker.iterate(fixtures::NOW, Wall::EPOCH);
        let io::Request::Spawn { owner, .. } = worker.io().pop().expect("assigned agent spawn") else {
            panic!("assignment spawns its agent");
        };
        let mut peer = Self::new(&fixtures::worker_agent(), AGENT_OWNER);
        peer.events = skein_lib::Queue::with_capacity(OPENING_EVENTS);
        worker.up(
            fixtures::NOW,
            io::Event::Spawned { owner, child: AGENT_CHILD, pipes: Box::new([AGENT_INPUT, AGENT_OUTPUT, AGENT_ERROR]) },
        );
        peer.settle(worker);
        // The peer has received Opened and the root's real Start.
        while peer.events.pop().is_some() {}
        peer.component.send_admitted(AGENT_OWNER, &mut peer.events, &mut peer.out).unwrap();
        peer.settle(worker);
        peer
    }

    /// Deliver one checked live frame into the process adapter without
    /// iterating the worker, leaving its progress for the admission stage.
    fn queue_progress(&mut self, worker: &mut crate::Worker) {
        self.component
            .send_live(
                smith_protocol_channel::Piece {
                    kind: smith_protocol_channel::PieceKind::Text,
                    turn: 1,
                    retry: 0,
                    offset: 0,
                    bytes: b"progress",
                },
                AGENT_OWNER,
                &mut self.events,
                &mut self.out,
            )
            .unwrap();
        while let Some(request) = self.out.pop() {
            match request {
                Lower::Write(stream::OutputDown::Room { right, .. }) => {
                    self.lower(LowerEvent::Write(stream::OutputUp::Settled {
                        right,
                        outcome: stream::OutputOutcome::Granted,
                    }));
                }
                Lower::Write(stream::OutputDown::Send { bytes, .. }) => self.worker_bytes.append(&bytes).unwrap(),
                other @ (Lower::Read(_) | Lower::Write(_) | Lower::FinishWrite) => {
                    panic!("live frame output: {other:?}");
                }
            }
        }
        for _ in 0..PEER_STEPS {
            if let Some(bytes) = demanded_bytes(&mut self.worker_read, &mut self.worker_bytes) {
                worker.up(fixtures::NOW, io::Event::Stream { owner: AGENT_OUTPUT, up: stream::Up::Bytes(bytes) });
                if self.worker_bytes.is_empty() {
                    return;
                }
            }
            // Header and body each spend their own demand; stop after the
            // body, before the root takes the resulting progress event.
            worker.iterate(fixtures::NOW, Wall::EPOCH);
            while let Some(request) = worker.io().pop() {
                self.worker_request(worker, request);
            }
        }
        panic!("the complete live frame reaches the process adapter");
    }
}

fn progress_at(entries: u32) -> (fixtures::WorkerStand, Time) {
    let mut stand = fixtures::started_worker();
    let mut agent = AgentPeer::assigned_for_admission(&mut stand.worker);
    agent.queue_progress(&mut stand.worker);
    load(&mut stand.worker, entries);
    stand.worker.link(fixtures::NOW, assignment());
    let before = stand.worker.root.next_deadline().expect("the admitted agent has a watchdog");
    stand.worker.iterate(fixtures::NOW.saturating_add(PROGRESS_AFTER), Wall::EPOCH);
    (stand, before)
}

#[test]
fn new_work_waits_at_the_watermark_while_admitted_work_is_taken() {
    let (mut stand, before) = progress_at(WATERMARK);
    assert_eq!(stand.worker.held.len(), HELD_ASSIGNMENTS, "the new assignment waits");
    assert!(stand.worker.engine.is_empty());
    assert_eq!(
        stand.worker.root.next_deadline(),
        Some(before.saturating_add(PROGRESS_AFTER)),
        "agent progress feeds its watchdog"
    );
    for request in stand.worker.io().iter() {
        if let io::Request::Spawn { .. } = request {
            panic!("waiting work prepares no checkout and spawns no agent");
        }
    }
}

#[test]
fn new_work_is_taken_one_entry_below_the_watermark() {
    let (stand, before) = progress_at(WATERMARK - 1);
    assert!(stand.worker.engine.is_empty(), "the assignment was stepped");
    assert!(stand.worker.held.is_empty());
    assert_eq!(stand.worker.root.host().hosted(), 1, "the occupied worker keeps its admitted run");
    assert_eq!(stand.worker.root.next_deadline(), Some(before.saturating_add(PROGRESS_AFTER)));
}

#[test]
fn the_engines_later_records_wait_behind_its_waiting_new_work() {
    let (mut stand, mut engine) = waiting_assignment();
    stand.worker.link(
        fixtures::NOW,
        link::Request::Down {
            down: hub::LinkDown::Cancel { run: NEXT_RUN, attempt: Token::new(2) },
            parts: Box::new([]),
        },
    );
    stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    let waiting = stand.worker.held.len();
    let hosted_while_waiting = stand.worker.root.host().hosted();
    // Release the IO backlog; both records can now cross, in their order.
    while stand.worker.io().pop().is_some() {}
    stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    let mut spawned = None;
    let mut link_output = skein_lib::Queue::uncapped();
    while let Some(request) = stand.worker.io().pop() {
        if let io::Request::Spawn { owner, .. } = request {
            spawned = Some(owner);
        } else {
            link_output.push(request);
        }
    }
    while let Some(request) = link_output.pop() {
        stand.worker.io().push(request);
    }
    let owner = spawned.expect("assignment starts before cancel");
    stand.worker.up(fixtures::NOW, io::Event::Failed { owner, error: io::Error::Unreachable });
    stand.worker.up(fixtures::NOW, io::Event::Closed { owner });
    let (up, _) = engine.answer(&mut stand.worker);
    assert_eq!((waiting, hosted_while_waiting), (2, 0));
    let hub::LinkUp::Answer { run, attempt, answer } = up else { panic!("the engine gets the cancellation answer") };
    assert_eq!((run, attempt), (NEXT_RUN, Token::new(2)));
    assert_eq!(
        answer.ending,
        hub::Ending::Failed {
            failure: hub::Failure::Cancelled(hub::Reason::Engine),
            model: None,
            detail: Box::new([]),
            work: hub::Work { left: None, saved: None },
        }
    );
    assert_eq!(stand.worker.root.kept_answers(), 1);
    assert!(stand.worker.engine.is_empty());
    assert!(stand.worker.held.is_empty());
}

#[test]
fn an_answer_about_admitted_turns_passes_a_held_assignment() {
    let mut stand = fixtures::started_worker();
    let settings = fixtures::worker_agent();
    let mut agent = AgentPeer::assigned(&mut stand.worker, &settings);
    let body = cancellation_turn(&settings);
    let turn = smith_protocol_channel::decode_turn(&body, SENT_TURNS, &settings.limits.transcript, &settings.endpoints)
        .unwrap();
    agent
        .component
        .send_turn(
            SENT_TURNS,
            None,
            smith_domain_run::Spend::ZERO,
            true,
            &turn,
            AGENT_OWNER,
            &mut agent.events,
            &mut agent.out,
        )
        .unwrap();
    agent.settle(&mut stand.worker);
    let retained = stand.worker.root.retained_turns();
    let hosting = stand.worker.root.hosting();
    let admitted = hosting.get(0).expect("the admitted run");
    load(&mut stand.worker, WATERMARK);
    stand.worker.link(fixtures::NOW, assignment());
    stand.worker.link(
        fixtures::NOW,
        link::Request::Down {
            down: hub::LinkDown::TurnKept { run: admitted.run, attempt: admitted.attempt, turn: SENT_TURNS },
            parts: Box::new([]),
        },
    );
    stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    assert_eq!(retained, SENT_TURNS, "the admitted turn waited for the engine to keep it");
    assert_eq!(stand.worker.root.retained_turns(), 0, "the kept turn passes and is released");
    assert_eq!(stand.worker.held.len(), HELD_ASSIGNMENTS, "the assignment still waits at the watermark");
    assert!(stand.worker.engine.is_empty(), "the answer was taken from behind the assignment");
}

#[test]
fn the_links_loss_waits_behind_a_held_assignment() {
    let (mut stand, _engine) = waiting_assignment();
    stand.worker.link(fixtures::NOW, link::Request::Lost);
    stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    let mut held = stand.worker.held.iter();
    let Some(root::Event::Link(hub::LinkDown::Assign { .. })) = held.next() else {
        panic!("the assignment is held first");
    };
    assert_eq!(held.next(), Some(&root::Event::Lost), "loss waits after the assignment (W5)");
    assert!(held.next().is_none());
    assert_eq!(stand.worker.root.host().hosted(), 0, "the assignment was held");
    assert!(stand.worker.engine.is_empty());
}

#[test]
fn a_worker_whose_engine_records_wait_keeps_its_link() {
    let mut stand = socket_stand();
    let mut peer = LinkPeer::new();
    peer.open(&mut stand.worker);
    while stand.worker.notes().pop().is_some() {}
    load(&mut stand.worker, WATERMARK);
    let room = stand.worker.held.room();
    for _ in 0..room {
        stand.worker.link(fixtures::NOW, assignment());
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    }
    stand.worker.link(fixtures::NOW, assignment());
    assert_eq!(stand.worker.held.room(), 0);
    assert_eq!(stand.worker.engine.len(), 1);
    let received = peer.received;
    let mut service = super::worker::alone_service();
    service.worker = stand.worker;
    for seconds in 1..=20 {
        let now = fixtures::NOW.saturating_add(Duration::from_secs(seconds));
        peer.now = now;
        peer.component.fire(now, &mut peer.out);
        // Settle demanded headers and bodies on this second's clock. The
        // backlog stays at the watermark throughout every service pass.
        for _ in 0_u8..32 {
            let worker = &mut service.worker;
            // The concrete peer takes the socket requests before the service
            // publishes them to its separate IO machine.
            peer.turn(worker);
            for record in &peer.records {
                assert!(
                    *record != LinkRecord::Lost && *record != LinkRecord::Ended,
                    "the engine lost a healthy worker at {seconds}s while its records waited"
                );
            }
            assert!(worker.notes().is_empty(), "the worker's linked link gives no loss fact");
        }
        crate::worker_alone::iterate(&mut service, now, Wall::EPOCH);
        load(&mut service.worker, WATERMARK);
    }
    assert!(peer.received > received, "the idle worker's pings reach the engine");
}

#[test]
fn a_new_wire_record_past_the_entrance_faults_the_link() {
    use jig_protocol_engine_workers as engine_link;
    let mut stand = socket_stand();
    let mut peer = LinkPeer::new();
    peer.open(&mut stand.worker);
    while stand.worker.notes().pop().is_some() {}
    load(&mut stand.worker, WATERMARK);
    let entrance = peer.engine_frames.checked_add(3).unwrap();
    for _ in 0..entrance {
        stand.worker.link(fixtures::NOW, assignment());
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    }
    assert_eq!(stand.worker.held.room(), 0, "the engine cap and three state slots fill the entrance");
    let link::Request::Down { down, parts } = assignment() else { panic!("assignment") };
    peer.component.down(
        fixtures::NOW,
        engine_link::Event::Down { channel: Token::new(1), down, parts, values: Box::new([]) },
        &mut peer.out,
    );
    for _ in 0..PEER_STEPS {
        peer.exchange(&mut stand.worker);
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
        if let Some(note) = stand.worker.notes().pop() {
            assert_eq!(
                note,
                crate::worker_alone::Note::Lost {
                    worker: b"worker-a".as_slice().into(),
                    address: Box::new(([127, 0, 0, 1], 7100).into()),
                    why: link::Loss::Fault,
                }
            );
            assert_eq!(stand.worker.held.len(), stand.worker.held.capacity());
            for event in &stand.worker.engine {
                assert!(!root::is_new_work(event), "the extra record is dropped");
            }
            assert!(stand.worker.notes().is_empty());
            return;
        }
    }
    panic!("a wire record past the entrance faults the link");
}

#[test]
fn the_link_is_read_while_its_last_output_waits() {
    let mut stand = socket_stand();
    let mut engine = LinkPeer::new();
    engine.open(&mut stand.worker);
    while stand.worker.notes().pop().is_some() {}
    stand.worker.engine.push(root::Event::Lost);
    // Reading never waits for the root (temper/docs/design/domain/worker/README.md, 3.1).
    let mut service = super::worker::alone_service();
    service.worker = stand.worker;
    let owner = engine.owner;
    service.process.told().push(io::Event::Closed { owner });
    crate::worker_alone::iterate(&mut service, fixtures::NOW, Wall::EPOCH);
    let waiting = service.process.told().len();
    assert_eq!(waiting, 0, "socket input is routed while the last engine record waits");
}

#[test]
fn a_full_held_entrance_lets_a_passing_answer_pass() {
    let mut stand = fixtures::started_worker();
    let settings = fixtures::worker_agent();
    let mut agent = AgentPeer::assigned(&mut stand.worker, &settings);
    let body = cancellation_turn(&settings);
    let turn = smith_protocol_channel::decode_turn(&body, SENT_TURNS, &settings.limits.transcript, &settings.endpoints)
        .unwrap();
    agent
        .component
        .send_turn(
            SENT_TURNS,
            None,
            smith_domain_run::Spend::ZERO,
            true,
            &turn,
            AGENT_OWNER,
            &mut agent.events,
            &mut agent.out,
        )
        .unwrap();
    agent.settle(&mut stand.worker);
    let retained = stand.worker.root.retained_turns();
    let hosting = stand.worker.root.hosting();
    let admitted = hosting.get(0).expect("the admitted run");
    load(&mut stand.worker, WATERMARK);
    for _ in 0..stand.worker.held.capacity() {
        stand.worker.link(fixtures::NOW, assignment());
        stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    }
    assert_eq!(stand.worker.held.room(), 0, "new assignments fill the held entrance");
    stand.worker.link(
        fixtures::NOW,
        link::Request::Down {
            down: hub::LinkDown::TurnKept { run: admitted.run, attempt: admitted.attempt, turn: SENT_TURNS },
            parts: Box::new([]),
        },
    );
    let answer = stand.worker.engine.iter().next().expect("the engine's answer");
    assert!(root::passes(answer), "the head answers an admitted turn");
    stand.worker.link(fixtures::NOW, assignment());
    assert_eq!(stand.worker.engine.len(), 2, "a new record follows the passing answer");
    assert_eq!(retained, SENT_TURNS, "the admitted turn waits for its answer");
    stand.worker.iterate(fixtures::NOW, Wall::EPOCH);
    assert_eq!(stand.worker.root.retained_turns(), 0, "the passing answer reaches its owner without a held slot");
    assert_eq!(stand.worker.held.room(), 0, "the held entrance remains full");
    assert_eq!(stand.worker.held.len(), stand.worker.held.capacity());
    assert_eq!(stand.worker.engine.len(), 1, "the new record behind the answer still waits");
    let waiting = stand.worker.engine.iter().next().expect("the next new record");
    assert!(root::is_new_work(waiting));
    assert!(!root::passes(waiting), "the scan stops at the first record that cannot pass");
}

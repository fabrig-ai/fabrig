//! Workers stopped mid-turn (`temper/docs/design/testing/restarts.md`, 3, 6, 9).
//! The agent is played at its process boundary: an LLM call has not said its
//! next turn; a delivery call runs its check program before asking the worker
//! to push. That program is held here, not a git operation in the worker.
use super::{ATTEMPT, CALL, Case, PATH, REMOTE, RUN, Request, World, agent, host, protocol, translate, wire};
use jig_domain_link::{PauseMode, PauseState};
use skein_lib::{Duration, Rng, Time, Token};
use skein_world::{
    faults::{FaultLedger, Injection, Struck},
    streams::{Stream, Streams},
};
use temper_fake_checkout::{Checkout, git::Remote};
use temper_faults::restarts::{PART, Point, Way, worker};
use temper_worker_domain::{Domain, Encodable, Encoded, Event};

/// Where a crashed worker's replacement process starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recovery {
    SameWorker,
    AnotherWorker,
}

/// Restart draws never advance the forge's or the workload's generator.
struct Injector {
    rng: Rng,
}

impl Injector {
    #[must_use]
    fn new(seed: u64) -> Self {
        Self { rng: Streams::new(seed).rng(Stream::Faults, PART) }
    }

    fn draw(&mut self) -> (Way, Point, Recovery) {
        let way = match self.rng.below(3) {
            0 => Way::Drained,
            1 => Way::Signalled,
            2 => Way::Crashed,
            _ => unreachable!("three ways"),
        };
        let point = match self.rng.below(4) {
            0 => Point::LlmCall,
            1 => Point::ToolCall,
            2 => Point::DeliveryCheck,
            3 => Point::Save,
            _ => unreachable!("four points"),
        };
        let recovery =
            if way == Way::Crashed && self.rng.below(2) == 1 { Recovery::AnotherWorker } else { Recovery::SameWorker };
        (way, point, recovery)
    }
}

/// The boundary referee reads each turn's complete body and accounting.
/// The one-turn stories can replay their turn; a named text script must
/// forward each turn once and in order.
#[derive(Default)]
pub(super) struct Observer {
    hellos: Vec<(Time, usize)>,
    turns: Vec<Token>,
    starts: Vec<Token>,
    texts: &'static [&'static [u8]],
    pub(super) signals: Vec<Request>,
}

impl Observer {
    pub(super) fn hello(&mut self, at: Time, hosting: usize) {
        self.hellos.push((at, hosting));
    }

    pub(super) fn started(&mut self, attempt: Token) {
        assert!(!self.starts.contains(&attempt), "each attempt starts once");
        self.starts.push(attempt);
    }

    pub(super) fn turned(&mut self, attempt: Token, turn: &host::Turn) {
        let body = if self.texts.is_empty() {
            assert_eq!(turn.turn, 1, "no unseen turn prefix");
            protocol::transcript_turn(2, 8, false)
        } else {
            let number = u32::try_from(self.turns.len() + 1).expect("bounded story");
            assert_eq!(turn.turn, number, "each turn is forwarded once and in order");
            let text = self.texts.get(self.turns.len()).expect("no turn beyond the released answer");
            let Encoded::Turn(expected) = protocol::encode(Encodable::Turn(protocol::text_turn(number, text))) else {
                panic!("encodable fixture turn");
            };
            expected.body
        };
        assert_eq!(turn.spent, 0);
        assert_eq!(turn.read, None);
        assert!(!turn.opens_window);
        assert!(turn.refused.is_empty());
        assert_eq!(turn.body, body, "complete turn body, including replay");
        self.turns.push(attempt);
    }

    pub(super) fn acknowledges(&self, turn: u32) -> bool {
        (1..=u32::try_from(self.texts.len()).expect("bounded story")).contains(&turn)
    }
}

/// The played agent's pending delivery call stands for its running check.
/// Dropping it kills that command without an answer or a push. The other
/// agent activity is held by World's read credit.
fn hold(world: &mut World, point: Point) -> Option<agent::Ask> {
    match point {
        Point::LlmCall => None,
        Point::ToolCall => {
            world
                .say(agent::Up::Call {
                    call: CALL,
                    name: agent::CallName { activation: world.attempt.raw(), completion: 1, position: 0 },
                    deadline: world.env.now.saturating_add(Duration::from_secs(60)),
                    ask: agent::Ask::Host {
                        tool: b"recall".as_slice().into(),
                        effect: agent::Effect::Read,
                        body: b"{}".as_slice().into(),
                    },
                })
                .expect("worker inspection");
            None
        }
        Point::DeliveryCheck => Some(protocol::delivery(b"Delivery", b"Worker story")),
        Point::Save => {
            world.hold_push = true;
            world.request_delivery(agent::Ask::Save).expect("worker inspection");
            None
        }
    }
}

fn held(world: &World, point: Point, check: Option<&agent::Ask>) {
    assert!(world.reading, "the admitted agent is live at its boundary");
    assert_eq!(world.domain.agent().agents(), 1);
    assert!(world.answer.is_none());
    assert!(world.restart_observer.as_ref().expect("referee").turns.is_empty(), "no next turn said");
    assert_eq!(world.deliveries, 0);
    assert_eq!(world.replies.len(), 0);
    match point {
        Point::LlmCall => {
            assert!(world.call.is_none());
            assert!(check.is_none());
            assert!(world.held.is_none());
        }
        Point::ToolCall => {
            assert!(world.call.is_some(), "host call asked, not answered");
            assert!(check.is_none());
            assert!(world.held.is_none());
        }
        Point::DeliveryCheck => {
            assert!(check.is_some(), "delivery check running");
            assert!(world.call.is_none());
            assert!(world.held.is_none(), "no push begun");
        }
        Point::Save => {
            assert!(check.is_none());
            assert!(matches!(world.held, Some((_, super::Op::Push { .. }))), "save push sent and held");
            assert!(world.delivery_reply.is_none(), "save has no terminal yet");
        }
    }
    assert_eq!(world.forge.branch(REMOTE, b"topic"), Some(world.first), "no push has completed");
}

fn release(world: &mut World, point: Point) {
    match point {
        Point::ToolCall => {
            let call = world.call.take().expect("pending host call");
            world
                .event(Event::Link(host::LinkDown::Relayed {
                    run: RUN,
                    attempt: world.attempt,
                    call,
                    answer: host::Relayed::Answer { error: false, text: b"answer".as_slice().into() },
                }))
                .expect("worker inspection");
            assert_eq!(world.replies.len(), 1);
        }
        Point::Save => {
            world.release_push().expect("worker inspection");
            assert!(matches!(world.delivery_reply, Some(agent::Delivery::Delivered(_))));
        }
        Point::LlmCall => {}
        Point::DeliveryCheck => unreachable!("a check still running is cut at the drain bound"),
    }
}

fn drain(world: &mut World, way: Way, point: Point, check: &mut Option<agent::Ask>) {
    match way {
        Way::Drained => world.event(Event::Link(host::LinkDown::Pause { mode: PauseMode::Drain })),
        Way::Signalled => world.event(Event::Shutdown { drain: true }),
        Way::Crashed => unreachable!("crash has no settle"),
    }
    .expect("worker inspection");
    assert_eq!(world.report.drains, 1, "only the worker receives the initial stop");
    assert_eq!(world.report.cancellations, 0, "the current turn goes on");
    let turns = if point == Point::DeliveryCheck {
        // The check outlives the drain, as a repository's check program can.
        world.env.now = world.env.now.saturating_add(world.env.limits.drain);
        world.settle().expect("worker inspection");
        assert_eq!(world.report.cancellations, 1, "the drain bound cuts the check");
        drop(check.take().expect("running check killed"));
        assert_eq!(world.forge.branch(REMOTE, b"topic"), Some(world.first), "a cut check never pushes");
        0
    } else {
        world.env.now = world.env.now.saturating_add(Duration::from_secs(1));
        release(world, point);
        world.say(agent::Up::Turn { turn: protocol::turn(1, 0, 8) }).expect("worker inspection");
        1
    };
    world
        .say(agent::Up::Answer {
            answer: agent::Answer {
                read: None,
                turns,
                spent: 0,
                result: agent::RunResult::Failed { failure: agent::RunFailure::Cancelled },
            },
        })
        .expect("worker inspection");
    world.gone().expect("worker inspection");
    let (answer, _, preparation) = world.answer.take().expect("drained attempt answered");
    assert_eq!(answer.turns, turns);
    assert_eq!(preparation, None);
    assert!(matches!(
        answer.ending,
        host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Shutdown), .. }
    ));
    assert_eq!(world.stops, 0, "wait for the engine to acknowledge");
    world
        .event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))
        .expect("worker inspection");
    if turns == 1 {
        assert_eq!(world.stops, 0, "wait for the last turn too");
        world
            .event(Event::Link(host::LinkDown::TurnKept { run: RUN, attempt: world.attempt, turn: 1 }))
            .expect("worker inspection");
    }
    assert_eq!(world.domain.kept_answers(), 0);
    assert_eq!(world.domain.agent().agents(), 0);
    assert_eq!(world.domain.workspaces(), 0);
    match way {
        Way::Drained => {
            assert_eq!(world.report.pauses, [PauseState::Draining, PauseState::Drained]);
            assert_eq!(world.root_pauses, world.report.pauses);
            assert!(world.up);
            assert_eq!(world.stops, 0, "link drain stays up");
            assert_eq!(world.shutdown_notices, 0);
        }
        Way::Signalled => {
            assert_eq!(world.shutdown_notices, 1);
            assert_eq!(world.stops, 1, "signal stops the worker after acknowledgement");
        }
        Way::Crashed => unreachable!("crash has no drain"),
    }
}

/// No settle, cancellation event, exit event or pending completion is run.
/// The disk and forge are outside the process; all other live work is dropped.
fn crash(world: &mut World, check: &mut Option<agent::Ask>, recovery: Recovery) {
    drop(check.take());
    let old = std::mem::replace(&mut world.domain, Domain::new(&world.env.limits, Box::new([])));
    drop(old);
    world.out.clear();
    world.owed.clear();
    world.held = None;
    world.process = None;
    world.reading = false;
    world.workspace = None;
    world.call = None;
    world.delivery_reply = None;
    world.up = false;
    world.hold_push = false;
    assert_eq!(world.report.saves, 0, "no settle saved the crashed run");
    assert_eq!(world.report.drains, 0);
    assert_eq!(world.report.cancellations, 0);
    if recovery == Recovery::AnotherWorker {
        world.disk = Checkout::new();
    }
    assert_eq!(
        world.disk.exists(b"worker-state"),
        recovery == Recovery::SameWorker,
        "same disk or another worker's disk"
    );
    world.env.now = world.env.now.saturating_add(Duration::from_secs(2));
    world.connect().expect("worker inspection");
    assert_eq!(world.domain.agent().agents(), 0, "new root has no old agents");
    assert!(world.workstreams.is_empty(), "no old claim in the new hello");
    next_attempt(world, false);
}

fn next_attempt(world: &mut World, saved: bool) {
    world.attempt = Token::new(world.attempt.raw() + 1);
    world.process = None;
    world.replies.clear();
    world.delivery_reply = None;
    world.deliveries = 0;
    world.first = world.forge.branch(REMOTE, b"topic").expect("producer branch");
    assert!(world.up, "the engine places the attempt after the hello or resume");
    let start = if saved {
        wire::Start::Saved { branch: b"saved".as_slice().into() }
    } else {
        wire::Start::Branch { branch: b"topic".as_slice().into() }
    };
    world.assign(start).expect("worker inspection");
    world.say(agent::Up::Admitted).expect("worker inspection");
    let place = temper_worker_domain::checkout::git::Place {
        git: None,
        workspace: world.workspace.expect("prepared next attempt"),
        repository: b"app".as_slice().into(),
    };
    world.disk.write(&[super::translate::path(&place).as_slice(), b"/code"].concat(), b"edited\n");
    assert_eq!(world.domain.agent().agents(), 1);
    assert!(world.reading, "the next attempt started and was admitted");
    assert_eq!(world.restart_observer.as_ref().expect("referee").starts, [ATTEMPT, world.attempt]);
}

fn injected(seed: u64, draw: (Way, Point, Recovery), ledger: &mut FaultLedger) -> (World, Injection) {
    let (way, point, recovery) = draw;
    let mut world = World::new(seed);
    world.restart_observer = Some(Observer::default());
    world.start().expect("worker inspection");
    world.disk.write(b"worker-state", b"old worker");
    let place = temper_worker_domain::checkout::git::Place {
        git: None,
        workspace: world.workspace.expect("prepared"),
        repository: b"app".as_slice().into(),
    };
    world.disk.write(&[super::translate::path(&place).as_slice(), b"/code"].concat(), b"before restart\n");
    let mut check = hold(&mut world, point);
    world.env.now = Time::from_nanos(7_000_000_000);
    held(&world, point, check.as_ref());
    let fault = ledger.inject(worker(way, point), world.env.now, Struck::default());
    match way {
        Way::Drained | Way::Signalled => {
            drain(&mut world, way, point, &mut check);
            assert_eq!(world.restart_observer.as_ref().expect("referee").hellos, [(Time::ZERO, 0)]);
            if way == Way::Drained {
                // A link drain has no new hello: its Drained notice closes the interval.
                ledger.stop(fault, world.env.now).expect("drained interval");
            }
        }
        Way::Crashed => {
            crash(&mut world, &mut check, recovery);
            let hellos = &world.restart_observer.as_ref().expect("referee").hellos;
            assert_eq!(hellos, &[(Time::ZERO, 0), (world.env.now, 0)], "new hello lists no old claim");
            ledger.stop(fault, hellos.last().expect("new hello").0).expect("restart interval");
        }
    }
    (world, fault)
}

/// Focused fault step, including the replacement's next attempt after a crash.
#[must_use]
pub fn at_point(seed: u64, way: Way, point: Point, recovery: Recovery) -> FaultLedger {
    let mut ledger = FaultLedger::new();
    let (mut world, _) = injected(seed, (way, point, recovery), &mut ledger);
    if way == Way::Crashed {
        super::finish_host_run(&mut world).expect("worker inspection");
    } else if way == Way::Drained {
        world.event(Event::Shutdown { drain: true }).expect("worker inspection");
        assert_eq!(world.stops, 1);
    }
    ledger
}

/// A checked delivery, not a save or its check, is in flight when the worker alone is signalled.
pub fn signalled_during_delivery(seed: u64) -> Result<(), String> {
    use super::Op;
    let mut world = World::new(seed);
    world.restart_observer = Some(Observer::default());
    world.start()?;
    world.disk.write(b"worker-state", b"same disk");

    world.request_delivery(agent::Ask::Save)?;
    assert_eq!(world.deliveries, 1, "the producer received its save receipt");
    let saved = world.forge.branch(REMOTE, b"topic").expect("save pushed");
    assert_eq!(world.forge.tree(saved), [(PATH.to_vec(), b"edited\n".to_vec())].into());
    assert!(world.forge.message(saved).starts_with(b"Unchecked:"));
    let pushes = world.pushes;

    let place = temper_worker_domain::checkout::git::Place {
        git: None,
        workspace: world.workspace.expect("prepared"),
        repository: b"app".as_slice().into(),
    };
    let both = b"edited\nlater\n";
    world.disk.write(&[translate::path(&place).as_slice(), b"/code"].concat(), both);
    let delivery = protocol::delivery(b"Delivery", b"Worker story");
    let agent::Ask::Deliver { fields } = &delivery else { panic!("checked delivery") };
    let change = fields.clone().into_value();
    world.hold_push = true;
    world.request_delivery(delivery)?;
    assert!(matches!(world.held, Some((_, Op::Push { .. }))));
    assert_eq!(world.forge.branch(REMOTE, b"topic"), Some(saved));
    assert!(world.delivery_reply.is_none());

    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.report.drains, 1, "the worker's first termination drains");
    assert_eq!(world.report.cancellations, 0);
    assert_eq!(world.stops, 0);
    assert!(world.answer.is_none());
    world.env.now = world.env.now.saturating_add(Duration::from_secs(1));
    world.release_push()?;
    assert_eq!(world.deliveries, 2, "exactly one checked delivery receipt after the save");
    let head = world.forge.branch(REMOTE, b"topic").expect("delivery pushed");
    assert_eq!(world.forge.parent(head), Some(saved));
    assert_eq!(world.forge.tree(head), [(PATH.to_vec(), both.to_vec())].into(), "the delivery head holds both lines");

    world.say(agent::Up::Turn { turn: protocol::turn(1, 0, 8) })?;
    world.accept_change(change, 1)?;
    world.gone()?;
    let (answer, work, preparation) = world.answer.take().expect("one delivery answer");
    assert_eq!(answer.turns, 1);
    assert!(matches!(answer.ending, host::Ending::Ended { .. }));
    assert_eq!(preparation, None);
    assert_eq!(work.landed.as_ref(), [wire::Landed { tag: 1, commit: translate::commit(head).raw(), checked: true }]);
    assert_eq!(world.stops, 0, "the final answer is not yet acknowledged");
    assert_eq!(world.domain.kept_answers(), 1);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: world.attempt }))?;
    assert_eq!(world.domain.kept_answers(), 0);
    assert_eq!(world.stops, 0, "the turn is not yet acknowledged");
    world.event(Event::Link(host::LinkDown::TurnKept { run: RUN, attempt: world.attempt, turn: 1 }))?;
    assert_eq!(world.pushes, pushes + 1, "one push for the checked delivery");

    reopen_settled(&mut world)?;
    assert_eq!(world.restart_observer.as_ref().expect("referee").turns, [ATTEMPT]);
    assert!(world.answer.is_none(), "the new worker does not replay the delivery's answer");
    assert_eq!(world.pushes, pushes + 1, "the restart does not push again");
    assert_eq!(world.forge.branch(REMOTE, b"topic"), Some(head));
    Ok(())
}

fn reopen_settled(world: &mut World) -> Result<(), String> {
    assert_eq!(world.stops, 1);
    assert_eq!(world.domain.agent().agents(), 0);
    assert_eq!(world.domain.workspaces(), 0);
    world.domain = Domain::new(&world.env.limits, Box::new([]));
    world.process = None;
    world.up = false;
    world.env.now = world.env.now.saturating_add(Duration::from_secs(2));
    world.connect()?;
    let observer = world.restart_observer.as_ref().expect("referee");
    assert_eq!(observer.hellos, [(Time::ZERO, 0), (world.env.now, 0)], "fresh hello with no old attempt");
    assert!(world.disk.exists(b"worker-state"), "the worker keeps its disk");
    Ok(())
}

const REPLIES: &[&[u8]] = &[b"I read the file.", b"I updated it.", b"Here is the held answer."];
const RELEASED_WRITE: &[u8] = b"released\n";

/// The played agent's model boundary. It starts a Completion before each answer,
/// and asks again after a turn only if its host has not sent Drain.
#[derive(Default)]
struct Model {
    queries: u32,
    pending: bool,
}

impl Model {
    fn ask(&mut self, world: &mut World) -> Result<(), String> {
        assert!(!self.pending, "one model query at a time");
        self.queries += 1;
        self.pending = true;
        world.say(agent::Up::Long { operation: agent::Operation::Completion, span: Duration::from_secs(60) })
    }

    fn release(&mut self, world: &mut World, write: Option<&[u8]>) -> Result<(), String> {
        assert!(self.pending, "release an outstanding answer");
        self.pending = false;
        world.say(agent::Up::LongDone)?;
        if let Some(bytes) = write {
            let place = temper_worker_domain::checkout::git::Place {
                git: None,
                workspace: world.workspace.expect("prepared"),
                repository: b"app".as_slice().into(),
            };
            world.disk.write(&[translate::path(&place).as_slice(), b"/code"].concat(), bytes);
        }
        let text = REPLIES[usize::try_from(self.queries - 1).expect("bounded script")];
        world.say(agent::Up::Turn { turn: protocol::text_turn(self.queries, text) })?;
        if world.report.drains == 0 {
            self.ask(world)?;
        }
        Ok(())
    }
}

/// The first worker signal drains; the live agent's own signal is only a notice.
pub fn signalled_during_model_answer(seed: u64) -> Result<(), String> {
    let mut world = World::new(seed);
    world.restart_observer = Some(Observer { texts: REPLIES, ..Observer::default() });
    world.start()?;
    world.disk.write(b"worker-state", b"same disk");
    let mut model = Model::default();
    model.ask(&mut world)?;
    for turn in [1, 2] {
        model.release(&mut world, None)?;
        world.event(Event::Link(host::LinkDown::TurnKept { run: RUN, attempt: ATTEMPT, turn }))?;
    }
    // Point::LlmCall: the third query is out, but the next turn has not been said.
    assert_eq!(world.restart_observer.as_ref().expect("referee").turns, [ATTEMPT; 2]);
    assert_eq!(model.queries, 3);
    assert!(model.pending && world.reading);
    assert_eq!(world.forge.branch(REMOTE, b"saved"), Some(world.first));

    world.event(Event::Shutdown { drain: true })?;
    assert_eq!(world.report.drains, 1);
    let signal = agent::parent::Signal::Terminate;
    let from = Some(42);
    world.say(agent::Up::Signalled { signal, from })?;
    assert_eq!(
        world.restart_observer.as_ref().expect("referee").signals,
        [Request::AgentSignalled { run: RUN, attempt: ATTEMPT, signal, from }]
    );
    assert_eq!(
        (world.report.cancellations, world.report.drops, world.stops),
        (0, 0, 0),
        "the stray signal is only a notice"
    );
    assert!(world.answer.is_none() && world.reading && model.pending, "the same attempt still waits on its answer");
    assert_eq!(world.restart_observer.as_ref().expect("referee").turns, [ATTEMPT; 2]);

    world.env.now = world.env.now.saturating_add(Duration::from_secs(1));
    model.release(&mut world, Some(RELEASED_WRITE))?;
    assert_eq!(model.queries, 3, "the drain commits this answer without asking the model again");
    assert!(!model.pending);
    let answer = agent::Answer {
        read: None,
        turns: model.queries,
        spent: 0,
        result: agent::RunResult::Failed { failure: agent::RunFailure::Cancelled },
    };
    world.say(agent::Up::Answer { answer })?;
    world.gone()?;
    let (answer, work, preparation) = world.answer.take().expect("one terminal after the released answer");
    assert_eq!(answer.turns, 3);
    // The hub maps this shutdown reason to Recheck, which spends no try.
    assert!(matches!(
        answer.ending,
        host::Ending::Failed { failure: host::Failure::Cancelled(host::Reason::Shutdown), .. }
    ));
    assert_eq!(preparation, None);
    let saved = world.forge.branch(REMOTE, b"saved").expect("saved branch");
    let saved_work = work.saved.expect("saved work in the answer");
    let [wire::SavedRepository { tag: 1, landing: wire::Landing::Landed { commit }, .. }] = saved_work.as_ref() else {
        panic!("one saved head: {saved_work:?}");
    };
    assert_eq!(*commit, translate::commit(saved).raw());
    assert_eq!(world.forge.tree(saved), [(PATH.to_vec(), RELEASED_WRITE.to_vec())].into());

    assert_eq!(world.stops, 0, "the final turn is not yet acknowledged");
    world.event(Event::Link(host::LinkDown::TurnKept { run: RUN, attempt: ATTEMPT, turn: 3 }))?;
    assert_eq!(world.stops, 0, "the terminal is not yet acknowledged");
    assert_eq!(world.domain.kept_answers(), 1);
    world.event(Event::Link(host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }))?;
    assert_eq!(world.domain.kept_answers(), 0);
    assert_eq!(world.report.cancellations, 0, "the host never cancelled the concluding turn");
    reopen_settled(&mut world)?;
    let observer = world.restart_observer.expect("referee");
    assert_eq!(observer.turns, [ATTEMPT; 3], "the three distinct texts, once each on the same attempt");
    assert_eq!(observer.starts, [ATTEMPT]);
    assert!(world.answer.is_none());
    Ok(())
}

/// Each existing delivery/contact story, preceded by the same seed's restart
/// at its drawn point. The next attempt continues from the interrupted one's
/// saved work where it has any. One seed's ledger covers all four stories.
#[must_use]
pub fn stories(seed: u64) -> FaultLedger {
    let draw = Injector::new(seed).draw();
    let mut ledger = FaultLedger::new();
    for case in [Case::Delivered, Case::CancelledDelivery, Case::ContactWithin, Case::ContactPast] {
        story(seed, case, draw, &mut ledger);
    }
    ledger
}

// No assertion here prescribes W3/W4's preservation of unsaved work.
fn story(seed: u64, case: Case, draw: (Way, Point, Recovery), ledger: &mut FaultLedger) {
    let (way, _, _) = draw;
    let (mut world, fault) = injected(seed, draw, ledger);
    match way {
        Way::Drained => {
            world.event(Event::Link(host::LinkDown::Pause { mode: PauseMode::Resume })).expect("worker inspection");
            next_attempt(&mut world, true);
        }
        Way::Signalled => {
            world.domain = Domain::new(&world.env.limits, Box::new([]));
            world.env.now = world.env.now.saturating_add(Duration::from_secs(2));
            world.connect().expect("worker inspection");
            let (hello, hosting) = *world.restart_observer.as_ref().expect("referee").hellos.last().expect("new hello");
            assert_eq!(hosting, 0, "stopped worker starts with no old claim");
            ledger.stop(fault, hello).expect("next hello closes the interval");
            next_attempt(&mut world, true);
        }
        Way::Crashed => {}
    }
    world.stops = 0;
    world.report.cancellations = 0;
    world.report.saves = 0;
    world.story(case).expect("worker inspection");
    world.completed(case).expect("worker inspection");
}

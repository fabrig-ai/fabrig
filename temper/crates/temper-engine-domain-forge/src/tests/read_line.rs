//! Forge admission turns (temper/docs/design/domain/forge/keeping-up.md, 3).
use super::*;
use skein_forge::api::{Answer, Error, Op, Read};
use skein_forge_client::Event as Client;
use temper_engine_domain_forge_landing::Preparation;

const MAKING: u64 = 12;
const LATER: u64 = 13;
const TAKING: u64 = 21;
const LANDING: u64 = 61;
const LANDING_ENTRY: u64 = 91;
const WRITER: u64 = 9;
const WRITER_ATTEMPT: u64 = 1;
const WRITER_HOLDER: u64 = 11;
const TIP: [u8; 32] = [4; 32];
const GOAL_BRANCH: &[u8] = b"temper/1/g12";
const LATER_BRANCH: &[u8] = b"temper/1/g13";
const LOST_BRANCH: &[u8] = b"temper/11";
const LANDING_BRANCH: &[u8] = b"temper/51";
const WATCHED: &[u8] = b"watched";
const WATCH_OWNER: Token = Token::new(777);
const AGENT_LATER: u64 = 100;
const STOCK_PENDING: u32 = CLIENT.pending;
const FREED_CALLS: u64 = 8;
// Longer than the observation of the capped waits; no rate reset is needed.
const RATE_WINDOW: Duration = Duration::from_secs(600);

fn branch(branch: &[u8]) -> Read {
    Read::Branch { branch: branch.into() }
}

fn clock() -> Env<Limits> {
    let mut env = env();
    env.limits.client.pending = STOCK_PENDING;
    // Put every admission place in flight so each turnover frees exactly one.
    env.limits.client.calls = STOCK_PENDING;
    env.limits.client.window = RATE_WINDOW;
    env
}

struct Pressure {
    d: Domain,
    clock: Env<Limits>,
    calls: Queue<Token>,
}

impl Pressure {
    fn new(mut d: Domain, clock: Env<Limits>) -> Self {
        let mut out = Queue::uncapped();
        step(&mut d, &clock, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, &mut out);
        step(&mut d, &clock, Event::CatchUp, &mut out);
        let mut pressure = Self { d, clock, calls: Queue::with_capacity(STOCK_PENDING) };
        for serial in 0..u64::from(STOCK_PENDING) {
            pressure.agent(serial);
            pressure.send();
        }
        pressure
    }

    fn adopted(clock: Env<Limits>, repository: Repository) -> Self {
        let mut d = domain_with_limits(&clock.limits);
        outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
        Self::new(d, clock)
    }

    fn step(&mut self, event: Event) -> Queue<Request> {
        let mut out = Queue::uncapped();
        step(&mut self.d, &self.clock, event, &mut out);
        out
    }

    fn agent(&mut self, serial: u64) -> Queue<Request> {
        self.step(Event::Client(Client::Read {
            owner: root_read(serial).expect("agent read owner"),
            repository: REPO,
            read: branch(format!("agent/{serial}").as_bytes()),
        }))
    }

    fn watch(&mut self) {
        self.step(Event::Client(Client::Keep {
            owner: WATCH_OWNER,
            watches: Box::new([skein_forge_client::Watch {
                resource: skein_forge_client::Resource {
                    repository: REPO,
                    what: skein_forge_client::What::Branch(WATCHED.into()),
                },
                participating: false,
            }]),
        }));
    }

    fn making(&mut self, task: u64, name: &[u8]) -> Queue<Request> {
        self.step(Event::Hold {
            entry: TAKING.checked_add(task.checked_sub(MAKING).expect("taking task")).expect("taking entry"),
            task,
            resource: goal_branch::name(name),
            from: None,
        })
    }

    fn lost(&mut self) {
        self.d
            .holds
            .insert(
                goal_branch::name(LOST_BRANCH),
                Hold {
                    name: goal_branch::name(LOST_BRANCH),
                    task: WRITER_HOLDER,
                    writer: Some(Writer::Run { task: WRITER, attempt: WRITER_ATTEMPT }),
                    drift: None,
                    beside: None,
                    design_writers: Box::new([]),
                },
            )
            .expect("writer's held branch");
        self.step(Event::Lost { task: WRITER, attempt: WRITER_ATTEMPT });
    }

    fn brief(&mut self, owner: Token) {
        self.step(Event::Client(Client::Read {
            owner,
            repository: REPO,
            read: Read::Checks { commit: ci_brief::HEAD },
        }));
    }

    fn document(&mut self) {
        let mut messages = brief_document::plan(&mut self.d, brief_document::document(), brief_document::BUDGET);
        let Some(Request::BriefClient { event }) = messages.pop() else { panic!("document read: {messages:?}") };
        self.step(Event::Client(event));
    }

    fn landing(&mut self) {
        self.step(Event::StepLanding {
            task: LANDING,
            entry: LANDING_ENTRY,
            heard: landing_routes::preparing_heard(Some(Preparation::Idle)),
        });
    }

    fn free(&mut self) {
        let call = self.calls.pop().expect("a call is out");
        fixtures::answer_at(&mut self.d, &self.clock, call, Ok(Answer::Commit(TIP)));
    }

    fn send(&mut self) -> Op {
        let (call, op) = fixtures::next_at(&mut self.d, &self.clock);
        self.calls.push(call);
        op
    }

    fn turn(&mut self) -> Queue<Request> {
        let mut out = Queue::uncapped();
        resume(&mut self.d, &self.clock, &mut out);
        out
    }

    fn advance(&mut self, seconds: u64) -> Queue<Request> {
        fixtures::advance(&mut self.d, &mut self.clock, Duration::from_secs(seconds))
    }
}

fn facts(out: &Queue<Request>) -> Box<[&[u8]]> {
    let mut facts = skein_lib::List::with_capacity(out.len());
    for request in out {
        if let Request::FullTableFact { words } = request {
            facts.push(words.as_ref()).expect("one fact per request");
        }
    }
    facts.into_boxed()
}

#[test]
fn a_goal_branch_making_waiting_for_room_takes_the_next_freed_call() {
    let mut p = Pressure::adopted(clock(), adopted());
    p.making(MAKING, GOAL_BRANCH);
    p.watch();
    let making_read = Op::Read(branch(GOAL_BRANCH));
    let mut took_turn = None;
    for turn in 1..=FREED_CALLS {
        p.free();
        p.agent(AGENT_LATER + turn);
        if p.send() == making_read {
            took_turn = Some(turn);
            break;
        }
    }
    assert_eq!(took_turn, Some(1), "the making must take the first freed call");
}

#[test]
fn a_brief_a_lost_writer_and_a_making_take_turns() {
    let clock = clock();
    let (d, owner) = ci_brief::pending_checks_with(domain_with_limits(&clock.limits));
    let mut p = Pressure::new(d, clock);
    p.brief(owner);
    p.lost();
    p.making(MAKING, GOAL_BRANCH);
    p.watch();
    for (turn, read) in
        [Read::Checks { commit: ci_brief::HEAD }, branch(LOST_BRANCH), branch(GOAL_BRANCH), branch(WATCHED)]
            .into_iter()
            .enumerate()
    {
        p.free();
        if read != branch(WATCHED) {
            p.agent(AGENT_LATER + u64::try_from(turn).unwrap());
        }
        assert_eq!(p.send(), Op::Read(read), "turn {turn}");
    }
}

fn member_pressure() -> Pressure {
    let clock = clock();
    let mut d = landing_routes::landing_fixture_with(domain_with_limits(&clock.limits));
    d.changes.get_mut(&fixtures::CHANGE).expect("change").change.state =
        temper_engine_domain_forge_change::State::Ready { head: fixtures::HEAD, ready_since: clock.wall };
    let mut p = Pressure::new(d, clock);
    landing_routes::member_landing(&mut p.d);
    p
}

#[test]
fn a_making_a_landing_and_a_document_take_turns() {
    let mut p = member_pressure();
    p.making(MAKING, GOAL_BRANCH);
    p.landing();
    p.document();
    p.watch();
    for read in [branch(GOAL_BRANCH), branch(LANDING_BRANCH), branch(b"main"), branch(WATCHED)] {
        p.free();
        assert_eq!(p.send(), Op::Read(read));
    }
}

#[test]
fn a_later_agent_is_refused_and_a_making_joins_the_tail() {
    let mut p = Pressure::adopted(clock(), adopted());
    p.making(MAKING, GOAL_BRANCH);
    p.free();
    let mut refused = p.agent(AGENT_LATER);
    p.making(LATER, LATER_BRANCH);
    let first = p.send();
    p.free();
    let second = p.send();
    assert_eq!(refused.pop(), Some(Request::Read { owner: root_read(AGENT_LATER).unwrap(), result: Err(Error::Busy) }));
    assert_eq!([first, second], [Op::Read(branch(GOAL_BRANCH)), Op::Read(branch(LATER_BRANCH))]);
}

#[test]
fn a_head_refused_by_the_reserve_yields_without_losing_its_place() {
    let mut clock = clock();
    // The four blocking calls spend the entire foreground share.
    clock.limits.client.rate = STOCK_PENDING * 2;
    clock.limits.client.reserve = STOCK_PENDING;
    let mut p = Pressure::adopted(clock, adopted());
    p.making(MAKING, GOAL_BRANCH);
    p.making(LATER, LATER_BRANCH);
    p.watch();
    p.free();
    p.free();
    let mut refused = p.turn();
    let Some(Request::Call { call: poll, op, .. }) = refused.pop() else {
        panic!("only the reserved poll: {refused:?}")
    };
    assert_eq!(op, Op::Read(branch(WATCHED)));
    assert!(refused.is_empty(), "another refusal writes nothing");
    assert!(!p.d.is_ready(), "room remains, but the head must yield the turn");
    assert!(p.turn().is_empty(), "no second ask before another reclaim or deadline");
    p.calls.push(poll);
    p.advance(1);
    assert_eq!(p.send(), Op::Read(branch(GOAL_BRANCH)), "the deadline wakes the same head");
}

#[test]
fn making_wait_deadlines_double_to_the_slow_cap() {
    let mut p = Pressure::adopted(clock(), adopted());
    p.making(MAKING, GOAL_BRANCH);
    let start = p.clock.now;
    for (step, elapsed, next) in [
        (1, 1, 3),
        (2, 3, 7),
        (4, 7, 15),
        (8, 15, 30),
        (15, 30, 31),
        (1, 31, 60),
        (29, 60, 63),
        (3, 63, 103),
        (40, 103, 120),
        (17, 120, 143),
        (23, 143, 183),
    ] {
        assert_eq!(p.d.next_deadline(), Some(start.saturating_add(Duration::from_secs(elapsed))));
        p.advance(step);
        assert_eq!(p.d.next_deadline(), Some(start.saturating_add(Duration::from_secs(next))));
        assert!(!p.d.is_ready(), "the deadline does not make room at {elapsed} s");
    }
}

#[test]
fn a_making_logs_its_place_age_and_admission() {
    let mut repository = adopted();
    repository.owner = b"work".as_slice().into();
    repository.name = b"hello".as_slice().into();
    let mut p = Pressure::adopted(clock(), repository);
    p.lost();
    p.making(LATER, LATER_BRANCH);
    let joined = p.making(MAKING, GOAL_BRANCH);
    assert_eq!(facts(&joined).as_ref(), [b"task 12's goal branch making waits for the forge's read admission: branch temper/1/g12 of work/hello, 2 ahead".as_slice()]);
    p.free();
    p.send();
    p.free();
    p.send();
    let first = p.advance(1);
    assert_eq!(facts(&first).as_ref(), [b"task 12's goal branch making waits for the forge's read admission for 1 s: branch temper/1/g12 of work/hello, 0 ahead".as_slice()]);
    let second = p.advance(2);
    assert_eq!(facts(&second).as_ref(), [b"task 12's goal branch making waits for the forge's read admission for 3 s: branch temper/1/g12 of work/hello, 0 ahead".as_slice()]);
    p.free();
    let admitted = p.turn();
    assert_eq!(
        facts(&admitted).as_ref(),
        [b"task 12's goal branch making has the forge's read admission after 3 s: branch temper/1/g12 of work/hello"
            .as_slice()]
    );
}

#[derive(Clone, Copy)]
enum Kind {
    Making,
    Landing,
    Document,
    Lost,
}

pub(crate) fn check_interleaving(seed: u64) {
    const KINDS: u32 = 4;
    let mut rng = skein_lib::Rng::new(seed);
    let mut p = member_pressure();
    // These four kinds compete for the four freed calls. Keep the oracle
    // in event order, independent of the owners' tokens and kind.
    let mut kinds = [Kind::Making, Kind::Landing, Kind::Document, Kind::Lost];
    let mut waiting = Queue::with_capacity(KINDS);
    let mut agent = AGENT_LATER;
    let watch_at = u32::try_from(rng.between(1, u64::from(KINDS))).unwrap();
    for remaining in (1..=KINDS).rev() {
        let index = usize::try_from(rng.below(u64::from(remaining))).unwrap();
        let last = usize::try_from(remaining.checked_sub(1).unwrap()).unwrap();
        let kind = *kinds.get(index).expect("kind to join");
        let tail = *kinds.get(last).expect("last remaining kind");
        *kinds.get_mut(index).expect("replace selected kind") = tail;
        let read = match kind {
            Kind::Making => {
                p.making(MAKING, GOAL_BRANCH);
                branch(GOAL_BRANCH)
            }
            Kind::Landing => {
                p.landing();
                branch(LANDING_BRANCH)
            }
            Kind::Document => {
                p.document();
                branch(b"main")
            }
            Kind::Lost => {
                p.lost();
                branch(LOST_BRANCH)
            }
        };
        waiting.push(read);
        if remaining == watch_at {
            p.watch();
        }
        for _ in 0..rng.below(3) {
            p.agent(agent);
            agent = agent.checked_add(1).unwrap();
            assert!(!p.d.is_ready(), "a full table cannot retry, seed {seed}");
        }
    }
    for _ in 0..KINDS {
        p.free();
        for _ in 0..rng.below(3) {
            let expected = Request::Read { owner: root_read(agent).unwrap(), result: Err(Error::Busy) };
            assert_eq!(p.agent(agent).pop(), Some(expected), "seed {seed}");
            agent = agent.checked_add(1).unwrap();
        }
        let head = waiting.pop().expect("one waiter per freed call");
        assert_eq!(p.send(), Op::Read(head), "seed {seed}");
        assert!(!p.d.is_ready(), "only one call admitted in the turn, seed {seed}");
    }
}

//! Starts retain durable diagnostic answers (jig/docs/design/domain/engine.md, 6).
use jig_core_world::effects::Setup;
use jig_core_world::{
    effects::{World, fixture},
    fixtures,
};
use jig_domain_core::{
    self as core,
    start::{Action, Phase, SweepInput},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

const SEED: u64 = 2901;

struct Harness {
    core: core::Core,
    env: Env<core::Limits>,
    out: Queue<core::Request>,
}

impl Harness {
    fn stored(world: &World) -> Self {
        let (config, limits) = fixture(SEED, false);
        Self::stored_with(world, config.core, Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits: limits.core })
    }

    fn stored_with(world: &World, config: core::Config, env: Env<core::Limits>) -> Self {
        let mut h =
            Self { core: core::Core::new(config, &env.limits).expect("configured core"), env, out: Queue::uncapped() };
        phase(&h.core.start_begin(false, h.env.wall), Phase::LoadCore);
        for range in core::RestoreRange::LIVE {
            for (key, record) in &world.store.rows {
                if let root::Key::Core(key) = key
                    && range.admits(key)
                {
                    let root::Record::Core(record) = record else { panic!("jig live row") };
                    h.record(record.clone());
                }
            }
        }
        h
    }

    fn record(&mut self, record: core::Record) {
        assert_ne!(self.core.restore(&self.env, record), core::Restore::Refused);
        assert_eq!(self.core.start_failure(), None);
    }

    fn link(&mut self) -> Action {
        phase(&self.core.start_done(&self.env, Phase::LoadCore, &mut self.out), Phase::LoadInbox);
        let _census = self.core.census(&self.env.limits);
        let mut previous = Phase::LoadInbox;
        for connector in self.core.connectors.clone() {
            let next = Phase::RestoreConnector { connector };
            phase(&self.core.start_done(&self.env, previous, &mut self.out), next);
            previous = next;
        }
        phase(&self.core.start_done(&self.env, previous, &mut self.out), Phase::Link);
        self.core.start_done(&self.env, Phase::Link, &mut self.out)
    }

    fn open(&mut self, mut action: Action) {
        for _ in 0..32 {
            action = match action {
                Action::Step(Phase::Sweep) => self.core.start_sweep(&self.env, SweepInput::Finished, &mut self.out),
                Action::Step(done) => self.core.start_done(&self.env, done, &mut self.out),
                Action::Commit(token) => self.core.start_resume(&self.env, token, &mut self.out),
                Action::Idle => return,
                Action::Refused(failure) => panic!("stored start fits: {failure:?}"),
            };
            self.out.clear();
        }
        panic!("bounded start opens");
    }
}

fn phase(action: &Action, expected: Phase) {
    assert!(matches!(action, Action::Step(found) if *found == expected));
}

#[test]
fn unknown_introduction_target_replays_its_original_diagnostic_after_start_link() {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    world.hold_procedures = true;
    world.delegate();
    world.tool_call(b"introduce", br#"{"left":2,"right":18446744073709551615}"#, true);
    let key = world.call_key(2);
    let original = world.domain.core_mut().call_record(key).expect("durable diagnostic");
    assert!(matches!(&original.part, core::CallPart::MessageRefused(problem)
        if problem.task == Some(u64::MAX) && problem.why == tasks::Refusal::Unknown));
    let answer = world.host_answers.last().expect("rendered diagnostic").1.answer.clone();
    assert_eq!(original.settled.as_ref().expect("retained rendering").answer, answer);
    let mut h = Harness::stored(&world);
    assert!(matches!(h.link(), Action::Step(Phase::Sweep) | Action::Commit(_)));
    assert_eq!(h.core.call_record(key), Some(original.clone()));
    assert!(matches!(h.core.replay_call(key), Some(core::CallReplay::Core(part)) if part == original.part));
    assert_eq!(h.core.call_record(key).expect("replayed record").settled.expect("same rendering").answer, answer);
}

#[test]
fn retained_malformed_create_keeps_and_replays_the_original_refusal() {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    let mut member = fixtures::people_member(b"malformed", 1);
    member.executor = people::TaskExecutor::Procedure { connector: 0, code: 0 };
    let ask = people::Ask::Create { project: 1, batch: Box::new([member]) };
    let key = people::RequestKey { person: 1, key: [91; 16] };
    let sign_in = world.sign_in.expect("owner sign-in");
    fixtures::ask_as(&mut world, sign_in, 91, ask.clone());
    let Some(people::Reply::Outcome(original @ people::Outcome::Refused(_))) = world.people_answers.last() else {
        panic!("malformed Create has a permanent refusal: {:?}", world.people_answers.last());
    };
    let original = original.clone();
    let mut h = Harness::stored(&world);
    let action = h.link();
    h.open(action);
    let (stored, outcome) = h
        .core
        .people
        .next_answer(Some(people::RequestKey { person: 1, key: [90; 16] }))
        .expect("retained malformed Create");
    assert_eq!(stored, key);
    assert_eq!(*outcome, original);
    let mut out = Queue::uncapped();
    people::step(
        &mut h.core.people,
        &Env { now: h.env.now, wall: h.env.wall, limits: h.env.limits.people },
        people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(900)),
            sign_in: world.sign_in.expect("owner sign-in"),
            key: key.key,
            ask,
            models: Box::new([]),
        },
        &mut out,
    );
    assert_eq!(
        out.pop(),
        Some(people::Request::Reply { to: ReplyTo::new(Token::new(900)), reply: people::Reply::Outcome(original) })
    );
    assert!(out.is_empty());
}

#[test]
fn recurring_lifetime_count_past_live_capacity_survives_start_link() {
    let world = World::new(Setup { seed: SEED, ..Setup::default() });
    let mut h = Harness::stored(&world);
    let task = h.core.tasks.task(1).expect("standing root");
    let parent = task.funder;
    let funder = tasks::Funder::Recurring { project: 1, task: 1, period: 1 };
    let made = h.env.limits.tasks.tasks.checked_add(1).expect("small live capacity");
    h.record(core::Record::Tasks(tasks::Stored::Ledger(tasks::FundingRecord {
        period: None,
        funder,
        parent: Some(parent),
        numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
        made,
        closed: false,
    })));
    assert!(matches!(h.link(), Action::Step(Phase::Sweep) | Action::Commit(_)));
    assert_eq!(h.core.tasks.funding(funder).expect("retained recurring funding").made, made);
}

/// Restore the direct core through durable Wires before testing runtime inputs.
pub(super) fn recovered(world: &World, config: core::Config, env: &Env<core::Limits>) -> core::Core {
    let mut harness = Harness::stored_with(world, config, Env { now: env.now, wall: env.wall, limits: env.limits });
    let action = harness.link();
    harness.open(action);
    assert!(harness.core.start_opened());
    harness.core
}

pub(super) fn first(config: core::Config, env: &Env<core::Limits>) -> core::Core {
    let mut harness = Harness {
        core: core::Core::new(config, &env.limits).expect("configured core"),
        env: Env { now: env.now, wall: env.wall, limits: env.limits },
        out: Queue::uncapped(),
    };
    let action = harness.core.start_begin(false, harness.env.wall);
    harness.open(action);
    assert!(harness.core.start_opened());
    harness.core
}

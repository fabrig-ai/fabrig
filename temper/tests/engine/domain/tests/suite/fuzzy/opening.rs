//! Crash cuts through a seeded workload (draft/reconciling-start.md, 7).
//! Referee observations are settled, but the cuts never drain first.
//! Campaign cases follow `temper/docs/design/testing/workloads.md`, 8.5.
use super::follows::referee::Referee;
use crate::forge_routes::follow_changes::proof;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, ReplyTo, Rng, StateHasher, Token};
use skein_world::sweep::{Test, cases};
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use temper_engine_domain::{self as engine, Record};
use temper_engine_domain_forge::{FollowState, follows::FollowAct};
use temper_engine_domain_world::{
    forge_routes::World,
    replay::{same_digest, same_text},
};

const SEEDS: u64 = 4;
const STEPS: u32 = 8;
const CUT_TICKS: u64 = 24;

#[derive(Clone, Copy, Debug, Hash)]
enum Step {
    Landing,
    Start,
    Hold,
    Skip,
    Merge,
    Port,
    Goal,
    Cancel,
    Restart,
}
fn drawn(rng: &mut Rng) -> Step {
    match rng.below(10) {
        0 | 1 => Step::Landing,
        2 => Step::Start,
        3 => Step::Hold,
        4 => Step::Skip,
        5 => Step::Merge,
        6 => Step::Port,
        7 => Step::Goal,
        8 => Step::Cancel,
        _ => Step::Restart,
    }
}

fn drawn_tail(seed: u64) -> impl Iterator<Item = (Step, u64, u64, u64)> {
    let mut rng = Rng::new(seed.wrapping_add(91_700));
    (0..STEPS).map(move |_| {
        let step = drawn(&mut rng);
        let cut = rng.below(CUT_TICKS);
        let fault = rng.below(3);
        let crash_seed = rng.next_u64();
        (step, cut, fault, crash_seed)
    })
}

struct Driver {
    world: World,
    referee: Referee,
    key: u8,
    goals: u8,
    cuts_with_commits: u32,
    cuts_with_calls: u32,
    max_open_ticks: usize,
    max_bound: usize,
}
impl Driver {
    fn new() -> Self {
        let world = proof::opening_fixture();
        assert_eq!(world.env.limits.forge.entries, 2);
        assert_eq!(world.env.limits.forge.client.pending, 2);
        assert_eq!(world.env.limits.forge.client.calls, 2);
        assert_eq!(world.env.limits.people.sign_ins, 2);
        Self {
            world,
            referee: Referee::default(),
            key: 150,
            goals: 0,
            cuts_with_commits: 0,
            cuts_with_calls: 0,
            max_open_ticks: 0,
            max_bound: 0,
        }
    }
    fn act(&mut self, act: FollowAct) {
        self.world.send(engine::Event::Party(engine::PartyInput::Application {
            reply_to: ReplyTo::new(Token::new(u64::from(self.key))),
            sign_in: self.world.signed_in.expect("owner session"),
            key: [self.key; 16],
            digest: [self.key; 32],
            scope: people::ApplicationScope::Project(2),
            carried: None,
            request: engine::Application::Follow { project: 2, follow: 1, act },
        }));
    }
    fn ask(&mut self, ask: people::Ask) {
        self.world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(u64::from(self.key))),
            sign_in: self.world.signed_in.expect("owner session"),
            key: [self.key; 16],
            ask,
        }));
    }
    fn dispatch(&mut self, step: Step) {
        self.key += 1;
        let entry = proof::follow(&self.world).queue.first().cloned();
        match step {
            Step::Landing => {
                if proof::follow(&self.world).queue.len() >= 2 {
                    return;
                }
                // In-scope moves are admitted even when the tiny read table
                // makes a comparison incomplete; filtering is the follows sweep's job.
                let after = proof::landing(&mut self.world, b"src/a.c", format!("move {}", self.key).as_bytes());
                self.referee.moves.push((after, true));
            }
            Step::Start | Step::Hold | Step::Skip | Step::Merge => {
                let Some(entry) = entry else { return };
                if matches!(entry.state, FollowState::Producing | FollowState::Held(_) | FollowState::Ended(_)) {
                    return;
                }
                let act = match step {
                    Step::Start => FollowAct::Start { after: entry.after },
                    Step::Hold => FollowAct::Hold { after: entry.after, reason: b"owner waits".as_slice().into() },
                    Step::Skip => FollowAct::Skip { after: entry.after, reason: b"cancel port".as_slice().into() },
                    Step::Merge => {
                        let Some(next) = proof::follow(&self.world).queue.get(1) else { return };
                        if next.change == entry.change {
                            return;
                        }
                        FollowAct::Merge { afters: Box::new([entry.after, next.after]) }
                    }
                    _ => unreachable!("owner action"),
                };
                self.act(act);
            }
            Step::Port => {
                if let Some(entry) = entry
                    && entry.state == FollowState::Producing
                {
                    let change = entry.change.expect("producing change");
                    let producer =
                        self.world.store.change_hot(change).and_then(|row| row.delegate).and_then(|(task, _)| {
                            let record = self
                                .world
                                .store
                                .rows
                                .get(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(task))))?;
                            let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) = record else {
                                return None;
                            };
                            let tasks::Phase::Active(tasks::Active::Running { attempt }) = row.phase else {
                                return None;
                            };
                            self.world
                                .assigned
                                .iter()
                                .rev()
                                .find(|run| run.task == task && run.attempt == attempt)
                                .cloned()
                        });
                    if let Some(producer) = producer {
                        proof::port(&mut self.world, change, &producer, false);
                    }
                }
            }
            Step::Goal => {
                // Leave two worker places for the follow and its producer.
                if self.goals < 2 {
                    self.goals += 1;
                    self.ask(temper_engine_domain_world::walking::goal_from(
                        self.world.chat(1, b"Review the restart"),
                        people::TaskExecutor::Agent { charter: 1 },
                        jig_test_profile::fixtures::Goal { budget: 100, priority: 3, deadline: None },
                    ));
                }
            }
            Step::Cancel => {
                let goal = self.world.store.rows.values().find_map(|row| match row {
                    Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
                        if task.root == task.number && task.project == 1 =>
                    {
                        Some(task.number)
                    }
                    _ => None,
                });
                if let Some(task) = goal {
                    self.ask(people::Ask::Cancel {
                        project: 1,
                        task,
                        reason: b"cancel goal".as_slice().into(),
                        grace: Some(Duration::ZERO),
                    });
                }
            }
            Step::Restart => {}
        }
    }
    fn crash(&mut self, seed: u64) {
        self.cuts_with_commits += u32::from(!self.world.store.pending.is_empty());
        self.cuts_with_calls += u32::from(!self.world.pending.is_empty());
        let durable = self.world.store.rows.clone();
        self.world.crash_restart(seed);
        assert_eq!(self.world.store.rows, durable, "crash loses pending commits, not durable rows");
        // Seven core ranges, census, forge and sweep can each be a one-row page.
        // A row can be visited by census/sweep as well as its owning range: allow
        // eight ticks per durable row for load, resume and commit terminals.
        // Another 128 ticks cover the ten range starts and nine local phases.
        // No term depends on a forge answer, retry, timeout or rate window.
        let bound = 128 + 8 * self.world.store.rows.len();
        self.max_bound = self.max_bound.max(bound);
        self.world.busy_reads = true;
        let mut opened = None;
        for ticks in 1..=bound {
            self.world.tick();
            if self.world.root.ready() && self.world.root.start_phase().is_none() {
                opened = Some(ticks);
                self.max_open_ticks = self.max_open_ticks.max(ticks);
                break;
            }
        }
        assert!(opened.is_some(), "opening exceeded {bound} local ticks; phase {:?}", self.world.root.start_phase());
        self.world.busy_reads = false;
        if let Some(reads) = self.world.held_reads.take() {
            for read in reads {
                self.world.out.push(read);
            }
            self.world.collect();
        }
    }
    fn step(&mut self, step: Step, cut: u64, fault: u64, seed: u64) {
        proof::owner_session(&mut self.world);
        self.dispatch(step);
        // Each spell lasts through the cut and the opening, never through the
        // settled referee. Outstanding remote operations still run after a crash.
        self.world.fake_env.limits.unavailable = if fault == 1 { 1000 } else { 0 };
        self.world.fake_env.limits.rate_limit = u32::from(fault == 2);
        self.world.fake_env.limits.rate_window = Duration::from_secs(1);
        for _ in 0..cut {
            self.world.tick();
        }
        self.crash(seed);
        self.world.fake_env.limits.unavailable = 0;
        self.world.fake_env.limits.rate_limit = 0;
        if let Some((after, _)) = self.referee.moves.last() {
            proof::heard(&mut self.world, *after);
        }
        self.world.drain_cut();
        self.referee.check(&mut self.world);
    }
}
fn same(first: &Driver, replay: &Driver, seed: u64) {
    same_digest(
        &(&first.world.root, &first.world.store.rows, &first.world.events),
        &(&replay.world.root, &replay.world.store.rows, &replay.world.events),
        &format_args!("opening seed {seed}"),
    );
    assert_eq!(first.world.writes, replay.world.writes);
    assert_eq!(first.world.assigned, replay.world.assigned);
}
fn run_replayed(seed: u64) {
    let mut first = Driver::new();
    let mut replay = Driver::new();
    // Exercise each stimulus even on a seed whose random tail omits it. The
    // zero-tick cuts also hit the commits made by the input itself, not a drain.
    for (step, cut) in [
        (Step::Goal, 4),
        (Step::Landing, 1),
        (Step::Hold, 6),
        (Step::Start, 6),
        (Step::Port, 0),
        (Step::Port, 4),
        (Step::Landing, 2),
        (Step::Landing, 3),
        (Step::Merge, 0),
        (Step::Skip, 6),
        (Step::Cancel, 0),
    ] {
        eprintln!("opening seed {seed} prelude: {step:?}, cut {cut}");
        first.step(step, cut, 0, seed);
        replay.step(step, cut, 0, seed);
        same(&first, &replay, seed);
    }
    let prelude_cuts = (first.cuts_with_commits, first.cuts_with_calls);
    for (number, (step, cut, fault, crash_seed)) in drawn_tail(seed).enumerate() {
        eprintln!("opening seed {seed} step {number}: {step:?}, cut {cut}, fault {fault}");
        first.step(step, cut, fault, crash_seed);
        replay.step(step, cut, fault, crash_seed);
        same(&first, &replay, seed);
    }
    eprintln!(
        "opening seed {seed}: cuts with commits {} ({} in prelude), with forge calls {} ({} in prelude); opening at most {} ticks, local bound at most {}",
        first.cuts_with_commits,
        prelude_cuts.0,
        first.cuts_with_calls,
        prelude_cuts.1,
        first.max_open_ticks,
        first.max_bound
    );
    same_text(&first.world.fake, &replay.world.fake, &format_args!("opening forge seed {seed}"));
    assert!(first.cuts_with_commits > 0, "seed cuts an undurable commit");
    assert!(first.cuts_with_calls > 0, "seed cuts an in-flight forge answer");
    first.world.close();
    replay.world.close();
}

#[test]
fn fuzzy_opening_draws_keep_gate_steps_and_accept_full_width_seeds() {
    let mut digest = StateHasher::new();
    for seed in 0..SEEDS {
        for draw in drawn_tail(seed) {
            draw.hash(&mut digest);
        }
    }
    assert_eq!(digest.finish(), 0x08d8_9c69_523e_0d3c, "tail draws in gate seed order");

    assert_eq!(drawn_tail(u64::MAX).count(), usize::try_from(STEPS).expect("bounded tail"));
}

fn shard(test: Test, seeds: Range<u64>) {
    cases(test, seeds, |seed| {
        run_replayed(seed);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

#[test]
fn fuzzy_opening_shard_0() {
    shard(skein_world::this_test!("fuzzy_opening_shard_0"), 0..SEEDS / 2);
}
#[test]
fn fuzzy_opening_shard_1() {
    shard(skein_world::this_test!("fuzzy_opening_shard_1"), SEEDS / 2..SEEDS);
}

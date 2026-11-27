//! Seeded goals over the forge-route world (`temper/docs/design/testing/workloads.md`, 4.1).

use std::collections::{BTreeMap, BTreeSet};

use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_forge_domain as fake;
use skein_lib::{Duration, ReplyTo, Token};
use skein_world::{
    faults::{FaultLedger, Kind},
    streams::{Stream, Streams},
    sweep::Sweep,
};
use temper_workload::parties::{self, Act, Action};

#[path = "workload/people.rs"]
mod acts_driver;
use temper_engine_domain::{self as engine, Key, Record};
use temper_engine_domain_forge as forge;
use temper_workload::{
    draw::{ENGINE, Party, Workload},
    names::Names,
};

use crate::{
    ends::{Bound, Ends, Expect, OPENING},
    forge_routes::{StoryLimits, World},
    protocol,
};

const CHANNEL: Token = Token::new(7);
// Includes three bounded CI rounds, the after-landing tier and all four goals.
const BOUND: Bound = Bound { minutes: 120, steps: 100_000 };

/// One run keeps its durable names available to the comparison and composed drivers.
pub struct Run {
    pub world: World,
    pub workload: Workload,
    pub names: Names,
    pub ledger: FaultLedger,
    pub main: BTreeMap<String, String>,
    pub left: Vec<temper_workload::outcome::Left>,
}

/// The engine adapter's four bounded tables (workloads.md, 2.4).
const TABLE_BOUNDS: [Kind; 4] = [
    temper_faults::bounds::FORGE_WRITE_ENTRIES,
    temper_faults::bounds::FORGE_READ_ADMISSIONS,
    temper_faults::bounds::BRANCH_HOLDS,
    temper_faults::bounds::LANDING_ROWS,
];

/// Run the engine shape without faults (`temper/docs/design/testing/workloads.md`, 9).
pub fn without_acts(streams: &Streams) -> Result<Run, String> {
    let workload = Workload::draw(streams, &ENGINE);
    run_actions(streams, workload, Vec::new(), &Sweep::new("no acts"), |_, _, _, _| {})
}

/// Draw acts independently from the workload and run one seed.
pub fn run(streams: &Streams, configured: &[Act], sweep: &Sweep) -> Result<Run, String> {
    let workload = Workload::draw(streams, &ENGINE);
    let actions = parties::draw(streams, &workload, configured);
    run_actions(streams, workload, actions, sweep, |_, _, _, _| {})
}

/// A later family supplies its own injector without changing the workload or parties' draws.
pub fn run_actions(
    streams: &Streams,
    workload: Workload,
    actions: Vec<Action>,
    sweep: &Sweep,
    mut inject: impl FnMut(&mut World, &Names, &mut FaultLedger, u64),
) -> Result<Run, String> {
    run_injected(*streams, workload, actions, sweep, None, None, &mut inject)
}

/// Observe the tables over the unchanged workload, with no parties' acts drawn.
pub fn run_bounds(streams: &Streams, sweep: &Sweep, limits: &engine::Limits) -> Result<Run, String> {
    run_injected(
        *streams,
        Workload::draw(streams, &ENGINE),
        Vec::new(),
        sweep,
        Some(limits),
        None,
        &mut |_, _, _, _| {},
    )
}

/// Replay the same workload and acts, optionally enabling the engine restart family.
pub fn with_restarts(
    streams: &Streams,
    workload: Workload,
    actions: Vec<Action>,
    sweep: &Sweep,
    injector: Option<crate::forge_routes::restarts::Injector>,
) -> Result<Run, String> {
    run_injected(*streams, workload, actions, sweep, None, injector, &mut |_, _, _, _| {})
}

fn run_injected(
    streams: Streams,
    workload: Workload,
    actions: Vec<Action>,
    sweep: &Sweep,
    bounds: Option<&engine::Limits>,
    injector: Option<crate::forge_routes::restarts::Injector>,
    inject: &mut impl FnMut(&mut World, &Names, &mut FaultLedger, u64),
) -> Result<Run, String> {
    skein_checks::count("workload-seeds", 1);
    let mut driver = Driver::new(streams, workload, actions, sweep, bounds, injector)
        .map_err(|finding| format!("workload seed {}: {finding}", streams.seed()))?;
    driver.run(sweep, inject).map_err(|finding| format!("workload seed {}: {finding}", streams.seed()))?;
    let ledger = driver.world.bounds.take().map_or(driver.ledger, |(_, ledger)| ledger);
    driver.world.check_workload_store()?;
    let tip = driver.world.fake.branch(b"org/repo", b"main").expect("main");
    let main = driver
        .world
        .fake
        .object(tip)
        .expect("main tree")
        .tree
        .iter()
        .map(|(path, text)| {
            (
                String::from_utf8(path.to_vec()).expect("workload path"),
                String::from_utf8(text.to_vec()).expect("workload text"),
            )
        })
        .collect();
    let left = driver
        .world
        .workload_deadlines
        .as_ref()
        .map_or_else(Vec::new, |deadlines| deadlines.left(&driver.world.store, &driver.names, &driver.workload));
    Ok(Run { world: driver.world, workload: driver.workload, names: driver.names, ledger, main, left })
}

struct Driver {
    world: World,
    workload: Workload,
    // A train drawn with one dependency common to all members uses that change's ending,
    // rather than the person's latch, to release the members together.
    cohort: Option<(String, String)>,
    names: Names,
    sessions: BTreeMap<Party, u64>,
    goals: BTreeMap<String, Goal>,
    producers: BTreeMap<u64, String>,
    completed: BTreeSet<(u64, u64)>,
    next_call: u64,
    next_request: u64,
    watch: crate::acts::Watch,
    actions: Vec<Action>,
    ledger: FaultLedger,
    step: u64,
    scripts_due: BTreeMap<(u64, u64), u64>,
    scheduling: skein_lib::Rng,
    unanswered: BTreeMap<String, Act>,
    fallen: Vec<Action>,
    ending_askers: BTreeSet<u64>,
    pending_acts: Vec<Action>,
    observations: Vec<(u64, u64, String)>,
    jobs_at: BTreeMap<u64, u64>,
    resume_after_answer: BTreeSet<u64>,
    excluded_loads: BTreeSet<(String, Kind)>,
    seed: u64,
}

#[derive(Default)]
struct Goal {
    changes: usize,
    closer_task: Option<u64>,
    reserved_changes: u32,
    latch: Option<u64>,
    released: bool,
    latched: bool,
    ends: Option<Ends>,
    expect: Option<Expect>,
    fault_at: Option<skein_lib::Time>,
}

impl Driver {
    fn new(
        streams: Streams,
        workload: Workload,
        actions: Vec<Action>,
        sweep: &Sweep,
        bounds: Option<&engine::Limits>,
        injector: Option<crate::forge_routes::restarts::Injector>,
    ) -> Result<Self, String> {
        let mut world = setup(streams, &workload, bounds);
        if let Some(injector) = injector {
            world.workload_deadlines = Some(crate::outcome::Deadlines::default());
            world.inject_restarts(injector);
        }
        world.adopt();
        let mut sessions = BTreeMap::new();
        sessions.insert(Party::First, world.signed_in.expect("first session"));
        // The second owner signs in separately; parties share no session.
        world.signed_in = None;
        world.send(engine::Event::Party(engine::PartyInput::SignedIn {
            until: None,
            held: None,
            reply_to: ReplyTo::new(Token::new(92)),
            identity: people::Identity {
                key: forge::user_key(1, 8),
                login: b"second".as_slice().into(),
                name: b"Second".as_slice().into(),
            },
        }));
        world.until(crate::forge_routes::Until::SignedIn);
        sessions.insert(Party::Second, world.signed_in.expect("second session"));
        assert_ne!(sessions[&Party::First], sessions[&Party::Second], "separate sessions");
        let mut hello = protocol::hello_slots(CHANNEL, world.env.limits.fleet.slots, Box::new([]));
        if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) =
            &mut hello
        {
            hello.bounds = jig_test_profile::fixtures::bounds();
            hello.bounds.turns = 800;
            hello.bounds.time = Duration::from_secs(28_800);
            let model = crate::walking::charter().policy.model;
            hello.bounds.endpoints = Box::new([jig_domain_link::EndpointBound {
                endpoint: model.endpoint,
                window: model.window,
                output: model.output,
            }]);
            hello.bounds.message_bytes = world.env.limits.tasks.message_bytes;
            hello.jobs = Box::new([jig_domain_link::JobBound {
                kind: jig_domain_link::JobKind(1),
                spec: world.env.limits.tasks.spec_bytes,
                result: world.env.limits.tasks.result_bytes,
                deadline: Duration::from_secs(600),
            }]);
            world.workload_worker = Some((hello.bounds.clone(), hello.jobs.clone(), hello.slots));
        }
        world.send(hello);
        let mut ledger = FaultLedger::new();
        let actions: Vec<_> = actions
            .into_iter()
            .filter(|action| {
                let holding = if action.act == Act::AmendPassedDeadline
                    && workload.goal(&action.target).is_some_and(|goal| goal.party == action.party)
                {
                    vec!["the amending party started the goal"]
                } else {
                    Vec::new()
                };
                if let Some(exclusion) = sweep.leaves_out(crate::acts::kind(action.act), &holding) {
                    ledger.leave_out(exclusion.scenario, crate::acts::kind(action.act), world.env.now);
                    false
                } else {
                    true
                }
            })
            .collect();
        let unanswered = actions
            .iter()
            .filter(|action| {
                matches!(action.act, Act::AnswerLate | Act::NeverAnswer | Act::AnswerTwice | Act::AnswerEndedRun)
            })
            .map(|action| (action.target.clone(), action.act))
            .collect();
        let mut driver = Self {
            cohort: drawn_cohort(&workload),
            world,
            workload,
            names: Names::new(),
            sessions,
            goals: BTreeMap::new(),
            producers: BTreeMap::new(),
            completed: BTreeSet::new(),
            next_call: 1000,
            next_request: 100,
            watch: crate::acts::Watch::new(actions.clone()),
            actions,
            ledger,
            step: 0,
            scripts_due: BTreeMap::new(),
            scheduling: streams.rng(Stream::Scheduling, "engine"),
            unanswered,
            fallen: Vec::new(),
            ending_askers: BTreeSet::new(),
            pending_acts: Vec::new(),
            observations: Vec::new(),
            jobs_at: BTreeMap::new(),
            resume_after_answer: BTreeSet::new(),
            excluded_loads: BTreeSet::new(),
            seed: streams.seed(),
        };
        driver.start_goals(sweep)?;
        Ok(driver)
    }

    fn request(&mut self, party: Party, ask: people::Ask) -> people::Outcome {
        self.next_request += 1;
        let id = self.next_request;
        let mut key = [0; 16];
        key[..8].copy_from_slice(&id.to_be_bytes());
        self.world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(id)),
            sign_in: self.sessions[&party],
            key,
            ask,
        }));
        for _ in 0..1000 {
            self.world.tick();
            if let Some(outcome) = self.world.store.rows.values().find_map(|row| match row {
                Record::Core(core::Record::People(people::Stored::Answer { key: request, outcome, .. }))
                    if request.key == key =>
                {
                    Some(outcome.clone())
                }
                _ => None,
            }) {
                return outcome;
            }
        }
        panic!("party request {id} unanswered")
    }

    fn call(&mut self, run: &engine::Assignment, tool: engine::Tool) -> engine::CallAnswer {
        self.next_call += 1;
        self.world.scripted_call(run, self.next_call, tool)
    }

    fn start_goals(&mut self, sweep: &Sweep) -> Result<(), String> {
        let mut chats = BTreeMap::new();
        for party in [Party::First, Party::Second] {
            let mut ask = self.world.chat(1, b"Run the workload");
            if let people::Ask::Create { batch, .. } = &mut ask {
                batch[0].budget = people::Budget::AsAllowed { cap: None };
            }
            let task = created(&self.request(party, ask)).map_err(|error| format!("{party:?} chat: {error}"))?;
            let run = self.await_run(task);
            chats.insert(party, run);
        }
        // The gathering owns the chain; the later goal carries the earlier task as its dependency.
        let milestone = if self.workload.goals.iter().any(|goal| goal.after.is_some()) {
            let people::Ask::Create { project, mut batch } = self.world.chat(1, b"Workload milestone") else {
                unreachable!()
            };
            batch[0].executor = people::TaskExecutor::Procedure { connector: core::RECURRING, code: core::GATHERING };
            batch[0].flags = people::Flags { ended_by: people::EndedBy::Requester, ..people::Flags::PLAIN };
            batch[0].tracked = Some(1);
            batch[0].budget = people::Budget::AsAllowed { cap: None };
            Some(created(&self.request(Party::First, people::Ask::Create { project, batch }))?)
        } else {
            None
        };
        for goal in self.workload.goals.clone() {
            let mut admitted = false;
            for _ in 0..BOUND.steps {
                if self.load_allowed(sweep, 0, &goal.name) {
                    admitted = true;
                    break;
                }
                self.world.tick();
            }
            if !admitted {
                return Err("goal load remained excluded through the workload bound".into());
            }
            let chat = &chats[&goal.party];
            let in_chain = goal.after.is_some()
                || self.workload.goals.iter().any(|later| later.after.as_deref() == Some(&goal.name));
            let task = if goal.train && !in_chain {
                delegated(self.call(
                    chat,
                    engine::Tool::Goal { words: goal.name.as_bytes().into(), advisor: false, agree: false },
                ))?
            } else {
                let mut ask = self.world.chat(1, goal.name.as_bytes());
                let people::Ask::Create { batch, .. } = &mut ask else { unreachable!() };
                batch[0].tracked = Some(0);
                batch[0].flags = people::Flags {
                    listed: true,
                    ended_by: people::EndedBy::Closer(people::Closer { connector: 1, code: 6 }),
                    ..people::Flags::PLAIN
                };
                batch[0].symbols = Box::new([people::Symbol { connector: 1, symbol: 1 }]);
                if goal.train {
                    batch[0].flags.ended_by = people::EndedBy::Result;
                    batch[0].symbols = Box::new([]);
                }
                batch[0].budget = people::Budget::AsAllowed {
                    cap: Some(task(&self.world, chat.task).expect("chat").numbers.budget / 4),
                };
                if let Some(after) = &goal.after {
                    batch[0].dependencies =
                        Box::new([people::Dependency::Existing(self.names.task(after).expect("earlier goal"))]);
                }
                let people::Ask::Create { project, mut batch } = ask else { unreachable!() };
                let parent = if in_chain { milestone.expect("chain") } else { chat.task };
                let parent_authority = &task(&self.world, parent).expect("parent").authority;
                batch[0].authority.delegation.depth = parent_authority.delegation.depth.saturating_sub(1);
                batch[0].authority.delegation.tasks = parent_authority.delegation.tasks / 2;
                if self
                    .actions
                    .iter()
                    .any(|action| action.target == goal.name && action.act == Act::AmendPassedDeadline)
                {
                    batch[0].authority.deadline = Some(skein_lib::Wall::from_nanos(
                        self.world.env.wall.as_nanos() + Duration::from_secs(1).as_nanos(),
                    ));
                    batch[0].deadline = people::Deadline::Named;
                }
                created(&self.request(goal.party, people::Ask::Delegate { project, task: parent, batch }))
                    .map_err(|error| format!("{} under {parent}: {error}", goal.name))?
            };
            self.names.name(&goal.name, task)?;
            if goal.train && !in_chain && shared_dependency(&goal).is_none() {
                self.authorize_latch(&goal, task);
            }
            self.goals.insert(goal.name, Goal::default());
        }
        for chat in chats.values() {
            self.finish(chat, tasks::TaskResult::Report { words: b"Goals started".as_slice().into() });
        }
        if let Some(task) = milestone {
            let outcome = self.request(Party::First, people::Ask::Conclude { project: 1, task });
            assert!(!matches!(outcome, people::Outcome::Refused { .. }), "conclude milestone: {outcome:?}");
        }
        Ok(())
    }

    fn authorize_latch(&mut self, goal: &temper_workload::draw::Goal, task: u64) {
        let row = task_row(&self.world, task).expect("new train goal");
        let revision = row.revisions.spec;
        let mut authority = acts_driver::person_authority(&row.authority);
        let mut kinds = authority.delegation.kinds.to_vec();
        kinds.push(people::Executor::Role(0));
        authority.delegation.kinds = kinds.into();
        if self.actions.iter().any(|action| action.target == goal.name && action.act == Act::AmendPassedDeadline) {
            authority.deadline =
                Some(skein_lib::Wall::from_nanos(self.world.env.wall.as_nanos() + Duration::from_secs(1).as_nanos()));
        }
        let outcome = self.request(
            goal.party,
            people::Ask::Amend {
                project: 1,
                task,
                revision,
                release: None,
                amendment: people::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    spec: None,
                    wake: None,
                    dependencies: None,
                    authority: Some(authority),
                    llm: None,
                    reason: b"Allow the person's readiness latch".as_slice().into(),
                },
            },
        );
        assert!(!matches!(outcome, people::Outcome::Refused(_)), "train latch authority: {outcome:?}");
    }

    fn delegate(&self, parent: &engine::Assignment, words: &[u8]) -> engine::Delegate {
        let mut delegate = crate::fixtures::report_delegate(words, Box::new([]));
        delegate.executor = tasks::Executor::Agent { charter: parent.charter };
        delegate.authority = task(&self.world, parent.task).expect("parent").authority.clone();
        delegate.authority.budget.spend /= 2;
        delegate.authority.delegation.tasks /= 2;
        delegate.authority.delegation.depth = delegate.authority.delegation.depth.saturating_sub(1);
        delegate.tracked = None;
        delegate.flags = tasks::Flags::PLAIN;
        delegate
    }

    fn await_run(&mut self, task: u64) -> engine::Assignment {
        for _ in 0..2000 {
            if let Some(run) = running(&self.world, task) {
                return run;
            }
            self.world.tick();
        }
        panic!(
            "task {task} never assigned: {:?}; assignments {}",
            task_row(&self.world, task).map(|row| &row.phase),
            self.world.assigned.len()
        );
    }

    fn finish(&mut self, run: &engine::Assignment, result: tasks::TaskResult) {
        self.world.scripted_read_inbox(run);
        let event = self.world.protocol.answer(
            CHANNEL,
            run.task,
            run.attempt,
            0,
            tasks::End::Finished { result, cancel_delegates: false },
            engine::Parts::none(),
        );
        self.world.send(event);
        self.completed.insert((run.task, run.attempt));
    }

    fn park(&mut self, run: &engine::Assignment) {
        self.world.scripted_read_inbox(run);
        let event =
            self.world.protocol.answer(CHANNEL, run.task, run.attempt, 0, tasks::End::Parked, engine::Parts::none());
        self.world.send(event);
        self.completed.insert((run.task, run.attempt));
    }

    fn finish_restarts(&mut self) -> Result<(), String> {
        if self.world.restarts.is_none() {
            return Ok(());
        }
        let restarts = self.world.finish_restarts();
        for fault in restarts.entries() {
            let entry = self.ledger.inject(fault.kind, fault.from, fault.struck.clone());
            if let Some(until) = fault.until {
                self.ledger.stop(entry, until)?;
            }
        }
        Ok(())
    }

    fn run(
        &mut self,
        sweep: &Sweep,
        inject: &mut impl FnMut(&mut World, &Names, &mut FaultLedger, u64),
    ) -> Result<(), String> {
        for step in 0..BOUND.steps {
            self.step = step;
            inject(&mut self.world, &self.names, &mut self.ledger, step);
            crate::tier_runs::try_tick(&mut self.world).map_err(|finding| {
                format!(
                    "{finding}; tasks {:?}; goals {:?}; assigned {:?}; changes {:?}; tier {:?}",
                    self.world
                        .store
                        .rows
                        .values()
                        .filter_map(|row| if let Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) = row {
                            Some((
                                task.number,
                                task.requester,
                                task.executor,
                                &task.phase,
                                task.numbers,
                                task.tries,
                                &task.waiting_on,
                            ))
                        } else {
                            None
                        })
                        .collect::<Vec<_>>(),
                    self.goals
                        .iter()
                        .map(|(name, goal)| (name, goal.changes, goal.closer_task.is_some()))
                        .collect::<Vec<_>>(),
                    self.world.assigned.iter().map(|run| (run.task, run.attempt)).collect::<Vec<_>>(),
                    self.workload
                        .goals
                        .iter()
                        .flat_map(|goal| &goal.changes)
                        .filter_map(|change| self.names.task(&change.name).and_then(|number| self
                            .world
                            .store
                            .change_hot(number)
                            .map(|row| (number, &row.change.state))))
                        .collect::<Vec<_>>(),
                    crate::tier_runs::tier(&self.world)
                )
            })?;
            self.fire_actions(sweep)?;
            self.end_askers();
            self.settle_stops();
            self.prepare_jobs();
            self.latch_trains();
            self.release_trains();
            self.run_due_assignments(sweep, step)?;
            self.judge_goals()?;
            if self.finished() {
                self.finish_restarts()?;
                self.judge_goals()?;
                self.check_ends()?;
                self.tier()?;
                return Ok(());
            }
        }
        Err(format!(
            "workload did not end: {:?}; fell {:?}; pending {:?}; changes {:?}",
            self.world
                .store
                .rows
                .values()
                .filter_map(|row| if let Record::Core(core::Record::Tasks(tasks::Stored::Live(task))) = row {
                    Some((task.number, &task.phase, &task.questions, &task.waiting_on))
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            self.fallen,
            self.pending_acts,
            self.workload
                .goals
                .iter()
                .flat_map(|goal| &goal.changes)
                .filter_map(|change| self.names.task(&change.name).and_then(|number| self
                    .world
                    .store
                    .change_hot(number)
                    .map(|row| (number, &row.change.state))))
                .collect::<Vec<_>>()
        ))
    }

    fn run_due_assignments(&mut self, sweep: &Sweep, step: u64) -> Result<(), String> {
        // Assignments only append. Keep this pass's horizon even when a script
        // ticks the world and appends another run; copy a payload only when due.
        for index in 0..self.world.assigned.len() {
            let run = &self.world.assigned[index];
            if self.completed.contains(&(run.task, run.attempt))
                || task(&self.world, run.task).is_none_or(|row| {
                    row.phase != tasks::Phase::Active(tasks::Active::Running { attempt: run.attempt })
                })
            {
                continue;
            }
            let due = self
                .scripts_due
                .entry((run.task, run.attempt))
                .or_insert_with(|| step + self.scheduling.between(1, 60));
            if step < *due {
                continue;
            }
            *due = step + self.scheduling.between(1, 60);
            let run = run.clone();
            if let Some(name) = self.names.logical(run.task).map(str::to_owned) {
                self.coordinate(&name, &run, sweep)?;
            } else {
                self.produce(&run);
            }
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the scripted coordinator keeps each phase and its load admission together"
    )]
    fn coordinate(&mut self, name: &str, run: &engine::Assignment, sweep: &Sweep) -> Result<(), String> {
        let Some(goal) = self.workload.goal(name).cloned() else {
            return Ok(());
        };
        let cohort = shared_dependency(&goal);
        if goal.train && self.goals[name].latch.is_none() {
            let latch = if let Some(dependency) = cohort {
                let Some(task) = self.names.task(dependency) else { return Ok(()) };
                task
            } else {
                let mut latch = crate::fixtures::report_delegate(b"Release the ready train", Box::new([]));
                latch.executor = tasks::Executor::Person(tasks::PersonAddress::Person(match goal.party {
                    Party::First => 1,
                    Party::Second => 2,
                }));
                latch.authority.tools = tasks::Tools(0);
                latch.authority.budget.spend = 0;
                delegated(self.call(run, engine::Tool::Delegate { batch: Box::new([latch]) }))?
            };
            self.goals.get_mut(name).expect("train").latch = Some(latch);
        }
        let progress = &self.goals[name];
        if progress.changes < goal.changes.len() {
            let next = &goal.changes[progress.changes];
            let remaining = u32::try_from(goal.changes.len() - progress.changes).expect("bounded train");
            let demand = if goal.train { if progress.changes == 0 { 2 * remaining } else { 0 } } else { 2 };
            if !self.load_allowed(sweep, demand, &next.name) {
                return Ok(());
            }
            if next.after.iter().filter_map(|name| self.names.task(name)).any(|number| {
                task(&self.world, number).is_some_and(|row| {
                    matches!(
                        &row.phase,
                        tasks::Phase::Ended(tasks::Ending::Cancelled { .. } | tasks::Ending::Failed { .. })
                            | tasks::Phase::Closing(tasks::Closing {
                                ending: tasks::Ending::Cancelled { .. } | tasks::Ending::Failed { .. },
                                ..
                            })
                    )
                })
            }) {
                self.abandon(run);
                return Ok(());
            }
            {
                let after = next
                    .after
                    .iter()
                    .filter(|name| {
                        self.workload.change(name).is_some_and(|(owner, _)| owner.name == goal.name)
                            && self.names.task(name).is_some_and(|number| !ended(&self.world, number))
                    })
                    .map(|name| engine::ChangeAfter::Task(self.names.task(name).expect("earlier change named")))
                    .collect();
                let mut change = engine::ChangeCall::words(next.name.as_bytes().into(), None);
                change.after = after;
                change.questions = Some(engine::ChangeQuestions::Up);
                let task = delegated(self.call(run, engine::Tool::Change { changes: Box::new([change]) }))?;
                self.names.name(&next.name, task)?;
                // For a cohort, latch_trains adds the shared dependency once the
                // producers have started, so they can do their work before it lands.
                if cohort.is_none() {
                    self.amend_after(&goal, next, task);
                }
                let progress = self.goals.get_mut(name).expect("goal");
                progress.changes += 1;
                if goal.train {
                    progress.reserved_changes = remaining - 1;
                }
                return Ok(());
            }
        }
        let open = goal.changes.iter().filter_map(|change| self.names.task(&change.name)).find_map(|change| {
            task(&self.world, change).and_then(|row| row.questions.first()).map(|question| (change, question.number))
        });
        if let Some((change, question)) = open {
            let logical = self.names.logical(change).expect("change name");
            if self.unanswered.contains_key(logical) {
                return Ok(());
            }
            self.next_call += 1;
            self.world.goal_answer(run, self.next_call, change, question, b"Proceed");
            return Ok(());
        }
        let late_words = self.actions.iter().any(|action| {
            action.act == Act::WordsToEndedProducer
                && goal.changes.iter().any(|change| change.name == action.target)
                && self.names.task(&action.target).is_some_and(|number| {
                    crate::acts::producers(&self.world.store, number)
                        .any(|producer| matches!(producer.phase, tasks::Phase::Ended(_)))
                })
                && !self.fallen.contains(action)
        });
        if late_words {
            return Ok(());
        }
        if progress.changes == goal.changes.len()
            && goal
                .changes
                .iter()
                .all(|change| self.names.task(&change.name).is_some_and(|task| ended(&self.world, task)))
            && goal.train
        {
            self.finish(run, tasks::TaskResult::Report { words: b"Train landed".as_slice().into() });
            return Ok(());
        }
        if progress.closer_task.is_none()
            && progress.changes == goal.changes.len()
            && goal
                .changes
                .iter()
                .all(|change| self.names.task(&change.name).is_some_and(|task| ended(&self.world, task)))
        {
            if !self.load_allowed(sweep, 1, &format!("{name}.closer")) {
                return Ok(());
            }
            let mut closer = self.delegate(run, b"Close the workload goal");
            closer.executor = tasks::Executor::Procedure { connector: 1, code: 6 };
            closer.authority.budget.spend = task(&self.world, run.task).expect("goal").numbers.budget / 2;
            let closer = delegated(self.call(run, engine::Tool::Delegate { batch: Box::new([closer]) }))?;
            let progress = self.goals.get_mut(name).expect("goal");
            progress.closer_task = Some(closer);
        }
        if self.goals[name].closer_task.is_some() {
            self.park(run);
        }
        Ok(())
    }

    // Exclusions act before adding load, not after an inspection reports a finding.
    // A left-out attempt may try again after the existing room wait clears.
    fn load_allowed(&mut self, sweep: &Sweep, added: u32, load: &str) -> bool {
        let stalled = self.world.root.forge().inspection_changes().iter().any(|(number, row)| {
            row.room.is_some()
                && !self.world.root.forge().room_asked(*number)
                && self.world.env.wall.as_nanos().saturating_sub(row.change.since.as_nanos())
                    > self.world.env.limits.forge.change_policy.stall.as_nanos()
        });
        let forge = self.world.root.forge();
        let holds = forge.inspection_holds();
        // prepare_workspace reserves one prefix per not-yet-prepared member;
        // each newly created change also needs its own branch's hold.
        let pending: u32 = forge
            .inspection_changes()
            .iter()
            .map(|(number, row)| {
                if ended(&self.world, *number) {
                    return 0;
                }
                let branch = u32::from(!branch_held(forge, &row.branch));
                let prepared =
                    forge.landing_of(row.task).and_then(|landing| forge.landing(landing)).is_some_and(|landing| {
                        landing.landing.members.iter().any(|member| {
                            member.change == *number
                                && (member.candidate.is_some() || branch_held(forge, &member.branch))
                        })
                    });
                branch + u32::from(!prepared)
            })
            .sum();
        // A started closer's change reuses the goal branch; reserve its one
        // prepare prefix until that change appears in the adapter's rows.
        let closers = self
            .goals
            .values()
            .filter_map(|goal| goal.closer_task)
            .filter(|closer| {
                !ended(&self.world, *closer)
                    && !forge.inspection_changes().iter().any(|(number, _)| {
                        task(&self.world, *number).is_some_and(|row| row.requester == tasks::Party::Task(*closer))
                    })
            })
            .count();
        let reserved = self.goals.values().map(|goal| 2 * goal.reserved_changes).sum::<u32>();
        let lacks_room = added > 0
            && holds.len() + pending + reserved + u32::try_from(closers).expect("bounded closers") + added
                > holds.capacity();
        let mut holding = Vec::new();
        if stalled {
            holding.push("a change waiting for room past its stall");
        }
        if lacks_room {
            holding.push("the holds and pending prepares, including the new change or train, exceed capacity");
        }
        for kind in [temper_faults::bounds::FORGE_WRITE_ENTRIES, temper_faults::bounds::BRANCH_HOLDS] {
            if let Some(exclusion) = sweep.leaves_out(kind, &holding) {
                let ledger = self.world.bounds.as_mut().map_or(&mut self.ledger, |(_, ledger)| ledger);
                if self.excluded_loads.insert((load.to_owned(), kind)) {
                    ledger.leave_out(exclusion.scenario, kind, self.world.env.now);
                }
                return false;
            }
        }
        true
    }

    fn amend_after(&mut self, goal: &temper_workload::draw::Goal, change: &temper_workload::draw::Change, number: u64) {
        let foreign: Vec<_> = change
            .after
            .iter()
            .filter(|name| {
                self.workload.change(name).is_some_and(|(owner, _)| owner.name != goal.name)
                    || self.names.task(name).is_some_and(|number| ended(&self.world, number))
            })
            .map(|name| self.names.task(name).expect("earlier milestone change named"))
            .collect();
        if foreign.is_empty() {
            return;
        }
        // The coordinator references its retained delegates. The owner can name
        // another goal's change or an ended delegate already evicted from that
        // cache through the authenticated amendment route.
        let row = task(&self.world, number).expect("new change");
        let revision = row.revisions.spec;
        let mut dependencies = row.dependencies.to_vec();
        dependencies.extend(foreign);
        let outcome = self.request(
            goal.party,
            people::Ask::Amend {
                project: 1,
                task: number,
                revision,
                release: None,
                amendment: people::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    spec: None,
                    wake: None,
                    dependencies: Some(dependencies.into()),
                    authority: None,
                    llm: None,
                    reason: b"Keep the drawn change dependency".as_slice().into(),
                },
            },
        );
        assert!(!matches!(outcome, people::Outcome::Refused(_)), "cross-goal dependency: {outcome:?}");
    }

    fn abandon(&mut self, run: &engine::Assignment) {
        self.world.scripted_read_inbox(run);
        let event = self.world.protocol.answer(
            CHANNEL,
            run.task,
            run.attempt,
            0,
            tasks::End::Finished {
                result: tasks::TaskResult::Report {
                    words: b"The cancelled dependency ends this plan".as_slice().into(),
                },
                cancel_delegates: true,
            },
            engine::Parts::none(),
        );
        self.world.send(event);
        self.completed.insert((run.task, run.attempt));
    }

    fn latch_trains(&mut self) {
        for index in 0..self.workload.goals.len() {
            let goal = &self.workload.goals[index];
            if !goal.train {
                continue;
            }
            let progress = &self.goals[&goal.name];
            if progress.latched || progress.changes != goal.changes.len() {
                continue;
            }
            let ready = goal.changes.iter().all(|change| {
                self.names.task(&change.name).is_some_and(|number| {
                    ended(&self.world, number)
                        || self.world.assigned.iter().any(|run| {
                            task(&self.world, run.task).is_some_and(|row| {
                                row.requester == tasks::Party::Task(number)
                                    && (!change.asks || self.producers.contains_key(&run.task))
                                    && row.questions.is_empty()
                            })
                        }) && self.world.store.change_hot(number).is_some_and(|row| row.questions.open.is_empty())
                })
            });
            if !ready {
                continue;
            }
            let latch = progress.latch.expect("train latch");
            let goal = goal.clone();
            for change in &goal.changes {
                let number = self.names.task(&change.name).expect("opened member");
                let row = task_row(&self.world, number).expect("train member");
                if matches!(row.phase, tasks::Phase::Ended(_) | tasks::Phase::Closing(_)) {
                    continue;
                }
                let mut dependencies = row.dependencies.to_vec();
                dependencies.push(latch);
                let revision = row.revisions.spec;
                let outcome = self.request(
                    goal.party,
                    people::Ask::Amend {
                        project: 1,
                        task: number,
                        revision,
                        release: None,
                        amendment: people::Amendment {
                            tracked: None,
                            addressed: None,
                            instructions: None,
                            spec: None,
                            wake: None,
                            dependencies: Some(dependencies.into()),
                            authority: None,
                            llm: None,
                            reason: b"Gather the ready train".as_slice().into(),
                        },
                    },
                );
                assert!(!matches!(outcome, people::Outcome::Refused(_)), "latch member: {outcome:?}");
            }
            self.goals.get_mut(&goal.name).expect("train").latched = true;
        }
    }

    fn release_trains(&mut self) {
        for index in 0..self.workload.goals.len() {
            let goal = &self.workload.goals[index];
            let progress = &self.goals[&goal.name];
            let Some(latch) = progress.latch else { continue };
            if progress.released || progress.changes != goal.changes.len() {
                continue;
            }
            let ready = goal.changes.iter().all(|change| {
                self.names.task(&change.name).is_some_and(|number| {
                    ended(&self.world, number)
                        || (self.world.store.change_hot(number).is_some_and(|row| {
                            matches!(row.change.state, temper_engine_domain_forge_change::State::Gating { .. })
                                && row.delegate.is_none()
                                && row.verdicts.iter().any(|verdict| {
                                    verdict.status == temper_engine_domain_forge_change::Status::Passed
                                        && Some(verdict.head) == row.change.last_head
                                })
                        }) && self.world.root.tasks().procedure_due(number).is_none())
                })
            });
            if ready {
                if shared_dependency(goal).is_some() {
                    // Every member is gating behind the named change. Let its
                    // producer finish; only that change's landing releases them.
                    self.goals.get_mut(&goal.name).expect("cohort").released = true;
                    continue;
                }
                let revision = task(&self.world, latch).expect("latch").revisions.spec;
                let name = goal.name.clone();
                let party = goal.party;
                let outcome = self.request(
                    party,
                    people::Ask::AnswerPerson {
                        project: 1,
                        task: latch,
                        revision,
                        result: people::PersonResult::Report { words: b"Ready together".as_slice().into() },
                    },
                );
                assert!(!matches!(outcome, people::Outcome::Refused(_)), "release train: {outcome:?}");
                self.goals.get_mut(&name).expect("train").released = true;
            }
        }
    }

    fn produce(&mut self, run: &engine::Assignment) {
        let Some(row) = task(&self.world, run.task) else { return };
        let tasks::Party::Task(parent) = row.requester else { return };
        if self.cohort.as_ref().is_some_and(|(train, dependency)| {
            !self.goals[train].released && self.names.task(dependency) == Some(parent)
        }) {
            return;
        }
        let Some(change_row) = self.world.store.change(parent) else { return };
        if let Some((delegate, temper_engine_domain_forge_change::Delegate::Gate { head, .. })) = change_row.delegate
            && delegate == run.task
        {
            let red =
                self.names.logical(parent).and_then(|name| self.workload.change(name)).is_some_and(|(_, change)| {
                    let commit = fake::commit_count(&head).expect("fake gate head");
                    let tree = &self.world.fake.object(commit).expect("candidate tree").tree;
                    change
                        .files
                        .iter()
                        .any(|file| tree.get(file.path.as_bytes()).map(AsRef::as_ref) != Some(file.text.as_bytes()))
                });
            self.finish(
                run,
                tasks::TaskResult::Verdict {
                    code: if red { 2 } else { 1 },
                    fields: Box::new([]),
                    words: if red {
                        b"Repair the drawn file contents".as_slice().into()
                    } else {
                        b"Approved workload tree".as_slice().into()
                    },
                },
            );
            return;
        }
        let Some(name) = self.names.logical(parent).map(str::to_owned) else { return };
        let (goal, change) = self.workload.change(&name).expect("named change");
        let train_waits = goal.train && !self.goals[&goal.name].latched;
        let change = change.clone();
        let is_repair = change_row.change.repairs > 0;
        if change.asks && !self.producers.contains_key(&run.task) && !is_repair {
            self.producers.insert(run.task, name);
            self.next_call += 1;
            self.world.producer_question(run, self.next_call, b"Proceed?");
            self.park(run);
            return;
        }
        if !row.questions.is_empty() {
            self.park(run);
            return;
        }
        if train_waits {
            return;
        }
        let branch = run.workspace.repositories.first().and_then(|repo| repo.push.clone()).expect("producer branch");
        if self.world.fake.branch(b"org/repo", &branch).is_none() {
            let base = self.world.fake.branch(b"org/repo", &change_row.base).expect("base branch");
            self.world.run_push(run, &branch, base, 1);
        }
        for (index, file) in change.files.iter().enumerate() {
            let text = if change.red_once && !is_repair && index == 0 {
                b"The first gate must reject this tree\n".as_slice()
            } else {
                file.text.as_bytes()
            };
            self.world.run_push_edit(run, &branch, file.path.as_bytes(), text, 1);
        }
        self.finish(
            run,
            tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(change.name.as_bytes().into()),
                connector: 1,
                kind: 2,
                resource: forge::tag(1, 2),
                words: b"Workload files delivered".as_slice().into(),
            },
        );
    }

    fn tier(&mut self) -> Result<(), String> {
        if !self.workload.tier {
            return Ok(());
        }
        let main = self.world.fake.branch(b"org/repo", b"main").expect("main");
        let main = temper_engine_forge_world::translate::commit(main);
        // A run at an earlier tip may still be in flight. Allow both it and the final
        // tip's run their workflow window and the repository's polling fallback
        // (temper/docs/design/domain/after-landing-runs.md, 2).
        let client = &self.world.env.limits.forge.client;
        let polling = client.poll.max(client.fallback);
        let run_ticks =
            u64::from(crate::tier_runs::RUN_TICKS) + polling.as_nanos().div_ceil(self.world.tick_interval.as_nanos());
        let final_tip_ticks = 2 * run_ticks;
        for _ in 0..final_tip_ticks {
            crate::tier_runs::try_tick(&mut self.world)?;
            let tier = crate::tier_runs::tier(&self.world);
            if tier.record.judged.as_ref().is_some_and(|judged| judged.tip == main)
                && matches!(&tier.record.failing, temper_engine_domain_forge_tiers::Failures::Tests(tests) if tests.is_empty())
            {
                return Ok(());
            }
        }
        let tier = crate::tier_runs::tier(&self.world);
        Err(format!(
            "main's after-landing tier did not judge the final tip green: main {main:?}; judged {:?}; failing {:?}; runs {:?}",
            tier.record.judged,
            tier.record.failing,
            crate::tier_runs::runs(&self.world)
                .iter()
                .map(|run| (run.number, run.commit, run.state, run.conclusion))
                .collect::<Vec<_>>()
        ))
    }
}

fn drawn_cohort(workload: &Workload) -> Option<(String, String)> {
    workload
        .goals
        .iter()
        .find_map(|goal| shared_dependency(goal).map(|dependency| (goal.name.clone(), dependency.to_owned())))
}

fn shared_dependency(goal: &temper_workload::draw::Goal) -> Option<&str> {
    if !goal.train || goal.changes.len() < 2 {
        return None;
    }
    let [dependency] = goal.changes.first()?.after.as_slice() else { return None };
    goal.changes.iter().all(|change| change.after.as_slice() == [dependency.as_str()]).then_some(dependency)
}

fn setup(streams: Streams, workload: &Workload, bounds: Option<&engine::Limits>) -> World {
    let limits = bounds.copied().unwrap_or_else(crate::forge_routes::box_limits);
    let model = crate::walking::charter().policy.model;
    let mut defaults = temper_engine_service_stock::stock::with_models(
        Box::new([core::settings::CatalogueModel {
            first_read: None,
            number: 1,
            model: model.clone(),
            shown: None,
            description: None,
            picker: true,
            withdrawn: false,
            source: core::settings::ModelSource::Stock,
            estimated: false,
        }]),
        limits.tasks.result_bytes,
    );
    // Share the existing owner allotment between two people; the period and all limits stay fixed.
    defaults
        .policies
        .get_mut(&1)
        .expect("project policy")
        .policy
        .roles
        .iter_mut()
        .find(|role| role.number == 0)
        .expect("owner role")
        .period_spend /= 2;
    let mut world = World::configured_stock_turn_with(
        &defaults,
        &model,
        &limits,
        StoryLimits {
            scheduling_seed: Some(streams.seed_of(Stream::Scheduling, "engine")),
            // Each input file, its repair and its prepared prefix; goal landings add two commits apiece.
            forge_files: Some(
                u32::try_from(
                    workload
                        .goals
                        .iter()
                        .flat_map(|goal| &goal.changes)
                        .map(|change| change.files.len())
                        .sum::<usize>(),
                )
                .expect("small workload"),
            ),
            forge_commits: Some(
                u32::try_from(
                    workload
                        .goals
                        .iter()
                        .map(|goal| 1 + goal.changes.iter().map(|change| 2 * change.files.len() + 2).sum::<usize>())
                        .sum::<usize>()
                        * 3,
                )
                .expect("bounded commits"),
            ),
            batch: workload
                .goals
                .iter()
                .find(|goal| goal.train)
                .map(|goal| u16::try_from(goal.changes.len()).expect("small train")),
            tier: Some(crate::tier_runs::Tier {
                wait: temper_engine_domain_forge_change::Wait::Landed,
                ..crate::tier_runs::Tier::default()
            }),
            ..StoryLimits::default()
        },
    )
    .with_workload_parties()
    .with_landing_deployment(Box::new([engine::LandingRule {
        sets: Box::new([]),
        connector: 1,
        kind: 4,
        enforce: true,
        pattern: jig_domain_authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: jig_domain_authority::Last::Open(Box::new([])),
        },
        ci: false,
        up_to_date: false,
        gates: Box::new([engine::Gate { number: 1, blocking: true, freshness: engine::Freshness::Clean }]),
        approvals: Box::new([]),
    }]));
    if bounds.is_some() {
        world.bounds = Some((TABLE_BOUNDS.to_vec(), FaultLedger::new()));
    }
    world.manual_prepare_jobs();
    world.fake_env.limits.latency_min = Duration::from_millis(1);
    world.fake_env.limits.latency_max = Duration::from_millis(5);
    world
}

pub(crate) fn task(world: &World, number: u64) -> Option<&tasks::TaskRecord> {
    task_row(world, number)
}
fn task_row(world: &World, number: u64) -> Option<&tasks::TaskRecord> {
    for key in [tasks::Key::Live(number), tasks::Key::Ended(number)] {
        if let Some(Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))) =
            world.store.rows.get(&Key::Core(core::Key::Tasks(key)))
        {
            return Some(row);
        }
    }
    None
}
fn branch_held(forge: &forge::Domain, branch: &[u8]) -> bool {
    forge.inspection_holds().iter().any(|(name, _)| match &name.what {
        forge::What::Branch(parts) => parts.iter().map(AsRef::as_ref).eq(branch.split(|byte| *byte == b'/')),
        _ => false,
    })
}

fn ended(world: &World, number: u64) -> bool {
    task(world, number).is_some_and(|row| matches!(row.phase, tasks::Phase::Ended(_)))
}
fn running(world: &World, number: u64) -> Option<engine::Assignment> {
    let tasks::Phase::Active(tasks::Active::Running { attempt }) = task(world, number)?.phase else { return None };
    world.assigned.iter().rev().find(|run| run.task == number && run.attempt == attempt).cloned()
}
fn created(outcome: &people::Outcome) -> Result<u64, String> {
    if let people::Outcome::Created { first, last, .. } = outcome {
        assert_eq!(first, last);
        Ok(*first)
    } else {
        Err(format!("creation: {outcome:?}"))
    }
}
fn delegated(answer: engine::CallAnswer) -> Result<u64, String> {
    if let engine::CallAnswer::Delegated(made) = answer {
        Ok(made.first().expect("one delegate").task)
    } else {
        Err(format!("delegation: {answer:?}"))
    }
}

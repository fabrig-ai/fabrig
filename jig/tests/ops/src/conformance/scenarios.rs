//! Shared scenarios at ops' root, host capability and production boundaries
//! (`jig/docs/design/domain/testing.md`, 5).
use super::{Config, Ops, Peers, Store, Story, core, infra, obs, production, root};
use jig_conformance::scenarios::{Action, Scenario, ScenarioApplication};
use jig_conformance::{
    Application, Clock, Harness, Input,
    referee::{Promise, Violation},
};
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_fake_workers as workers;
use skein_lib::{Duration, List, ReplyTo, Time, Token, Wall};
use std::collections::{BTreeMap, VecDeque};
type World = Harness<Ops>;
type Inputs = Vec<Input<root::Event, root::Record>>;
mod pool;
pub(super) mod reads;

/// Scenario peers retain only their own scripts, requests and observations.
pub(super) struct State {
    kind: Scenario,
    starts: VecDeque<workers::Assignment>,
    workers: BTreeMap<Token, workers::Worker>,
    turns: BTreeMap<Token, u32>,
    late: bool,
    hold: bool,
    pending: Vec<infra::SystemRequest>,
    pub(super) replaying: bool,
    deadline: Option<u64>,
    children: Vec<u64>,
    parts: Vec<core::CallPart>,
    answers: Vec<people::Outcome>,
    grace_until: u64,
    next: Option<core::reads::Cursor>,
    found: Vec<u64>,
    query_deliveries: u32,
    query_checks: u32,
    spent: u64,
    acknowledged: u32,
}
impl State {
    pub(super) fn new(kind: Scenario) -> Self {
        Self {
            kind,
            starts: VecDeque::new(),
            workers: BTreeMap::new(),
            turns: BTreeMap::new(),
            late: false,
            hold: false,
            pending: Vec::new(),
            replaying: false,
            deadline: None,
            children: Vec::new(),
            parts: Vec::new(),
            answers: Vec::new(),
            grace_until: 0,
            next: None,
            found: Vec::new(),
            query_deliveries: 0,
            query_checks: 0,
            spent: 0,
            acknowledged: 0,
        }
    }
    pub(super) fn assigned(&self, owner: Token) -> Option<(u64, u64)> {
        self.workers.get(&owner).and_then(|worker| worker.seen.last()).map(|a| (a.task, a.attempt))
    }
    pub(super) fn answer(&mut self, part: core::CallPart) {
        self.parts.push(part);
    }
    pub(super) fn cold(&mut self) {
        self.starts.clear();
        self.workers.clear();
        self.turns.clear();
    }
    pub(super) fn agent(&mut self, request: host::ToAgent) -> Inputs {
        match request {
            host::ToAgent::Start { owner, start } => {
                let mut assignment = self.starts.pop_front().expect("released host assignment");
                assignment.transcript = start.turns;
                let mut script = vec![workers::Script::Wait];
                if self.kind == Scenario::RunBudget {
                    // The host reports aggregate session and sub-agent charges.
                    for cost in [2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1] {
                        script.push(workers::Script::Turn { body: Box::new([]), cost, read: None });
                    }
                } else if assignment.transcript.is_empty() {
                    script.extend([
                        workers::Script::Turn { body: Box::new([]), cost: 1, read: None },
                        workers::Script::Wait,
                    ]);
                }
                script.push(workers::Script::Finish { report: Box::new([]) });
                let mut worker = workers::Worker::new(0, 1, vec![script.into_boxed_slice()]);
                worker.assign(assignment);
                self.workers.insert(owner, worker);
                self.turns.insert(owner, 0);
                vec![Input::Event(root::Event::Agent(host::FromAgent::Started { owner, agent: owner }))]
            }
            host::ToAgent::Message { agent, name, words, .. } => {
                if let Some(worker) = self.workers.get_mut(&agent) {
                    let assignment = worker.seen.last().expect("hosted assignment");
                    worker.message(assignment.task, assignment.attempt, name.raw(), words);
                }
                Vec::new()
            }
            host::ToAgent::Acknowledge { agent, turn } => {
                if let Some(worker) = self.workers.get_mut(&agent) {
                    let assignment = worker.seen.last().expect("hosted assignment");
                    worker.acknowledge_turn(assignment.task, assignment.attempt, turn);
                }
                Vec::new()
            }
            host::ToAgent::Cancel { agent } => {
                if let Some(worker) = self.workers.remove(&agent) {
                    self.spent += worker.actual_spent;
                    self.acknowledged += worker.turn_acks;
                }
                vec![Input::Event(root::Event::Agent(host::FromAgent::Gone { owner: agent, detail: Box::new([]) }))]
            }
            host::ToAgent::Grant { .. } | host::ToAgent::Answer { .. } => Vec::new(),
        }
    }
    pub(super) fn poll(&mut self, clock: Clock) -> Inputs {
        let mut events = Vec::new();
        for (&owner, worker) in &mut self.workers {
            for up in worker.tick(Time::from_nanos(clock.now)) {
                let report = match up {
                    workers::Up::Turn { turn, spent, read, opens_window, .. } => {
                        self.turns.insert(owner, turn);
                        host::FromAgent::Turn {
                            owner,
                            turn: host::Turn {
                                turn,
                                spent,
                                read: read.map(Token::new),
                                body: transcript(turn, spent),
                                opens_window,
                                refused: Box::new([]),
                            },
                        }
                    }
                    workers::Up::Answer { spent, read, end, .. } => host::FromAgent::Finished {
                        owner,
                        turns: self.turns[&owner],
                        spent,
                        read: read.map(Token::new),
                        finish: match end {
                            workers::End::Finished(_) => host::Finish::Ended {
                                outcome: super::scripts::result(
                                    smith_charter::Form::Report,
                                    None,
                                    b"shared work finished",
                                    &[],
                                ),
                            },
                            workers::End::Parked => host::Finish::Parked,
                            workers::End::Failed(_) => host::Finish::Failed { failure: host::RunFailure::Cancelled },
                        },
                    },
                    workers::Up::Hello { .. }
                    | workers::Up::Lost { .. }
                    | workers::Up::Writing { .. }
                    | workers::Up::Text { .. }
                    | workers::Up::Call { .. } => unreachable!("inline turn and result script"),
                };
                events.push(Input::Event(root::Event::Agent(report)));
            }
        }
        events
    }
}

fn transcript(turn: u32, spent: u64) -> Box<[u8]> {
    use smith_transcript::v6 as t;
    let limits = t::CEILINGS;
    let text = t::Text::new(&limits, t::TextParts { text: b"shared scenario turn".as_slice().into(), replay: None })
        .expect("text");
    let message = t::Message::new(
        &limits,
        t::MessageParts { role: t::Role::Assistant, blocks: List::from_boxed(Box::new([t::Block::Text(text)])) },
    )
    .expect("message");
    let value = t::Turn::new(
        &limits,
        t::TurnParts {
            model: b"model".as_slice().into(),
            effort: None,
            kind: t::TurnKind::Ordinary,
            opening: None,
            endpoint: b"fake".as_slice().into(),
            dialect: jig_client_test_domain::DIALECT,
            place: turn,
            usage: t::Usage::new(
                &limits,
                t::UsageParts { input: Some(1), output: Some(1), cache_read: None, cache_write: None, reasoning: None },
            )
            .expect("usage"),
            spent,
            jobs: 0,
            messages: List::from_boxed(Box::new([message])),
        },
    )
    .expect("turn");
    let mut writer = skein_lib::Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut writer).expect("measured turn");
    writer.finish()
}

fn state(world: &World) -> &State {
    world.peers.scenario.as_ref().expect("shared scenario")
}
fn state_mut(world: &mut World) -> &mut State {
    world.peers.scenario.as_mut().expect("shared scenario")
}
fn check(holds: bool, why: &str) -> Result<(), Violation> {
    if holds { Ok(()) } else { Err(Violation { promise: Promise::Lost, why: why.into() }) }
}
fn parent(world: &World) -> (u64, u64) {
    world.peers.assignments[0]
}
fn row(world: &World, task: u64) -> &tasks::TaskRecord {
    [tasks::Key::Live(task), tasks::Key::Ended(task)]
        .into_iter()
        .find_map(|key| match world.store.rows.get(&root::Key::Core(core::Key::Tasks(key))) {
            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))) => {
                Some(row)
            }
            _ => None,
        })
        .expect("durable task")
}
fn ended(world: &World, task: u64) -> bool {
    matches!(row(world, task).phase, tasks::Phase::Ended(_))
}
fn ask(world: &mut World, key: u8, ask: people::Ask) {
    world.send(root::Event::Core(core::Event::People(people::Event::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(6000 + u64::from(key))),
        sign_in: world.peers.sign_in.expect("on-call signed in"),
        key: [key; 16],
        ask,
    })));
}
fn message(world: &mut World, task: u64) {
    let state = state_mut(world);
    let worker = state
        .workers
        .values_mut()
        .find(|worker| worker.seen.iter().any(|assignment| assignment.task == task))
        .expect("independent host");
    let a = worker.seen.last().expect("assignment");
    worker.message(a.task, a.attempt, 100, b"continue".as_slice().into());
}
fn finish(world: &mut World, task: u64) -> Result<(), Violation> {
    if !ended(world, task) && !state(world).workers.values().any(|worker| worker.seen.iter().any(|a| a.task == task)) {
        world.advance(6_000_000_000);
        world.drain()?;
        world.advance(1_000_000_000);
        world.drain()?;
    }
    for _ in 0..2 {
        if ended(world, task) {
            break;
        }
        message(world, task);
        world.drain()?;
    }
    check(ended(world, task), "scripted work did not end")
}
fn chat(world: &mut World, key: u8, words: &[u8]) {
    ask(world, key, super::config::chat(words));
}
fn effect(world: &mut World, completion: u32, proposal: bool) {
    let (task, attempt) = parent(world);
    let service = infra::Service::new(b"staging".as_slice().into(), b"checkout".as_slice().into());
    let effect = if matches!(state(world).kind, Scenario::OtherHand | Scenario::ChangedJudge) {
        infra::Effect::Scale { service, from: 3, to: 1 }
    } else {
        infra::Effect::Restart { service, operation: 1 }
    };
    world.send(root::Event::Effect {
        to: ReplyTo::new(Token::new(7000 + u64::from(completion))),
        key: core::CallKey { task, attempt, completion, position: 0 },
        effect,
        deadline: Wall::from_nanos(world.clock().now + 1_000_000_000),
        proposal: proposal.then(|| b"on-call decides".as_slice().into()),
    });
}
fn delegate(world: &mut World, count: usize) -> Result<(), Violation> {
    let (task, attempt) = parent(world);
    let mut authority = row(world, task).authority.clone();
    authority.budget.spend = 10;
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let batch = (0..count)
        .map(|_| core::Delegate {
            deadline: tasks::Deadline::Named,
            label: 0,
            flags: tasks::Flags::PLAIN,
            tracked: None,
            budget: people::Budget::Named,
            executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec {
                job: None,
                asks: false,
                words: b"shared delegate".as_slice().into(),
                parameters: Box::new([]),
                inputs: Box::new([]),
            },
            contract: tasks::Contract::Report { words: 128 },
            authority: authority.clone(),
            llm: None,
            symbolic_grants: Box::new([]),
            dependencies: Box::new([]),
            wake: Some(tasks::WakePolicy::DEFAULT),
        })
        .collect();
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: ReplyTo::new(Token::new(8120)),
        key: core::CallKey { task, attempt, completion: 1, position: 0 },
        batch,
        stubs: Box::new([]),
    }));
    world.drain()?;
    let children = match state(world).parts.last() {
        Some(core::CallPart::Delegated(members)) => members.iter().map(|member| member.task).collect(),
        part => panic!("delegates answered: {part:?}"),
    };
    state_mut(world).children = children;
    Ok(())
}

impl ScenarioApplication for Ops {
    fn configuration(scenario: Scenario, seed: u64) -> Option<Config> {
        let mut config =
            Config::new(seed, if scenario == Scenario::ShrinkingPool { Story::Environment } else { Story::Night });
        config.scenario = Some(scenario);
        config.limits.core.fleet.engine_slots = 4;
        config.limits.host.slots = 4;
        config.limits.infrastructure.retry_after_seconds = 1;
        Some(config)
    }
    #[expect(clippy::too_many_lines, reason = "each shared action translates to ops' outside vocabulary")]
    fn action(world: &mut World, action: Action) -> Result<(), Violation> {
        match action {
            Action::Ready => {
                if state(world).kind == Scenario::ShrinkingPool {
                    return Ok(());
                }
                chat(world, 1, b"shared caller");
                world.drain()?;
                // Staging's observed requirement needs thirty minutes of low load.
                world.advance(1_800_000_000_000);
                world.drain()?;
                let event = world.systems.observability(obs::SystemRequest::Facts {
                    service: obs::Service::new(b"staging".as_slice().into(), b"checkout".as_slice().into()),
                });
                super::observations::fact(&mut world.peers.observed, &event);
                world.send(root::Event::Observability(obs::Event::System(event)));
                world.drain()?;
                check(!world.peers.assignments.is_empty(), "caller not assigned")?;
                if state(world).kind == Scenario::Compound {
                    delegate(world, 2)?;
                }
                if state(world).kind == Scenario::RacingDecisions {
                    delegate(world, 1)?;
                }
            }
            Action::Late => state_mut(world).late = true,
            Action::Effect(completion) => effect(world, completion, false),
            Action::DeliverLate | Action::ReleaseWrites => {
                state_mut(world).hold = false;
                let clock = world.clock();
                state_mut(world).replaying = true;
                for call in std::mem::take(&mut state_mut(world).pending) {
                    for input in Ops::deliver(
                        &mut world.peers,
                        &mut world.systems,
                        &mut world.store,
                        clock,
                        super::Delivery::Root(Box::new(root::Output::Infrastructure(call))),
                    ) {
                        if let Input::Event(event) = input {
                            world.send(event);
                        }
                    }
                }
                state_mut(world).replaying = false;
            }
            Action::HoldWrites => state_mut(world).hold = true,
            Action::OtherHand => world.systems.production.other_hand(production::Fault::HandScale {
                service: production::ServiceName::new("staging", "checkout"),
                replicas: 13,
            }),
            Action::CheckDeadline => {
                let deadline = world
                    .store
                    .rows
                    .values()
                    .find_map(|row| match row {
                        root::Record::Infrastructure(infra::Record::Outbox(entry)) => Some(entry.deadline),
                        root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => {
                            None
                        }
                    })
                    .expect("uncertain outbox");
                if let Some(saved) = state(world).deadline {
                    check(deadline == saved, "restart moved the uncertain deadline")?;
                }
                state_mut(world).deadline = Some(deadline);
            }
            Action::ChangeJudge => {
                world.systems.production.inject_incident(&production::ServiceName::new("staging", "checkout"));
                let event = world.systems.observability(obs::SystemRequest::Facts {
                    service: obs::Service::new(b"staging".as_slice().into(), b"checkout".as_slice().into()),
                });
                super::observations::fact(&mut world.peers.observed, &event);
                world.send(root::Event::Observability(obs::Event::System(event)));
            }
            Action::Propose => effect(world, 2, true),
            Action::NarrowPolicy => ask(
                world,
                43,
                people::Ask::ChangePolicy {
                    project: 1,
                    change: people::PolicyChange::Requirements {
                        requirements: Box::new([people::Requirement {
                            connector: 2,
                            kind: 1,
                            pattern: people::Pattern { segments: Box::new([]), last: people::Last::Open(Box::new([])) },
                            judge: people::Judge { connector: 1, requirement: 2, parameters: 0 },
                            guard: people::Guard::Guarded,
                            must_be_guarded: true,
                        }]),
                    },
                },
            ),
            Action::AcceptProposal => {
                let proposal = world
                    .store
                    .rows
                    .values()
                    .find_map(|record| match record {
                        root::Record::Infrastructure(infra::Record::Proposal { number, .. }) => Some(*number),
                        root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => {
                            None
                        }
                    })
                    .expect("waiting effect proposal");
                ask(
                    world,
                    44,
                    people::Ask::DecideProposal {
                        project: 1,
                        proposer: people::Proposer::Task(parent(world).0),
                        proposal,
                        decision: people::ProposalDecision::Accept,
                        running: None,
                    },
                );
            }
            Action::Compound | Action::CompoundAgain => {
                let children = state(world).children.clone();
                let operations = List::from_boxed(Box::new([
                    people::Operation::Move { task: children[0], reason: b"continue".as_slice().into() },
                    people::Operation::Cancel { task: children[1], reason: b"finish".as_slice().into(), grace: None },
                    people::Operation::Cancel {
                        task: parent(world).0,
                        reason: b"finish".as_slice().into(),
                        grace: Some(Duration::from_secs(2)),
                    },
                ]));
                ask(world, 120, people::Ask::Compound { project: 1, operations });
            }
            Action::CancelWithGrace => {
                state_mut(world).grace_until = world.clock().now + 2_000_000_000;
                ask(
                    world,
                    120,
                    people::Ask::Cancel {
                        project: 1,
                        task: parent(world).0,
                        reason: b"finish".as_slice().into(),
                        grace: Some(Duration::from_secs(2)),
                    },
                );
            }
            Action::Finish => {
                // Both cold starts discard the inline agent. Its task retains
                // the grace and is cancelled once that deadline arrives.
                check(world.clock().now < state(world).grace_until, "grace ended during restarts")?;
                check(row(world, parent(world).0).grace.is_some(), "restart lost grace")?;
            }
            Action::Resume => {
                message(world, parent(world).0);
            }
            Action::PartyAmend => {
                let target = state(world).children[0];
                ask(
                    world,
                    42,
                    people::Ask::Amend {
                        project: 1,
                        task: target,
                        revision: 0,
                        release: None,
                        amendment: people::Amendment {
                            spec: Some(people::Spec {
                                asks: false,
                                words: b"party choice".as_slice().into(),
                                parameters: Box::new([]),
                                inputs: Box::new([]),
                            }),
                            ..jig_test_profile::fixtures::person_unchanged(b"party evidence")
                        },
                    },
                );
            }
            Action::RunAmend => {
                let (task, attempt) = parent(world);
                world.send(root::Event::Core(core::Event::NamedAction {
                    to: ReplyTo::new(Token::new(8002)),
                    key: core::CallKey { task, attempt, completion: 2, position: 0 },
                    action: core::NamedAction::Amend {
                        target: state(world).children[0],
                        revision: 0,
                        amendment: tasks::Amendment {
                            tracked: None,
                            addressed: None,
                            release: None,
                            instructions: None,
                            wake: None,
                            dependencies: None,
                            authority: None,
                            llm: None,
                            reason: b"run evidence".as_slice().into(),
                            spec: Some(tasks::Spec {
                                job: None,
                                asks: false,
                                words: b"run choice".as_slice().into(),
                                parameters: Box::new([]),
                                inputs: Box::new([]),
                            }),
                        },
                    },
                }));
            }
            Action::Conversation { forgettable } => {
                let mut member = jig_test_profile::fixtures::conversation_member(forgettable);
                member.words =
                    if forgettable { b"forgettable conversation".as_slice() } else { b"kept conversation".as_slice() }
                        .into();
                member.authority.spend = 10;
                member.flags.listed = forgettable;
                member.authority.delegation = people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
                ask(
                    world,
                    if forgettable { 101 } else { 100 },
                    people::Ask::Create { project: 1, batch: Box::new([member]) },
                );
            }
            Action::Turn => {
                let children: Vec<_> = world
                    .store
                    .rows
                    .values()
                    .filter_map(|row| match row {
                        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))
                            if row.spec.words.ends_with(b"conversation") =>
                        {
                            Some(row.number)
                        }
                        root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => {
                            None
                        }
                    })
                    .collect();
                state_mut(world).children.clone_from(&children);
                for task in children {
                    message(world, task);
                }
            }
            Action::MarkTurns => {
                for (index, task) in state(world).children.clone().into_iter().enumerate() {
                    ask(
                        world,
                        102 + u8::try_from(index).expect("two chats"),
                        people::Ask::MarkSeen { seen: people::Seen::Conversation { task, through: 1 } },
                    );
                }
            }
            Action::EndConversations => {
                for task in state(world).children.clone() {
                    finish(world, task)?;
                }
            }
            Action::MarkResults => {
                let through = state(world)
                    .children
                    .iter()
                    .filter_map(|&task| {
                        world
                            .store
                            .rows
                            .get(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(task))))
                            .map(|_| row(world, task).result_position)
                    })
                    .max()
                    .expect("ended result");
                ask(world, 104, people::Ask::MarkSeen { seen: people::Seen::Results { through } });
            }
            Action::WatchQuery | Action::EndTask | Action::ReadEnded => reads::action(world, action)?,
            Action::StartStanding => standing(world),
            Action::Period(period) => {
                let task = world
                    .store
                    .rows
                    .values()
                    .find_map(|row| match row {
                        root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))
                            if row.spec.words.as_ref() == b"standing batch" =>
                        {
                            Some(row.number)
                        }
                        root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => {
                            None
                        }
                    })
                    .expect("standing batch");
                finish(world, task)?;
                for _ in 0..2 {
                    world.send(root::Event::Core(core::Event::Period { project: 1, period, budget: 1000, next: None }));
                    world.drain()?;
                }
            }
            Action::FillPool => pool::fill(world)?,
            Action::ShrinkPool => pool::quota(world, 1),
            Action::FinishHolder(index) => pool::finish(world, index)?,
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "each shared scenario checks its distinct ending at the outside boundary"
    )]
    fn check(world: &World, scenario: Scenario) -> Result<(), Violation> {
        let effects = world.systems.production.observed();
        let applied = effects
            .iter()
            .filter(|effect| match effect {
                production::ObservedEffect::Restart { applied, .. }
                | production::ObservedEffect::Scale { applied, .. } => *applied,
                production::ObservedEffect::Silence { .. }
                | production::ObservedEffect::Rollback { .. }
                | production::ObservedEffect::Create { .. }
                | production::ObservedEffect::TearDown { .. } => false,
            })
            .count();
        match scenario {
            Scenario::LateCopy => {
                check(applied == 1 && effects.len() == 2, "late and retried copies did not make exactly one effect")
            }
            Scenario::OtherHand => check(
                applied == 0
                    && effects
                        .iter()
                        .any(|effect| matches!(effect, production::ObservedEffect::Scale { applied: false, .. })),
                "conditional retry overwrote another hand",
            ),
            Scenario::UncertainRestarts => check(applied == 1, "uncertain entry did not settle after cold starts"),
            Scenario::ChangedJudge => check(
                applied == 1
                    && state(world).parts.iter().any(|part| matches!(part, core::CallPart::EffectDenied { .. })),
                "new effect ignored changed judge",
            ),
            Scenario::NarrowingPolicy => check(
                applied == 1
                    && world
                        .store
                        .rows
                        .values()
                        .any(|row| matches!(row, root::Record::Infrastructure(infra::Record::Proposal { .. }))),
                &format!(
                    "narrowed policy: applied {applied}, parts {:?}, answers {:?}",
                    state(world).parts,
                    state(world).answers
                ),
            ),
            Scenario::Compound => {
                check(
                    row(world, state(world).children[0]).requester == tasks::Party::Person(1),
                    "compound lost moved delegate",
                )?;
                check(
                    ended(world, state(world).children[1]) && ended(world, parent(world).0),
                    "compound lost cancellation",
                )?;
                let answers: Vec<_> = state(world)
                    .answers
                    .iter()
                    .filter(|answer| matches!(answer, people::Outcome::Compound { .. }))
                    .collect();
                check(
                    answers.len() >= 2 && answers.windows(2).all(|pair| pair[0] == pair[1]),
                    "compound retry changed keyed answer",
                )
            }
            Scenario::Grace => check(
                ended(world, parent(world).0)
                    && row(world, parent(world).0).ended_at.is_some_and(|at| at.as_nanos() >= state(world).grace_until),
                "grace cancelled before its retained deadline",
            ),
            Scenario::RunBudget => check(
                state(world).spent + state(world).workers.values().map(|worker| worker.actual_spent).sum::<u64>() == 19
                    && state(world).acknowledged
                        + state(world).workers.values().map(|worker| worker.turn_acks).sum::<u32>()
                        == 12,
                "priced completions did not share one run allowance",
            ),
            Scenario::RacingDecisions => check(
                row(world, state(world).children[0]).revisions.spec == 1
                    && row(world, state(world).children[0]).spec.words.as_ref() == b"party choice"
                    && state(world).parts.iter().any(|part| {
                        matches!(part, core::CallPart::Moved { now: people::Moved::Spec { revision: 1 }, .. })
                    }),
                "racing amendment overwrote the selected revision",
            ),
            Scenario::ConversationEndsWhileRead => {
                let children = &state(world).children;
                check(children.len() == 2, "two conversations created")?;
                check(
                    world.store.rows.contains_key(&root::Key::Core(core::Key::People(people::Key::EndedRead {
                        party: 1,
                        task: children[0],
                    }))) && !world
                        .store
                        .rows
                        .contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(children[1])))),
                    "read markers did not follow kept and forgettable conversations",
                )
            }
            Scenario::QueryWatchedWhileTasksEnd => check(
                state(world).query_checks == 3 && state(world).query_deliveries >= 4,
                "query was not watched and paged through three task ends",
            ),
            Scenario::StandingTask => check(
                world.peers.assignments.len() == 6,
                "standing task did not create exactly one batch per funding period",
            ),
            Scenario::ShrinkingPool => pool::check_ending(world),
        }
    }
}

pub(super) fn delivered(peers: &mut Peers, store: &Store, output: &root::Output) {
    let Some(state) = &mut peers.scenario else { return };
    match output {
        root::Output::ToChild(root::Released::Host(host::Event::Down(host::LinkDown::Assign {
            assignment, ..
        }))) => {
            let task = assignment.assignment.run.raw();
            let attempt = assignment.assignment.attempt.raw();
            let snapshot = super::observations::snapshot(store);
            let proof = store
                .rows
                .values()
                .find_map(|record| match record {
                    root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof))) if proof.task == task => {
                        Some(proof)
                    }
                    root::Record::Core(_) | root::Record::Observability(_) | root::Record::Infrastructure(_) => None,
                })
                .expect("durable run proof");
            state.starts.push_back(workers::Assignment {
                task,
                attempt,
                conversation: proof.conversation,
                budget: snapshot.tasks[&task].run_reserved,
                transcript: assignment.turns.clone(),
                messages: assignment.messages.iter().map(|message| message.name.raw()).collect(),
            });
        }
        root::Output::Core(core::Held::CallAnswer { part, .. }) => state.parts.push(part.clone()),
        root::Output::Core(core::Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. }) => {
            state.answers.push(outcome.clone());
        }
        root::Output::Core(core::Held::Delivery { watcher, .. }) if *watcher == Token::new(7090) => {
            state.query_deliveries += 1;
        }
        root::Output::Core(core::Held::Read { reply, answer: core::reads::Answer::Tasks(page) })
            if *reply == ReplyTo::new(Token::new(7091)) =>
        {
            if let core::reads::EndedGroup::Rows { rows, next, .. } = &page.ended {
                state.found.extend(rows.iter().map(|line| line.chip.number));
                state.next.clone_from(next);
            }
        }
        root::Output::ProjectMade { .. }
        | root::Output::ApplicationDocument { .. }
        | root::Output::ApplicationRead { .. }
        | root::Output::IncidentAssociationRefused { .. }
        | root::Output::Application { .. }
        | root::Output::Commit { .. }
        | root::Output::Switch
        | root::Output::Load { .. }
        | root::Output::CoreLoad(_)
        | root::Output::Core(_)
        | root::Output::Now(_)
        | root::Output::Observability(_)
        | root::Output::Infrastructure(_)
        | root::Output::ReadResult { .. }
        | root::Output::Read { .. }
        | root::Output::Agent(_)
        | root::Output::ToChild(_)
        | root::Output::Stop => {}
    }
}

pub(super) fn infrastructure(peers: &mut Peers, call: &infra::SystemRequest) -> Option<Inputs> {
    let state = peers.scenario.as_mut()?;
    if let infra::SystemRequest::Apply { key, attempt, .. } = call {
        if !state.replaying && (state.late || state.hold) {
            peers.observed.push(jig_conformance::referee::Observed::Released {
                requires: vec![jig_conformance::referee::Receipt::Outbox(2, super::observations::key(*key), *attempt)],
            });
        }
        if state.late {
            state.late = false;
            state.pending.push(call.clone());
            return Some(vec![Input::Event(root::Event::Infrastructure(infra::Event::System(
                infra::SystemEvent::Applied { key: *key, attempt: *attempt, result: infra::ApplyResult::Uncertain },
            )))]);
        }
        if state.hold {
            state.pending.push(call.clone());
            return Some(Vec::new());
        }
    }
    None
}

fn standing(world: &mut World) {
    let mut authority = super::config::task_authority(10);
    authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::StartRecurring {
        project: 1,
        authority: super::config::task_authority(100),
        template: tasks::RecurringTemplate {
            key: 1,
            overlap: tasks::RecurringOverlap::Skip,
            batch: Box::new([tasks::New {
                deadline: tasks::Deadline::Named,
                label: 0,
                flags: tasks::Flags::PLAIN,
                number: 1,
                project: 1,
                executor: tasks::Executor::Agent { charter: 1 },
                llm: None,
                spec: tasks::Spec {
                    job: None,
                    asks: false,
                    words: b"standing batch".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                authority,
                numbers: tasks::Numbers { budget: 10, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 0 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            }]),
        },
    }));
}

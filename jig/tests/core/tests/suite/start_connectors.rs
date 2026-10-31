//! Direct core starts and connector recovery (jig/docs/design/domain/engine.md, 6).
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_core::{
    self as core,
    start::{Action, Phase, SweepInput},
};
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use jig_test_system::{Fault, System};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use std::collections::{BTreeMap, VecDeque};

const SEED: u64 = 714;
const ENTRY: u64 = 1;

struct Harness {
    core: core::Core,
    env: Env<core::Limits>,
    connector: connector::Domain,
    connector_env: Env<connector::Limits>,
    system: System,
    records: BTreeMap<connector::RecordKey, connector::Record>,
    task: u64,
    resource: tasks::Name,
    outcome: Option<core::effects::connector::OutboxOutcome>,
    complete: bool,
    wires: bool,
}

fn phase(action: &Action, expected: Phase) {
    assert!(matches!(action, Action::Step(found) if *found == expected));
}

fn task_name(path: &connector::Path) -> tasks::Name {
    tasks::Name { connector: 1, path: path.segments().into() }
}

impl Harness {
    #[expect(
        clippy::too_many_lines,
        reason = "one fixture loads a sampled core store and hand-built connector records"
    )]
    fn new(recovery: u16) -> Self {
        let (configuration, limits) = jig_core_world::world::fixture(SEED, false);
        let mut source = World::new(Setup { seed: SEED, ..Setup::default() });
        let task = source.assigned.as_ref().expect("source run").task;
        source.end(tasks::End::Parked, 0);
        let path = jig_test_connector_world::path(1, 1);
        let resource = task_name(&path);
        let env = Env {
            now: Time::ZERO,
            wall: Wall::from_nanos(if recovery == 3 { 1_000 } else { 10 }),
            limits: limits.core,
        };
        let connector_env = env.with(limits.connector);
        let mut core = core::Core::new(configuration.core, &limits.core).expect("configured direct core");
        let mut configured = Queue::uncapped();
        tasks::step(
            &mut core.tasks,
            &env.with(limits.core.tasks),
            tasks::Event::Kinds {
                connector: 1,
                kinds: Box::new([tasks::Kind {
                    connector: 1,
                    kind: 1,
                    hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits },
                }]),
            },
            &mut configured,
        );
        assert!(configured.is_empty());
        {
            phase(&core.start_begin(false, env.wall), Phase::LoadCore);
        }
        let mut rows: Vec<_> = source
            .store
            .rows
            .values()
            .filter_map(|row| match row {
                root::Record::Core(row) => Some(row.clone()),
                root::Record::Connector { .. } => None,
            })
            .collect();
        for row in &mut rows {
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "only the durable effect counter and holder are changed in the sampled core store"
            )]
            match row {
                core::Record::Core(core::CoreRecord::Deployment(header)) => header.connector_rows = ENTRY,
                core::Record::Tasks(tasks::Stored::Live(row)) if row.number == task => {
                    row.holdings = Box::new([tasks::Holding::Write { resource: resource.clone(), kind: 1 }]);
                    row.holds_taken = true;
                }
                _ => {}
            }
        }
        // The settings range precedes every child row, as it does in the store's live walk.
        for row in &rows {
            if let core::Record::Core(row @ (core::CoreRecord::Deployment(_) | core::CoreRecord::Settings(_))) = row {
                assert!(!matches!(core.restore(&env, core::Record::Core(row.clone())), core::Restore::Refused));
            }
        }
        {}
        rows.sort_by_key(|row| match row {
            core::Record::Accounts(_) => 0,
            core::Record::People(_) => 1,
            core::Record::Tasks(_) => 2,
            core::Record::Core(_) => 3,
            core::Record::Notes(_) => 4,
        });
        for row in rows {
            match &row {
                core::Record::Core(core::CoreRecord::Deployment(_) | core::CoreRecord::Settings(_))
                | core::Record::Notes(_) => continue,
                core::Record::Tasks(row) if !row.key().live() => continue,
                core::Record::Core(_)
                | core::Record::Tasks(_)
                | core::Record::People(_)
                | core::Record::Accounts(_) => {}
            }
            assert_ne!(core.restore(&env, row), core::Restore::Refused);
        }
        let pool = tasks::Stored::Pool(tasks::PoolSlots {
            number: 1,
            pool: task_name(&jig_test_connector_world::path(1, 2)),
            slots: 0,
        });
        assert_eq!(core.restore(&env, core::Record::Tasks(pool)), core::Restore::Kept);
        let writer = tasks::Stored::Writer(tasks::WriterSlot {
            number: 1,
            resource: resource.clone(),
            writer: tasks::Writer::Effect { entry: ENTRY },
            lost: false,
        });
        assert_eq!(core.restore(&env, core::Record::Tasks(writer)), core::Restore::Kept);
        let kind = configuration
            .first
            .kinds
            .iter()
            .find(|kind| kind.kind == jig_test_connector_world::kind(1, recovery))
            .expect("recovery kind");
        let entry = connector::OutboxEntry {
            number: ENTRY,
            task,
            key: connector::Key {
                deployment: configuration.first.deployment,
                task,
                purpose: 7,
                origin: jig_test_connector::Purpose::Call { attempt: 1, completion: 1, position: 0 },
            },
            effect: connector::Effect {
                kind: kind.kind,
                resources: Box::new([path.clone()]),
                purpose: 7,
                condition: if kind.recovery == connector::Recovery::Conditional { Some(10) } else { None },
                target: 11,
                state: 10,
            },
            attempt: Some(connector::Attempt { number: 1, sent: Wall::EPOCH, deadline: Wall::from_nanos(1_000) }),
            phase: connector::EffectPhase::Sent,
        };
        let mut system = System::new();
        if kind.form != connector::Form::Creation {
            system.other_hand(&path, 10);
        }
        // The old attempt reached its system but its outcome never reached the durable store.
        let lost = system.answer(
            connector::SystemRequest::Apply {
                entry: ENTRY,
                attempt: 1,
                key: entry.key,
                effect: entry.effect.clone(),
                form: kind.form,
                recovery: kind.recovery,
            },
            Fault::AfterApply,
        );
        assert!(matches!(lost, connector::SystemEvent::Applied { result: connector::ApplyResult::Uncertain, .. }));
        let records = [
            connector::Record::Adoption { project: 1, path: path.clone(), role: connector::ResourceRole::Owned },
            connector::Record::Task { task, project: 1, resources: Box::new([path]) },
            connector::Record::Outbox(entry),
            connector::Record::Adoption {
                project: 1,
                path: jig_test_connector_world::path(1, 2),
                role: connector::ResourceRole::Owned,
            },
        ]
        .into_iter()
        .map(|record| (record.key(), record))
        .collect();
        Self {
            core,
            env,
            connector: connector::Domain::new(configuration.first, &limits.connector),
            connector_env,
            system,
            records,
            task,
            resource,
            outcome: None,
            complete: false,
            wires: false,
        }
    }

    fn route_core(&mut self, event: core::Event) {
        let mut out = core::step(&mut self.core, &self.env, event);
        self.outputs(&mut out);
    }

    fn outputs(&self, out: &mut Queue<core::Request>) {
        for _ in 0..out.len() {
            match out.pop().expect("core output") {
                core::Request::Write(_) => {}
                core::Request::Now(now) => {
                    assert!(self.wires, "no outward output before durable Wires");
                    assert!(matches!(*now, core::Now::Account(_)));
                }
                core::Request::Ask { .. } | core::Request::Held(_) | core::Request::ProjectMade { .. } => {
                    panic!("idle restored task needs no connector ask or external delivery")
                }
            }
        }
    }

    fn connector_event(&mut self, event: connector::Event) {
        let mut pending = VecDeque::from([event]);
        for _ in 0..32 {
            let Some(event) = pending.pop_front() else { break };
            let mut out = Queue::uncapped();
            connector::step(&mut self.connector, &self.connector_env, event, &mut out);
            for _ in 0..out.len() {
                #[expect(
                    clippy::wildcard_enum_match_arm,
                    reason = "the direct start fixture exercises only restore, fresh read and outbox outputs"
                )]
                match out.pop().expect("connector output") {
                    connector::Request::Resource { project, resource } => {
                        let role = match resource.role {
                            connector::ResourceRole::Owned => core::effects::connector::ResourceRole::Owned,
                            connector::ResourceRole::Participating => {
                                core::effects::connector::ResourceRole::Participant
                            }
                            connector::ResourceRole::Context => core::effects::connector::ResourceRole::Context,
                            connector::ResourceRole::Unavailable => core::effects::connector::ResourceRole::Unavailable,
                        };
                        let hold = match resource.hold {
                            connector::Hold::None => core::effects::connector::HoldKind::Shared,
                            connector::Hold::Exclusive { mode } => core::effects::connector::HoldKind::Exclusive {
                                wait: mode == connector::HoldMode::Wait,
                            },
                            connector::Hold::Pooled { mode } => core::effects::connector::HoldKind::Pooled {
                                slots: self.connector.slots(&resource.path).expect("configured pool"),
                                wait: mode == connector::HoldMode::Wait,
                            },
                        };
                        self.route_core(core::Event::EffectConnector(core::effects::connector::Event::Resource {
                            project,
                            name: task_name(&resource.path),
                            role,
                            hold,
                        }));
                    }
                    connector::Request::Save { record } => {
                        self.records.insert(record.key(), record);
                    }
                    connector::Request::Erase { key } => {
                        self.records.remove(&key);
                    }
                    connector::Request::System(request) => {
                        assert!(self.wires, "connectors reach their systems only after Wires resumes");
                        pending.push_back(connector::Event::System(self.system.answer(request, Fault::None)));
                    }
                    connector::Request::Outcome { entry, outcome } => {
                        let outcome = match outcome {
                            connector::Outcome::Made { .. } => core::effects::connector::OutboxOutcome::Made,
                            connector::Outcome::Held => core::effects::connector::OutboxOutcome::Held { entry },
                            connector::Outcome::Failed => core::effects::connector::OutboxOutcome::Failed,
                            connector::Outcome::Uncertain => core::effects::connector::OutboxOutcome::Uncertain,
                            connector::Outcome::Withdrawn => core::effects::connector::OutboxOutcome::Withdrawn,
                            connector::Outcome::Skipped => core::effects::connector::OutboxOutcome::NotMade,
                        };
                        self.outcome = Some(outcome);
                        self.route_core(core::Event::EffectConnector(core::effects::connector::Event::Outbox {
                            connector: 1,
                            entry,
                            task: self.task,
                            outcome,
                        }));
                    }
                    connector::Request::RestartDone => self.complete = true,
                    connector::Request::Changed { .. } => {}
                    _ => panic!("unexpected connector restart output"),
                }
            }
        }
        assert!(pending.is_empty());
    }

    fn restore_connector(&mut self) {
        for record in self.records.values().cloned().collect::<Vec<_>>() {
            self.connector_event(connector::Event::Restore { record });
        }
        assert_eq!(self.system.calls(), 1, "restoration made no system call");
        assert_eq!(self.core.tasks.holder(&self.resource), Some(self.task));
        assert_eq!(
            self.core.tasks.writer(&self.resource).expect("restored writer").writer,
            tasks::Writer::Effect { entry: ENTRY }
        );
    }

    fn read_afresh(&mut self) {
        assert!(self.wires);
        self.connector_event(connector::Event::Restart(connector::RestartStep::FreshRead {
            resource: jig_test_connector_world::path(1, 1),
        }));
        self.route_core(core::Event::Tasks(tasks::Event::ReadAfresh { resource: self.resource.clone() }));
        assert_eq!(self.system.reads().len(), 1);
        assert!(
            self.core.tasks.writer(&self.resource).is_some(),
            "fresh reads keep effect writers until their outcome"
        );
    }

    fn settle(&mut self) {
        self.connector_event(connector::Event::Restart(connector::RestartStep::SettleOutbox));
        assert!(self.complete, "the root advances only after the connector reports settled");
        // The core ignores completion; its root translates it to the matching phase completion.
        self.route_core(core::Event::EffectConnector(core::effects::connector::Event::RestartDone {
            stage: core::effects::connector::RestartStage::Settled,
        }));
    }

    fn started(&mut self) {
        let mut out = Queue::uncapped();
        phase(&self.core.start_done(&self.env, Phase::LoadCore, &mut out), Phase::LoadInbox);
        phase(&self.core.start_done(&self.env, Phase::LoadInbox, &mut out), Phase::RestoreConnector { connector: 1 });
        self.restore_connector();
        phase(
            &self.core.start_done(&self.env, Phase::RestoreConnector { connector: 1 }, &mut out),
            Phase::RestoreConnector { connector: 2 },
        );
        phase(&self.core.start_done(&self.env, Phase::RestoreConnector { connector: 2 }, &mut out), Phase::Link);
        let action = self.core.start_done(&self.env, Phase::Link, &mut out);
        match action {
            Action::Commit(token) => {
                self.outputs(&mut out);
                phase(&self.core.start_resume(&self.env, token, &mut out), Phase::Sweep);
            }
            action @ (Action::Step(_) | Action::Idle | Action::Refused(_)) => phase(&action, Phase::Sweep),
        }
        self.outputs(&mut out);
        phase(&self.core.start_sweep(&self.env, SweepInput::Finished, &mut out), Phase::Switch);
        phase(&self.core.start_done(&self.env, Phase::Switch, &mut out), Phase::Wires);
        let Action::Commit(wires) = self.core.start_done(&self.env, Phase::Wires, &mut out) else {
            panic!("Wires commits")
        };
        self.outputs(&mut out);
        assert_eq!(self.system.calls(), 1);
        self.wires = true;
        phase(&self.core.start_resume(&self.env, wires, &mut out), Phase::AdoptRuns);
        self.outputs(&mut out);
        phase(&self.core.start_done(&self.env, Phase::AdoptRuns, &mut out), Phase::ReadAfresh { connector: 1 });
        self.read_afresh();
        phase(
            &self.core.start_done(&self.env, Phase::ReadAfresh { connector: 1 }, &mut out),
            Phase::ReadAfresh { connector: 2 },
        );
        phase(
            &self.core.start_done(&self.env, Phase::ReadAfresh { connector: 2 }, &mut out),
            Phase::SettleOutbox { connector: 1 },
        );
        self.settle();
        assert_eq!(self.core.start_phase(), Some(Phase::SettleOutbox { connector: 1 }));
        phase(
            &self.core.start_done(&self.env, Phase::SettleOutbox { connector: 1 }, &mut out),
            Phase::SettleOutbox { connector: 2 },
        );
        phase(&self.core.start_done(&self.env, Phase::SettleOutbox { connector: 2 }, &mut out), Phase::Open);
        assert!(matches!(self.core.start_done(&self.env, Phase::Open, &mut out), Action::Idle));
        self.outputs(&mut out);
        assert_eq!(self.core.start_phase(), None);
        assert_eq!(self.core.start_failure(), None);
    }
}

fn recovered(recovery: u16, applications: usize, calls: u64) {
    let mut started = Harness::new(recovery);
    started.started();
    assert_eq!(started.outcome, Some(core::effects::connector::OutboxOutcome::Made));
    assert!(started.records.values().any(|row| matches!(row, connector::Record::Made { state: 11, .. })));
    assert!(!started.records.values().any(|row| matches!(row, connector::Record::Outbox(_))));
    assert_eq!(started.core.tasks.writer(&started.resource), None);
    assert_eq!(started.system.observed().iter().filter(|row| row.applied).count(), applications);
    assert_eq!(started.system.calls(), calls);
}

#[test]
fn uncertain_keyed_effect_and_writer_settle_after_fresh_reads() {
    recovered(1, 1, 3);
}

#[test]
fn uncertain_conditional_effect_is_found_without_another_write() {
    recovered(2, 1, 3);
}

#[test]
fn uncertain_idempotent_effect_repeats_after_its_saved_deadline() {
    recovered(3, 2, 3);
}

#[test]
fn unrecoverable_effect_holds_its_task_and_writer() {
    let mut started = Harness::new(4);
    started.started();
    assert_eq!(started.outcome, Some(core::effects::connector::OutboxOutcome::Held { entry: ENTRY }));
    assert!(started.records.values().any(|row| matches!(
        row,
        connector::Record::Outbox(connector::OutboxEntry { phase: connector::EffectPhase::Held, .. })
    )));
    assert!(started.core.tasks.writer(&started.resource).is_some());
    let row = started.core.tasks.task(started.task).expect("held task");
    assert!(
        matches!(&row.phase, tasks::Phase::Held { why: tasks::Hold::Uncertain { entry: ENTRY }, source: Some(source), .. } if **source == tasks::HoldSource::Effect { connector: 1, entry: ENTRY })
    );
    assert_eq!(started.system.calls(), 2, "an unrecoverable effect is neither looked up nor retried");
    assert_eq!(started.system.observed().iter().filter(|row| row.applied).count(), 1);
}

#[test]
fn restored_access_controls_workspace_writers_during_start() {
    for role in [connector::ResourceRole::Owned, connector::ResourceRole::Context] {
        {
            let mut h = Harness::new(1);
            for record in h.records.values_mut() {
                if let connector::Record::Adoption { role: restored, .. } = record {
                    *restored = role;
                }
            }
            {
                h.started();
            }
            assert_eq!(h.core.tasks.holder(&h.resource), Some(h.task));
            assert_eq!(
                h.core.tasks.resource_hold(&h.resource),
                Some(tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits })
            );
            assert_eq!(h.core.tasks.writer(&h.resource), None);
            let row = h.core.tasks.task(h.task).expect("idle holder");
            let attempt = row.attempt.checked_add(1).expect("next fixture attempt");
            let word = tasks::Word {
                number: row.last_message.checked_add(1).expect("next fixture word"),
                from: row.requester,
                kind: tasks::MessageKind::Words,
                words: b"write".as_slice().into(),
                at: h.env.wall,
                hits: 1,
                eligible: true,
            };
            let env = h.env.with(h.env.limits.tasks);
            tasks::step(
                &mut h.core.tasks,
                &env,
                tasks::Event::Message { reply_to: ReplyTo::new(Token::new(1)), project: 1, task: h.task, word },
                &mut Queue::uncapped(),
            );
            assert_eq!(
                h.core.tasks.task(h.task).expect("woken holder").phase,
                tasks::Phase::Active(tasks::Active::Due)
            );
            tasks::step(
                &mut h.core.tasks,
                &env,
                tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(2)), task: h.task },
                &mut Queue::uncapped(),
            );
            assert_eq!(
                h.core.tasks.task(h.task).expect("prepared holder").phase,
                tasks::Phase::Active(tasks::Active::Preparing)
            );
            let mut out = Queue::uncapped();
            tasks::step(
                &mut h.core.tasks,
                &env,
                tasks::Event::Claim {
                    reply_to: ReplyTo::new(Token::new(3)),
                    task: h.task,
                    attempt,
                    resumed: false,
                    budget: 1,
                    account: None,
                    writes: Box::new([h.resource.clone()]),
                },
                &mut out,
            );
            match role {
                connector::ResourceRole::Owned => {
                    assert_eq!(
                        h.core.tasks.writer(&h.resource).expect("owned workspace writer").writer,
                        tasks::Writer::Run { task: h.task, attempt }
                    );
                }
                connector::ResourceRole::Context => {
                    let Some(tasks::Request::Refused { problem, .. }) = out.pop() else {
                        panic!("context denies a workspace writer")
                    };
                    assert_eq!(problem.why, tasks::Refusal::Holds);
                    assert!(out.is_empty());
                    assert_eq!(h.core.tasks.writer(&h.resource), None);
                }
                connector::ResourceRole::Participating | connector::ResourceRole::Unavailable => {
                    unreachable!("fixture roles")
                }
            }
        }
    }
}

#[test]
fn loading_slot_reports_keep_the_durable_pool_capacity() {
    {
        let mut h = Harness::new(1);
        {
            h.started();
        }
        let pool = task_name(&jig_test_connector_world::path(1, 2));
        assert_eq!(h.connector.slots(&jig_test_connector_world::path(1, 2)), Some(2));
        assert_eq!(h.core.tasks.resource_hold(&pool), Some(tasks::HoldKind::Pooled { taken: tasks::Taken::Waits }));
        let holder = h.core.tasks.task(h.task).expect("sampled task");
        let mut authority = holder.authority.clone();
        authority.budget.spend = 1;
        let row = tasks::New {
            deadline: tasks::Deadline::Named,
            label: holder.label,
            flags: holder.flags,
            number: 2,
            project: holder.project,
            executor: holder.executor,
            llm: None,
            spec: holder.spec.clone(),
            contract: holder.contract.clone(),
            authority,
            numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
            funder: holder.funder,
            dependencies: Box::new([]),
            holdings: Box::new([tasks::Holding::Slot { pool, kind: 1 }]),
            wake: holder.wake,
            recurring: None,
            tracked: None,
        };
        let env = h.env.with(h.env.limits.tasks);
        let mut out = Queue::uncapped();
        tasks::step(
            &mut h.core.tasks,
            &env,
            tasks::Event::Make {
                reply_to: ReplyTo::new(Token::new(4)),
                creator: tasks::Party::Person(1),
                batch: Box::new([row]),
            },
            &mut out,
        );
        let row = h.core.tasks.task(2).expect("admitted pool waiter");
        assert_eq!(row.phase, tasks::Phase::Waiting);
        assert!(!row.holds_taken, "the restored zero-slot capacity survives the loading report of two slots");
    }
}

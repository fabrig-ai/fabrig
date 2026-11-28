//! Engine starts and connector recovery (jig/docs/design/domain/engine/restart.md, 4).
use jig_core_world::world::Setup;
use jig_core_world::world::World;
use jig_domain_contract as jig;
use jig_domain_core::engine::{self, Engine, In, Out, Peer, application, store};
use jig_domain_core::{self as core, start::Phase};
use jig_domain_tasks as tasks;
use jig_test_connector as connector;
use jig_test_domain as root;
use jig_test_system::{Fault, System};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use std::collections::{BTreeMap, VecDeque};

const SEED: u64 = 714;
const ENTRY: u64 = 1;

struct Harness {
    engine: Engine,
    env: Env<core::Limits>,
    connector: connector::Domain,
    connector_env: Env<connector::Limits>,
    system: System,
    records: BTreeMap<connector::RecordKey, connector::Record>,
    core_rows: BTreeMap<core::Key, core::Record>,
    task: u64,
    resource: tasks::Name,
    outcome: Option<jig::effects::OutboxOutcome>,
    arrivals: Vec<(u64, u32, u64, u32)>,
    complete: bool,
    catch_up: Vec<u16>,
    wires: bool,
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
        let (mut configuration, limits) = jig_core_world::world::fixture(SEED, false);
        let mut source = World::new(Setup { seed: SEED, ..Setup::default() });
        let task = source.assigned.as_ref().expect("source run").task;
        source.park_for_message(0);
        let path = jig_test_connector_kit::path(1, 1);
        let resource = task_name(&path);
        let env = Env {
            now: Time::ZERO,
            wall: Wall::from_nanos(if recovery == 3 { 1_000 } else { 10 }),
            limits: limits.core,
        };
        let connector_env = env.with(limits.connector);
        configuration.resource_kinds[0] = Box::new([tasks::ResourceKind {
            kind: 1,
            hold: tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits },
        }]);
        let declarations = configuration.declarations();
        let engine = Engine::new(configuration.core, &declarations, &limits.core).expect("configured engine");
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
            match row {
                core::Record::Core(core::CoreRecord::Deployment(header)) => header.connector_rows = ENTRY,
                core::Record::Tasks(tasks::Stored::Live(row)) if row.number == task => {
                    row.holdings = Box::new([tasks::Holding::Write { resource: resource.clone(), kind: 1 }]);
                    row.holds_taken = true;
                }
                _ => {}
            }
        }
        rows.push(core::Record::Tasks(tasks::Stored::Pool(tasks::PoolSlots {
            number: 1,
            pool: task_name(&jig_test_connector_kit::path(1, 2)),
            slots: 0,
        })));
        rows.push(core::Record::Tasks(tasks::Stored::Writer(tasks::WriterSlot {
            number: 1,
            resource: resource.clone(),
            writer: tasks::Writer::Effect { entry: ENTRY },
            lost: false,
        })));
        let core_rows = rows.into_iter().map(|row| (row.key(), row)).collect();
        let kind = configuration
            .first
            .kinds
            .iter()
            .find(|kind| kind.kind == jig_test_connector_kit::kind(1, recovery))
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
                path: jig_test_connector_kit::path(1, 2),
                role: connector::ResourceRole::Owned,
            },
        ]
        .into_iter()
        .map(|record| (record.key(), record))
        .collect();
        Self {
            engine,
            env,
            connector: connector::Domain::new(configuration.first, &limits.connector),
            connector_env,
            system,
            records,
            core_rows,
            task,
            resource,
            outcome: None,
            arrivals: Vec::new(),
            complete: false,
            catch_up: Vec::new(),
            wires: false,
        }
    }

    fn connector_up(&mut self, up: jig::Up) {
        let mut out = Queue::uncapped();
        engine::step(&mut self.engine, &self.env, In::Connector(jig::Connector(1), up), &mut out);
        while let Some(output) = out.pop() {
            self.engine_output(output);
        }
    }

    fn engine_output(&mut self, output: Out) {
        match output {
            Out::Write(_)
            | Out::Core(core::Request::Ask {
                ask: core::Ask::Catalogue { .. } | core::Ask::Workers(_) | core::Ask::Providers(_),
                ..
            }) => {}
            Out::Core(core::Request::Now(now)) => {
                assert!(self.wires, "no outward output before durable Wires");
                assert!(matches!(*now, core::Now::Account(_)));
            }
            Out::Core(core::Request::Held(held)) => {
                let core::Held::NeedArrived { party, project, task, needs } = *held else {
                    panic!("idle restored task needs no other external delivery")
                };
                assert!(self.wires, "a recovery decision follows durable Wires");
                self.arrivals.push((party, project, task, needs));
            }
            Out::Ask(jig::Connector(connector), jig::Ask::Lifecycle(jig::lifecycle::Ask::CatchUp)) => {
                assert!(self.engine.core().start_opened() && self.engine.core().ready());
                self.catch_up.push(connector);
            }
            other => panic!("unexpected engine output during start: {other:?}"),
        }
    }

    fn connector_event(&mut self, event: connector::Event) {
        let mut pending = VecDeque::from([event]);
        for _ in 0..32 {
            let Some(event) = pending.pop_front() else { break };
            let mut out = Queue::uncapped();
            connector::step(&mut self.connector, &self.connector_env, event, &mut out);
            while let Some(output) = out.pop() {
                match output {
                    connector::Out::Save(record) => {
                        self.records.insert(record.key(), record);
                    }
                    connector::Out::Erase(key) => {
                        self.records.remove(&key);
                    }
                    connector::Out::System(call) => {
                        assert!(self.wires, "connectors reach their systems only after Wires resumes");
                        pending.push_back(connector::Event::System(self.system.answer(call, Fault::None)));
                    }
                    connector::Out::Up(up) => {
                        if let jig::Up::Effects(jig::effects::Up::Outbox { outcome, .. }) = &up {
                            self.outcome = Some(*outcome);
                        }
                        if let jig::Up::Lifecycle(jig::lifecycle::Up::Done {
                            stage: jig::lifecycle::RestartStage::Settled,
                        }) = &up
                        {
                            self.complete = true;
                        }
                        self.connector_up(up);
                    }
                    other => panic!("unexpected connector restart output: {other:?}"),
                }
            }
        }
        assert!(pending.is_empty());
    }

    fn restore_connector(&mut self, pending: &mut VecDeque<In>) {
        let mut out = Queue::uncapped();
        let ask = jig::Ask::Lifecycle(jig::lifecycle::Ask::Restore);
        connector::step(&mut self.connector, &self.connector_env, connector::Event::Ask(ask), &mut out);
        assert!(matches!(out.pop(), Some(connector::Out::LoadLive { after: None })));
        assert!(out.is_empty(), "the first restore ask only loads the connector's records");
        let rows = self
            .records
            .values()
            .map(|record| connector::RestoreRow { record: record.clone(), interests: Box::new([]) })
            .collect();
        let page = connector::Event::RestoredPage { rows, after: None };
        connector::step(&mut self.connector, &self.connector_env, page, &mut out);
        while let Some(output) = out.pop() {
            let connector::Out::Up(up) = output else { panic!("restore only reports state: {output:?}") };
            pending.push_back(In::Connector(jig::Connector(1), up));
        }
    }

    fn check_restored_connector(&self) {
        assert_eq!(self.system.calls(), 1, "restoration made no system call");
        assert_eq!(self.engine.core().tasks.holder(&self.resource), Some(self.task));
        assert_eq!(
            self.engine.core().tasks.writer(&self.resource).expect("restored writer").writer,
            tasks::Writer::Effect { entry: ENTRY }
        );
    }

    fn read_afresh(&mut self) {
        assert!(self.wires && self.engine.core().start_opened());
        self.connector_event(connector::Event::Restart(connector::RestartStep::FreshRead {
            resource: jig_test_connector_kit::path(1, 1),
        }));
        self.connector_up(jig::Up::Resources(jig::resources::Up::ReadAfresh { resource: self.resource.clone() }));
        assert_eq!(self.system.reads().len(), 1);
        assert!(
            self.engine.core().tasks.writer(&self.resource).is_some(),
            "fresh reads keep effect writers until their outcome"
        );
    }

    fn settle(&mut self) {
        self.connector_event(connector::Event::Restart(connector::RestartStep::SettleOutbox));
        assert!(self.complete, "the root advances only after the connector reports settled");
    }

    fn store_answer(&self, out: store::Out) -> store::In {
        match out {
            store::Out::Switch => store::In::Switched,
            store::Out::Load { load, range: store::Range::Start { ranges, after: None } } => {
                let items = ranges
                    .iter()
                    .flat_map(|range| self.core_rows.iter().filter(move |(key, _)| range.admits(key)))
                    .map(|(_, row)| store::Item::Record(row.clone()))
                    .collect();
                store::In::Loaded { load, result: store::Answer::Rows(store::Rows::Start { items, next: None }) }
            }
            other @ store::Out::Load { .. } => panic!("unexpected start load: {other:?}"),
        }
    }

    fn start_ports(&mut self) {
        let mut pending = VecDeque::from([In::Store(store::In::Opened { pending: false })]);
        let mut phases = Vec::new();
        for _ in 0..64 {
            let Some(input) = pending.pop_front() else { break };
            let mut out = Queue::uncapped();
            engine::step(&mut self.engine, &self.env, input, &mut out);
            let mut writes = false;
            let mut wires = false;
            while let Some(output) = out.pop() {
                match output {
                    Out::Now(Peer::Application(application::Out::Start { phase })) => phases.push(phase),
                    Out::Now(Peer::Application(application::Out::Numbered { commits })) => {
                        let header = &self.core_rows[&core::Key::Core(core::CoreKey::Deployment)];
                        let core::Record::Core(core::CoreRecord::Deployment(header)) = header else {
                            panic!("deployment header")
                        };
                        assert_eq!(commits, header.commits);
                    }
                    Out::Held(Peer::Store(request)) => pending.push_back(In::Store(self.store_answer(request))),
                    Out::Ask(jig::Connector(1), jig::Ask::Lifecycle(jig::lifecycle::Ask::Restore)) => {
                        self.restore_connector(&mut pending);
                    }
                    Out::Ask(number @ jig::Connector(2), jig::Ask::Lifecycle(jig::lifecycle::Ask::Restore)) => {
                        self.check_restored_connector();
                        pending.push_back(In::Connector(
                            number,
                            jig::Up::Lifecycle(jig::lifecycle::Up::Done {
                                stage: jig::lifecycle::RestartStage::Restored,
                            }),
                        ));
                    }
                    Out::Write(write) => {
                        writes = true;
                        wires |= matches!(write, store::Write::Wires);
                    }
                    Out::Held(Peer::Resume(resume)) => {
                        assert!(writes, "a start continuation belongs to a writing decision");
                        assert!(out.is_empty(), "release the continuation after the decision's writes");
                        if wires {
                            assert_eq!(self.system.calls(), 1);
                            assert!(!self.engine.core().start_opened(), "Wires has not yet resumed");
                            self.wires = true;
                        }
                        pending.push_back(In::Resume(resume));
                    }
                    other => self.engine_output(other),
                }
            }
        }
        assert!(pending.is_empty(), "the port-driven start settles");
        assert!(self.wires, "Wires committed before opening");
        assert_eq!(
            phases,
            [
                Some(Phase::LoadCore),
                Some(Phase::LoadInbox),
                Some(Phase::RestoreConnector { connector: 1 }),
                Some(Phase::RestoreConnector { connector: 2 }),
                Some(Phase::Link),
                Some(Phase::Sweep),
                Some(Phase::Switch),
                Some(Phase::Wires),
                Some(Phase::AdoptRuns),
                Some(Phase::Open),
                None,
            ]
        );
    }

    fn started(&mut self) {
        self.start_ports();
        assert_eq!(self.catch_up, [1, 2]);
        assert_eq!(self.engine.core().start_phase(), None);
        assert!(self.engine.core().ready() && self.engine.core().start_opened());
        assert!(!self.engine.core().caught_up(1));
        self.read_afresh();
        self.connector_up(jig::Up::Lifecycle(jig::lifecycle::Up::Done {
            stage: jig::lifecycle::RestartStage::ReadAfresh,
        }));
        assert!(!self.engine.core().caught_up(1));
        self.settle();
        assert!(self.engine.core().caught_up(1));
        assert_eq!(self.engine.core().start_failure(), None);
    }
}

fn recovered(recovery: u16, applications: usize, calls: u64) {
    let mut started = Harness::new(recovery);
    started.started();
    assert_eq!(started.outcome, Some(jig::effects::OutboxOutcome::Made));
    assert!(started.records.values().any(|row| matches!(row, connector::Record::Made { state: 11, .. })));
    assert!(!started.records.values().any(|row| matches!(row, connector::Record::Outbox(_))));
    assert_eq!(started.engine.core().tasks.writer(&started.resource), None);
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
    assert_eq!(started.outcome, Some(jig::effects::OutboxOutcome::Held { entry: ENTRY }));
    assert!(started.records.values().any(|row| matches!(
        row,
        connector::Record::Outbox(connector::OutboxEntry { phase: connector::EffectPhase::Held, .. })
    )));
    assert!(started.engine.core().tasks.writer(&started.resource).is_some());
    let row = started.engine.core().tasks.task(started.task).expect("held task");
    assert!(
        matches!(&row.phase, tasks::Phase::Held { why: tasks::Hold::Uncertain { entry: ENTRY }, source: Some(source), .. } if **source == tasks::HoldSource::Effect { connector: 1, entry: ENTRY })
    );
    assert_eq!(started.arrivals, [(1, 1, started.task, 1)], "recovery creates a new escalation need");
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
            assert_eq!(h.engine.core().tasks.holder(&h.resource), Some(h.task));
            assert_eq!(
                h.engine.core().tasks.resource_hold(&h.resource),
                Some(tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits })
            );
            assert_eq!(h.engine.core().tasks.writer(&h.resource), None);
            let row = h.engine.core().tasks.task(h.task).expect("idle holder");
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
                &mut h.engine.core_mut().tasks,
                &env,
                tasks::Event::Message {
                    reply_to: ReplyTo::new(Token::new(1)),
                    project: 1,
                    operation: tasks::Message { task: h.task, word },
                },
                &mut Queue::uncapped(),
            );
            assert_eq!(
                h.engine.core().tasks.task(h.task).expect("woken holder").phase,
                tasks::Phase::Active(tasks::Active::Due)
            );
            tasks::step(
                &mut h.engine.core_mut().tasks,
                &env,
                tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(2)), task: h.task },
                &mut Queue::uncapped(),
            );
            assert_eq!(
                h.engine.core().tasks.task(h.task).expect("prepared holder").phase,
                tasks::Phase::Active(tasks::Active::Preparing)
            );
            let mut out = Queue::uncapped();
            tasks::step(
                &mut h.engine.core_mut().tasks,
                &env,
                tasks::Event::Claim {
                    sources: None,
                    fresh: None,
                    session_end: None,
                    offered: jig_domain_tasks::Reasons::ALL,
                    reasons: jig_domain_tasks::Reasons::FIRST,
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
                        h.engine.core().tasks.writer(&h.resource).expect("owned workspace writer").writer,
                        tasks::Writer::Run { task: h.task, attempt }
                    );
                }
                connector::ResourceRole::Context => {
                    let Some(tasks::Request::Refused { problem, .. }) = out.pop() else {
                        panic!("context denies a workspace writer")
                    };
                    assert_eq!(problem.why, tasks::Refusal::Holds);
                    assert!(out.is_empty());
                    assert_eq!(h.engine.core().tasks.writer(&h.resource), None);
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
        let pool = task_name(&jig_test_connector_kit::path(1, 2));
        assert_eq!(h.connector.slots(&jig_test_connector_kit::path(1, 2)), Some(2));
        assert_eq!(
            h.engine.core().tasks.resource_hold(&pool),
            Some(tasks::HoldKind::Pooled { taken: tasks::Taken::Waits })
        );
        let holder = h.engine.core().tasks.task(h.task).expect("sampled task");
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
            &mut h.engine.core_mut().tasks,
            &env,
            tasks::Event::Make {
                reads_project: false,
                maker: None,
                places: Box::new([]),
                reply_to: ReplyTo::new(Token::new(4)),
                creator: tasks::Party::Person(1),
                batch: Box::new([row]),
            },
            &mut out,
        );
        let row = h.engine.core().tasks.task(2).expect("admitted pool waiter");
        assert_eq!(row.phase, tasks::Phase::Waiting);
        assert!(!row.holds_taken, "the restored zero-slot capacity survives the loading report of two slots");
    }
}

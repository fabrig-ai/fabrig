//! Scripted store and commit releases (jig/docs/design/domain/engine.md, 6).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "the scripted host is ordinary Rust with Vec-backed store pages and typed assertions"
)]
use super::fixtures;
use crate::{Delivery, Domain, Event, Record, Request, StartInput, Work, Write};
use ::core::{mem, panic::AssertUnwindSafe};
use jig_domain_core::{
    self as core,
    start::{Continuation, Failure, Phase},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

struct Host {
    domain: Domain,
    env: Env<crate::Limits>,
    rows: Vec<Record>,
    phases: Vec<Phase>,
    commits: Vec<(u64, bool)>,
    cut_wires: bool,
    wires: Option<Continuation>,
    reads: Vec<(u16, u32)>,
    stopped: bool,
}

impl Host {
    fn new(rows: Vec<Record>) -> Self {
        Self {
            domain: fixtures::domain(),
            env: fixtures::env(),
            rows,
            phases: Vec::new(),
            commits: Vec::new(),
            cut_wires: false,
            wires: None,
            reads: Vec::new(),
            stopped: false,
        }
    }

    fn begin(&mut self) {
        crate::step(&mut self.domain, &self.env, Event::StartBegin { pending: false });
        assert!(!self.domain.ready());
    }

    fn page(&mut self, phase: Phase) {
        let mut rows = self.rows.clone();
        rows.sort_by_key(Record::key);
        for row in rows {
            let selected = match (&phase, &row) {
                (Phase::LoadCore, Record::Core(_)) | (Phase::Sweep, Record::Core(core::Record::People(_))) => true,
                (Phase::RestoreConnector { connector }, Record::Connector { number, .. }) => connector == number,
                _ => false,
            };
            if selected {
                crate::start_record(&mut self.domain, &self.env, row);
            }
            if self.domain.start_stopped() {
                break;
            }
        }
        crate::step(&mut self.domain, &self.env, Event::StartDone(phase));
    }

    fn apply(&mut self, number: u64, mut writes: Queue<Write>) -> bool {
        let mut wires = false;
        for _ in 0..writes.len() {
            match writes.pop().expect("commit write") {
                Write::Save(row) => {
                    self.rows.retain(|old| old.key() != row.key());
                    self.rows.push(row);
                }
                Write::Erase(key) => self.rows.retain(|row| row.key() != key),
                Write::Wires => wires = true,
                Write::CopySettingsRevision(_) => panic!("fixture keeps the configured settings"),
            }
        }
        self.commits.push((number, wires));
        crate::step(&mut self.domain, &self.env, Event::Committed { number, fill: 0 });
        wires
    }

    fn drive(&mut self) {
        for _ in 0..128_u32 {
            let mut out = Queue::uncapped();
            crate::release(&mut self.domain, &self.env, &mut out);
            for _ in 0..out.len() {
                match out.pop().expect("host request") {
                    Request::Start { phase, census } => {
                        assert_eq!(census.is_some(), phase == Phase::LoadInbox);
                        self.phases.push(phase);
                        self.page(phase);
                    }
                    Request::Commit { number, writes } => {
                        if self.apply(number, writes) {
                            let mut held = Queue::uncapped();
                            let _released = self.domain.journal.release(&mut held);
                            let Some(Delivery::Start(continuation)) = held.pop() else {
                                panic!("Wires and its continuation belong to one decision");
                            };
                            assert!(held.is_empty());
                            self.wires = Some(continuation.clone());
                            assert!(!self.domain.ready());
                            if self.cut_wires {
                                return;
                            }
                            crate::decide(&mut self.domain, &self.env, Work::Start(StartInput::Resume(continuation)));
                        }
                    }
                    Request::Deliver(Delivery::System {
                        connector,
                        call: crate::connector::SystemRequest::ReadFact { resource, .. },
                    }) => {
                        crate::step(
                            &mut self.domain,
                            &self.env,
                            Event::Connector {
                                number: connector,
                                event: crate::connector::Event::System(crate::connector::SystemEvent::Fact {
                                    resource,
                                    fact: crate::connector::Fact {
                                        state: Some(1),
                                        pending: false,
                                        observed: self.env.wall,
                                    },
                                    origin: crate::connector::Origin::Own,
                                }),
                            },
                        );
                        let seen = match self.reads.iter_mut().find(|(number, _)| *number == connector) {
                            Some((_, count)) => {
                                *count = count.checked_add(1).expect("configured read count");
                                *count
                            }
                            None => {
                                self.reads.push((connector, 1));
                                1
                            }
                        };
                        if crate::numbered(&mut self.domain, connector)
                            .specification_resource(u64::from(seen) + 1)
                            .is_none()
                        {
                            crate::step(&mut self.domain, &self.env, Event::StartDone(Phase::ReadAfresh { connector }));
                        }
                    }
                    Request::Stop => {
                        self.stopped = true;
                        return;
                    }
                    Request::Deliver(_) | Request::Now(_) => {}
                    Request::Restart(_) | Request::Busy { .. } | Request::ReadResult { .. } => {
                        panic!("start host vocabulary")
                    }
                }
            }
            if self.domain.ready() {
                return;
            }
        }
        panic!("scripted start settles within its phase and commit bound");
    }
}

fn seeded_rows() -> Vec<Record> {
    let mut host = Host::new(Vec::new());
    host.begin();
    host.page(Phase::LoadCore);
    let commit = host.domain.journal.commit().expect("seed commits before inbox loading");
    host.apply(commit.number, commit.writes);
    host.rows
}

#[test]
fn begin_holds_the_seed_until_its_commit_is_released() {
    let mut host = Host::new(Vec::new());
    host.begin();
    let mut out = Queue::uncapped();
    crate::release(&mut host.domain, &host.env, &mut out);
    let Some(Request::Start { phase: Phase::LoadCore, census: None }) = out.pop() else {
        panic!("LoadCore starts the walk")
    };
    assert!(out.is_empty());
    host.page(Phase::LoadCore);
    assert!(!host.domain.ready());
    let commit = host.domain.journal.commit().expect("empty store seeds in one decision");
    let mut held = Queue::uncapped();
    let _released = host.domain.journal.release(&mut held);
    assert!(held.is_empty(), "seed continuation waits for durability");
    host.apply(commit.number, commit.writes);
    let _released = host.domain.journal.release(&mut held);
    let Some(Delivery::Start(_)) = held.pop() else { panic!("seed holds its start continuation") };
    assert!(held.is_empty());
    let mut settings = false;
    for row in &host.rows {
        if let Record::Core(core::Record::Core(core::CoreRecord::Settings(_))) = row {
            settings = true;
        }
    }
    assert!(settings);
    assert_eq!(host.commits, [(1, false)]);
}

#[test]
#[ignore = "needs core-inbox"]
fn empty_store_opens_after_the_wires_continuation() {
    let mut host = Host::new(Vec::new());
    host.begin();
    host.drive();
    assert!(host.domain.ready());
    assert_eq!(
        host.phases,
        [
            Phase::LoadCore,
            Phase::LoadInbox,
            Phase::RestoreConnector { connector: 1 },
            Phase::RestoreConnector { connector: 2 },
            Phase::Sweep,
            Phase::Switch
        ]
    );
    assert!(host.wires.is_some());
    assert_eq!(host.commits.first(), Some(&(1, false)));
    assert!(host.commits.iter().any(|(_, wires)| *wires));
}

fn policy() -> people::PolicyValue {
    people::PolicyValue {
        ceiling: people::Authority {
            tools: 1023,
            models: None,
            grants: Box::new([]),
            delegation: people::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            spend: 100,
            deadline: None,
            notes: 0,
            note_resources: Box::new([]),
        },
        escalation_role: None,
        period_spend: 100,
        projections: Box::new([]),
        roles: Box::new([]),
        requirements: Box::new([]),
        permissions: Box::new([]),
        templates: Box::new([]),
    }
}

fn populated_rows() -> Vec<Record> {
    let mut rows = seeded_rows();
    for row in &mut rows {
        if let Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = row {
            header.projects = 1;
            header.people = 1;
            header.tasks = 1;
        }
    }
    rows.extend([
        Record::Core(core::Record::People(people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"owner".as_slice().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        })),
        Record::Core(core::Record::People(people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        })),
        Record::Core(core::Record::People(people::Stored::Policy {
            project: 1,
            name: Box::from("Project"),
            value: policy(),
        })),
    ]);
    let mut source = fixtures::domain();
    let env = fixtures::env();
    let mut out = Queue::uncapped();
    let task_env = Env { now: env.now, wall: env.wall, limits: env.limits.core.tasks };
    for event in [
        tasks::Event::Check,
        tasks::Event::Restored,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 1, period: 1, budget: 100 },
        tasks::Event::Make {
            reply_to: ReplyTo::new(Token::new(2)),
            creator: tasks::Party::Person(1),
            batch: Box::new([tasks::New {
                number: 1,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Agent { charter: 1 },
                llm: None,
                spec: tasks::Spec {
                    asks: false,
                    words: b"live work".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                deadline: tasks::Deadline::Named,
                authority: fixtures::run().authority,
                numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: 1, period: 1 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            }]),
        },
    ] {
        tasks::step(&mut source.core.tasks, &task_env, event, &mut out);
    }
    assert!(source.core.tasks.task(1).is_some());
    for _ in 0..out.len() {
        let request = out.pop().expect("fixture task output");
        if let tasks::Request::Save { record } = request {
            rows.retain(|old| old.key() != Record::Core(core::Record::Tasks(record.clone())).key());
            rows.push(Record::Core(core::Record::Tasks(record)));
        }
    }
    rows.sort_by_key(Record::key);
    rows
}

#[test]
#[ignore = "needs core-inbox"]
fn people_project_and_live_task_open_in_the_core_phase_order() {
    let mut host = Host::new(populated_rows());
    host.begin();
    host.drive();
    assert!(host.domain.ready());
    assert_eq!(
        host.phases,
        [
            Phase::LoadCore,
            Phase::LoadInbox,
            Phase::RestoreConnector { connector: 1 },
            Phase::RestoreConnector { connector: 2 },
            Phase::Sweep,
            Phase::Switch
        ]
    );
    assert!(host.domain.core.tasks.task(1).is_some());
}

#[test]
#[ignore = "needs core-inbox"]
fn cold_start_discards_the_unreleased_wires_continuation() {
    let mut first = Host::new(Vec::new());
    first.cut_wires = true;
    first.begin();
    first.drive();
    let stale = first.wires.take().expect("crash at durable Wires before its release");
    let rows = mem::take(&mut first.rows);
    drop(first);
    let mut restarted = Host::new(rows);
    restarted.begin();
    restarted.drive();
    assert!(restarted.domain.ready());
    let refused = std::panic::catch_unwind(AssertUnwindSafe(|| {
        crate::decide(&mut restarted.domain, &restarted.env, Work::Start(StartInput::Resume(stale)));
    }));
    assert!(refused.is_err());
}

#[test]
#[ignore = "needs core-settings"]
fn project_policy_past_its_bytes_refuses_at_load_core_before_switch() {
    let mut rows = seeded_rows();
    for row in &mut rows {
        if let Record::Core(core::Record::Core(core::CoreRecord::Deployment(header))) = row {
            header.projects = 1;
        }
    }
    rows.push(Record::Core(core::Record::People(people::Stored::Policy {
        project: 1,
        name: Box::from("Project"),
        value: policy(),
    })));
    let mut host = Host::new(rows);
    host.env.limits.core.policy_bytes = 1;
    host.begin();
    host.drive();
    assert_eq!(host.domain.start_failure(), Some((Some(Phase::LoadCore), Failure::ProjectPolicy { project: 1 })));
    assert!(!host.domain.ready());
    assert!(host.stopped);
    assert_eq!(host.phases, [Phase::LoadCore]);
    assert!(host.commits.iter().all(|(_, wires)| !wires));
}

#[test]
fn missing_link_is_retained_until_the_core_link_pass() {
    let mut host = Host::new(Vec::new());
    host.begin();
    crate::start_missing(
        &mut host.domain,
        core::Key::People(people::Key::Roles(1)),
        core::Key::People(people::Key::Person(1)),
    );
    assert_eq!(host.domain.core.start_phase(), Some(Phase::LoadCore));
    assert_eq!(host.domain.start_failure(), None);
    assert!(!host.domain.ready());
}

#[test]
fn populated_fixture_contains_a_live_task_and_its_people_and_project() {
    let rows = populated_rows();
    let (mut live, mut person, mut project) = (false, false, false);
    for row in &rows {
        if let Record::Core(core::Record::Tasks(tasks::Stored::Live(_))) = row {
            live = true;
        }
        if let Record::Core(core::Record::People(people::Stored::Person { .. })) = row {
            person = true;
        }
        if let Record::Core(core::Record::People(people::Stored::Policy { .. })) = row {
            project = true;
        }
    }
    assert!(live && person && project);
}

#[test]
fn wires_commit_holds_its_continuation_before_any_connector_read() {
    use core::start::{Action, SweepInput};
    let mut host = Host::new(Vec::new());
    let env = Env { now: host.env.now, wall: host.env.wall, limits: host.env.limits.core };
    let mut out = Queue::uncapped();
    // Prepare the core at Wires without asking the host for an inbox census.
    let mut action = host.domain.core.start_begin(false, env.wall);
    for _ in 0..32_u32 {
        action = match action {
            Action::Step(Phase::Wires) => break,
            Action::Step(Phase::Sweep) => host.domain.core.start_sweep(&env, SweepInput::Finished, &mut out),
            Action::Step(phase) => host.domain.core.start_done(&env, phase, &mut out),
            Action::Commit(continuation) => host.domain.core.start_resume(&env, continuation, &mut out),
            Action::Idle | Action::Refused(_) => panic!("prepared core reaches Wires"),
        };
        while out.pop().is_some() {}
    }
    assert_eq!(host.domain.core.start_phase(), Some(Phase::Wires));
    crate::step(&mut host.domain, &host.env, Event::StartDone(Phase::Wires));
    let commit = host.domain.journal.commit().expect("Wires asks for durability");
    let mut held = Queue::uncapped();
    let _released = host.domain.journal.release(&mut held);
    assert!(held.is_empty());
    assert!(!host.domain.ready());
    assert!(host.apply(commit.number, commit.writes));
    let _released = host.domain.journal.release(&mut held);
    let Some(Delivery::Start(continuation)) = held.pop() else { panic!("Wires holds its own continuation") };
    assert!(held.is_empty());
    crate::decide(&mut host.domain, &host.env, Work::Start(StartInput::Resume(continuation)));
    assert_eq!(host.domain.core.start_phase(), Some(Phase::ReadAfresh { connector: 1 }));
    let mut released = Queue::uncapped();
    crate::release(&mut host.domain, &host.env, &mut released);
    let mut read = false;
    for request in &released {
        if let Request::Deliver(Delivery::System { call: crate::connector::SystemRequest::ReadFact { .. }, .. }) =
            request
        {
            read = true;
        }
    }
    assert!(read);
}

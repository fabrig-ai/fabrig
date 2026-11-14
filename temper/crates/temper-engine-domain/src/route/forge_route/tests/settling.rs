//! A shared dependency's readiness settles before sealing a train (domain/landing.md, 9).
use super::super::{Domain, Limits, Work, forge, forge_change as change, tasks};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};

const LATCH: u64 = 7;

#[expect(clippy::too_many_lines, reason = "one root fixture couples the shared dependency, gated changes and due task steps")]
fn kept_changes(changes: &[u64]) -> (Domain, Env<Limits>) {
    let mut limits = crate::tests::resume_fixture::limits();
    limits.tasks.tasks = 4;
    limits.tasks.project_tasks = 4;
    limits.people.inbox_tasks = 8;
    limits.tasks.tree_tasks = 4;
    limits.tasks.batch = 4;
    limits.tasks.batch_bytes = 16_384;
    limits.forge.landing_members = 3;
    let (mut domain, env) = super::procedure_root_with_limits(limits);
    let (fixture, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let template = fixture.core.tasks.task(task).unwrap();
    let mut batch = List::with_capacity(4);
    for number in core::iter::once(LATCH).chain(changes.iter().copied()) {
        batch.push(tasks::New {
            number,
            project: 1,
            label: 0,
            flags: tasks::Flags::PLAIN,
            executor: tasks::Executor::Procedure { connector: 0, code: 2 },
            llm: None,
            spec: tasks::Spec {
                parameters: if number == LATCH {
                    Box::new([])
                } else {
                    Box::new([
                        tasks::Parameter::Resource { name: 1, connector: 0, resource: forge::tag(1, 1) },
                        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
                    ])
                },
                ..template.spec.clone()
            },
            contract: if number == LATCH {
                tasks::Contract::Report { words: 32 }
            } else {
                tasks::Contract::Change { connector: 0, kind: 1, words: 32 }
            },
            deadline: tasks::Deadline::Named,
            authority: template.authority.clone(),
            numbers: template.numbers,
            funder: tasks::Funder::Period { project: 1, period: 1 },
            dependencies: if number == LATCH { Box::new([]) } else { Box::new([LATCH]) },
            holdings: Box::new([]),
            wake: tasks::WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        }).unwrap();
    }
    let mut made = Queue::uncapped();
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Make {
            reply_to: super::super::super::internal(u64::MAX),
            creator: tasks::Party::Deployment { project: 1 },
            batch: batch.into_boxed(),
        },
        &mut made,
    );
    assert!(domain.core.tasks.task(LATCH).is_some(), "{made:?}");
    for task in changes {
        assert_eq!(domain.core.tasks.task(*task).unwrap().waiting_on.as_ref(), &[LATCH]);
        let mut row = super::change_row(Some(*task));
        row.task = *task;
        row.change.task = *task;
        let repository = super::adopted(1);
        row.repository = repository.provider;
        row.branch = super::super::tree_branch(&repository, *task, b'c', *task, None, env.limits.forge.name_bytes).unwrap();
        row.change.way = change::Way::Turn;
        row.change.state = change::State::Gating { head: super::SHA1, asked: Box::new([1]) };
        row.change.gates = Box::new([change::Gate {
            number: 1,
            kind: change::GateKind::Agent,
            blocking: true,
            freshness: change::Freshness::Exact,
            eager: false,
        }]);
        row.verdicts = Box::new([change::GateReport { number: 1, head: super::SHA1, status: change::Status::Passed }]);
        row.change.last_head = Some(super::SHA1);
        forge::step(
            &mut domain.forge,
            &crate::translate::environment_forge(&env),
            forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
            &mut Queue::uncapped(),
        );
    }
    super::turn_project(&mut domain);
    assert!(domain.forge.project_batch(1, 3));
    domain.work.push(Work::Tasks(tasks::Event::Procedure {
        reply_to: super::super::super::internal(u64::MAX),
        task: LATCH,
        step: 1,
        read: None,
        decision: tasks::ProcedureDecision::new(tasks::ProcedureEnd::Result(tasks::TaskResult::Report {
            words: b"Ready together".as_slice().into(),
        })),
    }));
    super::judge_trains(&mut domain, &env);
    for task in changes {
        let row = domain.core.tasks.task(*task).unwrap();
        assert!(row.waiting_on.is_empty(), "the shared non-change dependency ended");
        assert!(domain.core.tasks.procedure_due(*task).is_some(), "its readiness reevaluation is due");
    }
    (domain, env)
}

/// Deliver the forge child's completed readiness read, one change at a time.
fn ready(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    let mut row = domain.forge.change(task).unwrap().clone();
    row.change.state = change::State::Ready { head: super::SHA1, ready_since: env.wall };
    row.change.ready_since = Some(env.wall);
    domain.work.push(Work::Forge(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) }));
    let mut answered = Queue::uncapped();
    answered.push(forge::Request::ChangeDecision { task, decision: change::Decision::Ready, entry: None, evidence: None });
    domain.work.push(Work::ForgeSettings(answered));
    super::judge_trains(domain, env);
    assert!(domain.core.tasks.procedure_due(task).is_none(), "readiness read settled: {:?}", domain.core.tasks.task(task));
}

#[test]
fn a_shared_dependency_forms_one_landing_after_all_three_readiness_reads_settle() {
    let changes = [8, 9, 10];
    let (mut domain, env) = kept_changes(&changes);
    for task in [10, 8] {
        ready(&mut domain, &env, task);
        assert!(domain.forge.change(task).unwrap().change.landing.is_none(), "another readiness read is due");
    }
    ready(&mut domain, &env, 9);
    let landing = domain.forge.change(8).unwrap().change.landing.expect("settled readiness forms at once");
    let row = domain.forge.landing(landing).unwrap();
    assert_eq!(row.landing.members.len(), changes.len());
    for (member, expected) in row.landing.members.iter().zip(changes) {
        assert_eq!(member.change, expected);
    }
    for task in changes {
        assert_eq!(domain.forge.change(task).unwrap().change.landing, Some(landing));
    }
}

#[test]
fn a_lone_change_forms_as_soon_as_its_dependency_readiness_settles() {
    let (mut domain, env) = kept_changes(&[8]);
    ready(&mut domain, &env, 8);
    let landing = domain.forge.change(8).unwrap().change.landing.expect("no other readiness is due");
    let row = domain.forge.landing(landing).unwrap();
    assert_eq!(row.landing.members.len(), 1);
    assert_eq!(row.landing.members[0].change, 8);
}

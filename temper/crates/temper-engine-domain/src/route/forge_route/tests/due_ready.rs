//! Due waiting changes leave only their own units out (domain/landing.md, 9).
use super::super::{Domain, Limits, Work, forge, forge_change as change, tasks};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};

fn ready_changes(changes: &[u64], batch_size: u16) -> (Domain, Env<Limits>) {
    let mut limits = crate::tests::resume_fixture::limits();
    limits.tasks.tasks = 16;
    limits.tasks.project_tasks = 16;
    limits.tasks.tree_tasks = 16;
    limits.tasks.batch = 8;
    limits.tasks.batch_bytes = 16_384;
    limits.people.inbox_tasks = 32;
    limits.forge.landing_members = 4;
    let (mut domain, env) = super::procedure_root_with_limits(limits);
    let (fixture, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let template = fixture.core.tasks.task(task).unwrap();
    let mut batch = List::with_capacity(8);
    for number in changes {
        batch
            .push(tasks::New {
                number: *number,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Procedure { connector: 0, code: 2 },
                llm: None,
                spec: tasks::Spec {
                    parameters: Box::new([
                        tasks::Parameter::Resource { name: 1, connector: 0, resource: forge::tag(1, 1) },
                        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
                    ]),
                    ..template.spec.clone()
                },
                contract: template.contract.clone(),
                deadline: tasks::Deadline::Named,
                authority: template.authority.clone(),
                numbers: template.numbers,
                funder: tasks::Funder::Period { project: 1, period: 1 },
                dependencies: Box::new([]),
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            })
            .unwrap();
        let mut row = super::change_row(Some(*number));
        row.task = *number;
        row.change.task = *number;
        let repository = super::adopted(1);
        row.repository = repository.provider;
        row.branch =
            super::super::tree_branch(&repository, *number, b'c', *number, None, env.limits.forge.name_bytes).unwrap();
        row.change.way = change::Way::Turn;
        row.change.state = change::State::Ready { head: super::SHA1, ready_since: env.wall };
        row.change.ready_since = Some(env.wall);
        forge::step(
            &mut domain.forge,
            &crate::translate::environment_forge(&env),
            forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
            &mut Queue::uncapped(),
        );
    }
    let mut made = Queue::uncapped();
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Make {
            maker: None,
            places: Box::new([]),
            reply_to: super::super::super::internal(u64::MAX),
            creator: tasks::Party::Deployment { project: 1 },
            batch: batch.into_boxed(),
        },
        &mut made,
    );
    for task in changes {
        assert!(domain.core.tasks.task(*task).is_some(), "{made:?}");
        settle(&mut domain, &env, *task);
    }
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Connection {
            forge: 1,
            state: forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
        },
        &mut Queue::uncapped(),
    );
    for event in [
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        forge::Event::ReadAfresh,
        forge::Event::SettleOutbox,
    ] {
        forge::step(&mut domain.forge, &crate::translate::environment_forge(&env), event, &mut Queue::uncapped());
    }
    super::turn_project(&mut domain);
    assert!(domain.forge.project_batch(1, batch_size));
    (domain, env)
}

// As in train_root, end the completed readiness step without changing its ready row.
fn settle(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    let (_, _, step) = domain.core.tasks.procedure_due(task).expect("step due before settling");
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Procedure {
            reply_to: super::super::super::internal(u64::MAX),
            task,
            step,
            read: None,
            decision: tasks::ProcedureDecision::new(tasks::ProcedureEnd::Wait),
        },
        &mut Queue::uncapped(),
    );
    assert!(domain.core.tasks.procedure_due(task).is_none());
    domain.forge.offered_change(task);
    domain.forge.wait_change(&crate::translate::environment_forge(env), task, None);
}

fn wake(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::WakeProcedure { task },
        &mut Queue::uncapped(),
    );
    assert!(domain.core.tasks.procedure_due(task).is_some());
}

#[test]
fn rotating_due_ready_steps_do_not_starve_a_free_train() {
    let (mut domain, env) = ready_changes(&[7, 8, 9, 10], 3);
    let mut previous = None;
    for task in [7, 8, 9, 10, 7, 8, 9, 10] {
        // The next window's wake arrives before the preceding step's read settles.
        wake(&mut domain, &env, task);
        if let Some(previous) = previous {
            settle(&mut domain, &env, previous);
        }
        super::judge_trains(&mut domain, &env);
        if domain.forge.change(8).unwrap().change.landing.is_some() {
            return;
        }
        previous = Some(task);
    }
    panic!("a free train must form despite a ready step due at every judgement");
}

fn members(domain: &Domain, task: u64) -> Box<[u64]> {
    let landing = &domain.forge.landing(task).unwrap().landing;
    let mut changes = List::with_capacity(u32::try_from(landing.members.len()).unwrap());
    for member in &landing.members {
        changes.push(member.change).unwrap();
    }
    changes.into_boxed()
}

fn cancel_landing(domain: &mut Domain, task: u64) {
    // Its former members stay out while the waiting changes take the next turn.
    for member in members(domain, task).iter().copied() {
        domain.work.push(Work::Tasks(stop_member(member)));
    }
    domain.work.push(Work::Forge(forge::Event::SettleEffects {
        task,
        root: task,
        ending: forge::ReleaseEnding::Cancelled,
    }));
}

fn stop_member(task: u64) -> tasks::Event {
    tasks::Event::HoldFrom {
        task,
        why: tasks::Hold::Stopped,
        source: Box::new(tasks::HoldSource::Account {
            connector: 0,
            entry: None,
            words: b"cancelled".as_slice().into(),
        }),
    }
}

#[test]
fn a_freed_train_forms_right_after_the_first_woken_waiter_settles() {
    let (mut domain, env) = ready_changes(&[7, 8, 9, 10, 11], 3);
    for task in [8, 9, 10, 11] {
        wake(&mut domain, &env, task);
    }
    super::judge_trains(&mut domain, &env);
    let first = domain.forge.change(7).unwrap().change.landing.expect("one settled member");
    assert_eq!(members(&domain, first).as_ref(), [7]);
    for task in [8, 9, 10, 11] {
        settle(&mut domain, &env, task);
    }
    let mut previous = None;
    for task in [8, 9, 10, 11] {
        wake(&mut domain, &env, task);
        if let Some(previous) = previous {
            settle(&mut domain, &env, previous);
        }
        super::judge_trains(&mut domain, &env);
        assert_eq!(members(&domain, first).as_ref(), [7]);
        for waiting in [8, 9, 10, 11] {
            assert_eq!(domain.forge.change(waiting).unwrap().change.landing, None);
        }
        previous = Some(task);
    }
    cancel_landing(&mut domain, first);
    super::judge_trains(&mut domain, &env);
    assert!(domain.forge.train_free(super::adopted(1).provider, b"main"));
    for task in [8, 9, 10, 11] {
        assert!(domain.core.tasks.procedure_due(task).is_some(), "landing end wakes every waiter");
        assert_eq!(domain.forge.change(task).unwrap().change.landing, None);
    }
    settle(&mut domain, &env, 9);
    super::judge_trains(&mut domain, &env);
    let next = domain.forge.change(9).unwrap().change.landing.expect("the first settled waiter forms at once");
    assert_ne!(next, first);
    assert_eq!(members(&domain, next).as_ref(), [9]);
    for task in [8, 10, 11] {
        assert!(domain.core.tasks.procedure_due(task).is_some());
        assert_eq!(domain.forge.change(task).unwrap().change.landing, None);
    }
}

#[test]
fn a_due_middle_waiter_keeps_its_place_and_joins_the_next_landing() {
    let (mut domain, env) = ready_changes(&[7, 8, 9], 3);
    let before = domain.forge.change(8).unwrap().clone();
    wake(&mut domain, &env, 8);
    super::judge_trains(&mut domain, &env);
    let first = domain.forge.change(7).unwrap().change.landing.expect("the settled units form");
    assert_eq!(members(&domain, first).as_ref(), [7, 9]);
    assert_eq!(domain.forge.change(8).unwrap(), &before, "exclusion changes neither queue time nor priority");
    settle(&mut domain, &env, 8);
    super::judge_trains(&mut domain, &env);
    assert_eq!(domain.forge.change(8).unwrap().change.landing, None, "the train is still held");
    cancel_landing(&mut domain, first);
    super::judge_trains(&mut domain, &env);
    assert!(domain.core.tasks.procedure_due(8).is_some(), "landing end wakes the remaining waiter again");
    settle(&mut domain, &env, 8);
    super::judge_trains(&mut domain, &env);
    let next = domain.forge.change(8).unwrap().change.landing.expect("the excluded waiter joins later");
    assert_ne!(next, first);
    assert_eq!(members(&domain, next).as_ref(), [8]);
}

#[test]
fn queue_windows_and_landing_end_wakes_keep_forming_until_every_waiter_joins() {
    let changes = [7, 8, 9, 10, 11, 12];
    let (mut domain, mut env) = ready_changes(&changes, 3);
    queue_window(&mut domain, &mut env);
    let mut joined = List::with_capacity(6);
    for task in changes {
        assert!(domain.forge.train_free(super::adopted(1).provider, b"main"));
        for waiting in changes {
            if waiting < task {
                continue;
            }
            assert!(domain.core.tasks.procedure_due(waiting).is_some());
            assert_eq!(domain.forge.change(waiting).unwrap().change.landing, None);
        }
        settle(&mut domain, &env, task);
        super::judge_trains(&mut domain, &env);
        let formed =
            domain.forge.change(task).unwrap().change.landing.expect("a free train forms at its first settled waiter");
        assert_eq!(members(&domain, formed).as_ref(), [task]);
        for member in members(&domain, formed).iter().copied() {
            joined.push(member).unwrap();
        }
        // Finish the reads woken by the preceding end, while the train is held.
        for waiting in changes {
            if waiting <= task {
                continue;
            }
            settle(&mut domain, &env, waiting);
            super::judge_trains(&mut domain, &env);
            assert_eq!(domain.forge.change(waiting).unwrap().change.landing, None);
        }
        // Another queue window re-steps the now-idle waiters while the train is held.
        queue_window(&mut domain, &mut env);
        for waiting in changes {
            if waiting <= task {
                continue;
            }
            assert!(domain.core.tasks.procedure_due(waiting).is_some(), "the queue window re-steps each waiter");
            settle(&mut domain, &env, waiting);
            super::judge_trains(&mut domain, &env);
            assert_eq!(domain.forge.change(waiting).unwrap().change.landing, None, "the live landing holds the train");
        }
        end_at_read_boundary(&mut domain, &env, formed);
        for waiting in changes {
            if waiting <= task {
                continue;
            }
            assert!(domain.core.tasks.procedure_due(waiting).is_some(), "the landing end wakes the settled waiter");
            assert_eq!(domain.forge.change(waiting).unwrap().change.landing, None);
        }
    }
    assert_eq!(joined.as_slice(), &changes);
}

fn queue_window(domain: &mut Domain, env: &mut Env<Limits>) {
    let elapsed = env.limits.forge.queue_window.as_nanos().checked_add(1).unwrap();
    env.wall = skein_lib::Wall::from_nanos(env.wall.as_nanos().checked_add(elapsed).unwrap());
    env.now = skein_lib::Time::from_nanos(env.now.as_nanos().checked_add(elapsed).unwrap());
    let mut out = Queue::uncapped();
    forge::fire(&mut domain.forge, &crate::translate::environment_forge(env), &mut out);
    wake_at_read_boundary(domain, env, out);
}

// Keep the fixture at the asynchronous read boundary: wakes make tasks due,
// and only settle delivers their completed reads, one at a time.
fn wake_at_read_boundary(domain: &mut Domain, env: &Env<Limits>, mut out: Queue<forge::Request>) {
    let mut rest = Queue::uncapped();
    while let Some(request) = out.pop() {
        if let forge::Request::WakeProcedure { task } = request {
            if let tasks::Phase::Active(_) = domain.core.tasks.task(task).unwrap().phase
                && domain.core.tasks.procedure_due(task).is_none()
            {
                wake(domain, env, task);
            }
        } else {
            rest.push(request);
        }
    }
    domain.work.push(Work::ForgeSettings(rest));
    super::judge_trains(domain, env);
}

fn end_at_read_boundary(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    for member in members(domain, task).iter().copied() {
        tasks::step(&mut domain.core.tasks, &env.with(env.limits.tasks), stop_member(member), &mut Queue::uncapped());
    }
    let mut out = Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(env),
        forge::Event::SettleEffects { task, root: task, ending: forge::ReleaseEnding::Cancelled },
        &mut out,
    );
    wake_at_read_boundary(domain, env, out);
}

#[test]
fn a_step_begun_before_readiness_settles_its_coalesced_wake_before_forming() {
    let (mut domain, env) = ready_changes(&[7, 8], 3);
    let mut row = domain.forge.change(8).unwrap().clone();
    row.change.state = change::State::Gating { head: super::SHA1, asked: Box::new([]) };
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row.clone())) },
        &mut Queue::uncapped(),
    );
    wake(&mut domain, &env, 8);
    let context = domain.core.tasks.run_context(8).unwrap();
    assert!(super::super::start_change(&mut domain, &env, &context));
    super::judge_trains(&mut domain, &env);
    assert!(domain.forge.change(7).unwrap().change.landing.is_none());
    // A second wake reaches the same step before its readiness read completes.
    domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task: 8 }));
    row.change.state = change::State::Ready { head: super::SHA1, ready_since: env.wall };
    domain.work.push(Work::Forge(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) }));
    ready_answer(&mut domain, 8);
    super::judge_trains(&mut domain, &env);
    assert!(domain.core.tasks.procedure_due(8).is_some(), "the readiness step has a coalesced wake");
    assert!(domain.forge.change(7).unwrap().change.landing.is_none(), "the newly ready member still settles");
    ready_answer(&mut domain, 8);
    super::judge_trains(&mut domain, &env);
    let formed = domain.forge.change(7).unwrap().change.landing.expect("both readiness reads settled");
    assert_eq!(members(&domain, formed).as_ref(), [7, 8]);
}

fn ready_answer(domain: &mut Domain, task: u64) {
    let mut out = Queue::uncapped();
    out.push(forge::Request::ChangeDecision { task, decision: change::Decision::Ready, entry: None, evidence: None });
    domain.work.push(Work::ForgeSettings(out));
}

#[test]
fn due_ready_and_gating_changes_do_not_hold_another_branchs_train() {
    for due in [7, 8] {
        for gating in [false, true] {
            let (mut domain, env) = ready_changes(&[7, 8], 3);
            let mut goal = domain.forge.change(8).unwrap().clone();
            goal.base = b"goal".as_slice().into();
            forge::step(
                &mut domain.forge,
                &crate::translate::environment_forge(&env),
                forge::Event::Restore { record: forge::Stored::Change(Box::new(goal)) },
                &mut Queue::uncapped(),
            );
            if gating {
                let mut row = domain.forge.change(due).unwrap().clone();
                row.change.state = change::State::Gating { head: super::SHA1, asked: Box::new([1]) };
                row.change.gates = Box::new([change::Gate {
                    number: 1,
                    kind: change::GateKind::Agent,
                    blocking: true,
                    freshness: change::Freshness::Exact,
                    eager: false,
                }]);
                row.verdicts =
                    Box::new([change::GateReport { number: 1, head: super::SHA1, status: change::Status::Passed }]);
                row.change.last_head = Some(super::SHA1);
                forge::step(
                    &mut domain.forge,
                    &crate::translate::environment_forge(&env),
                    forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
                    &mut Queue::uncapped(),
                );
            }
            wake(&mut domain, &env, due);
            super::judge_trains(&mut domain, &env);
            let other = if due == 7 { 8 } else { 7 };
            let formed =
                domain.forge.change(other).unwrap().change.landing.expect("the other branch forms independently");
            assert_eq!(members(&domain, formed).as_ref(), [other]);
            assert_eq!(domain.forge.change(due).unwrap().change.landing, None);
            assert!(domain.core.tasks.procedure_due(due).is_some());
        }
    }
}

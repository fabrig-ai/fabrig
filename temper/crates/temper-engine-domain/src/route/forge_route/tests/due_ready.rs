//! Due ready changes join in queue order and never hold forming (domain/landing-turn.md, 2).
use super::super::{Domain, Limits, Work, forge, forge_change as change, tasks};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue};

pub(super) fn ready_changes(changes: &[u64], batch_size: u16) -> (Domain, Env<Limits>) {
    let mut limits = crate::tests::resume_fixture::limits();
    limits.tasks.tasks = 16;
    limits.tasks.project_tasks = 16;
    limits.tasks.tree_tasks = 16;
    limits.tasks.batch = 8;
    limits.tasks.batch_bytes = 16_384;
    limits.people.inbox_tasks = 32;
    limits.forge.landing_members = 4;
    limits.tasks.holdings = limits.forge.landing_members;
    let (mut domain, env) = super::procedure_root_with_limits(limits);
    let (fixture, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let template = fixture.engine.core().tasks.task(task).unwrap();
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
        super::restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    }
    let mut made = Queue::uncapped();
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: super::super::super::internal(u64::MAX),
            creator: tasks::Party::Deployment { project: 1 },
            batch: batch.into_boxed(),
        },
        &mut made,
    );
    for task in changes {
        assert!(domain.engine.core().tasks.task(*task).is_some(), "{made:?}");
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
    for event in [forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, forge::Event::CatchUp]
    {
        forge::step(&mut domain.forge, &crate::translate::environment_forge(&env), event, &mut Queue::uncapped());
    }
    super::turn_project(&mut domain);
    assert!(domain.forge.project_batch(1, batch_size));
    (domain, env)
}

// As in train_root, end the completed readiness step without changing its ready row.
pub(super) fn settle(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    let (_, _, step) = domain.engine.core().tasks.procedure_due(task).expect("step due before settling");
    super::task_event(
        domain,
        env,
        tasks::Event::Procedure {
            reads_project: false,
            reply_to: super::super::super::internal(u64::MAX),
            task,
            step,
            read: None,
            decision: tasks::ProcedureDecision::new(tasks::ProcedureEnd::Wait),
        },
    );
    assert!(domain.engine.core().tasks.procedure_due(task).is_none());
    domain.forge.offered_change(task);
    domain.forge.wait_change(&crate::translate::environment_forge(env), task, None);
}

pub(super) fn wake(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    super::task_event(domain, env, tasks::Event::WakeProcedure { task });
    assert!(domain.engine.core().tasks.procedure_due(task).is_some());
}

fn restore_change(domain: &mut Domain, env: &Env<Limits>, row: forge::ChangeRow) {
    super::restore_forge_with_engine_waits(domain, env, forge::Stored::Change(Box::new(row)));
}

fn judge_main(domain: &mut Domain, env: &Env<Limits>) {
    super::judge_trains(domain, env);
    let mut live = 0_u32;
    for task in domain.engine.core().tasks.view_tasks() {
        if let Some(row) = domain.forge.landing(task.number)
            && row.landing.branch.as_ref() == b"main"
            && row.landing.phase != super::super::landing::Phase::Landed
            && row.landing.phase != super::super::landing::Phase::Cancelled
        {
            live = live.checked_add(1).unwrap();
        }
    }
    assert!(live <= 1, "main has {live} live landing rows");
}

#[test]
fn a_full_batch_forms_in_queue_order_while_every_step_stays_due() {
    let changes = [7, 8, 9, 10, 11, 12, 13, 14];
    let (mut domain, mut env) = ready_changes(&changes, 4);
    let mut priority = domain.forge.change(12).unwrap().clone();
    priority.priority = -1_i32;
    restore_change(&mut domain, &env, priority);
    for task in changes {
        wake(&mut domain, &env, task);
    }
    judge_main(&mut domain, &env);
    let first = domain.forge.landing_of(12);
    assert!(first.is_some(), "all eight waiting changes are due, but a full batch must form");
    let first = first.unwrap();
    assert_eq!(members(&domain, first).as_ref(), [12, 7, 8, 9]);
    for _ in 0_u8..3 {
        judge_main(&mut domain, &env);
        assert!(!domain.forge.train_free(super::adopted(1).provider, b"main"));
        for task in [10, 11, 13, 14] {
            assert!(domain.engine.core().tasks.procedure_due(task).is_some());
            assert_eq!(domain.forge.landing_of(task), None);
        }
    }
    queue_window(&mut domain, &mut env);
    judge_main(&mut domain, &env);
    for task in [10, 11, 13, 14] {
        settle(&mut domain, &env, task);
        wake(&mut domain, &env, task);
        judge_main(&mut domain, &env);
        assert_eq!(domain.forge.landing_of(task), None);
    }
    cancel_landing(&mut domain, first);
    judge_main(&mut domain, &env);
    for task in [10, 11, 13, 14] {
        assert!(domain.engine.core().tasks.procedure_due(task).is_some(), "the end wakes every waiter");
    }
    let next = domain.forge.landing_of(10).expect("the next four join at once");
    assert_ne!(first, next);
    assert_eq!(members(&domain, next).as_ref(), [10, 11, 13, 14]);
}

#[test]
fn two_landings_form_within_one_minute_without_turning_main() {
    let (mut domain, env) = ready_changes(&[7, 8, 9, 10], 3);
    let repository = super::adopted(1).provider;
    let main = forge::Name {
        forge: repository.forge,
        repository: repository.repository,
        what: forge::What::Branch(Box::new([b"main".as_slice().into()])),
    };
    super::restore_forge_with_engine_waits(
        &mut domain,
        &env,
        forge::Stored::BranchHead(forge::BranchHead { name: main.clone(), commit: super::SHA1, owned: false }),
    );
    let tip = domain.forge.branch_head(&main);
    assert_eq!(tip, Some(super::SHA1));
    let minute = env.wall.as_nanos() / 60_000_000_000;
    let ready_ten = domain.forge.change(10).unwrap().clone();
    let mut checking = ready_ten.clone();
    checking.change.state = change::State::Checking { head: super::SHA1 };
    restore_change(&mut domain, &env, checking);
    for task in [8, 9] {
        wake(&mut domain, &env, task);
    }
    judge_main(&mut domain, &env);
    let first = domain.forge.landing_of(7).expect("the quiet waiter forms");
    assert_eq!(members(&domain, first).as_ref(), [7, 8, 9]);
    assert!(!domain.forge.train_free(repository, b"main"));
    // The real Ready decision ends a step that joined at the same head while due.
    ready_answer(&mut domain, 8);
    judge_main(&mut domain, &env);
    assert!(domain.engine.core().tasks.procedure_due(8).is_none());
    assert_eq!(domain.forge.landing_of(8), Some(first));
    assert!(!domain.forge.train_free(repository, b"main"));
    assert_eq!(members(&domain, first).as_ref(), [7, 8, 9]);
    for member in &domain.forge.landing(first).unwrap().landing.members {
        assert_eq!(member.left, None);
    }
    settle(&mut domain, &env, 9);
    restore_change(&mut domain, &env, ready_ten);
    judge_main(&mut domain, &env);
    assert!(!domain.forge.train_free(super::adopted(1).provider, b"main"));
    assert_eq!(domain.forge.landing_of(10), None);
    cancel_landing(&mut domain, first);
    judge_main(&mut domain, &env);
    let next = domain.forge.landing_of(10).expect("the end's due waiter joins at once");
    assert_ne!(first, next);
    assert_eq!(members(&domain, next).as_ref(), [10]);
    assert_eq!(domain.forge.branch_head(&main), tip, "main never turned");
    assert_eq!(env.wall.as_nanos() / 60_000_000_000, minute);
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
        if domain.forge.landing_of(8).is_some() {
            return;
        }
        previous = Some(task);
    }
    panic!("a free train must form despite a ready step due at every judgement");
}

pub(super) fn members(domain: &Domain, task: u64) -> Box<[u64]> {
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
fn a_freed_train_forms_at_once_with_due_and_quiet_waiters() {
    let (mut domain, env) = ready_changes(&[7, 8, 9, 10, 11], 3);
    for task in [8, 9, 10, 11] {
        wake(&mut domain, &env, task);
    }
    super::judge_trains(&mut domain, &env);
    let first = domain.forge.landing_of(7).expect("a full batch forms");
    assert_eq!(members(&domain, first).as_ref(), [7, 8, 9]);
    for task in [8, 9, 10, 11] {
        settle(&mut domain, &env, task);
    }
    let mut previous = None;
    for task in [10, 11, 10, 11] {
        wake(&mut domain, &env, task);
        if let Some(previous) = previous {
            settle(&mut domain, &env, previous);
        }
        super::judge_trains(&mut domain, &env);
        assert_eq!(members(&domain, first).as_ref(), [7, 8, 9]);
        for waiting in [10, 11] {
            assert_eq!(domain.forge.landing_of(waiting), None);
        }
        previous = Some(task);
    }
    cancel_landing(&mut domain, first);
    super::judge_trains(&mut domain, &env);
    let next = domain.forge.landing_of(10).expect("due waiters join immediately at the end");
    assert_ne!(next, first);
    assert_eq!(members(&domain, next).as_ref(), [10, 11]);
    for task in [10, 11] {
        assert!(domain.engine.core().tasks.procedure_due(task).is_some(), "landing end wakes every waiter");
    }
}

#[test]
fn a_due_middle_waiter_keeps_its_place_and_joins_the_same_landing() {
    let (mut domain, env) = ready_changes(&[7, 8, 9], 3);
    let before = domain.forge.change(8).unwrap().clone();
    wake(&mut domain, &env, 8);
    super::judge_trains(&mut domain, &env);
    let first = domain.forge.landing_of(7).expect("due and quiet units form together");
    assert_eq!(members(&domain, first).as_ref(), [7, 8, 9]);
    assert_eq!(domain.forge.landing_of(8), Some(first));
    assert_eq!(domain.forge.change(8).unwrap(), &before, "joining changes neither queue time nor priority");
    settle(&mut domain, &env, 8);
    super::judge_trains(&mut domain, &env);
    assert_eq!(domain.forge.landing_of(8), Some(first), "the train is still held");
}

#[test]
fn queue_windows_and_landing_end_wakes_keep_forming_until_every_waiter_joins() {
    let changes = [7, 8, 9, 10, 11, 12];
    let (mut domain, mut env) = ready_changes(&changes, 3);
    for task in changes {
        wake(&mut domain, &env, task);
    }
    let mut joined = List::with_capacity(6);
    for batch in [[7, 8, 9], [10, 11, 12]] {
        super::judge_trains(&mut domain, &env);
        let formed = domain.forge.landing_of(batch[0]).expect("three due waiters form together");
        assert_eq!(members(&domain, formed).as_ref(), batch);
        for member in batch {
            assert!(domain.engine.core().tasks.procedure_due(member).is_some());
            joined.push(member).unwrap();
        }
        // Finish waiting reads while the landing holds its train.
        for waiting in changes {
            if waiting <= batch[2] {
                continue;
            }
            settle(&mut domain, &env, waiting);
            super::judge_trains(&mut domain, &env);
            assert_eq!(domain.forge.landing_of(waiting), None);
        }
        queue_window(&mut domain, &mut env);
        for waiting in changes {
            if waiting <= batch[2] {
                continue;
            }
            assert!(
                domain.engine.core().tasks.procedure_due(waiting).is_some(),
                "the queue window re-steps each waiter"
            );
            settle(&mut domain, &env, waiting);
            super::judge_trains(&mut domain, &env);
            assert_eq!(domain.forge.landing_of(waiting), None, "the live landing holds the train");
        }
        end_at_read_boundary(&mut domain, &env, formed);
        for waiting in changes {
            if waiting <= batch[2] {
                continue;
            }
            assert!(domain.engine.core().tasks.procedure_due(waiting).is_some(), "the landing end wakes each waiter");
            assert!(domain.forge.landing_of(waiting).is_some(), "the freed train forms at once");
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
            if let tasks::Phase::Active(_) = domain.engine.core().tasks.task(task).unwrap().phase
                && domain.engine.core().tasks.procedure_due(task).is_none()
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
        tasks::step(
            &mut domain.engine.core_mut().tasks,
            &env.with(env.limits.tasks),
            stop_member(member),
            &mut Queue::uncapped(),
        );
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
fn a_step_begun_before_readiness_does_not_hold_forming_once_ready() {
    let (mut domain, env) = ready_changes(&[7, 8], 3);
    let mut row = domain.forge.change(8).unwrap().clone();
    row.change.state = change::State::Gating { head: super::SHA1, asked: Box::new([]) };
    row.change.last_head = Some(super::SHA1);
    // The last blocking gate's result has released change 8 into the cohort.
    row.settling = true;
    restore_change(&mut domain, &env, row.clone());
    wake(&mut domain, &env, 8);
    let context = domain.engine.core().tasks.run_context(8).unwrap();
    assert!(super::super::start_change(&mut domain, &env, &context));
    super::judge_trains(&mut domain, &env);
    assert!(domain.forge.landing_of(7).is_none(), "change 8 is settling into the train");
    // A second wake reaches the same step before its readiness read completes.
    domain.work.push(Work::Tasks(tasks::Event::WakeProcedure { task: 8 }));
    // The Ready answer accompanies the row written by the first decision, which leaves the cohort.
    row.change.state = change::State::Ready { head: super::SHA1, ready_since: env.wall };
    row.settling = false;
    domain.work.push(Work::Forge(forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) }));
    ready_answer(&mut domain, 8);
    super::judge_trains(&mut domain, &env);
    assert!(!domain.forge.change(8).unwrap().settling, "the first decision leaves the cohort");
    assert!(domain.engine.core().tasks.procedure_due(8).is_some(), "the readiness step has a coalesced wake");
    let formed = domain.forge.landing_of(7).expect("the ready member no longer holds forming");
    assert_eq!(members(&domain, formed).as_ref(), [7, 8]);
    ready_answer(&mut domain, 8);
    super::judge_trains(&mut domain, &env);
    assert_eq!(domain.forge.landing_of(8), Some(formed));
}

fn ready_answer(domain: &mut Domain, task: u64) {
    let mut out = Queue::uncapped();
    out.push(forge::Request::ChangeDecision {
        task,
        decision: change::Decision::Ready,
        entry: None,
        evidence: None,
        close_at: None,
    });
    domain.work.push(Work::ForgeSettings(out));
}

#[test]
fn due_ready_and_gating_changes_do_not_hold_another_branchs_train() {
    for due in [7, 8] {
        for gating in [false, true] {
            let (mut domain, env) = ready_changes(&[7, 8], 3);
            let mut goal = domain.forge.change(8).unwrap().clone();
            goal.base = b"goal".as_slice().into();
            super::restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(goal)));
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
                row.verdicts = Box::new([change::GateReport {
                    revision: 0,
                    number: 1,
                    head: super::SHA1,
                    status: change::Status::Passed,
                }]);
                row.change.last_head = Some(super::SHA1);
                super::restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
            }
            wake(&mut domain, &env, due);
            super::judge_trains(&mut domain, &env);
            let other = if due == 7 { 8 } else { 7 };
            let formed = domain.forge.landing_of(other).expect("the other branch forms independently");
            assert_eq!(members(&domain, formed).as_ref(), [other]);
            if gating {
                assert_eq!(domain.forge.landing_of(due), None);
            } else {
                let own = domain.forge.landing_of(due).expect("ready due changes form on their own train");
                assert_ne!(own, formed);
                assert_eq!(members(&domain, own).as_ref(), [due]);
            }
            assert!(domain.engine.core().tasks.procedure_due(due).is_some());
        }
    }
}

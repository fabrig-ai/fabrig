//! Only one readiness step at a gating head holds forming (domain/landing.md, 9).
use super::super::{Domain, Limits, forge, forge_change as change, tasks};
use super::due_ready::{members, ready_changes, settle, wake};
use alloc::boxed::Box;
use skein_lib::{Env, Queue};

fn restore(domain: &mut Domain, env: &Env<Limits>, row: forge::ChangeRow) {
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(env),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
}

fn restore_task(domain: &mut Domain, env: &Env<Limits>, changed: tasks::TaskRecord) {
    let mut rows = skein_lib::List::with_capacity(env.limits.tasks.tasks);
    for view in domain.core.tasks.view_tasks() {
        let row = if view.number == changed.number {
            changed.clone()
        } else {
            domain.core.tasks.task(view.number).unwrap().clone()
        };
        rows.push(row).unwrap();
    }
    domain.core.tasks = tasks::Domain::new(
        &env.limits.tasks,
        tasks::Kept { revisions: env.limits.settings.revisions, versions: env.limits.settings.versions },
        100,
        0,
        Box::new([]),
        Box::new([]),
    );
    for row in rows.into_boxed() {
        tasks::step(
            &mut domain.core.tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
            &mut Queue::uncapped(),
        );
    }
}

fn gating(domain: &mut Domain, env: &Env<Limits>, status: change::Status) {
    let mut row = domain.forge.change(8).unwrap().clone();
    row.change.state = change::State::Gating { head: super::SHA1, asked: Box::new([1]) };
    row.change.last_head = Some(super::SHA1);
    row.change.gates = Box::new([change::Gate {
        number: 1,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: false,
    }]);
    row.verdicts = Box::new([change::GateReport { number: 1, head: super::SHA1, status }]);
    restore(domain, env, row);
    wake(domain, env, 8);
}

fn only_waiter_forms(domain: &mut Domain, env: &Env<Limits>) {
    super::judge_trains(domain, env);
    let formed = domain.forge.change(7).unwrap().change.landing.expect("the settled waiter forms at this judgement");
    assert_eq!(members(domain, formed).as_ref(), [7]);
    assert_eq!(domain.forge.change(8).unwrap().change.landing, None);
    assert!(domain.core.tasks.procedure_due(8).is_some());
}

#[test]
fn a_due_checking_change_does_not_hold_its_train() {
    let (mut domain, env) = ready_changes(&[7, 8], 3);
    let mut row = domain.forge.change(8).unwrap().clone();
    row.change.state = change::State::Checking { head: super::SHA1 };
    restore(&mut domain, &env, row);
    wake(&mut domain, &env, 8);
    only_waiter_forms(&mut domain, &env);
}

#[test]
fn outstanding_blocking_gates_do_not_hold_a_due_changes_train() {
    for status in [change::Status::Pending, change::Status::Unknown, change::Status::Failed] {
        let (mut domain, env) = ready_changes(&[7, 8], 3);
        gating(&mut domain, &env, status);
        only_waiter_forms(&mut domain, &env);
    }
}

#[test]
fn a_due_gated_change_kept_by_a_dependency_does_not_hold_its_train() {
    let (mut domain, env) = ready_changes(&[7, 8, 9], 3);
    gating(&mut domain, &env, change::Status::Passed);
    let mut dependency = domain.forge.change(9).unwrap().clone();
    dependency.change.state = change::State::Checking { head: super::SHA1 };
    restore(&mut domain, &env, dependency);
    let mut task = domain.core.tasks.task(8).unwrap().clone();
    task.dependencies = Box::new([9]);
    task.waiting_on = Box::new([9]);
    restore_task(&mut domain, &env, task);
    assert!(super::super::landing_route::kept(&domain, &env, 8));
    only_waiter_forms(&mut domain, &env);
}

#[test]
fn a_gating_head_holds_for_one_step_only_even_when_woken_again() {
    let (mut domain, env) = ready_changes(&[7, 8], 3);
    gating(&mut domain, &env, change::Status::Passed);
    let (_, _, first) = domain.core.tasks.procedure_due(8).unwrap();
    for _ in 0_u8..2 {
        super::judge_trains(&mut domain, &env);
        assert_eq!(domain.forge.change(7).unwrap().change.landing, None, "the same due step keeps holding");
    }
    settle(&mut domain, &env, 8);
    wake(&mut domain, &env, 8);
    assert_ne!(domain.core.tasks.procedure_due(8).unwrap().2, first);
    only_waiter_forms(&mut domain, &env);
}

#[test]
fn a_new_head_or_a_return_to_gating_can_hold_for_one_step_again() {
    for moves in [false, true] {
        let (mut domain, env) = ready_changes(&[7, 8], 3);
        gating(&mut domain, &env, change::Status::Passed);
        super::judge_trains(&mut domain, &env);
        assert_eq!(domain.forge.change(7).unwrap().change.landing, None);
        settle(&mut domain, &env, 8);
        // Leave no waiting unit while proving the old hold has been spent.
        wake(&mut domain, &env, 7);
        wake(&mut domain, &env, 8);
        super::judge_trains(&mut domain, &env);
        assert!(!domain.forge.hold_forming(8, domain.core.tasks.procedure_due(8).unwrap().2));
        let mut row = domain.forge.change(8).unwrap().clone();
        if moves {
            row.change.state = change::State::Gating { head: super::SHA256, asked: Box::new([1]) };
            row.change.last_head = Some(super::SHA256);
            row.verdicts.first_mut().unwrap().head = super::SHA256;
        } else {
            row.change.state = change::State::Checking { head: super::SHA1 };
            restore(&mut domain, &env, row.clone());
            row.change.state = change::State::Gating { head: super::SHA1, asked: Box::new([1]) };
        }
        restore(&mut domain, &env, row);
        settle(&mut domain, &env, 7);
        super::judge_trains(&mut domain, &env);
        assert_eq!(domain.forge.change(7).unwrap().change.landing, None, "the new gating episode settles again");
        settle(&mut domain, &env, 8);
        wake(&mut domain, &env, 8);
        only_waiter_forms(&mut domain, &env);
    }
}

#[test]
fn a_returned_gate_terminal_holds_until_the_due_step_consumes_it() {
    // The gate's task has ended and left jig's live arena; its owner's unread
    // terminal still names the current delegate and the checked head.
    for (head, sender, status, holds) in [
        (super::SHA1, 99, change::Status::Pending, true),
        (super::SHA256, 99, change::Status::Pending, false),
        (super::SHA1, 98, change::Status::Pending, false),
        (super::SHA256, 99, change::Status::Passed, true),
    ] {
        let (mut domain, env) = ready_changes(&[7, 8], 3);
        gating(&mut domain, &env, status);
        let mut row = domain.forge.change(8).unwrap().clone();
        row.delegate = Some((99, change::Delegate::Gate { number: 1, head }));
        restore(&mut domain, &env, row);
        let mut task = domain.core.tasks.task(8).unwrap().clone();
        task.inbox = Box::new([tasks::Word {
            number: 1,
            from: tasks::Party::Task(sender),
            kind: tasks::MessageKind::Result(tasks::ResultKind::Verdict { code: 1, fields: Box::new([]) }),
            words: b"Passed".as_slice().into(),
            at: env.wall,
            hits: 1,
            eligible: true,
        }]);
        restore_task(&mut domain, &env, task);
        super::judge_trains(&mut domain, &env);
        assert_eq!(
            domain.forge.change(7).unwrap().change.landing.is_none(),
            holds,
            "only the current gate at the checked head settles"
        );
        assert_eq!(domain.forge.change(8).unwrap().verdicts.first().unwrap().status, status);
    }
}

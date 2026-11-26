//! Cohort enrollment belongs to a release, not to being due (domain/landing-turn.md, 2).
use super::super::{Domain, Limits, Work, forge, forge_change as change, tasks};
use super::due_ready::{members, ready_changes, wake};
use alloc::boxed::Box;
use skein_lib::{Env, Queue};

const WAITER: u64 = 7;
const CANDIDATE: u64 = 8;
const DEPENDENCY: u64 = 9;
const GATE_TASK: u64 = 99;
const NEXT_GATE_TASK: u64 = 100;

fn restore(domain: &mut Domain, env: &Env<Limits>, row: forge::ChangeRow) {
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(env),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
}

fn restore_task(domain: &mut Domain, env: &Env<Limits>, changed: tasks::TaskRecord) {
    let period = *domain
        .engine
        .core()
        .tasks
        .funding(tasks::Funder::Period { project: 1, period: domain.engine.core().period(1) })
        .expect("the fixture's open period");
    let mut rows = skein_lib::List::with_capacity(env.limits.tasks.tasks);
    for view in domain.engine.core().tasks.view_tasks() {
        let row = if view.number == changed.number {
            changed.clone()
        } else {
            domain.engine.core().tasks.task(view.number).unwrap().clone()
        };
        rows.push(row).unwrap();
    }
    domain.engine.core_mut().tasks = tasks::Domain::new(
        &env.limits.tasks,
        tasks::Kept { revisions: env.limits.settings.revisions, versions: env.limits.settings.versions },
        100,
        0,
        Box::new([]),
        Box::new([]),
    );
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Restore { record: tasks::Stored::Ledger(period) },
        &mut Queue::uncapped(),
    );
    for row in rows.into_boxed() {
        tasks::step(
            &mut domain.engine.core_mut().tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
            &mut Queue::uncapped(),
        );
    }
    for event in [tasks::Event::Check, tasks::Event::Restored] {
        tasks::step(&mut domain.engine.core_mut().tasks, &env.with(env.limits.tasks), event, &mut Queue::uncapped());
    }
}

fn gating(domain: &mut Domain, env: &Env<Limits>, status: change::Status) {
    let mut row = domain.forge.change(CANDIDATE).unwrap().clone();
    row.change.state = change::State::Gating { head: super::SHA1, asked: Box::new([1]) };
    row.change.last_head = Some(super::SHA1);
    row.change.gates = Box::new([change::Gate {
        number: 1,
        kind: change::GateKind::Agent,
        blocking: true,
        freshness: change::Freshness::Exact,
        eager: false,
    }]);
    row.verdicts = Box::new([change::GateReport { revision: 0, number: 1, head: super::SHA1, status }]);
    restore(domain, env, row);
    wake(domain, env, CANDIDATE);
}

fn only_waiter_forms(domain: &mut Domain, env: &Env<Limits>) {
    super::judge_trains(domain, env);
    let formed = domain.forge.landing_of(WAITER).expect("the settled waiter forms at this judgement");
    assert_eq!(members(domain, formed).as_ref(), [WAITER]);
    assert_eq!(domain.forge.landing_of(CANDIDATE), None);
    assert!(domain.engine.core().tasks.procedure_due(CANDIDATE).is_some());
}

#[test]
fn a_due_checking_change_does_not_hold_its_train() {
    let (mut domain, env) = ready_changes(&[WAITER, CANDIDATE], 3);
    let mut row = domain.forge.change(CANDIDATE).unwrap().clone();
    row.change.state = change::State::Checking { head: super::SHA1 };
    restore(&mut domain, &env, row);
    wake(&mut domain, &env, CANDIDATE);
    only_waiter_forms(&mut domain, &env);
}

#[test]
fn a_due_gate_with_no_release_does_not_hold_its_train() {
    let (mut domain, env) = ready_changes(&[WAITER, CANDIDATE], 3);
    gating(&mut domain, &env, change::Status::Passed);
    only_waiter_forms(&mut domain, &env);
}

fn release_gate(
    domain: &mut Domain,
    env: &Env<Limits>,
    head: [u8; 32],
    current: u64,
    sender: u64,
) -> Queue<forge::Request> {
    let mut row = domain.forge.change(CANDIDATE).unwrap().clone();
    row.delegate = Some((current, change::Delegate::Gate { number: 1, head }));
    restore(domain, env, row);
    let mut ended = domain.engine.core().tasks.task(CANDIDATE).unwrap().clone();
    ended.number = sender;
    ended.requester = tasks::Party::Task(CANDIDATE);
    ended.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: b"Gate failed".as_slice().into() });
    let mut out = Queue::uncapped();
    super::super::landing_route::cohort_saved(domain, env, &ended, &mut out);
    out
}

fn finish_wait(domain: &mut Domain, env: &Env<Limits>) {
    let mut out = Queue::uncapped();
    out.push(forge::Request::ChangeDecision {
        task: CANDIDATE,
        decision: change::Decision::Wait { until: None },
        entry: None,
        evidence: None,
    });
    domain.work.push(Work::ForgeSettings(out));
    super::judge_trains(domain, env);
}

#[test]
fn only_the_current_last_blocking_gate_at_the_head_releases_the_change() {
    for (head, sender, other, blocking, released) in [
        (super::SHA1, GATE_TASK, change::Status::Passed, true, true),
        (super::SHA256, GATE_TASK, change::Status::Passed, true, false),
        (super::SHA1, NEXT_GATE_TASK, change::Status::Passed, true, false),
        (super::SHA1, GATE_TASK, change::Status::Pending, true, false),
        (super::SHA1, GATE_TASK, change::Status::Passed, false, false),
    ] {
        let (mut domain, env) = ready_changes(&[WAITER, CANDIDATE], 3);
        gating(&mut domain, &env, change::Status::Pending);
        let mut row = domain.forge.change(CANDIDATE).unwrap().clone();
        let mut gate = *row.change.gates.first().unwrap();
        gate.blocking = blocking;
        row.change.gates = Box::new([gate, change::Gate { number: 2, blocking: true, ..gate }]);
        row.verdicts = Box::new([change::GateReport { revision: 0, number: 2, head: super::SHA1, status: other }]);
        restore(&mut domain, &env, row);
        let out = release_gate(&mut domain, &env, head, GATE_TASK, sender);
        assert_eq!(domain.forge.change(CANDIDATE).unwrap().settling, released);
        let mut saved_mark = false;
        for request in out.into_boxed() {
            if let forge::Request::Save { record: forge::Stored::ChangeHot(hot), .. } = request
                && hot.task == CANDIDATE
                && hot.settling
            {
                saved_mark = true;
            }
        }
        assert_eq!(saved_mark, released, "the release is saved in the decision that takes the result");
    }
}

#[test]
fn a_change_that_waits_leaves_the_cohort_and_a_later_release_can_join_again() {
    let (mut domain, env) = ready_changes(&[WAITER, CANDIDATE], 3);
    gating(&mut domain, &env, change::Status::Pending);
    let out = release_gate(&mut domain, &env, super::SHA1, GATE_TASK, GATE_TASK);
    domain.work.push(Work::ForgeSettings(out));
    super::judge_trains(&mut domain, &env);
    assert!(domain.forge.landing_of(WAITER).is_none());

    // The first decision waits (for example, for a sign-off); it didn't become ready.
    // Keep forming withheld so the next release can be observed before it forms.
    domain.hold_landings(true);
    finish_wait(&mut domain, &env);
    assert!(!domain.forge.change(CANDIDATE).unwrap().settling);
    wake(&mut domain, &env, CANDIDATE);
    assert!(!domain.forge.change(CANDIDATE).unwrap().settling, "a wake is not a release");

    let mut row = domain.forge.change(CANDIDATE).unwrap().clone();
    row.change.state = change::State::Gating { head: super::SHA256, asked: Box::new([1]) };
    row.change.last_head = Some(super::SHA256);
    restore(&mut domain, &env, row);
    let out = release_gate(&mut domain, &env, super::SHA256, NEXT_GATE_TASK, NEXT_GATE_TASK);
    domain.work.push(Work::ForgeSettings(out));
    domain.hold_landings(false);
    super::judge_trains(&mut domain, &env);
    assert!(domain.forge.landing_of(WAITER).is_none(), "the newly released head settles");
    finish_wait(&mut domain, &env);
    let landing = domain.forge.landing_of(WAITER).expect("a change that did not become ready leaves the cohort");
    assert_eq!(members(&domain, landing).as_ref(), [WAITER]);
}

fn gate_released_with_a_waiter() -> (Domain, Env<Limits>) {
    let (mut domain, env) = ready_changes(&[WAITER, CANDIDATE, DEPENDENCY], 3);
    gating(&mut domain, &env, change::Status::Pending);
    let mut other = domain.forge.change(DEPENDENCY).unwrap().clone();
    other.change.state = change::State::Checking { head: super::SHA1 };
    restore(&mut domain, &env, other);
    let out = release_gate(&mut domain, &env, super::SHA1, GATE_TASK, GATE_TASK);
    domain.work.push(Work::ForgeSettings(out));
    super::judge_trains(&mut domain, &env);
    assert!(domain.forge.change(CANDIDATE).unwrap().settling, "the gate result released the candidate");
    (domain, env)
}

fn candidate_left(domain: &mut Domain, env: &Env<Limits>, changed: &tasks::TaskRecord) {
    let mut out = Queue::uncapped();
    super::super::landing_route::cohort_saved(domain, env, changed, &mut out);
    domain.work.push(Work::ForgeSettings(out));
    super::judge_trains(domain, env);
    assert!(!domain.forge.change(CANDIDATE).unwrap().settling, "the unavailable change left the cohort");
    let landing = domain.forge.landing_of(WAITER).expect("an unavailable change holds no train");
    assert_eq!(members(domain, landing).as_ref(), [WAITER]);
}

#[test]
fn a_held_closing_or_ended_candidate_leaves_its_cohort() {
    let ending = tasks::Ending::Failed { reason: b"Stopped".as_slice().into() };
    for phase in [
        tasks::Phase::Held { was: tasks::Was::Waiting, why: tasks::Hold::Stopped, source: None },
        tasks::Phase::Closing(tasks::Closing { stage: tasks::Stage::Effects, ending: ending.clone() }),
        tasks::Phase::Ended(ending),
    ] {
        let (mut domain, env) = gate_released_with_a_waiter();
        let mut task = domain.engine.core().tasks.task(CANDIDATE).unwrap().clone();
        task.phase = phase;
        candidate_left(&mut domain, &env, &task);
    }
}

#[test]
fn a_new_pending_dependency_removes_a_candidate_from_its_cohort() {
    let (mut domain, env) = gate_released_with_a_waiter();
    let mut task = domain.engine.core().tasks.task(CANDIDATE).unwrap().clone();
    task.dependencies = Box::new([DEPENDENCY]);
    task.waiting_on = Box::new([DEPENDENCY]);
    restore_task(&mut domain, &env, task.clone());
    candidate_left(&mut domain, &env, &task);
}

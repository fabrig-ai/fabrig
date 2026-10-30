//! Start selection and phase discipline (jig/docs/design/domain/engine.md, 6).
use super::{Core, Env, Queue, Time, Wall, config, limits, tasks};
use crate::start::{Action, Failure, Phase, SweepInput};
use crate::{Key, Request, Write, people};

#[test]
fn actual_stored_call_capacity_keeps_its_key() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let capacity = limits.call_records;
    for task in 1..=u64::from(capacity) {
        assert!(core.restore_call(crate::CallKey { task, attempt: 1, completion: 1, position: 0 }, crate::CallPart::Controlled));
    }
    let key = crate::CallKey { task: u64::from(capacity) + 1, attempt: 1, completion: 1, position: 0 };
    assert!(!core.restore_call(key, crate::CallPart::Controlled));
    assert_eq!(core.start_failure(), Some(Failure::LiveSet {
        set: crate::start::LiveSet::Calls, key: Key::Core(crate::CoreKey::Call(key)),
        held: u64::from(capacity) + 1, limit: u64::from(capacity),
    }));
}

#[test]
#[should_panic]
fn empty_finishing_is_an_input_before_selection() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _outputs = crate::finish_decision(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits });
    let _late = core.start_begin(false, Wall::EPOCH);
}

#[test]
#[should_panic]
fn runtime_resume_cannot_cross_wires_early() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let _outputs = crate::resume_fleet(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits });
}

#[test]
fn actual_account_outputs_wait_for_wires_release() {
    let limits = limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, env.wall);
    super::restore_key_account(&mut core, &limits, 1, 2, 1);
    let mut child = Queue::uncapped();
    crate::accounts::step(
        &mut core.accounts,
        &Env { now: env.now, wall: env.wall, limits: limits.accounts },
        crate::accounts::Event::Restore { record: crate::accounts::Stored::Next { number: 3 } },
        &mut child,
    );
    assert!(child.is_empty());
    let mut out = Queue::uncapped();
    through_link(&mut core, &env, &mut out);
    assert!(out.is_empty());
    assert!(!core.start.held.is_empty());
    step_phase(core.start_sweep(&env, SweepInput::Finished, &mut out), Phase::Switch);
    step_phase(core.start_done(&env, Phase::Switch, &mut out), Phase::Wires);
    let Action::Commit(token) = core.start_done(&env, Phase::Wires, &mut out) else { panic!("Wires waits for release") };
    assert!(out.is_empty());
    step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
    let mut signing = false;
    let mut component = false;
    for _ in 0..out.len() {
        let request = out.pop().expect("released account output");
        if let Request::Now(now) = request {
            match *now {
                crate::Now::Account(crate::accounts::Request::Signing(crate::accounts::Signing::Restore { source, .. })) => {
                    assert_eq!(source, 2);
                    signing = true;
                }
                crate::Now::Account(crate::accounts::Request::Component(crate::accounts::ToComponent::Want { account })) => {
                    assert_eq!(account, 2);
                    component = true;
                }
                _ => {}
            }
        }
    }
    assert!(signing && component);
    assert!(core.start.held.is_empty());
}

#[test]
#[should_panic]
fn a_loaded_header_prevents_late_selection() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let header = core.counters.deployment();
    assert!(matches!(core.restore_core(crate::CoreRecord::Deployment(header), &limits), crate::Restored::Deployment { .. }));
    let _late = core.start_begin(false, Wall::EPOCH);
}

fn step_phase(action: Action, phase: Phase) {
    assert!(matches!(action, Action::Step(found) if found == phase));
}

fn through_link(core: &mut Core, env: &Env<crate::Limits>, out: &mut Queue<Request>) {
    step_phase(core.start_done(env, Phase::LoadCore, out), Phase::LoadInbox);
    let connectors = core.connectors.clone();
    let mut phase = Phase::LoadInbox;
    for connector in connectors {
        let next = Phase::RestoreConnector { connector };
        step_phase(core.start_done(env, phase, out), next);
        phase = next;
    }
    step_phase(core.start_done(env, phase, out), Phase::Link);
    step_phase(core.start_done(env, Phase::Link, out), Phase::Sweep);
}

#[test]
fn every_start_reaches_switch_and_waits_for_wires() {
    let limits = limits();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    for pending in [false, true] {
        for connectors in [Box::<[u16]>::from([]), Box::from([1, 2])] {
            let mut configured = config(41);
            configured.connectors = connectors;
            configured.grants = Box::new([]);
            let mut core = Core::new(configured, &limits).expect("valid core configuration");
            let mut out = Queue::uncapped();
            step_phase(core.start_begin(pending, env.wall), Phase::LoadCore);
            through_link(&mut core, &env, &mut out);
            step_phase(core.start_sweep(&env, SweepInput::Finished, &mut out), Phase::Switch);
            step_phase(core.start_done(&env, Phase::Switch, &mut out), Phase::Wires);
            let Action::Commit(token) = core.start_done(&env, Phase::Wires, &mut out) else { panic!("Wires waits for release") };
            assert_eq!(core.start_phase(), Some(Phase::Wires));
            assert!(!core.people.ready());
            assert!(out.is_empty());
            step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
            assert!(core.people.ready());
        }
    }
}

#[test]
fn frozen_expiry_and_stale_cleanup_continuation() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let mut env = Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits };
    let mut out = Queue::uncapped();
    step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    let row = people::Stored::SignIn { number: 7, person: 99, expires: Wall::from_nanos(5) };
    let mut child = Queue::uncapped();
    people::step(
        &mut core.people,
        &Env { now: env.now, wall: env.wall, limits: limits.people },
        people::Event::Restore { record: row.clone() },
        &mut child,
    );
    assert!(child.is_empty());
    through_link(&mut core, &env, &mut out);
    env.wall = Wall::EPOCH;
    let Action::Commit(token) = core.start_sweep(&env, SweepInput::Record(row), &mut out) else { panic!("expired row commits") };
    assert!(matches!(out.pop(), Some(Request::Write(Write::Erase(Key::People(people::Key::SignIn(7)))))));
    assert!(out.is_empty());
    let stale = token.clone();
    step_phase(core.start_resume(&env, token, &mut out), Phase::Sweep);
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| core.start_resume(&env, stale, &mut out)));
    assert!(rejected.is_err());
    assert!(out.is_empty());
    assert_eq!(core.start_phase(), Some(Phase::Sweep));
}

#[test]
#[should_panic]
fn first_external_missing_link_stops_before_sweep() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    let _begin = core.start_begin(false, env.wall);
    let record = Key::Tasks(tasks::Key::Live(1));
    let target = Key::People(people::Key::Person(99));
    core.start_missing(record.clone(), target.clone());
    core.start_missing(Key::Tasks(tasks::Key::Live(2)), Key::People(people::Key::Person(98)));
    assert_eq!(core.start.missing, Some((record, target)));
    through_link(&mut core, &env, &mut out);
}

#[test]
fn begin_selects_before_the_people_lifecycle_input() {
    let limits = limits();
    for pending in [false, true] {
        let mut core = Core::new(config(41), &limits).expect("valid core configuration");
        assert!(matches!(core.start_begin(pending, Wall::EPOCH), Action::Step(Phase::LoadCore)));
        assert_eq!(core.start_phase(), Some(Phase::LoadCore));
        assert_eq!(core.start_failure(), None);
    }
}

#[test]
#[should_panic]
fn begin_is_selected_once() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _first = core.start_begin(false, Wall::EPOCH);
    let _second = core.start_begin(false, Wall::EPOCH);
}

#[test]
fn first_failure_is_sticky() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let _begin = core.start_begin(false, Wall::EPOCH);
    let first = Failure::ProjectPolicy { project: 1 };
    assert!(matches!(core.start_refuse(first.clone()), Action::Refused(failure) if failure == first));
    assert!(matches!(core.start_refuse(Failure::ProjectPolicy { project: 2 }), Action::Refused(failure) if failure == first));
    assert_eq!(core.start_failure(), Some(first));
}

#[test]
#[should_panic]
fn direct_resource_input_prevents_late_start_selection() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("valid core configuration");
    let name = tasks::Name { connector: core.connectors[0], path: Box::new([Box::from(b"resource".as_slice())]) };
    let mut out = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
        tasks::Event::Resource { name: name.clone(), hold: tasks::HoldKind::Shared },
        &mut out,
    );
    assert!(out.is_empty());
    assert_eq!(core.tasks.resource_hold(&name), Some(tasks::HoldKind::Shared));
    core.tasks.start_restore();
}

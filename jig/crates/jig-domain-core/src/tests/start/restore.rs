//! The core owns the start's record families and child sequencing (domain/engine.md, 6).
use super::{
    Action, Core, Env, Phase, Queue, Request, SweepInput, Time, Wall, config, limits, people, step_phase, tasks,
};
use crate::{CoreRecord, Record, Restore, Write, accounts};

fn loading() -> (Core, Env<crate::Limits>) {
    let limits = limits();
    let mut core = Core::new(config(401), &limits).expect("configured core");
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits };
    step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    let mut header = core.counters.deployment();
    header.commits = 7;
    header.tasks = 100;
    header.people = 100;
    header.projects = 1;
    assert_eq!(core.restore(&env, Record::Core(CoreRecord::Deployment(header))), Restore::Deployment { commits: 7 });
    assert_eq!(core.counters.deployment(), header);
    (core, env)
}

#[test]
fn the_deployment_row_returns_the_journals_commit_number() {
    let (core, _) = loading();
    assert_eq!(core.counters.deployment().commits, 7);
}

#[test]
fn a_refused_family_keeps_its_failure_and_stops_later_rows() {
    let (mut core, env) = loading();
    let row = people::Stored::Person {
        number: u64::from(env.limits.people.people) + 1,
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
            login: Box::new([]),
            name: Box::new([]),
        },
    };
    // Fill the table to its configured limit, then refuse the next identity.
    for number in 1..=env.limits.people.people {
        let mut row = row.clone();
        let people::Stored::Person { number: stored, identity } = &mut row else { panic!("person row") };
        *stored = u64::from(number);
        identity.key.subject = u64::from(number).to_le_bytes().into();
        assert_eq!(core.restore(&env, Record::People(row)), Restore::Kept);
    }
    assert_eq!(core.restore(&env, Record::People(row)), Restore::Refused);
    let failure = core.start_failure().expect("the refused family names its limit");
    assert_eq!(core.restore(&env, Record::Accounts(accounts::Stored::Next { number: 9 })), Restore::Refused);
    assert_eq!(core.start_failure(), Some(failure));
}

#[test]
fn notes_rows_keep_no_restored_state() {
    let (mut core, env) = loading();
    let row = crate::notes::Record::Line(crate::notes::Line {
        scope: crate::notes::Scope::Deployment,
        name: 1,
        description: Box::new([]),
        revision: 1,
    });
    assert_eq!(core.restore(&env, Record::Notes(row)), Restore::Kept);
    assert_eq!(core.start_failure(), None);
}

#[test]
fn a_task_row_restores_the_child_and_its_run_correlation() {
    let (mut source, limits) = super::super::tree_core();
    super::super::make_task(&mut source, &limits, 1);
    let mut row = source.tasks.task(1).expect("source task").clone();
    row.attempt = 3;
    let (mut core, env) = loading();
    assert_eq!(core.restore(&env, Record::Tasks(tasks::Stored::Live(Box::new(row.clone())))), Restore::Kept);
    assert_eq!(core.tasks.task(1), Some(&row));
    assert_eq!(core.restoring_proofs.get(&1).expect("run correlation").attempt, 3);
}

#[test]
fn a_policy_row_restores_the_project_and_its_name() {
    let (mut core, env) = loading();
    let value = crate::policy::snapshot(&config(401).seed_projects[0].policy.policy, &[], &[]).expect("stored policy");
    assert_eq!(
        core.restore(&env, Record::People(people::Stored::Policy { project: 1, name: "Project 1".into(), value })),
        Restore::Kept
    );
    assert_eq!(core.names.get(&1).expect("project name").as_ref(), "Project 1");
    assert!(core.authority.policy(1).is_some());
}

#[test]
fn a_people_row_reaches_the_people_child() {
    let (mut core, env) = loading();
    let identity = people::Identity {
        key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
        login: Box::new([]),
        name: Box::new([]),
    };
    assert_eq!(
        core.restore(&env, Record::People(people::Stored::Person { number: 1, identity: identity.clone() })),
        Restore::Kept
    );
    assert_eq!(core.people.identity(1), Some(&identity));
}

#[test]
fn an_accounts_row_reaches_the_accounts_child() {
    let (mut core, env) = loading();
    let record = accounts::Stored::Account {
        number: 1,
        provider: 0,
        holder: accounts::Holder::Deployment,
        place: 0,
        uses: None,
        subject: Box::new([]),
        display: Box::new([]),
    };
    assert_eq!(core.restore(&env, Record::Accounts(record)), Restore::Kept);
    assert!(core.accounts.account(1).is_some());
}

#[test]
fn an_inbox_row_uses_the_inbox_phase() {
    let (mut core, env) = super::inbox::loaded();
    super::inbox::inbox(&mut core, &env);
    let row = super::inbox::ended();
    assert_eq!(core.restore(&env, Record::Tasks(tasks::Stored::Ended(Box::new(row)))), Restore::Kept);
    assert_eq!(core.people.cached_updates(&env.limits.people, 1).expect("inbox").len(), 1);
}

#[test]
fn sweep_rows_wait_inside_the_core_for_their_decisions() {
    let limits = limits();
    let mut core = Core::new(config(401), &limits).expect("configured core");
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits };
    let mut out = Queue::uncapped();
    step_phase(core.start_begin(false, env.wall), Phase::LoadCore);
    super::through_link(&mut core, &env, &mut out);
    for number in [7, 8] {
        assert_eq!(
            core.restore(
                &env,
                Record::People(people::Stored::SignIn { number, person: 99, expires: Wall::from_nanos(5) })
            ),
            Restore::Swept
        );
    }
    for number in [7, 8] {
        let Action::Commit(token) = core.start_sweep(&env, SweepInput::Next, &mut out) else {
            panic!("each expired row needs its own decision")
        };
        let Some(Request::Write(Write::Erase(key))) = out.pop() else { panic!("expiry erase") };
        assert_eq!(key, crate::Key::People(people::Key::SignIn(number)));
        assert!(out.is_empty());
        step_phase(core.start_resume(&env, token, &mut out), Phase::Sweep);
    }
    step_phase(core.start_sweep(&env, SweepInput::Finished, &mut out), Phase::Switch);
}

#[test]
fn adopted_runs_reach_the_fleet_without_a_root_event() {
    let (mut core, env) = super::runs::loaded(false, 1);
    let mut out = Queue::uncapped();
    let token = super::runs::wires(&mut core, &env, &mut out);
    step_phase(core.start_resume(&env, token, &mut out), Phase::AdoptRuns);
    assert_eq!(core.adopted.len(), 2);
    assert_eq!(core.fleet.attempts(), 0);
    let Action::Step(Phase::ReadAfresh { .. }) = core.start_done(&env, Phase::AdoptRuns, &mut out) else {
        panic!("the fleet adopts before connector reads")
    };
    assert!(core.adopted.is_empty());
    assert!(core.unreported_restored.contains_key(&2));
    assert_eq!(core.fleet.attempts(), 1);
    assert_eq!(core.tasks.task(1).expect("engine claim settled").refusals, 1);
    assert!(!core.start_opened());
}

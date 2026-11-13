//! Goal reds at a run's check start (domain/forge/reds.md, 5.1).
use super::{adopted, change_row, change_start_fixture};
use crate::{Domain, Limits, RunRed, reds_for_run};
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Token};
use temper_engine_domain_forge as forge;

fn proof(domain: &mut Domain, task: u64) {
    domain
        .core
        .proofs
        .insert(
            task,
            jig_domain_core::RunProof {
                task,
                attempt: 1,
                conversation: 1,
                credentials: Box::new([]),
                accounts: Box::new([]),
                placement: Some(Token::new(7)),
                account_failure: None,
                account_terminal: false,
                window: None,
                crossed: false,
                host: jig_domain_fleet::HostKind::Worker,
                offered: None,
                outstanding: Box::new([]),
                turn: None,
                terminal: None,
            },
        )
        .unwrap();
}
fn restore(domain: &mut Domain, env: &Env<Limits>, row: forge::Stored) {
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: row },
        &mut Queue::uncapped(),
    );
}
fn hold(domain: &mut Domain, env: &Env<Limits>, task: u64, branch: &[u8]) {
    restore(
        domain,
        env,
        forge::Stored::Hold(forge::Hold {
            name: forge::Name {
                forge: 1,
                repository: 1,
                what: super::super::branch_what(branch, env.limits.forge.name_bytes).unwrap(),
            },
            task,
            writer: None,
            design_writers: Box::new([]),
            drift: None,
        }),
    );
}
fn fixture(base: &[u8]) -> (Domain, Env<Limits>) {
    let (mut domain, mut env, context) = change_start_fixture();
    env.limits.tasks.tasks = 8;
    env.limits.tasks.depth = 3;
    domain.core.proofs = skein_lib::Map::with_capacity(8);
    let template = domain.core.tasks.task(context.task).unwrap().clone();
    domain.core.tasks = tasks::Domain::new(
        &env.limits.tasks,
        tasks::Kept { revisions: env.limits.settings.revisions, versions: env.limits.settings.versions },
        7,
        0,
        Box::new([]),
        Box::new([]),
    );
    for (number, parent, depth) in [
        (7, None, 0),
        (8, None, 0),
        (20, Some(8), 1),
        (21, Some(20), 2),
        (22, Some(20), 2),
        (23, Some(20), 2),
        (24, Some(23), 3),
    ] {
        let mut row = template.clone();
        row.number = number;
        row.root = 8;
        row.depth = depth;
        if let Some(parent) = parent {
            row.requester = tasks::Party::Task(parent);
        }
        tasks::step(
            &mut domain.core.tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
            &mut Queue::uncapped(),
        );
        assert!(domain.core.tasks.task(number).is_some(), "task {number}: {:?}", domain.core.tasks.restore_failure());
        proof(&mut domain, number);
    }
    hold(&mut domain, &env, 7, b"temper/8/g7");
    hold(&mut domain, &env, 8, b"temper/8/g8");
    for (goal, key, byte) in [(7, b"z", 3), (8, b"a", 8), (7, b"a", 4)] {
        domain
            .forge
            .record_red(
                &env.limits.forge,
                forge::RecordRed {
                    goal,
                    key: key.as_slice().into(),
                    head: Box::new([byte; 20]),
                    why: b"base failure".as_slice().into(),
                    by: forge::Recorder::Task(goal),
                    at: 9,
                },
                &mut Queue::uncapped(),
            )
            .unwrap();
    }
    let mut change = change_row(None);
    change.task = 20;
    change.change.task = 20;
    change.repository = adopted(1).provider;
    change.base = base.into();
    change.goal = Some(8); // The destination's holding, not this historical hint, selects the reds.
    restore(&mut domain, &env, forge::Stored::Change(Box::new(change)));
    (domain, env)
}
fn expected() -> Box<[RunRed]> {
    Box::new([
        RunRed { key: b"a".as_slice().into(), head: Box::new([4; 20]) },
        RunRed { key: b"z".as_slice().into(), head: Box::new([3; 20]) },
    ])
}
#[test]
fn change_producer_repair_and_resolution_read_the_goal_holding_their_changes_base() {
    let (domain, _) = fixture(b"temper/8/g7");
    for task in [20, 21, 22, 23, 24] {
        assert_eq!(reds_for_run(&domain, task, 1), expected(), "task {task}");
    }
}
#[test]
fn a_goals_own_run_reads_its_own_reds_even_when_its_change_lands_in_main() {
    let (mut domain, env) = fixture(b"main");
    let mut change = change_row(None);
    change.task = 7;
    change.change.task = 7;
    change.repository = adopted(1).provider;
    restore(&mut domain, &env, forge::Stored::Change(Box::new(change)));
    assert_eq!(reds_for_run(&domain, 7, 1), expected());
    assert_eq!(
        reds_for_run(&domain, 8, 1).as_ref(),
        &[RunRed { key: b"a".as_slice().into(), head: Box::new([8; 20]) }]
    );
}
#[test]
fn reads_for_main_nonlive_and_stale_attempts_are_empty() {
    let (mut domain, env) = fixture(b"main");
    // A holder of main who also holds a goal branch is not main's goal.
    hold(&mut domain, &env, 8, b"main");
    assert!(reds_for_run(&domain, 22, 1).is_empty());
    proof(&mut domain, 999);
    assert!(reds_for_run(&domain, 999, 1).is_empty());
    assert!(reds_for_run(&domain, 7, 2).is_empty());
    assert!(reds_for_run(&domain, 500, 1).is_empty());
}

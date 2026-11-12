//! Settled panels give their room to new changes (domain/forge/change/README.md, 7).
use super::{change_row, change_start_fixture};
use crate::{Decision, Key, Record, Write, route::forge_route::start_change};
use skein_lib::{Env, Queue};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;

const FIRST: u64 = 100;
const KEPT: u32 = 128;

fn kept(count: u32) -> (crate::Domain, Env<crate::Limits>, Box<jig_domain_tasks::RunContext>) {
    let (mut domain, mut env, context) = change_start_fixture();
    env.limits.forge.changes = KEPT;
    let config = crate::tests::resume_fixture::config(7);
    domain.forge =
        forge::Domain::new(&env.limits.forge, 4, config.forge, &config.identities, &config.forges, false).unwrap();
    domain.forge.restore_begin(env.wall);
    let mut repository = super::adopted(1);
    repository.project = context.project;
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Repository(repository.clone()) },
        &mut Queue::uncapped(),
    );
    // Reverse insertion makes the oldest choice independent of insertion order.
    for number in (FIRST..FIRST + u64::from(count)).rev() {
        keep(&mut domain, &env, number, change::State::Ended, 999);
    }
    (domain, env, context)
}

fn keep(domain: &mut crate::Domain, env: &Env<crate::Limits>, number: u64, state: change::State, goal: u64) {
    let mut row = change_row(None);
    row.task = number;
    row.change.task = number;
    row.repository = super::adopted(1).provider;
    row.change.state = state;
    row.ending = Some(forge::Ending::Cancelled);
    row.goal = Some(goal);
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
}

fn verifying_table(settled: bool) -> (crate::Domain, Env<crate::Limits>, Box<jig_domain_tasks::RunContext>) {
    let (mut domain, env, context) = kept(0);
    let pinned = if settled { KEPT - 3 } else { KEPT - 2 };
    for number in (FIRST..FIRST + u64::from(pinned)).rev() {
        keep(&mut domain, &env, number, change::State::Uncleared { merge: [1; 32] }, context.task);
    }
    keep(&mut domain, &env, 9, change::State::Red { merge: [1; 32], since: env.wall }, 999);
    keep(&mut domain, &env, 5, change::State::Uncleared { merge: [1; 32] }, 999);
    if settled {
        keep(&mut domain, &env, 50, change::State::Ended, 999);
    }
    (domain, env, context)
}

fn run_start(domain: &mut crate::Domain, env: &Env<crate::Limits>, context: &jig_domain_tasks::RunContext) {
    assert!(start_change(domain, env, context));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_into(domain, env, &mut decision);
    crate::domain::close(domain, env, decision, &mut Queue::uncapped());
}

#[test]
fn free_room_keeps_every_settled_panel() {
    let (mut domain, env, context) = kept(KEPT - 1);
    run_start(&mut domain, &env, &context);
    assert!(domain.forge.change(context.task).is_some());
    for number in FIRST..FIRST + u64::from(KEPT - 1) {
        assert!(domain.forge.change(number).is_some(), "settled panel {number}");
    }
}

fn pin_goal(
    domain: &mut crate::Domain,
    env: &Env<crate::Limits>,
    context: &jig_domain_tasks::RunContext,
    number: u64,
    explicit: bool,
) {
    use jig_domain_tasks as tasks;
    let mut goal = domain.core.tasks.task(context.task).unwrap().clone();
    domain.core.tasks = crate::Domain::new(crate::tests::resume_fixture::config(7), &env.limits).unwrap().core.tasks;
    goal.tracked = Some(1);
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Restore { record: tasks::Stored::Live(Box::new(goal.clone())) },
        &mut Queue::uncapped(),
    );
    let mut row = domain.forge.change(number).unwrap().clone();
    row.goal = if explicit { Some(context.task) } else { None };
    if !explicit {
        goal.number = number;
        goal.tracked = None;
        goal.requester = tasks::Party::Task(context.task);
        tasks::step(
            &mut domain.core.tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(goal)) },
            &mut Queue::uncapped(),
        );
    }
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
}

#[test]
fn a_live_goal_pins_the_oldest_panel_by_row_or_tracked_ancestor() {
    for explicit in [true, false] {
        let (mut domain, env, context) = kept(KEPT);
        pin_goal(&mut domain, &env, &context, FIRST, explicit);
        assert!(start_change(&mut domain, &env, &context));
        assert!(domain.forge.change(FIRST).is_some(), "live goal, explicit {explicit}");
        assert!(domain.forge.change(FIRST + 1).is_none(), "next oldest, explicit {explicit}");
        assert!(domain.forge.change(context.task).is_some());
    }
}

#[test]
fn a_live_goal_pins_a_dropped_last_panel_by_row_or_tracked_ancestor() {
    for explicit in [true, false] {
        let (mut domain, env, context) = verifying_table(false);
        pin_goal(&mut domain, &env, &context, 5, explicit);
        assert!(start_change(&mut domain, &env, &context));
        assert!(domain.forge.change(5).is_some(), "live goal, explicit {explicit}");
        assert!(domain.forge.change(9).is_none(), "next oldest, explicit {explicit}");
        assert!(domain.forge.change(context.task).is_some());
    }
}

#[test]
fn a_settled_panel_goes_before_an_older_ended_verifying_panel() {
    let (mut domain, env, context) = verifying_table(true);
    run_start(&mut domain, &env, &context);
    assert!(domain.forge.change(context.task).is_some(), "the 129th change registers");
    assert!(domain.forge.change(50).is_none(), "a settled task goes first");
    assert!(domain.forge.change(5).is_some(), "the older Uncleared panel stays");
    assert!(domain.forge.change(9).is_some(), "the Red panel stays");
    assert_eq!(domain.forge.change_tasks().len(), 128);
}

#[test]
fn a_gone_goals_verifying_panel_gives_room_beside_live_goals_settled_panels() {
    let (mut domain, env, context) = kept(KEPT);
    for number in domain.forge.change_tasks() {
        keep(&mut domain, &env, number, change::State::Ended, context.task);
    }
    keep(&mut domain, &env, FIRST + 1, change::State::Uncleared { merge: [1; 32] }, 999);
    run_start(&mut domain, &env, &context);
    assert!(domain.forge.change(context.task).is_some(), "the new change registers");
    assert!(domain.forge.change(FIRST + 1).is_none(), "the root's selected row goes");
    assert!(domain.forge.change(FIRST).is_some(), "the live goal's settled panel stays");
    assert_eq!(domain.forge.change_tasks().len(), skein_lib::usize_of(KEPT));
}

#[test]
fn failed_registration_queues_only_the_named_refusal_without_a_hold_or_step() {
    let (mut domain, env, context) = kept(KEPT);
    for number in domain.forge.change_tasks() {
        keep(&mut domain, &env, number, change::State::Ended, context.task);
    }
    assert!(start_change(&mut domain, &env, &context));
    let Some(crate::Work::ForgeSettings(mut out)) = domain.work.pop() else {
        panic!("registration is answered before any hold or step is queued")
    };
    assert_eq!(
        out.pop(),
        Some(forge::Request::ResourceRefused {
            task: context.task,
            operation: b"register change",
            resource: Some(forge::Name { forge: 1, repository: 1, what: forge::What::Repository }),
            cause: forge::RefusalCause::Limit {
                member: "forge.changes",
                configured: u64::from(KEPT),
                needed: u64::from(KEPT) + 1,
            },
        })
    );
    assert!(out.is_empty());
    assert!(domain.work.is_empty(), "a refused registration queues no names, hold or step");
    assert!(domain.forge.change(context.task).is_none());
}

#[test]
fn a_full_pinned_table_holds_with_the_named_limit_and_release_registers_after_unpinning() {
    use jig_domain_tasks as tasks;
    let (mut domain, env, context) = kept(KEPT);
    for number in domain.forge.change_tasks() {
        let mut row = domain.forge.change(number).unwrap().clone();
        row.goal = Some(context.task);
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
            &mut Queue::uncapped(),
        );
    }
    run_start(&mut domain, &env, &context);
    assert!(domain.forge.change(context.task).is_none());
    let tasks::Phase::Held { source: Some(source), .. } = &domain.core.tasks.task(context.task).unwrap().phase else {
        panic!("the new change is held")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("forge hold words") };
    assert!(skein_lib::bytes::find(words, b"forge.changes = 128; required usage 129; reduce the named work or raise this limit in the deployment configuration, restart, then release").is_some(), "{words:?}");
    let mut row = domain.forge.change(FIRST).unwrap().clone();
    row.goal = Some(999);
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut Queue::uncapped(),
    );
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Release {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(999)),
            by: tasks::Party::Person(1),
            operation: tasks::Release { task: context.task, again: None, message: None },
        },
        &mut Queue::uncapped(),
    );
    assert_eq!(domain.core.tasks.task(context.task).unwrap().due_reason, tasks::Why::Released);
    run_start(&mut domain, &env, &context);
    assert!(domain.forge.change(context.task).is_some());
    assert!(domain.forge.change(FIRST).is_none());
}

#[test]
fn the_129th_change_replaces_the_oldest_ended_verifying_panel_in_one_decision() {
    let (mut domain, env, context) = verifying_table(false);
    assert_eq!(domain.forge.change_tasks().len(), 128);
    assert!(start_change(&mut domain, &env, &context));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_into(&mut domain, &env, &mut decision);
    assert!(domain.forge.change(context.task).is_some(), "the 129th change registers");
    assert!(domain.forge.change(5).is_none(), "the oldest ended verifying task gives its room");
    assert!(domain.forge.change(9).is_some(), "the newer Red panel stays");
    assert_eq!(domain.forge.change_tasks().len(), 128);
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
    let Some(crate::decision::JournalOutput::Commit { writes, .. }) =
        crate::decision::commit(&mut domain.journal, &env.limits.journal)
    else {
        panic!("one root decision")
    };
    assert!(writes.contains(&Write::Erase(Key::Forge(forge::Key::Change(5)))));
    let mut registered = false;
    for write in &writes {
        if let Write::Save(Record::Forge { row }) = write
            && let forge::Stored::Change(row) = row.as_ref()
        {
            registered |= row.task == context.task;
        }
    }
    assert!(registered);
}

#[test]
fn the_129th_change_replaces_the_oldest_settled_panel_in_one_decision() {
    let (mut domain, env, context) = kept(KEPT);
    assert_eq!(domain.forge.change_tasks().len(), 128);
    assert!(start_change(&mut domain, &env, &context));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_into(&mut domain, &env, &mut decision);
    assert!(domain.forge.change(context.task).is_some(), "the 129th change registers");
    assert!(domain.forge.change(FIRST).is_none(), "the lowest task gives its room");
    assert_eq!(domain.forge.change_tasks().len(), 128);
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
    let Some(crate::decision::JournalOutput::Commit { writes, .. }) =
        crate::decision::commit(&mut domain.journal, &env.limits.journal)
    else {
        panic!("one root decision")
    };
    assert!(writes.contains(&Write::Erase(Key::Forge(forge::Key::Change(FIRST)))));
    let mut registered = false;
    for write in &writes {
        if let Write::Save(Record::Forge { row }) = write
            && let forge::Stored::Change(row) = row.as_ref()
        {
            registered |= row.task == context.task;
        }
    }
    assert!(registered);
}

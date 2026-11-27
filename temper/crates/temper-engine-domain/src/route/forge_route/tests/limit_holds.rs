//! Named holds retain the exact task revision and give freed room to the oldest waiter.
use super::adopted;
use alloc::boxed::Box;
const FIRST: u64 = 1;
const SECOND: u64 = 2;
// A separate connector name holder fills the table before either live waiter asks.
const THIRD: u64 = 3;
use crate::route::forge_route::{forge, tasks};
use skein_lib::{Env, Queue, ReplyTo, Token, Wall};

fn drive(domain: &mut crate::Domain, env: &Env<crate::Limits>, decision: &mut crate::Decision, event: forge::Event) {
    domain.work.push(crate::Work::Forge(event));
    crate::route::route_into(domain, env, decision);
}
fn kept_tasks(domain: &crate::Domain) -> Box<[u64]> {
    let mut tasks = skein_lib::List::uncapped();
    for row in domain.forge_limit_holds() {
        tasks.push(row.task).unwrap();
    }
    tasks.into_boxed()
}
fn held(domain: &crate::Domain, task: u64) -> bool {
    match domain.engine.core().tasks.task(task).unwrap().phase {
        tasks::Phase::Held { .. } => true,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    }
}
fn two_tasks() -> (crate::Domain, Env<crate::Limits>) {
    two_tasks_with_fresh_reads(true)
}
fn two_tasks_with_fresh_reads(read: bool) -> (crate::Domain, Env<crate::Limits>) {
    let (mut domain, env, source) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 1 });
    let original = domain.engine.core().tasks.task(source).unwrap().clone();
    let number = SECOND;
    {
        super::task_event(
            &mut domain,
            &env,
            tasks::Event::Make {
                reads_project: false,
                maker: None,
                places: Box::new([]),
                reply_to: ReplyTo::new(Token::new(number)),
                creator: tasks::Party::Deployment { project: 1 },
                batch: Box::new([tasks::New {
                    number,
                    project: 1,
                    label: 0,
                    flags: tasks::Flags::PLAIN,
                    executor: original.executor,
                    llm: None,
                    spec: original.spec,
                    contract: original.contract,
                    deadline: tasks::Deadline::Named,
                    authority: original.authority,
                    numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                    funder: original.funder,
                    dependencies: Box::new([]),
                    holdings: Box::new([]),
                    wake: tasks::WakePolicy::DEFAULT,
                    recurring: None,
                    tracked: None,
                }]),
            },
        );
        assert!(domain.engine.core().tasks.task(number).is_some());
    }
    let mut repository = adopted(1);
    repository.project = 1;
    domain.forge.restore_begin(env.wall);
    super::restore_forge(&mut domain, &env, forge::Stored::Repository(repository));
    if read {
        fresh(&mut domain, &env);
    }
    (domain, env)
}

fn fresh(domain: &mut crate::Domain, env: &Env<crate::Limits>) {
    for event in [forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, forge::Event::CatchUp]
    {
        forge::step(&mut domain.forge, &env.with(env.limits.forge), event, &mut Queue::uncapped());
    }
    assert!(domain.forge.fresh_reads_in());
}
#[test]
fn one_task_name_place_ends_only_the_oldest_limit_hold() {
    let (mut domain, mut env) = two_tasks();
    env.limits.forge.tasks = 1;
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    drive(&mut domain, &env, &mut decision, forge::Event::Names { task: THIRD, resources: Box::new([]) });
    for task in [SECOND, FIRST] {
        env.wall = Wall::from_nanos(1);
        drive(&mut domain, &env, &mut decision, forge::Event::Names { task, resources: Box::new([]) });
        assert!(held(&domain, task));
    }
    let kept = domain.forge_limit_holds();
    assert_eq!(kept_tasks(&domain).as_ref(), &[SECOND, FIRST]);
    assert_eq!((kept[0].configured, kept[0].needed), (1, 2));
    assert_eq!(kept[0].revision, domain.engine.core().tasks.task(SECOND).unwrap().revisions.hold);
    let tasks::Phase::Held { source: Some(source), .. } = &domain.engine.core().tasks.task(SECOND).unwrap().phase
    else {
        panic!("held task")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("limit account") };
    assert_eq!(words.as_ref(), b"watch resources: forge.tasks is 1, and this needs 2: a limit in temper-limits. Ask temper to raise it, or raise it from Needs you. This goes on by itself once room frees or the limit rises.");
    drive(&mut domain, &env, &mut decision, forge::Event::Unnamed { task: THIRD });
    assert_eq!(domain.engine.core().tasks.task(SECOND).unwrap().due_reason, tasks::Why::Released);
    assert!(!held(&domain, SECOND));
    assert!(held(&domain, FIRST));
    assert_eq!(kept_tasks(&domain).as_ref(), &[FIRST]);
    drive(&mut domain, &env, &mut decision, forge::Event::Names { task: SECOND, resources: Box::new([]) });
    assert_eq!(domain.forge.limit_usage(&forge::LimitSet::Tasks), Some(1));
}
#[test]
fn a_raise_ends_every_wait_even_when_only_one_operation_will_fit() {
    let (mut domain, mut env) = two_tasks();
    env.limits.forge.tasks = 1;
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    drive(&mut domain, &env, &mut decision, forge::Event::Names { task: THIRD, resources: Box::new([]) });
    for task in [FIRST, SECOND] {
        drive(&mut domain, &env, &mut decision, forge::Event::Names { task, resources: Box::new([]) });
        assert!(held(&domain, task));
    }
    env.limits.forge.tasks = 2;
    domain.end_forge_limit_holds(b"forge.tasks", 2);
    crate::route::route_into(&mut domain, &env, &mut decision);
    assert!(domain.forge_limit_holds().is_empty());
    for task in [FIRST, SECOND] {
        assert_eq!(domain.engine.core().tasks.task(task).unwrap().due_reason, tasks::Why::Released);
    }
}

#[test]
fn freed_resource_holds_and_task_names_resume_their_waiters() {
    for holds in [true, false] {
        let (mut domain, mut env) = two_tasks();
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        let first = if holds {
            forge::Event::Hold {
                entry: 0,
                task: FIRST,
                resource: forge::branch_name(adopted(1).provider, b"temper/1/c1"),
                from: None,
            }
        } else {
            forge::Event::Names { task: FIRST, resources: Box::new([]) }
        };
        drive(&mut domain, &env, &mut decision, first);
        let set = if holds { forge::LimitSet::Holds } else { forge::LimitSet::Tasks };
        assert_eq!(domain.forge.limit_usage(&set), Some(1));
        if holds {
            env.limits.forge.holds = 1;
        } else {
            env.limits.forge.tasks = 1;
        }
        let second = if holds {
            forge::Event::Hold {
                entry: 0,
                task: SECOND,
                resource: forge::branch_name(adopted(1).provider, b"temper/2/c2"),
                from: None,
            }
        } else {
            forge::Event::Names { task: SECOND, resources: Box::new([]) }
        };
        drive(&mut domain, &env, &mut decision, second);
        assert!(held(&domain, SECOND), "holds={holds}");
        let freed = if holds {
            forge::Event::Release {
                task: FIRST,
                root: FIRST,
                ending: forge::ReleaseEnding::Done,
                reason: None,
                entry: 500,
            }
        } else {
            forge::Event::Unnamed { task: FIRST }
        };
        drive(&mut domain, &env, &mut decision, freed);
        assert_eq!(domain.engine.core().tasks.task(SECOND).unwrap().due_reason, tasks::Why::Released, "holds={holds}");
        assert!(domain.forge_limit_holds().is_empty());
    }
}

fn refuse(
    domain: &mut crate::Domain,
    env: &Env<crate::Limits>,
    decision: &mut crate::Decision,
    task: u64,
    needed: u64,
) {
    let mut out = Queue::uncapped();
    out.push(forge::Request::ResourceRefused {
        task,
        operation: b"subscribe",
        resource: None,
        cause: forge::RefusalCause::Limit { member: "forge.paths_per_subscription", configured: 4, needed },
    });
    super::super::outputs(domain, env, decision, &mut out);
    crate::route::route_into(domain, env, decision);
}
fn task_event(
    domain: &mut crate::Domain,
    env: &Env<crate::Limits>,
    decision: &mut crate::Decision,
    event: tasks::Event,
) {
    domain.work.push(crate::Work::Tasks(event));
    crate::route::route_into(domain, env, decision);
}
#[test]
fn person_release_and_cancellation_remove_the_kept_rows() {
    let (mut domain, env) = two_tasks();
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    refuse(&mut domain, &env, &mut decision, FIRST, 5);
    refuse(&mut domain, &env, &mut decision, SECOND, 5);
    assert_eq!(domain.forge_limit_holds().len(), 2);
    task_event(
        &mut domain,
        &env,
        &mut decision,
        tasks::Event::Release {
            reply_to: ReplyTo::new(Token::new(800)),
            by: tasks::Party::Person(1),
            operation: tasks::Release { task: FIRST, again: None, message: None },
        },
    );
    assert_eq!(kept_tasks(&domain).as_ref(), &[SECOND]);
    task_event(
        &mut domain,
        &env,
        &mut decision,
        tasks::Event::Cancel {
            reply_to: ReplyTo::new(Token::new(801)),
            by: tasks::Party::Deployment { project: 1 },
            operation: tasks::Cancel {
                task: SECOND,
                reason: b"no longer needed".as_slice().into(),
                grace: None,
                message: None,
            },
        },
    );
    assert!(domain.forge_limit_holds().is_empty());
    domain.end_forge_limit_holds(b"forge.paths_per_subscription", 8);
    crate::route::route_into(&mut domain, &env, &mut decision);
    assert!(domain.forge_limit_holds().is_empty(), "no release is queued twice");
}
#[test]
fn a_stale_kept_revision_cannot_release_the_next_limit_hold() {
    let (mut domain, env) = two_tasks();
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    refuse(&mut domain, &env, &mut decision, FIRST, 5);
    let stale = domain.forge_limit_holds()[0].clone();
    task_event(
        &mut domain,
        &env,
        &mut decision,
        tasks::Event::Release {
            reply_to: ReplyTo::new(Token::new(800)),
            by: tasks::Party::Person(1),
            operation: tasks::Release { task: FIRST, again: None, message: None },
        },
    );
    refuse(&mut domain, &env, &mut decision, FIRST, 6);
    let revision = domain.engine.core().tasks.task(FIRST).unwrap().revisions.hold;
    assert!(revision > stale.revision);
    super::super::limit_holds::restore(&mut domain, stale);
    super::super::limit_holds::restart(&mut domain);
    crate::route::route_into(&mut domain, &env, &mut decision);
    assert!(domain.forge_limit_holds().is_empty());
    let task = domain.engine.core().tasks.task(FIRST).unwrap();
    assert_eq!(task.revisions.hold, revision);
    let tasks::Phase::Held { why, .. } = task.phase else { panic!("the next hold stays in place") };
    assert_eq!(why, tasks::Hold::ResourceUnavailable);
    task_event(&mut domain, &env, &mut decision, tasks::Event::Hold { task: FIRST, why: tasks::Hold::Stopped });
    let stopped = domain.engine.core().tasks.task(FIRST).unwrap().clone();
    refuse(&mut domain, &env, &mut decision, FIRST, 7);
    assert_eq!(
        domain.engine.core().tasks.task(FIRST),
        Some(&stopped),
        "a late limit refusal cannot replace a party's stop"
    );
    assert!(domain.forge_limit_holds().is_empty());
}

#[test]
fn reconcile_keeps_restored_limit_holds_until_fresh_reads_are_in() {
    let (mut domain, env) = two_tasks_with_fresh_reads(false);
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    refuse(&mut domain, &env, &mut decision, FIRST, 5);
    let mut stale = domain.forge_limit_holds()[0].clone();
    stale.task = 999;
    super::super::limit_holds::restore(&mut domain, stale);
    assert!(!domain.forge.fresh_reads_in());
    super::super::limit_holds::reconcile(&mut domain, &env, &mut decision);
    assert_eq!(domain.forge_limit_holds().len(), 2, "restored facts are not judged");
    fresh(&mut domain, &env);
    super::super::limit_holds::reconcile(&mut domain, &env, &mut decision);
    assert_eq!(kept_tasks(&domain).as_ref(), &[FIRST]);
}

#[test]
fn room_freed_waits_for_fresh_reads_before_releasing_limit_holds() {
    let (mut domain, mut env) = two_tasks_with_fresh_reads(false);
    env.limits.forge.tasks = 1;
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    drive(&mut domain, &env, &mut decision, forge::Event::Names { task: THIRD, resources: Box::new([]) });
    drive(&mut domain, &env, &mut decision, forge::Event::Names { task: FIRST, resources: Box::new([]) });
    assert!(held(&domain, FIRST));
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Unnamed { task: THIRD },
        &mut Queue::uncapped(),
    );
    super::super::limit_holds::room_freed(&mut domain, &env.limits.forge);
    assert!(domain.work.is_empty(), "restored usage queues no release");
    fresh(&mut domain, &env);
    super::super::limit_holds::room_freed(&mut domain, &env.limits.forge);
    assert!(!domain.work.is_empty(), "fresh usage notices the room that freed");
    crate::route::route_into(&mut domain, &env, &mut decision);
    assert!(!held(&domain, FIRST));
    assert!(domain.forge_limit_holds().is_empty());
}

//! A refused forming waits on its train alone (domain/landing-forming.md, 3).
use super::super::{Domain, Env, Limits, forge, landing_route, tasks};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Wall};

fn advance_window(env: &mut Env<Limits>) {
    let span = env.limits.forge.queue_window;
    env.now = env.now.saturating_add(span);
    env.wall = Wall::from_nanos(env.wall.as_nanos().checked_add(span.as_nanos()).expect("fixture wall span"));
}

fn reasons(domain: &mut Domain) -> Box<[Option<tasks::Refusal>]> {
    let mut reasons = List::with_capacity(crate::FACTS);
    while let Some(fact) = domain.pop_fact() {
        if let crate::Fact::LandingRefused { why, .. } = fact {
            reasons.push(why).expect("bounded diagnostic facts");
        }
    }
    reasons.into_boxed()
}

#[test]
fn a_refused_train_waits_exactly_one_window_while_another_forms() {
    let (mut domain, mut env) = super::train_root(false);
    let tasks = env.limits.tasks.tasks;
    env.limits.tasks.tasks = 2; // The two ready changes fill the live room.
    super::judge_trains(&mut domain, &env);
    assert_eq!(reasons(&mut domain).as_ref(), &[Some(tasks::Refusal::Live)]);
    let refusal = domain.core.counters.deployment.tasks;
    let until = Wall::from_nanos(
        env.wall.as_nanos().checked_add(env.limits.forge.queue_window.as_nanos()).expect("fixture queue window"),
    );
    assert_eq!(landing_route::refusal_deadline(&domain, &env), Some(until));

    env.limits.tasks.tasks = tasks;
    let mut other = domain.forge.change(8).unwrap().clone();
    other.base = b"goal".as_slice().into();
    domain
        .work
        .push(super::super::Work::Forge(forge::Event::Restore { record: forge::Stored::Change(Box::new(other)) }));
    super::judge_trains(&mut domain, &env);
    let beside = domain.forge.change(8).unwrap().change.landing.expect("the other train forms meanwhile");
    assert_eq!(domain.forge.landing(beside).unwrap().landing.branch.as_ref(), b"goal");
    assert_eq!(domain.forge.change(7).unwrap().change.landing, None);
    assert_eq!(
        domain.core.counters.deployment.tasks,
        refusal.checked_add(1).expect("one landing beside the refused train")
    );

    env.wall = Wall::from_nanos(until.as_nanos().checked_sub(1).expect("one nanosecond before expiry"));
    super::judge_trains(&mut domain, &env);
    assert_eq!(domain.forge.change(7).unwrap().change.landing, None);
    assert_eq!(
        domain.core.counters.deployment.tasks,
        refusal.checked_add(1).expect("one landing beside the refused train"),
        "not a nanosecond early"
    );
    env.wall = until;
    super::judge_trains(&mut domain, &env);
    let formed = domain.forge.change(7).unwrap().change.landing.expect("first judgement at expiry forms");
    assert_ne!(formed, beside);
    assert!(domain.core.tasks.task(formed).is_some());
    assert_eq!(landing_route::refusal_deadline(&domain, &env), None);
}

#[test]
fn a_refused_train_logs_only_changed_reasons_until_it_forms() {
    let (mut domain, mut env) = super::train_root(false);
    let tasks = env.limits.tasks;
    env.limits.tasks.tasks = 2;
    super::judge_trains(&mut domain, &env);
    assert_eq!(reasons(&mut domain).as_ref(), &[Some(tasks::Refusal::Live)]);
    advance_window(&mut env);
    super::judge_trains(&mut domain, &env);
    assert!(reasons(&mut domain).is_empty(), "the same reason is silent");

    env.limits.tasks.tasks = tasks.tasks;
    env.limits.tasks.project_tasks = 2;
    advance_window(&mut env);
    super::judge_trains(&mut domain, &env);
    assert_eq!(reasons(&mut domain).as_ref(), &[Some(tasks::Refusal::Project)]);
    env.limits.tasks.project_tasks = tasks.project_tasks;
    advance_window(&mut env);
    super::judge_trains(&mut domain, &env);
    let made = domain.forge.change(7).unwrap().change.landing.expect("a successful forming resets the reason");
    assert!(domain.core.tasks.task(made).is_some());
    // End the train without waking its member's procedure: it remains ready for this judgement.
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::SettleEffects { task: made, root: made, ending: forge::ReleaseEnding::Cancelled },
        &mut Queue::uncapped(),
    );
    env.limits.tasks.project_tasks = 2;
    super::judge_trains(&mut domain, &env);
    assert_eq!(
        reasons(&mut domain).as_ref(),
        &[Some(tasks::Refusal::Project)],
        "a made forming forgets the last reason"
    );
}

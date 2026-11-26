//! A refused forming waits on its train alone (domain/landing-forming.md, 3).
use super::super::{Domain, Env, Limits, forge, landing_route};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Wall};

fn advance_window(env: &mut Env<Limits>) {
    let span = env.limits.forge.queue_window;
    env.now = env.now.saturating_add(span);
    env.wall = Wall::from_nanos(env.wall.as_nanos().checked_add(span.as_nanos()).expect("fixture wall span"));
}

fn reasons(domain: &mut Domain) -> Box<[Box<[u8]>]> {
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
    assert_eq!(reasons(&mut domain).as_ref(), &[Box::from(b"Live".as_slice())]);
    let refusal = domain.core.counters.deployment.tasks;
    let until = Wall::from_nanos(
        env.wall.as_nanos().checked_add(env.limits.forge.queue_window.as_nanos()).expect("fixture queue window"),
    );
    assert_eq!(landing_route::refusal_deadline(&domain), Some(until));
    let stored = stored_refusal(&domain).expect("jig refused this train");
    assert_eq!(stored.since, env.wall);
    assert_eq!(
        stored.cause,
        forge::FormingRefusal::Jig { number: refusal, why: Box::from(b"Live".as_slice()), retry: until }
    );

    env.limits.tasks.tasks = tasks;
    let mut other = domain.forge.change(8).unwrap().clone();
    other.base = b"goal".as_slice().into();
    domain
        .work
        .push(super::super::Work::Forge(forge::Event::Restore { record: forge::Stored::Change(Box::new(other)) }));
    super::judge_trains(&mut domain, &env);
    let beside = domain.forge.landing_of(8).expect("the other train forms meanwhile");
    assert_eq!(domain.forge.landing(beside).unwrap().landing.branch.as_ref(), b"goal");
    assert_eq!(domain.forge.landing_of(7), None);
    assert_eq!(
        domain.core.counters.deployment.tasks,
        refusal.checked_add(1).expect("one landing beside the refused train")
    );

    env.wall = Wall::from_nanos(until.as_nanos().checked_sub(1).expect("one nanosecond before expiry"));
    super::judge_trains(&mut domain, &env);
    assert_eq!(domain.forge.landing_of(7), None);
    assert_eq!(
        domain.core.counters.deployment.tasks,
        refusal.checked_add(1).expect("one landing beside the refused train"),
        "not a nanosecond early"
    );
    env.wall = until;
    super::judge_trains(&mut domain, &env);
    let formed = domain.forge.landing_of(7).expect("first judgement at expiry forms");
    assert_ne!(formed, beside);
    assert!(domain.core.tasks.task(formed).is_some());
    assert_eq!(landing_route::refusal_deadline(&domain), None);
    assert_eq!(stored_refusal(&domain), None);
}

#[test]
fn a_refused_train_logs_only_changed_reasons_until_it_forms() {
    let (mut domain, mut env) = super::train_root(false);
    let tasks = env.limits.tasks;
    let since = env.wall;
    env.limits.tasks.tasks = 2;
    super::judge_trains(&mut domain, &env);
    assert_eq!(reasons(&mut domain).as_ref(), &[Box::from(b"Live".as_slice())]);
    advance_window(&mut env);
    super::judge_trains(&mut domain, &env);
    assert!(reasons(&mut domain).is_empty(), "the same reason is silent");
    assert_eq!(stored_refusal(&domain).expect("repeated refusal").since, since);

    env.limits.tasks.tasks = tasks.tasks;
    env.limits.tasks.project_tasks = 2;
    advance_window(&mut env);
    super::judge_trains(&mut domain, &env);
    assert_eq!(reasons(&mut domain).as_ref(), &[Box::from(b"Project".as_slice())]);
    assert_eq!(stored_refusal(&domain).expect("new cause").since, since);
    env.limits.tasks.project_tasks = tasks.project_tasks;
    advance_window(&mut env);
    super::judge_trains(&mut domain, &env);
    let made = domain.forge.landing_of(7).expect("a successful forming resets the reason");
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
        &[Box::from(b"Project".as_slice())],
        "a made forming forgets the last reason"
    );
}

fn stored_refusal(domain: &Domain) -> Option<&forge::TrainRefusal> {
    let change = domain.forge.change(7).expect("ready change");
    let (refusal, _) = domain.forge.forming_refusal(change.repository, &change.base)?;
    Some(refusal)
}

#[test]
fn a_refused_train_restarts_inside_its_stored_window() {
    let (mut domain, mut env) = super::train_root(false);
    let open = env.limits.tasks.tasks;
    env.limits.tasks.tasks = 2;
    super::judge_trains(&mut domain, &env);
    let expected = stored_refusal(&domain).expect("refused train").clone();
    let forge::FormingRefusal::Jig { number, retry, .. } = expected.cause else { panic!("jig refused") };
    let crate::decision::JournalOutput::Commit { writes, .. } =
        crate::decision::commit(&mut domain.journal, &env.limits.journal).expect("a refusal writes a commit")
    else {
        panic!("commit output")
    };
    let mut saved = None;
    for write in &writes {
        if let crate::Write::Save(crate::Record::Forge { row }) = write
            && let forge::Stored::Repository(_) = row.as_ref()
        {
            saved = Some(row.as_ref().clone());
            break;
        }
    }
    let saved = saved.expect("the refused episode was saved on its repository");

    let (mut restarted, _) = super::train_root(false);
    restarted.core.counters.deployment.tasks = number;
    env.limits.tasks.tasks = open;
    env.limits.forge.queue_window = skein_lib::Duration::from_nanos(1);
    env.wall = Wall::from_nanos(retry.as_nanos().checked_sub(1).expect("inside queue window"));
    restarted.work.push(super::super::Work::Forge(forge::Event::Restore { record: saved }));
    super::judge_trains(&mut restarted, &env);
    assert_eq!(restarted.core.counters.deployment.tasks, number, "no make before stored retry");
    assert_eq!(stored_refusal(&restarted), Some(&expected));
    assert!(reasons(&mut restarted).is_empty());
    assert_eq!(landing_route::refusal_deadline(&restarted), Some(retry));

    env.wall = retry;
    super::judge_trains(&mut restarted, &env);
    assert!(restarted.forge.landing_of(7).is_some(), "the first judgement at retry forms");
    assert_eq!(stored_refusal(&restarted), None);
}

#[test]
fn a_refusal_is_erased_when_its_last_waiting_changes_leave() {
    let (mut domain, env) = super::train_root(false);
    let change = domain.forge.change(7).expect("ready change");
    let repository = change.repository;
    let branch = change.base.clone();
    let forge_env = crate::translate::environment_forge(&env);
    for task in [7, 8] {
        let mut row = domain.forge.change(task).expect("ready change").clone();
        row.change.state = temper_engine_domain_forge_change::State::Ended;
        forge::step(
            &mut domain.forge,
            &forge_env,
            forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
            &mut Queue::uncapped(),
        );
    }
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::TrainRefusal { repository, branch: branch.clone(), cause: Some(forge::FormingRefusal::Landings) },
        &mut Queue::uncapped(),
    );
    assert!(domain.forge.forming_refusal(repository, &branch).is_some());
    super::judge_trains(&mut domain, &env);
    assert!(domain.forge.forming_refusal(repository, &branch).is_none());
    assert!(reasons(&mut domain).is_empty());
}

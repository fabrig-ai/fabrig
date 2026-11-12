//! Task holders and charter standing at the synchronous root seam.
#![expect(clippy::disallowed_macros, clippy::disallowed_methods, reason = "test assertions format handles and inspect queued news")]
use super::*;
use skein_lib::{ReplyTo, Token};
fn add(domain: &mut Domain, env: &Env<Limits>, number: u64, charter: u32, label: u32, source: &tasks::TaskRecord) {
    let source = source.clone();
    let mut out = Queue::uncapped();
    tasks::step(&mut domain.core.tasks, &env.with(env.limits.tasks), tasks::Event::Make {
        reply_to: ReplyTo::new(Token::new(1000_u64.checked_add(number).expect("fixture token"))), creator: tasks::Party::Deployment { project: 1 },
        batch: Box::new([tasks::New {
            number, project: 1, label, flags: tasks::Flags::PLAIN, executor: tasks::Executor::Agent { charter }, llm: None,
            spec: source.spec, contract: source.contract, deadline: tasks::Deadline::Named, authority: source.authority,
            numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 }, funder: source.funder,
            dependencies: Box::default(), holdings: Box::default(), wake: tasks::WakePolicy::DEFAULT, recurring: None, tracked: None,
        }]),
    }, &mut out);
    assert!(domain.core.tasks.task(number).is_some(), "{out:?}");
}
fn setting(domain: &mut Domain, env: &Env<Limits>, kind: u32, value: forge::settings::Value) {
    domain.forge.keep_settings(&super::super::environment_forge(env), 1, None, &[forge::settings::Asked { kind, scope: 1, number: 1, edit: forge::settings::Edit::Set(forge::settings::SizedValue { value, size: 1 }) }], &mut Queue::uncapped());
}
#[test]
fn task_hand_news_and_charter_standing_use_the_same_decision() {
    let (source, source_env, source_task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    let template = source.core.tasks.task(source_task).unwrap().clone();
    let mut limits = source_env.limits; limits.tasks.message_bytes = 128; limits.tasks.inbox_bytes = 512;
    let (mut domain, env) = crate::tests::resume_fixture::root_with_config_and_limits(crate::tests::resume_fixture::config(91), &limits);
    let task = 52;
    tasks::step(&mut domain.core.tasks, &env.with(env.limits.tasks), tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(2)), project: 1, period: 1, budget: 100 }, &mut Queue::uncapped());
    add(&mut domain, &env, task, 1, 3, &template); add(&mut domain, &env, 47, 1, 3, &template);
    assert_eq!(stands(&domain, 1, AreaOwner::Charter(1)), Some(Holder::Task(47)));
    setting(&mut domain, &env, 12, forge::settings::Value::ProjectAreas(forge::settings::ProjectAreas { areas: Box::new([forge::settings::Area { name: b"tests".as_slice().into(), words: b"test suite".as_slice().into(), owner: AreaOwner::Charter(1) }]) }));
    setting(&mut domain, &env, 13, forge::settings::Value::ProjectTriager(forge::settings::ProjectTriager { owner: AreaOwner::Task(task) }));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let call = forge::FileFinding { project: 1, kind: forge::FindingKind::Qa, severity: forge::Severity::High, area: None, title: b"Broken test".as_slice().into(), body: b"reproduction".as_slice().into() };
    assert_eq!(file_finding(&mut domain, &env, &mut decision, Filer::Party(1), call), forge::FileFindingAnswer::Filed { number: 1 });
    let hand = forge::HandFinding { project: 1, number: 1, area: b"tests".as_slice().into(), why: b"your suite".as_slice().into() };
    let why = hand_finding(&mut domain, &env, &mut decision, Filer::Task(47), hand.clone());
    assert_eq!(why, forge::HandFindingAnswer::Refused(FindingRefusal::NotYours(Holder::Task(task))));
    assert_eq!(FindingRefusal::NotYours(Holder::Task(task)).words(1, 1, b"").as_ref(), format!("finding 1/1 is not yours: its owner is task {task}").as_bytes());
    assert_eq!(hand_finding(&mut domain, &env, &mut decision, Filer::Task(task), hand), forge::HandFindingAnswer::Handed);
    let subscription = domain.forge.topic_subscriber(47, &forge::Topic::Findings { project: 1 }).unwrap();
    assert!(domain.core.tasks.task(47).unwrap().subscriptions.iter().any(|row| row.number == subscription.number));
    let news = domain.work.iter().find_map(|work| if let Work::Tasks(tasks::Event::Notice { task: 47, word }) = work { Some(word) } else { None }).unwrap();
    assert_eq!(news.words.as_ref(), format!("Finding 1/1 is yours: Broken test\nqa, high, area tests\nHanded on by task {task}: your suite\nreproduction").as_bytes());
    assert_eq!(domain.forge.finding(1, 1).unwrap().why.as_deref(), Some(b"your suite".as_slice()));
}

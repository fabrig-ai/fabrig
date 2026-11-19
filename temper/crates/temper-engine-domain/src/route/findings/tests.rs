//! Task holders and charter standing at the synchronous root seam.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "test assertions format handles and inspect queued news"
)]
use super::*;
use skein_lib::{ReplyTo, Token};
fn add(domain: &mut Domain, env: &Env<Limits>, number: u64, charter: u32, label: u32, source: &tasks::TaskRecord) {
    let source = source.clone();
    let mut out = Queue::uncapped();
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Make {
            maker: None,
            places: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1000_u64.checked_add(number).expect("fixture token"))),
            creator: tasks::Party::Deployment { project: 1 },
            batch: Box::new([tasks::New {
                number,
                project: 1,
                label,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Agent { charter },
                llm: None,
                spec: source.spec,
                contract: source.contract,
                deadline: tasks::Deadline::Named,
                authority: source.authority,
                numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                funder: source.funder,
                dependencies: Box::default(),
                holdings: Box::default(),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: source.tracked,
            }]),
        },
        &mut out,
    );
    assert!(domain.core.tasks.task(number).is_some(), "{out:?}");
}
fn setting(domain: &mut Domain, env: &Env<Limits>, kind: u32, value: forge::settings::Value) {
    domain.forge.keep_settings(
        &super::super::environment_forge(env),
        1,
        None,
        &[forge::settings::Asked {
            kind,
            scope: 1,
            number: 1,
            edit: forge::settings::Edit::Set(forge::settings::SizedValue { value, size: 1 }),
        }],
        &mut Queue::uncapped(),
    );
}
fn set_area(domain: &mut Domain, env: &Env<Limits>, owner: AreaOwner) {
    setting(
        domain,
        env,
        14,
        forge::settings::Value::ProjectAreas(forge::settings::ProjectAreas {
            areas: Box::new([forge::settings::Area {
                name: b"tests".as_slice().into(),
                words: b"test suite".as_slice().into(),
                owner,
            }]),
        }),
    );
}
fn set_triager(domain: &mut Domain, env: &Env<Limits>, owner: AreaOwner) {
    setting(domain, env, 15, forge::settings::Value::ProjectTriager(forge::settings::ProjectTriager { owner }));
}
fn task_event(domain: &mut Domain, env: &Env<Limits>, event: tasks::Event) {
    tasks::step(&mut domain.core.tasks, &env.with(env.limits.tasks), event, &mut Queue::uncapped());
}
#[test]
fn task_hand_news_and_charter_standing_use_the_same_decision() {
    let (source, source_env, source_task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    let template = source.core.tasks.task(source_task).unwrap().clone();
    let mut limits = source_env.limits;
    limits.tasks.message_bytes = 128;
    limits.tasks.inbox_bytes = 512;
    limits.tasks.tasks = 4;
    limits.tasks.project_tasks = 4;
    limits.people.inbox_tasks = 8;
    limits.tasks.charters = 14;
    let mut config = crate::tests::resume_fixture::config(91);
    let mut charter = config.core.defaults.charters.first().unwrap().clone();
    charter.number = 14;
    config.core.defaults.charters = Box::new([config.core.defaults.charters.first().unwrap().clone(), charter]);
    let (mut domain, env) = crate::tests::resume_fixture::root_with_config_and_limits(config, &limits);
    let task = 40;
    task_event(
        &mut domain,
        &env,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(2)), project: 1, period: 1, budget: 100 },
    );
    add(&mut domain, &env, task, 1, 0, &template);
    add(&mut domain, &env, 52, 14, 3, &template);
    add(&mut domain, &env, 47, 14, 3, &template);
    assert_eq!(stands(&domain, 1, AreaOwner::Charter(14)), Some(Holder::Task(47)));
    set_area(&mut domain, &env, AreaOwner::Charter(14));
    set_triager(&mut domain, &env, AreaOwner::Task(task));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let call = forge::FileFinding {
        project: 1,
        kind: forge::FindingKind::Qa,
        severity: forge::Severity::High,
        area: None,
        title: b"Broken test".as_slice().into(),
        body: b"reproduction".as_slice().into(),
    };
    assert_eq!(
        file_finding(&mut domain, &env, &mut decision, Filer::Party(1), call),
        forge::FileFindingAnswer::Filed { number: 1 }
    );
    let hand = forge::HandFinding {
        project: 1,
        number: 1,
        area: b"tests".as_slice().into(),
        why: b"your suite".as_slice().into(),
    };
    let why = hand_finding(&mut domain, &env, &mut decision, Filer::Task(47), hand.clone());
    assert_eq!(why, forge::HandFindingAnswer::Refused(FindingRefusal::NotYours(Holder::Task(task))));
    assert_eq!(
        FindingRefusal::NotYours(Holder::Task(task)).words(1, 1, b"").as_ref(),
        format!("finding 1/1 is not yours: its owner is task {task}").as_bytes()
    );
    assert_eq!(
        hand_finding(&mut domain, &env, &mut decision, Filer::Task(task), hand),
        forge::HandFindingAnswer::Handed
    );
    let subscription = domain.forge.topic_subscriber(47, &forge::Topic::Findings { project: 1 }).unwrap();
    assert!(domain.core.tasks.task(47).unwrap().subscriptions.iter().any(|row| row.number == subscription.number));
    let news = queued_news(&domain, 47);
    let news = news.first().unwrap();
    assert_eq!(news.words.as_ref(), format!("Finding 1/1 is yours: Broken test\nqa, high, area tests\nHanded on by task {task}: your suite\nreproduction").as_bytes());
    assert_eq!(domain.forge.finding(1, 1).unwrap().why.as_deref(), Some(b"your suite".as_slice()));
    let findings =
        document(&domain, 1, 1, forge::FindingsFilter::Triage { state: Some(forge::FindingState::New), mine: false });
    let handed = findings.rows.first().unwrap();
    assert_eq!(handed.hander, Some(Filer::Task(task)));
    assert_eq!(handed.why.as_deref(), Some(b"your suite".as_slice()));
    let detail = read(&domain, 1, 1).unwrap();
    assert_eq!(detail.row.hander, Some(Filer::Task(task)));
    assert_eq!(detail.row, *handed);
    assert_eq!(
        triage_finding(
            &mut domain,
            &env,
            &mut decision,
            Filer::Task(52),
            forge::TriageFinding { project: 1, number: 1, act: forge::Triage::Backlog }
        ),
        forge::TriageFindingAnswer::Refused(FindingRefusal::NotYours(Holder::Task(47)))
    );
    assert_eq!(cut_words("ééééé".as_bytes(), 8).as_ref(), "é[cut]".as_bytes());
    set_area(&mut domain, &env, AreaOwner::Task(40));
    set_triager(&mut domain, &env, AreaOwner::Charter(14));
    assert_eq!(holder(&domain, 1, Some(b"tests")), Holder::Task(40));
    task_event(
        &mut domain,
        &env,
        tasks::Event::Cancel {
            reply_to: ReplyTo::new(Token::new(998)),
            by: tasks::Party::Deployment { project: 1 },
            operation: tasks::Cancel { task: 40, reason: b"ended".as_slice().into(), grace: None, message: None },
        },
    );
    assert_eq!(stands(&domain, 1, AreaOwner::Task(40)), None);
    assert_eq!(holder(&domain, 1, Some(b"tests")), Holder::Task(47));
}
fn triager() -> (Domain, Env<Limits>, u64) {
    let (mut domain, mut env, task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    env.limits.tasks.inbox_bytes = 512;
    set_triager(&mut domain, &env, AreaOwner::Task(task));
    (domain, env, task)
}
fn fill_subscriptions(domain: &mut Domain, env: &Env<Limits>, task: u64) {
    for index in 1..=u64::from(env.limits.tasks.subscriptions) {
        let number = index.checked_add(1000).expect("fixture subscription");
        task_event(
            domain,
            env,
            tasks::Event::SubscribeTopic {
                reply_to: ReplyTo::new(Token::new(number)),
                task,
                subscription: tasks::Subscription {
                    number,
                    kind: tasks::SubscriptionKind::Topic { connector: 0, topic: number },
                },
            },
        );
    }
    assert_eq!(
        domain.core.tasks.task(task).unwrap().subscriptions.len(),
        skein_lib::usize_of(env.limits.tasks.subscriptions)
    );
}
fn file_red(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) -> forge::FileFindingAnswer {
    let call = forge::FileFinding {
        project: 1,
        kind: forge::FindingKind::Red,
        severity: forge::Severity::High,
        area: None,
        title: b"Red".as_slice().into(),
        body: Box::default(),
    };
    file_finding(domain, env, decision, Filer::Party(1), call)
}
fn queued_news(domain: &Domain, task: u64) -> Box<[&tasks::Word]> {
    domain
        .work
        .iter()
        .filter_map(|work| {
            if let Work::Tasks(tasks::Event::Notice { task: to, word }) = work {
                return (*to == task).then_some(word);
            }
            None
        })
        .collect()
}
#[test]
fn a_holder_at_its_subscription_bound_takes_the_filing_and_news() {
    let (mut domain, mut env, task) = triager();
    env.limits.tasks.subscriptions = 1;
    fill_subscriptions(&mut domain, &env, task);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    assert_eq!(file_red(&mut domain, &env, &mut decision), forge::FileFindingAnswer::Filed { number: 1 });
    assert_eq!(domain.forge.finding(1, 1).unwrap().state, forge::FindingState::New);
    let subscription = domain.forge.topic_subscriber(task, &forge::Topic::Findings { project: 1 }).unwrap();
    let subscriptions = &domain.core.tasks.task(task).unwrap().subscriptions;
    assert_eq!(subscriptions.len(), 2);
    assert!(subscriptions.iter().any(|row| row.number == subscription.number));
    let news = queued_news(&domain, task);
    assert_eq!(news.len(), 1);
    assert_eq!(news[0].words.as_ref(), b"Finding 1/1 is yours: Red\nred, high, no area\n");
    assert_eq!(
        news[0].kind,
        tasks::MessageKind::News { subscription: subscription.number, class: tasks::NewsClass::Wakes }
    );
}
#[test]
fn a_forge_subscription_refusal_keeps_the_filing_and_retries_at_the_next() {
    let (mut domain, mut env, task) = triager();
    let room = env.limits.forge.subscriptions;
    env.limits.forge.subscriptions = 0;
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    assert_eq!(file_red(&mut domain, &env, &mut decision), forge::FileFindingAnswer::Filed { number: 1 });
    assert_eq!(domain.forge.finding(1, 1).unwrap().state, forge::FindingState::New);
    assert!(domain.forge.topic_subscriber(task, &forge::Topic::Findings { project: 1 }).is_none());
    assert!(domain.core.tasks.task(task).unwrap().subscriptions.is_empty());
    assert_eq!(queued_news(&domain, task).len(), 0);
    env.limits.forge.subscriptions = room;
    assert_eq!(file_red(&mut domain, &env, &mut decision), forge::FileFindingAnswer::Filed { number: 2 });
    let subscription = domain.forge.topic_subscriber(task, &forge::Topic::Findings { project: 1 }).unwrap();
    assert_eq!(domain.core.tasks.task(task).unwrap().subscriptions.len(), 1);
    assert!(subscribed(&domain, task, subscription.number));
    let news = queued_news(&domain, task);
    assert_eq!(news.len(), 1);
    assert!(news[0].words.starts_with(b"Finding 1/2 is yours: Red\n"));
}
#[test]
fn the_next_finding_restores_an_unsubscribed_holder_under_the_forge_number() {
    let (mut domain, env, task) = triager();
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    assert_eq!(file_red(&mut domain, &env, &mut decision), forge::FileFindingAnswer::Filed { number: 1 });
    let number = domain.forge.topic_subscriber(task, &forge::Topic::Findings { project: 1 }).unwrap().number;
    task_event(
        &mut domain,
        &env,
        tasks::Event::Unsubscribe { reply_to: ReplyTo::new(Token::new(88)), task, subscription: number },
    );
    assert!(domain.core.tasks.task(task).unwrap().subscriptions.is_empty());
    while domain.work.pop().is_some() {}
    assert_eq!(file_red(&mut domain, &env, &mut decision), forge::FileFindingAnswer::Filed { number: 2 });
    assert_eq!(domain.forge.topic_subscriber(task, &forge::Topic::Findings { project: 1 }).unwrap().number, number);
    let subscriptions = &domain.core.tasks.task(task).unwrap().subscriptions;
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].number, number);
    let news = queued_news(&domain, task);
    assert_eq!(news.len(), 1);
    assert_eq!(news[0].kind, tasks::MessageKind::News { subscription: number, class: tasks::NewsClass::Wakes });
    assert!(news[0].words.starts_with(b"Finding 1/2 is yours: Red\n"));
}
#[test]
fn routing_to_a_goal_at_its_subscription_bound_writes_and_tells_it() {
    let (mut domain, mut env, source) = crate::tests::live_task(Some(1), tasks::Executor::Agent { charter: 1 });
    env.limits.tasks.subscriptions = 1;
    env.limits.tasks.inbox_bytes = 512;
    env.limits.tasks.inbox_messages = 4;
    let template = domain.core.tasks.task(source).unwrap().clone();
    let goal = 40;
    add(&mut domain, &env, goal, 1, 2, &template);
    set_triager(&mut domain, &env, AreaOwner::Task(source));
    fill_subscriptions(&mut domain, &env, goal);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    assert_eq!(file_red(&mut domain, &env, &mut decision), forge::FileFindingAnswer::Filed { number: 1 });
    assert_eq!(
        triage_finding(
            &mut domain,
            &env,
            &mut decision,
            Filer::Task(source),
            forge::TriageFinding { project: 1, number: 1, act: forge::Triage::Route(goal) },
        ),
        forge::TriageFindingAnswer::Triaged
    );
    let row = domain.forge.finding(1, 1).unwrap();
    assert_eq!(row.state, forge::FindingState::Routed);
    assert_eq!(row.goal, Some(goal));
    let number = domain.forge.topic_subscriber(goal, &forge::Topic::Findings { project: 1 }).unwrap().number;
    assert_eq!(domain.core.tasks.task(goal).unwrap().subscriptions.len(), 2);
    assert!(subscribed(&domain, goal, number));
    let news = queued_news(&domain, goal);
    assert_eq!(news.len(), 1);
    assert!(news[0].words.starts_with(b"Finding 1/1 for you: Red\n"));
}

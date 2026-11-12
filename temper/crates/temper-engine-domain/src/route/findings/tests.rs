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
                tracked: None,
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
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one decision pins charter standing, task hand-off and fallback after an end"
)]
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
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(2)), project: 1, period: 1, budget: 100 },
        &mut Queue::uncapped(),
    );
    add(&mut domain, &env, task, 1, 0, &template);
    add(&mut domain, &env, 52, 14, 3, &template);
    add(&mut domain, &env, 47, 14, 3, &template);
    assert_eq!(stands(&domain, 1, AreaOwner::Charter(14)), Some(Holder::Task(47)));
    setting(
        &mut domain,
        &env,
        12,
        forge::settings::Value::ProjectAreas(forge::settings::ProjectAreas {
            areas: Box::new([forge::settings::Area {
                name: b"tests".as_slice().into(),
                words: b"test suite".as_slice().into(),
                owner: AreaOwner::Charter(14),
            }]),
        }),
    );
    setting(
        &mut domain,
        &env,
        13,
        forge::settings::Value::ProjectTriager(forge::settings::ProjectTriager { owner: AreaOwner::Task(task) }),
    );
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
    let news = domain
        .work
        .iter()
        .find_map(
            |work| if let Work::Tasks(tasks::Event::Notice { task: 47, word }) = work { Some(word) } else { None },
        )
        .unwrap();
    assert_eq!(news.words.as_ref(), format!("Finding 1/1 is yours: Broken test\nqa, high, area tests\nHanded on by task {task}: your suite\nreproduction").as_bytes());
    assert_eq!(domain.forge.finding(1, 1).unwrap().why.as_deref(), Some(b"your suite".as_slice()));
    let findings = document(&domain, 1, 1, Some(forge::FindingState::New), false);
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
    setting(
        &mut domain,
        &env,
        12,
        forge::settings::Value::ProjectAreas(forge::settings::ProjectAreas {
            areas: Box::new([forge::settings::Area {
                name: b"tests".as_slice().into(),
                words: b"test suite".as_slice().into(),
                owner: AreaOwner::Task(40),
            }]),
        }),
    );
    setting(
        &mut domain,
        &env,
        13,
        forge::settings::Value::ProjectTriager(forge::settings::ProjectTriager { owner: AreaOwner::Charter(14) }),
    );
    assert_eq!(holder(&domain, 1, Some(b"tests")), Holder::Task(40));
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Cancel {
            reply_to: ReplyTo::new(Token::new(998)),
            by: tasks::Party::Deployment { project: 1 },
            operation: tasks::Cancel { task: 40, reason: b"ended".as_slice().into(), grace: None, message: None },
        },
        &mut Queue::uncapped(),
    );
    assert_eq!(stands(&domain, 1, AreaOwner::Task(40)), None);
    assert_eq!(holder(&domain, 1, Some(b"tests")), Holder::Task(47));
}
#[test]
fn a_subscription_refusal_leaves_the_filing_unwritten() {
    let (mut domain, mut env, task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    env.limits.tasks.subscriptions = 1;
    for index in 1..=u64::from(env.limits.tasks.subscriptions) {
        let number = index.checked_add(1000).expect("fixture subscription");
        tasks::step(
            &mut domain.core.tasks,
            &env.with(env.limits.tasks),
            tasks::Event::SubscribeTopic {
                reply_to: ReplyTo::new(Token::new(number)),
                task,
                subscription: tasks::Subscription {
                    number,
                    kind: tasks::SubscriptionKind::Topic { connector: 0, topic: number },
                },
            },
            &mut Queue::uncapped(),
        );
    }
    assert_eq!(
        domain.core.tasks.task(task).unwrap().subscriptions.len(),
        skein_lib::usize_of(env.limits.tasks.subscriptions)
    );
    setting(
        &mut domain,
        &env,
        13,
        forge::settings::Value::ProjectTriager(forge::settings::ProjectTriager { owner: AreaOwner::Task(task) }),
    );
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let answer = file_finding(
        &mut domain,
        &env,
        &mut decision,
        Filer::Party(1),
        forge::FileFinding {
            project: 1,
            kind: forge::FindingKind::Red,
            severity: forge::Severity::High,
            area: None,
            title: b"Red".as_slice().into(),
            body: Box::default(),
        },
    );
    let forge::FileFindingAnswer::Refused(FindingRefusal::Words(words)) = answer else {
        panic!("subscription bound refuses the filing")
    };
    assert_eq!(
        words.as_ref(),
        format!("task {task} cannot subscribe to findings: tasks.subscriptions or inbox bound").as_bytes()
    );
    assert!(domain.forge.finding(1, 1).is_none());
    assert!(domain.forge.topic_subscriber(task, &forge::Topic::Findings { project: 1 }).is_none());
}

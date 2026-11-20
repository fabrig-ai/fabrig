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
#[expect(clippy::too_many_lines, reason = "one scenario checks ownership, unchanged facts and no news in every state")]
fn blame_checks_ownership_without_news_and_repair_checks_settings_first_in_any_state() {
    use super::super::people;
    use forge::findings::{BlameAct, BlameMove, BlameStrength, FindingBlame, FindingSignature, RepairAct};
    for state in [
        forge::FindingState::New,
        forge::FindingState::Routed,
        forge::FindingState::Backlog,
        forge::FindingState::Dropped,
    ] {
        let (mut domain, env, task) = triager();
        domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
        domain.core.people.start_restore(env.wall);
        for record in [
            people::Stored::Person {
                number: 1,
                identity: people::Identity { key: forge::user_key(1, 7), login: Box::default(), name: Box::default() },
            },
            people::Stored::Roles {
                project: 1,
                holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
            },
        ] {
            people::step(
                &mut domain.core.people,
                &env.with(env.limits.people),
                people::Event::Restore { record },
                &mut Queue::uncapped(),
            );
        }
        assert!(domain.core.people.start_link());
        domain.core.people.start_sweep();
        assert!(domain.core.people.start_complete(&env.with(env.limits.people)));
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        let filing = forge::VerdictFiling {
            project: 1,
            kind: forge::FindingKind::Red,
            severity: forge::Severity::High,
            title: b"Red".as_slice().into(),
            body: b"its evidence".as_slice().into(),
            follow: None,
            signature: FindingSignature {
                repository: skein_forge::api::Repository { forge: 1, repository: 2 },
                source: b"browser".as_slice().into(),
                ids: Box::new([b"test a::t x".as_slice().into()]),
                first: Box::default(),
                many: false,
            },
            blame: Some(FindingBlame { strength: BlameStrength::Unblamed, landings: Box::new([]), moved: None }),
        };
        assert_eq!(
            file_verdict(&mut domain, &env, &mut decision, task, filing),
            forge::FileFindingAnswer::Filed { number: 1 }
        );
        let mut expected = domain.forge.finding(1, 1).unwrap().clone();
        expected.state = state;
        expected.goal = Some(task);
        domain.forge.commit_finding(
            &env.limits.forge,
            forge::FindingWrite { row: expected.clone(), erase: None },
            &mut Queue::uncapped(),
        );
        let blame = forge::Triage::Blame(BlameAct { landing: None, why: b"unrelated landing".as_slice().into() });
        let repair = forge::Triage::Repair(RepairAct { more: Box::new([5]), words: b"fix".as_slice().into() });
        assert_eq!(
            triage_finding(
                &mut domain,
                &env,
                &mut decision,
                Filer::Task(u64::MAX),
                forge::TriageFinding { project: 1, number: 1, act: blame.clone() }
            ),
            forge::TriageFindingAnswer::Refused(FindingRefusal::NotYours(Holder::Task(task)))
        );
        assert_eq!(domain.forge.finding(1, 1), Some(&expected));
        assert_eq!(
            triage_finding(
                &mut domain,
                &env,
                &mut decision,
                Filer::Task(u64::MAX),
                forge::TriageFinding { project: 1, number: 1, act: repair.clone() }
            ),
            forge::TriageFindingAnswer::Refused(FindingRefusal::Words(
                b"repairs are off in project 1".as_slice().into()
            ))
        );
        for by in [Filer::Task(task), Filer::Party(1)] {
            assert_eq!(domain.core.people.role(1, 1), Some(people::Role::Owner));
            let news = queued_news(&domain, task).len();
            assert_eq!(
                triage_finding(
                    &mut domain,
                    &env,
                    &mut decision,
                    by,
                    forge::TriageFinding { project: 1, number: 1, act: blame.clone() }
                ),
                forge::TriageFindingAnswer::Triaged
            );
            expected.blame = Some(FindingBlame {
                strength: BlameStrength::Moved,
                landings: Box::new([]),
                moved: Some(BlameMove { by, why: b"unrelated landing".as_slice().into() }),
            });
            assert_eq!(domain.forge.finding(1, 1), Some(&expected));
            assert_eq!(holder(&domain, 1, None), Holder::Task(task));
            assert_eq!(queued_news(&domain, task).len(), news, "moving blame routes nothing");
            let answer = triage_finding(
                &mut domain,
                &env,
                &mut decision,
                by,
                forge::TriageFinding { project: 1, number: 1, act: repair.clone() },
            );
            assert_eq!(
                answer,
                forge::TriageFindingAnswer::Refused(FindingRefusal::Words(
                    b"repairs are off in project 1".as_slice().into()
                ))
            );
            assert_eq!(domain.forge.finding(1, 1), Some(&expected));
        }
    }
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
fn a_cancelled_routed_red_returns_through_release_to_its_holder_with_back_news() {
    let (mut domain, mut env, source) = crate::tests::live_task(Some(1), tasks::Executor::Agent { charter: 1 });
    env.limits.tasks.inbox_bytes = 512;
    env.limits.tasks.message_bytes = 128;
    let template = domain.core.tasks.task(source).unwrap().clone();
    let goal = 40;
    add(&mut domain, &env, goal, 1, 2, &template);
    set_area(&mut domain, &env, AreaOwner::Task(source));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let mut write = domain
        .forge
        .prepare_verdict_filing(
            &env.limits.forge,
            env.wall,
            Filer::Task(source),
            &forge::VerdictFiling {
                project: 1,
                kind: forge::FindingKind::Red,
                severity: forge::Severity::High,
                title: b"Red".as_slice().into(),
                body: b"reproduce".as_slice().into(),
                signature: forge::FindingSignature {
                    repository: skein_forge::api::Repository { forge: 1, repository: 1 },
                    source: b"browser".as_slice().into(),
                    ids: Box::new([b"suite::fails".as_slice().into()]),
                    first: Box::new([]),
                    many: false,
                },
                blame: Some(forge::FindingBlame {
                    strength: forge::BlameStrength::Unblamed,
                    landings: Box::new([]),
                    moved: None,
                }),
                follow: None,
            },
        )
        .unwrap();
    let number = write.row.number;
    write.row.area = Some(b"tests".as_slice().into());
    domain.forge.commit_finding(&env.limits.forge, write, &mut Queue::uncapped());
    assert_eq!(
        triage_finding(
            &mut domain,
            &env,
            &mut decision,
            Filer::Task(source),
            forge::TriageFinding { project: 1, number, act: forge::Triage::Route(goal) }
        ),
        forge::TriageFindingAnswer::Triaged
    );
    while domain.work.pop().is_some() {}
    let connector = domain.config.forge_connector;
    super::super::asks::ask(
        &mut domain,
        &env,
        &mut decision,
        connector,
        jig_domain_core::Ask::Release {
            task: goal,
            root: goal,
            ending: tasks::Ending::Cancelled {
                by: tasks::Party::Deployment { project: 1 },
                at: env.wall,
                reason: b"stop".as_slice().into(),
                result: None,
            },
            entry: 80,
        },
    );
    let finding = domain.forge.finding(1, number).unwrap();
    assert_eq!(finding.state, forge::FindingState::New);
    assert_eq!(finding.goal, None);
    assert_eq!(finding.hander, None);
    assert!(finding.repair.as_ref().unwrap().cancelled);
    assert_eq!(finding.repair.as_ref().unwrap().attempts, 0);
    let news = queued_news(&domain, source);
    assert_eq!(news.len(), 1);
    assert_eq!(
        news[0].words.as_ref(),
        b"Finding 1/1 is yours: Red\nred, high, area tests\nBack: goal 40 was cancelled\nreproduce"
    );
}

#[test]
fn a_repair_deadline_sends_jigs_application_failure_and_a_closing_refusal_writes_nothing() {
    let (mut domain, env, task) = crate::tests::live_task(Some(1), tasks::Executor::Agent { charter: 1 });
    let created = domain.core.tasks.task(task).unwrap().created_at;
    let until = skein_lib::Wall::from_nanos(created.as_nanos().checked_add(7_200_000_000_000).unwrap());
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let mut out = Queue::uncapped();
    out.push(forge::Request::FailRepair { project: 1, task, until });
    forge_route::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Tasks(tasks::Event::Fail { reply_to, by, operation })) = domain.work.pop() else {
        panic!("jig Fail event")
    };
    assert_eq!(by, tasks::Party::Deployment { project: 1 });
    assert_eq!(operation.task, task);
    assert_eq!(operation.reason.as_ref(), b"past its repair deadline of 2 hours");
    let reason = operation.reason.clone();
    let mut out = Queue::uncapped();
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Fail { reply_to, by, operation },
        &mut out,
    );
    let mut written = 0_u32;
    for request in &out {
        if let tasks::Request::Save { .. } = request {
            written += 1;
        }
    }
    assert!(written > 0);
    let mut refused = Queue::uncapped();
    tasks::step(
        &mut domain.core.tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Fail { reply_to: ReplyTo::new(Token::new(902)), by, operation: tasks::Fail { task, reason } },
        &mut refused,
    );
    let mut saves = 0_u32;
    let mut refusals = 0_u32;
    for request in &refused {
        if let tasks::Request::Save { .. } = request {
            saves += 1;
        }
        if let tasks::Request::Refused { .. } = request {
            refusals += 1;
        }
    }
    assert_eq!(saves, 0);
    assert_eq!(refusals, 1);
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

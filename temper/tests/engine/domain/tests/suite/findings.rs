//! Findings through the real root, decisions and restart (domain/areas.md, 8).
use jig_domain_core::documents::{Watched, Window};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Application, ApplicationAnswer, Delivery, Key, Record, Write};
use temper_engine_domain_forge::{self as forge, settings as s};
use temper_engine_domain_world::{direct::Driver, kinds, walking};

pub(super) fn limits() -> engine::Limits {
    let mut limits = kinds::limits();
    limits.people.requests = 512; limits.forge.applications = 512;
    limits.tasks.message_bytes = 256; limits.tasks.inbox_bytes = 65_536;
    limits.tasks.inbox_messages = 128; limits.tasks.run_inbox = 128;
    limits.journal.transcript_bytes = 131_072;
    limits
}
pub(super) fn configuration() -> engine::Config {
    let mut config = kinds::configuration(&limits());
    for template in &mut config.core.defaults.templates { if template.number == 1 { template.label = 3; } }
    config
}
pub(super) fn driver() -> (Driver, engine::Assignment) { Driver::configured_chat([configuration(), configuration()], &limits()) }
pub(super) fn file(area: Option<&[u8]>) -> forge::FileFinding {
    forge::FileFinding { project: 1, kind: forge::FindingKind::Qa, severity: forge::Severity::High, area: area.map(Into::into), title: b"Broken test".as_slice().into(), body: b"reproduce from a fresh start".as_slice().into() }
}
pub(super) fn application(driver: &mut Driver, request: Application) -> ApplicationAnswer {
    driver.serial += 1;
    let serial = driver.serial;
    let mut key = [0; 16]; key[..8].copy_from_slice(&serial.to_be_bytes());
    let before = driver.delivered.len();
    driver.send(engine::Event::Party(engine::PartyInput::Application { scope: people::ApplicationScope::Project(1), reply_to: ReplyTo::new(Token::new(10_000 + serial)), sign_in: driver.session(), key, digest: [1; 32], carried: None, request }));
    driver.settle();
    driver.delivered[before..].iter().find_map(|delivery| if let Delivery::Application { outcome, .. } = delivery { outcome.clone() } else { None }).unwrap_or_else(|| panic!("no finding outcome: {:?}", &driver.delivered[before..]))
}
pub(super) fn settings(driver: &mut Driver, areas: Option<s::ProjectAreas>, triager: Option<s::AreaOwner>) {
    let mut items = vec![];
    if let Some(areas) = areas { items.push(kinds::item(12, 1, 1, &s::Value::ProjectAreas(areas))); }
    if let Some(owner) = triager { items.push(kinds::item(13, 1, 1, &s::Value::ProjectTriager(s::ProjectTriager { owner }))); }
    driver.serial += 1;
    let serial = driver.serial;
    let mut key = [0; 16]; key[..8].copy_from_slice(&serial.to_be_bytes());
    let items = items.into_boxed_slice();
    driver.send(engine::Event::Party(engine::PartyInput::ChangeSettings { reply_to: ReplyTo::new(Token::new(serial)), sign_in: driver.session(), key, digest: [2; 32], kinds: kinds::decoded(&items), change: Box::new(jig_domain_core::settings::Change { against: kinds::revision(&driver.store.rows), items, running: jig_domain_core::settings::Running::Stays }) }));
    driver.settle();
}
pub(super) fn area(owner: s::AreaOwner) -> s::ProjectAreas { s::ProjectAreas { areas: Box::new([s::Area { name: b"tests".as_slice().into(), words: b"The test suite".as_slice().into(), owner }]) } }
pub(super) fn row(driver: &Driver, number: u32) -> &forge::Finding {
    let Some(Record::Forge { row }) = driver.store.rows.get(&Key::Forge(forge::Key::Finding { project: 1, number })) else { panic!("finding {number} missing") };
    let forge::Stored::Finding(row) = row.as_ref() else { panic!("finding family") }; row
}
fn news(driver: &Driver, task: u64) -> Vec<&tasks::Word> { driver.root.tasks().task(task).expect("live holder").inbox.iter().filter(|word| matches!(word.kind, tasks::MessageKind::News { .. })).collect() }
fn watch(driver: &mut Driver, state: Option<forge::FindingState>, mine: bool) {
    driver.send(engine::Event::Party(engine::PartyInput::Watch { watcher: Token::new(900), sign_in: driver.session(), subject: Watched::Application { scope: people::ApplicationScope::Project(1), connector: 0, subject: Token::new(forge::findings_subject(1, state, mine)) }, window: Window { rows: 128, after: None } })); driver.settle();
}
fn documents(driver: &Driver) -> Vec<&forge::Findings> { driver.delivered.iter().filter_map(|delivery| if let Delivery::ForgeDocument { document, .. } = delivery && let forge::Document::Findings(document) = document.as_ref() { Some(document) } else { None }).collect() }

#[test]
fn filing_subscribes_and_tells_the_holder_in_that_decision_and_survives_restart() {
    let (mut driver, run) = driver();
    settings(&mut driver, Some(area(s::AreaOwner::Task(run.task))), Some(s::AreaOwner::Task(run.task)));
    assert_eq!(driver.root.tasks().task(run.task).expect("role task").label, 3);
    let before = driver.transactions.len();
    let runs = driver.delivered.iter().filter(|d| matches!(d, Delivery::Assigned { .. })).count();
    assert_eq!(application(&mut driver, Application::FileFinding(file(Some(b"tests")))), ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Filed { number: 1 }));
    let holder_task = driver.root.tasks().task(run.task).expect("holder");
    let subscription = holder_task.subscriptions.iter().find(|row| matches!(row.kind, tasks::SubscriptionKind::Topic { connector: 0, .. })).expect("root subscribed the task").number;
    assert!(news(&driver, run.task).iter().any(|word| word.words.starts_with(b"Finding 1/1 is yours: Broken test\n") && matches!(word.kind, tasks::MessageKind::News { subscription: number, class: tasks::NewsClass::Wakes } if number == subscription)));
    assert!(driver.transactions[before..].iter().any(|writes| writes.iter().any(|write| matches!(write, Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Finding(_)))) && writes.iter().any(|write| matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.number == run.task && row.subscriptions.iter().any(|s| s.number == subscription) && row.inbox.iter().any(|word| word.words.starts_with(b"Finding 1/1 is yours:"))))));
    assert_eq!(driver.delivered.iter().filter(|d| matches!(d, Delivery::Assigned { .. })).count(), runs, "no agent runs between filing and news");
    assert_eq!(application(&mut driver, Application::FileFinding(file(None))), ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Filed { number: 2 }));
    assert!(news(&driver, run.task).iter().any(|word| word.words.starts_with(b"Finding 1/2 is yours:")));
    let kept = row(&driver, 1).clone();
    driver.reopen(configuration()); driver.settle();
    assert_eq!(row(&driver, 1), &kept);
    assert_eq!(engine::finding_holder(&driver.root, 1, Some(b"tests")), forge::Holder::Task(run.task));
}

#[test]
fn people_reads_triage_and_capacity_keep_their_rules() {
    let (mut driver, _) = driver();
    watch(&mut driver, Some(forge::FindingState::New), true);
    assert!(documents(&driver).last().expect("initial findings").rows.is_empty());
    assert_eq!(application(&mut driver, Application::FileFinding(file(None))), ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Filed { number: 1 }));
    assert_eq!(documents(&driver).last().expect("marked at filing").rows[0].holder, forge::Holder::Role(0));
    assert!(driver.root.tasks().view_tasks().all(|task| news(&driver, task.number).is_empty()));
    let marked = documents(&driver).len();
    settings(&mut driver, Some(area(s::AreaOwner::Party(1))), Some(s::AreaOwner::Party(1)));
    assert!(documents(&driver).len() > marked, "settings saves mark findings");
    assert_eq!(documents(&driver).last().expect("marked at save").rows[0].holder, forge::Holder::Party(1));
    let bad = application(&mut driver, Application::FileFinding(file(Some(b"unknown"))));
    let ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Refused(why)) = bad else { panic!("unknown area refused") };
    assert_eq!(why.words(1, 0, b"unknown").as_ref(), b"no area unknown in project 1; its areas: tests");
    assert_eq!(application(&mut driver, Application::TriageFinding(forge::TriageFinding { project: 1, number: 1, act: forge::Triage::Backlog })), ApplicationAnswer::TriageFinding(forge::TriageFindingAnswer::Triaged));
    assert_eq!(row(&driver, 1).state, forge::FindingState::Backlog);
    assert!(documents(&driver).last().expect("backlog mark").rows.is_empty());
    assert_eq!(application(&mut driver, Application::HandFinding(forge::HandFinding { project: 1, number: 1, area: b"tests".as_slice().into(), why: b"test scope".as_slice().into() })), ApplicationAnswer::HandFinding(forge::HandFindingAnswer::Handed));
    assert_eq!(application(&mut driver, Application::TriageFinding(forge::TriageFinding { project: 1, number: 1, act: forge::Triage::Drop(b"duplicate".as_slice().into()) })), ApplicationAnswer::TriageFinding(forge::TriageFindingAnswer::Triaged));
    assert_eq!(row(&driver, 1).reason.as_deref(), Some(b"duplicate".as_slice()));
    for number in 2..=256 { assert_eq!(application(&mut driver, Application::FileFinding(file(None))), ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Filed { number })); }
    assert_eq!(application(&mut driver, Application::FileFinding(file(None))), ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Filed { number: 257 }));
    assert!(!driver.store.rows.contains_key(&Key::Forge(forge::Key::Finding { project: 1, number: 1 })));
    let answer = application(&mut driver, Application::FileFinding(file(None)));
    assert_eq!(answer, ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Refused(forge::FindingRefusal::Full(256))));
    assert_eq!(forge::FindingRefusal::Full(256).words(1, 0, b"").as_ref(), b"the project's findings are full: 256 new");
    let before = engine::finding_read(&driver.root, 1, 257).expect("read full finding");
    driver.reopen(configuration()); driver.settle();
    assert_eq!(engine::finding_read(&driver.root, 1, 257), Some(before));
}

#[test]
fn a_route_tells_a_live_goal_but_refuses_a_chat() {
    let (mut driver, run) = driver();
    application(&mut driver, Application::FileFinding(file(None)));
    let request = Application::TriageFinding(forge::TriageFinding { project: 1, number: 1, act: forge::Triage::Route(run.task) });
    assert_eq!(application(&mut driver, request), ApplicationAnswer::TriageFinding(forge::TriageFindingAnswer::Refused(forge::FindingRefusal::NoGoal)));
    let ask = walking::goal_from(driver.chat_ask(1, b"fix finding"), people::TaskExecutor::Agent { charter: 1 }, jig_test_profile::fixtures::Goal { budget: 1, deadline: None, priority: 1 });
    driver.send(engine::Event::Party(engine::PartyInput::Ask { models: Box::default(), reply_to: ReplyTo::new(Token::new(800)), sign_in: driver.session(), key: [80; 16], ask })); driver.settle();
    let goal = driver.root.tasks().view_tasks().find(|task| task.number != run.task).expect("goal created").number;
    assert_eq!(application(&mut driver, Application::TriageFinding(forge::TriageFinding { project: 1, number: 1, act: forge::Triage::Route(goal) })), ApplicationAnswer::TriageFinding(forge::TriageFindingAnswer::Triaged));
    assert!(news(&driver, goal).iter().any(|word| word.words.starts_with(b"Finding 1/1 for you: Broken test\n")));
    assert_eq!(row(&driver, 1).state, forge::FindingState::Routed); assert_eq!(row(&driver, 1).goal, Some(goal));
}

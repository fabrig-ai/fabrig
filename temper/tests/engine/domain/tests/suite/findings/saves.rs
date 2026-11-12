//! Settings ownership news crosses the save's commit barrier (domain/areas.md, 5).
use super::*;

fn role_driver(charter: u32) -> (Driver, engine::Assignment) {
    let mut limits = limits();
    limits.tasks.charters = 14;
    let mut config = configuration();
    if charter == 14 {
        let mut role = config.core.defaults.charters[0].clone();
        role.number = 14;
        let mut charters = config.core.defaults.charters.into_vec();
        charters.push(role);
        config.core.defaults.charters = charters.into_boxed_slice();
        config.core.defaults.rules.ceiling.delegation.kinds = Box::new([jig_domain_authority::Executor::Charter(14)]);
        for project in &mut config.core.seed_projects {
            project.policy.policy.ceiling.delegation.kinds = Box::new([jig_domain_authority::Executor::Charter(14)]);
            for role in &mut project.policy.policy.roles {
                role.authority.delegation.kinds = Box::new([jig_domain_authority::Executor::Charter(14)]);
            }
        }
    }
    let (driver, run) = configured_driver(&limits, config, 39, charter);
    assert_eq!(run.task, 40);
    assert_eq!(driver.root.tasks().task(40).expect("standing task").label, 3);
    (driver, run)
}
fn triage(driver: &mut Driver, number: u32, act: forge::Triage) {
    assert_eq!(
        application(driver, Application::TriageFinding(forge::TriageFinding { project: 1, number, act })),
        ApplicationAnswer::TriageFinding(forge::TriageFindingAnswer::Triaged)
    );
}
fn filed(driver: &mut Driver, number: u32, area: Option<&[u8]>) {
    assert_eq!(application(driver, Application::FileFinding(file(area))),
        ApplicationAnswer::FileFinding(forge::FileFindingAnswer::Filed { number }));
}
fn ownership_news(driver: &Driver, task: u64) -> Vec<&tasks::Word> {
    news(driver, task).into_iter().filter(|word| word.words.starts_with(b"Findings on project ")).collect()
}
fn finding_subscription(driver: &Driver, task: u64) -> Option<u64> {
    driver.store.rows.values().find_map(|record| {
        if let Record::Forge { row } = record
            && let forge::Stored::Subscription(row) = row.as_ref()
            && row.task == task && row.topic == (forge::Topic::Findings { project: 1 }) {
            Some(row.number)
        } else { None }
    })
}
#[test]
fn an_areas_save_tells_the_new_task_once_in_the_next_decision() {
    let (mut driver, run) = role_driver(1);
    settings(&mut driver, Some(area(s::AreaOwner::Role(0))), None);
    filed(&mut driver, 1, Some(b"tests"));
    filed(&mut driver, 2, Some(b"tests"));
    triage(&mut driver, 2, forge::Triage::Backlog);
    filed(&mut driver, 3, Some(b"tests"));
    triage(&mut driver, 3, forge::Triage::Drop(b"not needed".as_slice().into()));
    filed(&mut driver, 4, None);
    assert_eq!(finding_subscription(&driver, run.task), None);
    let before = driver.transactions.len();
    send_settings(&mut driver, Some(area(s::AreaOwner::Task(run.task))), None);
    for _ in 0..10 { driver.advance(false); }
    assert!(news(&driver, run.task).is_empty(), "nothing before the settings commit");
    assert_eq!(finding_subscription(&driver, run.task), None);
    driver.settle();
    let subscription = finding_subscription(&driver, run.task).expect("automatic subscription");
    let heard = ownership_news(&driver, run.task);
    assert_eq!(heard.len(), 1);
    assert_eq!(heard[0].words.as_ref(), b"Findings on project 1 are yours now: 1, 2");
    assert!(matches!(heard[0].kind, tasks::MessageKind::News { subscription: n, class: tasks::NewsClass::Wakes } if n == subscription));
    let save = driver.transactions[before..].iter().position(|writes| writes.iter().any(|write|
        matches!(write, Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Setting(row) if row.key.kind == 12 && row.key.slot == s::SettingSlot::Change)))).expect("areas commit");
    let notice = driver.transactions[before..].iter().position(|writes| writes.iter().any(|write|
        matches!(write, Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.number == run.task && row.inbox.iter().any(|word| word.words.starts_with(b"Findings on project 1"))))).expect("news commit");
    assert_eq!(notice, save + 1, "news in the following decision");
    let writes = &driver.transactions[before + notice];
    assert!(writes.iter().any(|write| matches!(write, Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Subscription(row) if row.task == run.task && row.number == subscription))), "subscription commits with news");
    let notice = ownership_news(&driver, run.task)[0].clone();
    settings(&mut driver, Some(area(s::AreaOwner::Task(run.task))), None);
    settings(&mut driver, Some(area(s::AreaOwner::Charter(1))), None);
    assert_eq!(ownership_news(&driver, run.task), [&notice], "same holder, even through a different owner");
}

#[test]
fn a_triager_save_tells_its_live_charter_role_but_not_a_person() {
    let (mut driver, run) = role_driver(14);
    settings(&mut driver, Some(area(s::AreaOwner::Role(0))), Some(s::AreaOwner::Role(0)));
    filed(&mut driver, 1, None);
    filed(&mut driver, 2, None);
    triage(&mut driver, 2, forge::Triage::Backlog);
    filed(&mut driver, 3, Some(b"tests"));
    settings(&mut driver, None, Some(s::AreaOwner::Charter(14)));
    let subscription = finding_subscription(&driver, run.task).expect("charter holder subscribed");
    assert_eq!(ownership_news(&driver, run.task).len(), 1);
    assert_eq!(ownership_news(&driver, run.task)[0].words.as_ref(), b"Findings on project 1 are yours now: 1, 2");
    let hits = ownership_news(&driver, run.task)[0].hits;
    // A second save uses the existing subscription; both kinds form one group.
    settings(&mut driver, Some(area(s::AreaOwner::Task(run.task))), Some(s::AreaOwner::Task(run.task)));
    assert_eq!(ownership_news(&driver, run.task).len(), 1);
    assert_eq!(ownership_news(&driver, run.task)[0].hits, hits + 1);
    assert_eq!(ownership_news(&driver, run.task)[0].words.as_ref(), b"Findings on project 1 are yours now: 3");
    assert_eq!(finding_subscription(&driver, run.task), Some(subscription));
    let notice = ownership_news(&driver, run.task)[0].clone();
    settings(&mut driver, Some(area(s::AreaOwner::Party(1))), Some(s::AreaOwner::Party(1)));
    assert_eq!(ownership_news(&driver, run.task), [&notice], "people hear through Needs you, not news");
    assert_eq!(engine::finding_read(&driver.root, 1, 1).expect("finding").row.holder, forge::Holder::Party(1));
}

#[test]
fn a_save_groups_both_kinds_and_cuts_ordered_numbers_with_the_omitted_count() {
    let (mut driver, run) = role_driver(1);
    settings(&mut driver, Some(area(s::AreaOwner::Role(0))), Some(s::AreaOwner::Role(0)));
    for number in 1..=40 { filed(&mut driver, number, if number % 2 == 0 { Some(b"tests") } else { None }); }
    settings(&mut driver, Some(area(s::AreaOwner::Task(run.task))), Some(s::AreaOwner::Task(run.task)));
    let words = ownership_news(&driver, run.task);
    assert_eq!(words.len(), 1, "one group across kinds 12 and 13");
    let expected = format!("Findings on project 1 are yours now: {}, and 18 more", (1..=22).map(|n| n.to_string()).collect::<Vec<_>>().join(", "));
    assert_eq!(words[0].words.as_ref(), expected.as_bytes());
    assert!(words[0].words.len() <= usize::try_from(driver.env.limits.tasks.message_bytes).expect("message bound"));
}

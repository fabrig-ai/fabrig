//! A seed replays owner changes, standing, filings, hand-offs and triage (domain/areas.md, 8).
use crate::findings as f;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Rng, Token};
use std::collections::BTreeSet;
use temper_engine_domain::{self as engine, Application, Delivery, Key, Record};
use temper_engine_domain_forge::{self as forge, settings as s};
use temper_engine_domain_world::direct::Driver;

fn ask(driver: &mut Driver, value: people::Ask) {
    driver.serial += 1;
    let serial = driver.serial;
    let mut key = [0; 16];
    key[..8].copy_from_slice(&serial.to_be_bytes());
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::default(),
        reply_to: ReplyTo::new(Token::new(serial)),
        sign_in: driver.session(),
        key,
        ask: value,
    }));
    driver.settle();
}
fn roles(driver: &mut Driver, party: u64, present: bool) {
    let mut holdings = vec![people::Holding { person: 1, role: people::Role::Owner }];
    if present {
        holdings.push(people::Holding { person: party, role: people::Role::Owner });
    }
    ask(driver, people::Ask::SetRoles { project: 1, holdings: holdings.into_boxed_slice() });
}
fn is_live(driver: &Driver, task: u64) -> bool {
    let Some(row) = driver.root.tasks().task(task) else { return false };
    row.project == 1
        && matches!(
            row.phase,
            tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. }
        )
}
fn stands(driver: &Driver, owner: s::AreaOwner) -> Option<forge::Holder> {
    match owner {
        s::AreaOwner::Role(number) => Some(forge::Holder::Role(number)),
        s::AreaOwner::Task(number) => is_live(driver, number).then_some(forge::Holder::Task(number)),
        s::AreaOwner::Party(number) => {
            let row = driver.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::Roles(1))));
            if let Some(Record::Core(jig_domain_core::Record::People(people::Stored::Roles { holdings, .. }))) = row {
                holdings.iter().any(|h| h.person == number).then_some(forge::Holder::Party(number))
            } else {
                None
            }
        }
        s::AreaOwner::Charter(charter) => driver
            .root
            .tasks()
            .view_tasks()
            .iter()
            .filter_map(|view| {
                let row = driver.root.tasks().task(view.number)?;
                (is_live(driver, row.number) && row.label == 3 && row.executor == tasks::Executor::Agent { charter })
                    .then_some(row.number)
            })
            .min()
            .map(forge::Holder::Task),
    }
}
fn expected(driver: &Driver, finding: &forge::Finding, doc: &forge::Findings) -> forge::Holder {
    let area = doc.areas.iter().find(|area| finding.area.as_deref() == Some(area.name.as_ref()));
    area.and_then(|area| stands(driver, area.owner))
        .or_else(|| doc.triager.and_then(|owner| stands(driver, owner)))
        .unwrap_or(forge::Holder::Role(0))
}
fn check(driver: &Driver, heard: &mut BTreeSet<(u64, u32, Box<[u8]>)>, sent: &mut Vec<(u64, u64, Box<[u8]>)>, before_save: Option<&forge::Findings>) {
    let doc = engine::findings_document(&driver.root, 1, 1, None, false);
    let mut numbers = BTreeSet::new();
    for row in &doc.rows {
        assert!(numbers.insert(row.number), "a finding has one row and one holder");
        let value = engine::finding_read(&driver.root, 1, row.number).expect("listed finding");
        let Some(Record::Forge { row: saved }) =
            driver.store.rows.get(&Key::Forge(forge::Key::Finding { project: 1, number: row.number }))
        else {
            panic!("durable finding")
        };
        let forge::Stored::Finding(saved) = saved.as_ref() else { panic!("finding family") };
        assert_eq!(row.holder, expected(driver, saved, &doc));
        assert_eq!(value.row, *row);
    }
    let mut saved_news = BTreeSet::new();
    for view in driver.root.tasks().view_tasks() {
        let Some(task) = driver.root.tasks().task(view.number) else { continue };
        for word in &task.inbox {
            if !matches!(word.kind, tasks::MessageKind::News { .. })
                || !heard.insert((word.number, word.hits, word.words.clone()))
            {
                continue;
            }
            let text = std::str::from_utf8(&word.words).expect("news is UTF-8");
            if let Some(numbers) = text.strip_prefix("Findings on project 1 are yours now: ") {
                let before = before_save.expect("owner news only follows a save");
                let gained: Vec<_> = doc.rows.iter().filter(|row| !row.state.ended()
                    && row.holder == forge::Holder::Task(task.number)
                    && before.rows.iter().any(|old| old.number == row.number && old.holder != row.holder))
                    .map(|row| row.number).collect();
                assert!(!gained.is_empty(), "news after a save needs a newly held finding");
                assert!(saved_news.insert(task.number), "one news per new task holder");
                let mut named = Vec::new();
                let mut omitted = 0_usize;
                for number in numbers.split(", ") {
                    if let Some(more) = number.strip_prefix("and ").and_then(|s| s.strip_suffix(" more")) {
                        omitted = more.parse().expect("omitted finding count");
                    } else {
                        named.push(number.parse::<u32>().expect("ordered finding number"));
                    }
                }
                assert_eq!(named, gained[..named.len()], "only gained findings, in number order");
                assert_eq!(named.len() + omitted, gained.len(), "all gained findings accounted for");
                sent.push((word.number, task.number, word.words.clone()));
                continue;
            }
            let address = text.strip_prefix("Finding 1/").expect("finding news");
            let number: u32 = address.split(' ').next().expect("number").parse().expect("decimal finding number");
            let finding = engine::finding_read(&driver.root, 1, number).expect("news names a finding");
            if address.contains(" for you:") {
                assert_eq!(finding.goal, Some(task.number));
            } else {
                assert_eq!(finding.row.holder, forge::Holder::Task(task.number), "news uses the rule in that step");
            }
            sent.push((word.number, task.number, word.words.clone()));
        }
    }
    if let Some(before) = before_save {
        let expected: BTreeSet<_> = doc.rows.iter().filter_map(|row| {
            let forge::Holder::Task(task) = row.holder else { return None };
            (!row.state.ended() && before.rows.iter().any(|old| old.number == row.number && old.holder != row.holder)).then_some(task)
        }).collect();
        assert_eq!(saved_news, expected, "every new task holder hears once after a save");
    }
}
#[derive(Debug, PartialEq, Eq)]
struct Replay {
    findings: Vec<(u32, forge::FindingAnswer)>,
    news: Vec<(u64, u64, Box<[u8]>)>,
}
fn scenario() -> (Driver, engine::Assignment, u64, u64) {
    let (mut driver, run) = f::driver();
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(991)),
        identity: people::Identity {
            key: forge::user_key(1, 8),
            login: b"second".as_slice().into(),
            name: b"Second".as_slice().into(),
        },
    }));
    driver.settle();
    let party = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::WebReply { reply: people::Reply::SignedIn { person, .. }, .. } = delivery {
                Some(*person)
            } else {
                None
            }
        })
        .expect("second party");
    roles(&mut driver, party, true);
    let goal_ask = temper_engine_domain_world::walking::goal_from(
        driver.chat_ask(1, b"fix findings"),
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 1, deadline: None, priority: 1 },
    );
    ask(&mut driver, goal_ask);
    let goal = driver.root.tasks().view_tasks().iter().find(|task| task.number != run.task).expect("goal").number;
    (driver, run, goal, party)
}
fn run(seed: u64) -> Replay {
    let (mut driver, run, goal, party) = scenario();
    let mut rng = Rng::new(seed);
    let mut heard = BTreeSet::new();
    let mut sent = Vec::new();
    f::settings(&mut driver, Some(f::area(s::AreaOwner::Task(run.task))), Some(s::AreaOwner::Role(0)));
    f::application(&mut driver, Application::FileFinding(f::file(Some(b"tests"))));
    check(&driver, &mut heard, &mut sent, None);
    let mut ended = false;
    for _ in 0..80 {
        let draw = rng.below(5);
        let owner = match draw {
            0 => s::AreaOwner::Task(run.task),
            1 => s::AreaOwner::Party(party),
            2 => s::AreaOwner::Role(0),
            3 => s::AreaOwner::Role(2),
            _ => s::AreaOwner::Charter(1),
        };
        let step = rng.below(9);
        let before_save = (step == 0 || step == 1).then(|| engine::findings_document(&driver.root, 1, 1, None, false));
        match step {
            0 => f::settings(
                &mut driver,
                Some(if rng.below(2) == 0 { f::area(owner) } else { s::ProjectAreas { areas: Box::default() } }),
                None,
            ),
            1 => f::settings(&mut driver, None, Some(owner)),
            2 => {
                roles(&mut driver, party, rng.below(2) == 0);
            }
            3 if !ended => {
                ask(
                    &mut driver,
                    people::Ask::Cancel {
                        project: 1,
                        task: run.task,
                        reason: b"role ended".as_slice().into(),
                        grace: None,
                    },
                );
                ended = true;
            }
            4 | 5 => {
                f::application(
                    &mut driver,
                    Application::FileFinding(f::file(if rng.below(2) == 0 { Some(b"tests") } else { None })),
                );
            }
            _ => {
                let doc = engine::findings_document(&driver.root, 1, 1, None, false);
                if !doc.rows.is_empty() {
                    let row = &doc.rows[usize::try_from(rng.below(u64::try_from(doc.rows.len()).expect("row count")))
                        .expect("row index")];
                    let request = if rng.below(2) == 0 {
                        Application::HandFinding(forge::HandFinding {
                            project: 1,
                            number: row.number,
                            area: b"tests".as_slice().into(),
                            why: b"drawn hand".as_slice().into(),
                        })
                    } else {
                        Application::TriageFinding(forge::TriageFinding {
                            project: 1,
                            number: row.number,
                            act: match rng.below(3) {
                                0 => forge::Triage::Backlog,
                                1 => forge::Triage::Drop(b"drawn drop".as_slice().into()),
                                _ => forge::Triage::Route(goal),
                            },
                        })
                    };
                    f::application(&mut driver, request);
                }
            }
        }
        check(&driver, &mut heard, &mut sent, before_save.as_ref());
    }
    let doc = engine::findings_document(&driver.root, 1, 1, None, false);
    Replay {
        findings: doc
            .rows
            .iter()
            .map(|row| (row.number, engine::finding_read(&driver.root, 1, row.number).expect("final finding")))
            .collect(),
        news: sent,
    }
}
#[test]
fn every_step_has_one_derived_holder_and_news_uses_that_steps_holder() {
    for seed in [0xa4ea_0012, 0xf1ad_0013] {
        assert_eq!(run(seed), run(seed), "seed {seed}");
    }
}

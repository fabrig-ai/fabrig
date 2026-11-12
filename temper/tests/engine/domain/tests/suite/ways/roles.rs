//! Standing roles and follow producers keep their first charter's place in the reader's view.
use super::*;
use jig_domain_tasks as tasks;
use temper_engine_domain_world::oracle;

const CHARTERS: u32 = 20;

fn follow(producer: Option<&[u8]>) -> settings::Follow {
    settings::Follow {
        source: settings::FollowSource {
            repository: skein_forge::api::Repository { forge: FORGE, repository: 3 },
            branch: b"main".as_slice().into(),
            paths: Box::new([]),
        },
        target: settings::FollowTarget {
            repository: skein_forge::api::Repository { forge: FORGE, repository: 2 },
            branch: b"main".as_slice().into(),
        },
        owner: settings::FollowOwner::Person(1),
        start: settings::FollowStart::Owner,
        producer: producer.map(Box::from),
        check: settings::FollowCheck {
            workflow: b"tests.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Lines,
        },
    }
}

fn driver(producers: &[(u32, Option<&[u8]>)]) -> (Driver, jig_domain_core::Template) {
    let mut limits = kinds::limits();
    limits.tasks.charters = CHARTERS;
    limits.tasks.tasks = 8;
    limits.tasks.project_tasks = 8;
    limits.tasks.project_goals = 8;
    limits.people.requests = 16;
    limits.people.inbox_tasks = 16;
    limits.documents.executors = CHARTERS;
    limits.authority.executors = CHARTERS;
    limits.tasks.executor_kinds = CHARTERS;
    let mut config = kinds::configuration(&limits);
    let exemplar = config.core.defaults.charters[0].clone();
    config.core.defaults.charters = (1..=CHARTERS)
        .map(|number| {
            let mut charter = exemplar.clone();
            charter.number = number;
            charter.policy.instructions = b"Keep this project's work in order. Read its records.".as_slice().into();
            charter
        })
        .collect();
    let executors: Box<[_]> = (1..=CHARTERS).map(jig_domain_authority::Executor::Charter).collect();
    config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&executors);
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds.clone_from(&executors);
    policy.roles[0].authority.delegation.kinds = executors;
    let stock = temper_engine_service::stock::with_models(
        Box::new([core_settings::CatalogueModel {
            first_read: None,
            number: 1,
            model: config.core.settings.models[0].clone(),
            shown: None,
            description: None,
            picker: true,
            withdrawn: false,
            source: core_settings::ModelSource::Stock,
            estimated: false,
        }]),
        limits.tasks.result_bytes,
    );
    let template = stock.templates.iter().find(|template| template.number == 7).expect("standing template").clone();
    let mut templates = config.core.defaults.templates.into_vec();
    templates.push(template.clone());
    config.core.defaults.templates = templates.into_boxed_slice();
    let mut bootstrap = Driver::cold(Store::with_limits(&limits), oracle::config::copy(&config), &limits);
    bootstrap.settle();
    let mut store = bootstrap.take_store();
    // Restore current kind 12 values; no follow changes need to run to read their producers.
    store.seed(
        producers
            .iter()
            .map(|(number, producer)| {
                let value = follow(*producer);
                engine::Record::Forge {
                    row: Box::new(forge::Stored::Setting(settings::SettingRow {
                        key: settings::SettingKey {
                            kind: 12,
                            scope: PROJECT,
                            number: *number,
                            slot: settings::SettingSlot::Change,
                        },
                        edit: settings::Edit::Set(settings::SizedValue {
                            size: settings::follow_record(&value).expect("follow record").measure(),
                            value: settings::Value::Follow(value),
                        }),
                    })),
                }
            })
            .collect(),
        Box::new([]),
    );
    let mut driver = Driver::cold(store, config, &limits);
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.period();
    driver.settle();
    (driver, template)
}

fn start(driver: &mut Driver, session: u64, template: &jig_domain_core::Template, key: u8, charter: u32) -> u64 {
    let people::Ask::Create { project, mut batch } = driver.chat_ask(PROJECT, b"Keep the work in order") else {
        panic!("task fixture")
    };
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Agent { charter };
    assert_eq!(template.flags.ended_by, tasks::EndedBy::Standing);
    member.flags = people::Flags {
        ended_by: people::EndedBy::Standing,
        conversation: template.flags.conversation,
        listed: template.flags.listed,
        retention: people::Retention::Kept,
    };
    member.label = template.label;
    member.tracked = template.tracked;
    member.budget = people::Budget::Named;
    member.authority.spend = 1;
    driver.delivered.clear();
    driver.send(Event::Party(PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: session,
        key: [key; 16],
        models: Box::new([]),
        ask: people::Ask::Create { project, batch },
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Created { first, .. }), .. } => {
                Some(*first)
            }
            _ => None,
        })
        .expect("standing role created")
}

fn cancel(driver: &mut Driver, session: u64, key: u8, task: u64) {
    driver.send(Event::Party(PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: session,
        key: [key; 16],
        models: Box::new([]),
        ask: people::Ask::Cancel { project: PROJECT, task, reason: b"Done".as_slice().into(), grace: None },
    }));
    driver.settle();
    assert!(driver.store.rows.contains_key(&engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task)))));
}

fn cards(driver: &mut Driver, session: u64) -> Vec<(u32, String, String)> {
    let ways::View::Roles(roles) = read_view(driver, session, "roles").expect("reader's roles") else {
        panic!("roles view")
    };
    roles
        .into_iter()
        .map(|(number, card)| {
            (
                number,
                String::from_utf8(card.name.into()).expect("role name"),
                String::from_utf8(card.who.into()).expect("role who"),
            )
        })
        .collect()
}

#[test]
fn roles_append_live_standing_charters_then_follow_producers_once_in_source_order() {
    // Insert out of number order; 13 is both a standing charter and a follow producer.
    let (mut driver, template) = driver(&[
        (9, Some(b"16;;")),
        (1, None),
        (2, Some(b";;")),
        (8, Some(b"17;0.1;")),
        (3, Some(b"2;;")),
        (4, Some(b"1;;")),
        (5, Some(b"21;;")), // Not held in settings.
        (6, Some(b"13;;")),
        (7, Some(b"")),
        (10, Some(b"17;;")),
    ]);
    let session = driver.session();
    let role = start(&mut driver, session, &template, 50, 13);
    assert_eq!(driver.task(role).expect("live role").label, 3);
    let initial = cards(&mut driver, session);
    assert_eq!(initial.iter().map(|(number, _, _)| *number).collect::<Vec<_>>(), [1, 2, 4, 9, 13, 17, 16]);
    assert_eq!(initial[3].1, "Status agent");
    assert_eq!(initial[4].1, "Standing role");
    assert_eq!(initial[4].2, "For each standing role: owns its work for the project's life and keeps its Now");

    cancel(&mut driver, session, 51, role);
    // Task number order is the opposite of charter order.
    start(&mut driver, session, &template, 52, 14);
    start(&mut driver, session, &template, 53, 13);
    start(&mut driver, session, &template, 54, 13);
    start(&mut driver, session, &template, 55, 1); // Chat keeps the charter's first place.
    let ended = start(&mut driver, session, &template, 56, 15);
    cancel(&mut driver, session, 57, ended);
    let got = cards(&mut driver, session);
    assert_eq!(got.iter().map(|(number, _, _)| *number).collect::<Vec<_>>(), [1, 2, 4, 9, 13, 14, 17, 16]);
    assert_eq!(got[4], initial[4], "two roles under 13 still give one card");
    assert_eq!(got[5].1, "Standing role");
    assert_eq!(
        got[5].2,
        "For each standing role: Routes what comes in to its area's owner, and triages what fits no area."
    );
    for (number, name, who) in &got[6..] {
        assert_eq!(name, "Follow's coding agent");
        let words = if *number == 16 {
            "For each follow change: Keeps its area's port in step with its source."
        } else {
            "For each follow change: Keep this project's work in order."
        };
        assert_eq!(who, words);
    }
}

#[test]
fn roles_have_room_for_every_configured_charter_not_just_sixteen_cards() {
    let producers: Vec<_> = (1..=CHARTERS).map(|number| (number, format!("{number};;"))).collect();
    let follows: Vec<_> = producers.iter().map(|(number, spelling)| (*number, Some(spelling.as_bytes()))).collect();
    let (mut driver, _) = driver(&follows);
    let session = driver.session();
    let got = cards(&mut driver, session);
    assert_eq!(got.len(), usize::try_from(CHARTERS).expect("charter bound"));
    let mut numbers: Vec<_> = got.iter().map(|(number, _, _)| *number).collect();
    numbers.sort_unstable();
    assert_eq!(numbers, (1..=CHARTERS).collect::<Vec<_>>());
}

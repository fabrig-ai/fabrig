//! A change keeps the forge's charters from its start (temper/docs/design/domain/forge.md, 8.9).
use super::*;
use jig_domain_core::settings as s;
use temper_engine_domain_forge::settings;

const STEPS: u32 = 200;

fn world(producer: u32) -> World {
    World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate { agent: true, ..Gate::default() },
        None,
        StoryLimits { kinds: true, forge_charter: Some(producer), ..StoryLimits::default() },
    )
}

fn row(world: &World, task: u64) -> &forge_top::ChangeRow {
    match world.store.rows.get(&Key::Forge(forge_top::Key::Change(task))) {
        Some(Record::Forge { row }) => match row.as_ref() {
            forge_top::Stored::Change(row) => row,
            _ => panic!("change row"),
        },
        _ => panic!("durable change"),
    }
}

fn revision(world: &World) -> u32 {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                s::Record::Revision { history: false, revision },
            ))) => Some(revision.head.number),
            _ => None,
        })
        .max()
        .expect("settings head")
}

fn charter(number: u32, value: u32) -> (s::Item, engine::KindValue) {
    (
        s::Item {
            setting: s::Setting::Application { kind: 3, scope: 0, number },
            edit: s::Edit::Set(s::Value::Application(value.to_le_bytes().into())),
        },
        engine::KindValue {
            kind: 3,
            scope: 0,
            number,
            value: settings::SizedValue { size: 4, value: settings::Value::Charter(value) },
        },
    )
}

fn apply(world: &mut World, items: Box<[s::Item]>, kinds: Box<[engine::KindValue]>) {
    let against = revision(world);
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(300)),
        sign_in: world.signed_in.expect("deployment owner"),
        key: [30; 16],
        digest: [30; 32],
        change: Box::new(s::Change { against, items, running: s::Running::Stays }),
        kinds,
    }));
    world.until(Until::Ready);
    assert_eq!(revision(world), against + 1, "the settings change commits: {:?}", world.web_replies);
}

fn delegate(world: &mut World, change: u64, charter: u32) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if let Some(run) = world.assigned.iter().find(|run| {
            run.charter == charter
                && world
                    .store
                    .rows
                    .get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(run.task))))
                    .or_else(|| {
                        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(run.task))))
                    })
                    .is_some_and(|record| {
                        matches!(record,
                    Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                    if row.requester == tasks::Party::Task(change))
                    })
        }) {
            return run.clone();
        }
    }
    panic!(
        "change {change} has no charter {charter} delegate; phase={:?}, row={:?}, assigned={:?}",
        live_task(world, change).phase,
        row(world, change),
        world.assigned.iter().map(|run| (run.task, run.charter)).collect::<Vec<_>>()
    );
}

#[test]
fn a_change_reads_the_forges_charters_when_made() {
    let (mut world, chat, producer, branch) = change_world_in(world(1), Change::default());
    let tasks::Party::Task(first) = live_task(&world, producer.task).requester else { panic!("producer's change") };
    assert_eq!(row(&world, first).review_charter, 4);
    let (item, value) = charter(2, 2);
    apply(&mut world, Box::new([item]), Box::new([value]));
    reviews::produce(&mut world, &producer, &branch);
    let first_review = delegate(&mut world, first, 4);
    assert_eq!(first_review.charter, 4, "a live change keeps its review charter");

    let engine::CallAnswer::Delegated(made) = change_tool::change(&mut world, &chat, 94, None, b"Next fix") else {
        panic!("next change delegated")
    };
    let second = made[0].task;
    // The two worker places now belong to the two changes' agents.
    let event =
        world.protocol.answer(Token::new(7), chat.task, chat.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    let producer = delegate(&mut world, second, 1);
    let branch = row(&world, second).branch.clone();
    assert_eq!(row(&world, second).review_charter, 2);
    reviews::produce(&mut world, &producer, &branch);
    let second_review = delegate(&mut world, second, 2);
    assert_eq!(second_review.charter, 2);
    assert_eq!(row(&world, first).review_charter, 4);
    assert!(matches!(live_task(&world, first_review.task).executor, tasks::Executor::Agent { charter: 4 }));
    world.close();
}

#[test]
fn a_change_whose_charter_is_no_longer_offered_is_held() {
    let (mut world, _, producer, branch) = change_world_in(world(2), Change::default());
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("producer's change") };
    let mut rules = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                s::Record::Default {
                    setting: s::Setting::Rules,
                    default: s::Default { value: s::Value::Rules(rules), .. },
                },
            ))) => Some(rules.as_ref().clone()),
            _ => None,
        })
        .expect("default rules");
    rules.ceiling.delegation.kinds = rules
        .ceiling
        .delegation
        .kinds
        .iter()
        .filter(|kind| **kind != authority::Executor::Charter(2))
        .copied()
        .collect();
    let (item, value) = charter(1, 1);
    apply(
        &mut world,
        Box::new([item, s::Item { setting: s::Setting::Rules, edit: s::Edit::Set(s::Value::Rules(Box::new(rules))) }]),
        Box::new([value]),
    );
    reviews::produce(&mut world, &producer, &branch);
    let reviewer = delegate(&mut world, change, 4);
    let assigned = world.assigned.len();
    reviews::finish_review(
        &mut world,
        &reviewer,
        tasks::TaskResult::Verdict {
            code: 2,
            fields: Box::default(),
            words: b"repair the unsafe edge".as_slice().into(),
        },
    );
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, change).phase else {
        panic!("unoffered producer holds its change");
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("forge's hold account") };
    assert_eq!(
        words.as_ref(),
        b"the change's producer charter 2 is no longer offered; cancel the change or make it again"
    );
    assert_eq!(world.assigned.len(), assigned, "the repair never starts");
    world.close();
}

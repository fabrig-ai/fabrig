//! Kinds through the party entrance and the root's real store
//! (temper/docs/design/domain/routing.md, 6, and temper/docs/design/domain/root.md, 8.1;
//! temper/docs/design/protocol/configuration/settings.md, 2 to 3 and temper/docs/design/protocol/configuration/stock.md).
use jig_domain_core::settings as s;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Key, Record};
use temper_engine_domain_forge::settings;

use crate::{direct::Driver, store::Store, walking};

pub const SEED: u64 = 9763;

/// Stock kinds, with the walking world's small agents and deterministic clock.
#[must_use]
pub fn configuration(limits: &engine::Limits) -> engine::Config {
    let mut config = walking::config(SEED);
    config.core.kinds = temper_service::stock::kinds(limits, config.core.connectors[0]);
    config.kinds_stock = temper_service::stock::forge(limits);
    config.core.defaults.charters = [1, 2, 4]
        .map(|number| {
            let mut charter = walking::charter();
            charter.number = number;
            charter
        })
        .into();
    config
}

#[must_use]
pub fn limits() -> engine::Limits {
    let mut limits = walking::limits();
    // The stock's charter lines, kind 9, run up to charter 16.
    limits.tasks.charters = 16;
    // A complete duplicate-gate value must reach the forge judge.
    limits.people.amendment_bytes = 2048;
    limits
}

#[must_use]
pub fn driver() -> Driver {
    let limits = limits();
    let mut driver = Driver::cold(Store::with_limits(&limits), configuration(&limits), &limits);
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}

/// The raw item and the entrance-decoded value travel beside one another.
#[must_use]
pub fn item(kind: u32, scope: u32, number: u32, value: &settings::Value) -> s::Item {
    s::Item {
        setting: s::Setting::Application { kind, scope, number },
        edit: s::Edit::Set(s::Value::Application(temper_engine_smith::kinds::encode(value))),
    }
}

#[must_use]
pub fn decoded(items: &[s::Item]) -> Box<[engine::KindValue]> {
    items
        .iter()
        .filter_map(|item| {
            let s::Setting::Application { kind, scope, number } = item.setting else { return None };
            let s::Edit::Set(s::Value::Application(bytes)) = &item.edit else { return None };
            // The party entrance decodes at the wire's schema ceilings;
            // the forge judges the deployment's smaller bounds afterwards.
            // temper-client-protocol-wire/src/generated/v1.rs, CEILINGS.
            let bounds =
                temper_engine_smith::kinds::Bounds { segments: 16, segment_bytes: 4096, gates: 64, approvals: 64 };
            Some(engine::KindValue {
                kind,
                scope,
                number,
                value: temper_engine_smith::kinds::decode(kind, bytes, &bounds).expect("entrance decodes value"),
            })
        })
        .collect()
}

#[must_use]
pub fn change(sign_in: u64, against: u32, key: u8, items: Box<[s::Item]>) -> engine::Event {
    engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in,
        key: [key; 16],
        digest: [key; 32],
        kinds: decoded(&items),
        change: Box::new(s::Change { against, items, running: s::Running::Stays }),
    })
}

#[must_use]
pub fn proposal(sign_in: u64, against: u32, key: u8, items: Box<[s::Item]>) -> engine::Event {
    engine::Event::Party(engine::PartyInput::ProposeSettings {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in,
        key: [key; 16],
        digest: [key; 32],
        kinds: decoded(&items),
        change: Box::new(s::Proposed { against, items, reason: b"change the landing rules".as_slice().into() }),
    })
}

#[must_use]
pub fn revision(rows: &std::collections::BTreeMap<Key, Record>) -> u32 {
    rows.values()
        .filter_map(|row| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                s::Record::Revision { history: false, revision },
            ))) = row
            {
                Some(revision.head.number)
            } else {
                None
            }
        })
        .max()
        .expect("current settings revision")
}

/// The requester-backed forge world replays seed 71. Its own small agents
/// and empty landing defaults leave one project's goal rule to the story.
#[must_use]
pub fn forge_world() -> crate::forge_routes::World {
    use crate::forge_routes::{Gate, StoryLimits, World};
    World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { kinds: true, forge_charter: Some(2), ..StoryLimits::default() },
    )
}

/// Adopt the repository and start a tracked goal holding its own branch.
#[must_use]
pub fn goal(world: &mut crate::forge_routes::World) -> engine::Assignment {
    use crate::forge_routes::Until;
    use jig_domain_people as people;
    world.adopt();
    world.send(crate::protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = crate::tier_runs::goal(world) else { panic!("goal batch") };
    batch[0].symbols = Box::new([people::Symbol { connector: 0, symbol: 1 }]);
    batch[0].contract = people::Contract::Report { words: 128 };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(92)),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: people::Ask::Create { project, batch },
    }));
    world.until(Until::Assigned);
    world.assigned[0].clone()
}

/// A change made by the goal, using its held branch as the landing.
#[must_use]
pub fn goal_change(world: &mut crate::forge_routes::World, goal: &engine::Assignment) -> (u64, engine::Assignment) {
    use crate::forge_routes::Until;
    world.until(Until::Ready);
    let event = world.protocol.call(
        Token::new(7),
        goal.task,
        goal.attempt,
        93,
        engine::Call {
            completion: 1,
            position: 1,
            tool: engine::Tool::Change {
                changes: Box::new([engine::ChangeCall {
                    landing: Some(engine::ChangeLanding::Goal),
                    ..engine::ChangeCall::words(b"Small fix".as_slice().into(), None)
                }]),
            },
        },
    );
    world.send(event);
    world.until(Until::Delegated);
    let engine::CallAnswer::Delegated(made) = world.answers.last().expect("change answer") else {
        panic!("change delegated: {:?}", world.answers)
    };
    let task = made[0].task;
    world.until(Until::SecondAssignment);
    (task, world.assigned[1].clone())
}

#[must_use]
pub fn change_row(world: &crate::forge_routes::World, task: u64) -> &temper_engine_domain_forge::ChangeRow {
    use temper_engine_domain_forge as forge;
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge::Key::Change(task))) else {
        panic!("durable change")
    };
    let forge::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

/// Push and finish the producer without giving its head any CI verdict.
pub fn produce(world: &mut crate::forge_routes::World, change: u64, producer: &engine::Assignment) {
    use jig_domain_tasks as tasks;
    use skein_fake_forge_domain::{self as fake, api as raw};
    let branch = change_row(world, change).branch.clone();
    let landing = change_row(world, change).base.clone();
    let tip = world.fake.branch(b"org/repo", &landing).expect("goal branch exists");
    assert_eq!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: tip })),
        raw::Answer::Branch(raw::Created::Created)
    );
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"small fix", 1)
        .expect("producer pushed");
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(1, 2),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

/// Cut after the real store keeps the rule decision, before the root hears
/// its commit terminal and releases any steps of the changes it covers.
pub fn restart_after_rule_commit(world: &mut crate::forge_routes::World, key: u8, item: s::Item) {
    use temper_engine_domain_forge as forge;
    for _ in 0..200 {
        if world.pending.is_empty() && world.events.is_empty() && world.store.pending.is_empty() {
            break;
        }
        world.tick();
    }
    assert!(world.pending.is_empty(), "forge reads settle before the settings cut");
    let against = revision(&world.store.rows);
    world.send(change(world.signed_in.expect("owner session"), against, key, Box::new([item])));
    for _ in 0..200 {
        let target = world.store.pending.front().is_some_and(|(_, writes)| {
            writes.iter().any(|write| {
                matches!(write, engine::Write::Save(Record::Forge { row })
                if matches!(row.as_ref(), forge::Stored::Setting(settings::SettingRow {
                    key: settings::SettingKey { kind: 2, scope: 1, number: 1, slot: settings::SettingSlot::Change }, ..
                })))
            })
        });
        if target {
            // apply uses the world's real store; only its terminal is lost.
            let _commit = world.store.apply();
            assert_eq!(revision(&world.store.rows), against + 1);
            assert!(world.store.pending.is_empty());
            assert_eq!(world.merges, 0, "no merge precedes the restart cut");
            world.store.crash(0);
            world.restart_same();
            return;
        }
        world.tick();
    }
    panic!("settings decision did not reach the store");
}

/// Restore through the same party entrance as a settings proposal
/// (jig/docs/design/domain/engine/settings.md, 10).
#[must_use]
pub fn restore(sign_in: u64, revision: u32, key: u8) -> engine::Event {
    engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in,
        key: [key; 16],
        ask: jig_domain_people::Ask::Restore { revision, reason: b"Return to the reviewed rules".as_slice().into() },
    })
}

#[must_use]
pub fn restored(driver: &Driver) -> (u64, Box<[s::Setting]>) {
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let engine::Delivery::WebReply {
                reply:
                    jig_domain_people::Reply::Outcome(jig_domain_people::Outcome::Restored { proposal, to, left_out }),
                ..
            } = delivery
            {
                assert_eq!(*to, jig_domain_people::Role::Owner.number());
                Some((*proposal, left_out.clone()))
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("restore answer: {:?}", driver.delivered))
}

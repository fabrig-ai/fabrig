//! The stock ways' acts cross the client and engine's commit boundary
//! (temper/docs/design/client/README.md, 5.13; temper/docs/design/domain/testing.md, 2.1).
use jig_client::contract as c;
use jig_client_kit::harness::{App, ClientId, Settings, Until, World};
use skein_world::domain::Verdict;
use temper_client_domain::{
    Action, Address, Event,
    forge::stock_ways::{State, StockWays, WayAct},
    pages::{Act, home},
};
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{
        Joint,
        ways::{self, Reader},
    },
};
use temper_engine_domain as engine;
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_domain_world::kinds;

const PROJECT: u32 = 1;
const APPROVE: u32 = 0;
const REVIEW: u32 = 1;
const DRAFT: u32 = 2;
type Clients = World<Temper, Joint>;

fn world(reader: Reader) -> (Clients, ClientId) {
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() },
        ways::fixture(fixtures::SEED, &fixtures::domain_limits(), reader),
    );
    world.randomness(0, 0, 0);
    let caller = world.front().caller;
    let person = world.client(caller, Address::Ask { project: PROJECT });
    settle(&mut world);
    (world, person)
}
fn settle(world: &mut Clients) {
    let outcome = world.run(Until::Time(world.now().saturating_add(fixtures::WITHIN)));
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn page(world: &Clients, person: ClientId) -> &StockWays {
    world.root(person).person().pages().stock_ways().expect("Ask holds StockWays")
}
fn panel(world: &Clients, person: ClientId) -> &temper_client_protocol_wire::v1::WaysPanel {
    let State::Shown(panel) = page(world, person).state() else { panic!("ways shown: {:?}", page(world, person)) };
    panel
}
fn act(world: &mut Clients, person: ClientId, act: Act) {
    world.event(person, Event::Act { action: Action::Pages(act) }.into());
    settle(world);
}
fn current(world: &Clients, person: ClientId, index: u32, name: &[u8]) {
    assert_eq!(*panel(world, person).current(), Some(index));
    let tabs = page(world, person).tabs();
    let current = tabs.iter().filter(|tab| tab.current).collect::<Vec<_>>();
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].index, Some(index));
    assert_eq!(current[0].name.as_ref(), name);
}

#[test]
fn keep_the_current_way_completes_to_begin_and_is_not_offered_again() {
    let (mut world, person) = world(Reader::Owner);
    current(&world, person, REVIEW, b"Agents review");
    assert_eq!(page(&world, person).act(REVIEW), Some(WayAct::Keep));
    let before = kinds::revision(&world.front().engine.store.rows);
    act(&mut world, person, Act::KeepWay { project: PROJECT });
    assert_eq!(kinds::revision(&world.front().engine.store.rows), before + 1, "Keep commits its settings");
    assert_eq!(page(&world, person).refused(), None);
    current(&world, person, REVIEW, b"Agents review");
    assert_eq!(page(&world, person).act(REVIEW), None, "Keep goes after the step is done");
    world.event(person, Temper::went(Address::Home { project: PROJECT }));
    settle(&mut world);
    let root = world.root(person).person();
    let facts = home::facts(root, PROJECT, skein_lib::Wall::EPOCH);
    assert!(facts.to_begin(), "repository step is still left: {facts:?}");
    assert!(facts.configured, "Tell temper how you work is done: {facts:?}");
    assert!(
        world.tree(person).nodes().iter().any(|node| {
            node.role == skein_ui::Role::Heading { level: 3 }
                && node.name.as_deref() == Some(b"2. Done. Tell temper how you work".as_slice())
        }),
        "Home draws the completed To begin step"
    );
    world.event(person, Temper::went(Address::Ask { project: PROJECT }));
    settle(&mut world);
    current(&world, person, REVIEW, b"Agents review");
    assert_eq!(page(&world, person).act(REVIEW), None, "a fresh document keeps the choice without Keep");
}

#[test]
fn use_draft_first_commits_the_goal_way() {
    let (mut world, person) = world(Reader::Owner);
    assert_eq!(page(&world, person).act(DRAFT), Some(WayAct::Use));
    let before = kinds::revision(&world.front().engine.store.rows);
    act(&mut world, person, Act::UseWay { project: PROJECT, way: DRAFT });
    assert_eq!(page(&world, person).refused(), None);
    assert_eq!(kinds::revision(&world.front().engine.store.rows), before + 1);
    current(&world, person, DRAFT, b"Draft first");
    let saved = world
        .front()
        .engine
        .store
        .rows
        .values()
        .find_map(|record| match record {
            engine::Record::Forge { row } => match row.as_ref() {
                forge::Stored::Setting(row)
                    if row.key
                        == (settings::SettingKey {
                            kind: 7,
                            scope: PROJECT,
                            number: 1,
                            slot: settings::SettingSlot::Change,
                        }) =>
                {
                    Some(row.edit.clone())
                }
                _ => None,
            },
            engine::Record::Core(_) => None,
        })
        .expect("committed kind 7");
    let settings::Edit::Set(settings::SizedValue { value: settings::Value::Ways(value), .. }) = saved else {
        panic!("kind 7 value")
    };
    assert!(value.goals);
    assert_eq!(value.way, 4);
}

#[test]
fn use_you_approve_refuses_a_box_owner_unknown_on_the_forge() {
    let (mut world, person) = world(Reader::BoxOwner);
    assert_eq!(panel(&world, person).own().repository().as_deref(), Some(b"acme/shop".as_slice()));
    let before = panel(&world, person).clone();
    let revision = kinds::revision(&world.front().engine.store.rows);
    act(&mut world, person, Act::UseWay { project: PROJECT, way: APPROVE });
    let Some(c::Refusal::Judged { setting, why, .. }) = page(&world, person).refused() else {
        panic!("approval refusal on StockWays: {:?}", page(&world, person));
    };
    assert_eq!(*setting, c::Setting::Application { kind: 2, scope: PROJECT, number: 1 });
    let why = std::str::from_utf8(why.bytes()).expect("refusal words");
    assert!(
        why.contains(
            "project 1 role 0 on forge forge.example requires 1 distinct known forge people; currently 0 eligible"
        ),
        "{why}"
    );
    assert_eq!(panel(&world, person), &before, "a refusal leaves the document unchanged");
    assert_eq!(kinds::revision(&world.front().engine.store.rows), revision, "refused settings do not commit");
}

#[test]
fn a_member_reads_the_ways_without_an_act() {
    let (world, person) = world(Reader::Member);
    current(&world, person, REVIEW, b"Agents review");
    let panel = panel(&world, person);
    assert!(!panel.may_change());
    assert_eq!(panel.ways().len(), 3);
    for index in [APPROVE, REVIEW, DRAFT] {
        assert_eq!(page(&world, person).act(index), None);
    }
}

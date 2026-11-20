//! The limits read and kept limit holds reach the shared page and one-act card.
use jig_client::contract as c;
use jig_client_kit::harness::{ClientId, Settings, Until, World};
use skein_fake_person::{Find, Step};
use skein_lib::List;
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::{
    Action, Address, Event,
    pages::{Act, limit_settings},
};
use temper_client_protocol_wire::v1 as w;
use temper_client_protocol_wire::v1::Record;
use temper_client_world::{
    app::Temper,
    fixtures,
    front::{
        Joint,
        ways::{self, Reader},
    },
};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

type Clients = World<Temper, Joint>;
fn world(front: Joint) -> Clients {
    let mut world =
        World::new(Settings { seed: fixtures::SEED, words: fixtures::decision_words(), ..Settings::default() }, front);
    world.randomness(0, 0, 0);
    world
}
fn settle(world: &mut Clients) {
    let outcome = world.run(Until::Time(world.now().saturating_add(fixtures::WITHIN)));
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn follow(world: &mut Clients, person: ClientId, steps: Vec<Step<Address>>) {
    world.person(person, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn see(role: Role, name: &[u8]) -> Step<Address> {
    Step::See { find: Find::exactly_named(role, name), within: fixtures::WITHIN }
}
fn page() -> w::LimitsAnswer {
    let mut rows = List::with_capacity(1);
    rows.push(w::LimitRow {
        name: b"forge.entries".as_slice().into(),
        line: b"Bounds the forge outbox.".as_slice().into(),
        unit: w::LimitUnit::Count,
        stock: Some(8),
        setting: Some(w::LimitSetting {
            kind: w::LimitKind::Temper,
            value: 32,
            reason: b"dev's outbox".as_slice().into(),
        }),
        in_force: Some(16),
        applies: w::LimitApplies::AtOnce,
        waiting: true,
        left_out: false,
        held: None,
    })
    .expect("limits fixture");
    w::LimitsAnswer { rows, next: None, revision: 1 }
}
#[test]
fn how_work_runs_links_to_limits_with_settings_and_existing_capacity_facts() {
    let mut front = ways::fixture(fixtures::SEED, &fixtures::domain_limits(), Reader::Owner);
    front.limit_pages.push_back(page());
    let caller = front.caller;
    let mut world = world(front);
    let person = world.client(caller, Address::Settings { project: 1 });
    follow(
        &mut world,
        person,
        vec![
            see(Role::Link, b"Limits"),
            Step::Press { find: Find::exactly_named(Role::Link, b"Limits") },
            see(Role::Heading { level: 3 }, b"forge.entries"),
            see(Role::Text, b"Stock: 8"),
            see(Role::Text, b"32 (temper-limits: dev's outbox)"),
            see(Role::Text, b"In force: 16"),
            see(Role::Text, b"Waits for the next start"),
            see(Role::Heading { level: 2 }, b"Capacities"),
        ],
    );
    assert_eq!(world.root(person).person().pages().address(), &Address::Limits);
    assert!(world.front().limit_pages.is_empty());
}
fn held(reader: Reader) -> (Clients, u64) {
    let mut front = ways::fixture(fixtures::SEED, &fixtures::domain_limits(), reader);
    let ask = front.engine.chat_ask(1, b"Waiting for limit room");
    front.engine.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1005)),
        sign_in: front.engine.session(),
        key: [35; 16],
        ask,
    }));
    front.engine.settle();
    let task = front.engine.last_assignment().task;
    front.engine.send(engine::Event::Forge(Box::new(forge::Event::Subscribe {
        subscription: forge::Subscriber {
            task,
            number: 100,
            topic: forge::Topic::Landings {
                repository: skein_forge::api::Repository { forge: 1, repository: 7 },
                branch: b"main".as_slice().into(),
            },
            own_change: None,
            goal_tasks: None,
            paths: vec![Box::from(b"src".as_slice()); 5].into_boxed_slice(),
        },
    })));
    front.engine.settle();
    let holds = front.engine.root.forge_limit_holds();
    assert_eq!(holds.len(), 1);
    assert_eq!((holds[0].task, holds[0].configured, holds[0].needed), (task, 4, 5));
    (world(front), task)
}
#[test]
fn a_limit_held_card_sends_one_whole_list_at_the_current_revision_and_stays_running() {
    let (mut world, task) = held(Reader::Owner);
    let caller = world.front().caller;
    let person = world.client(caller, fixtures::inbox());
    follow(&mut world, person, vec![see(Role::Button, b"Raise forge.paths_per_subscription to 8")]);
    let root = world.root(person).person();
    let inbox = root.pages().inbox().expect("limits fixture");
    let row = root
        .pages()
        .limit_settings()
        .hold(inbox.cards.iter().find(|card| card.source() == Some(task)).expect("limits fixture"))
        .expect("limits fixture")
        .clone();
    let against = root.forge().settings_newest().expect("limits fixture");
    world.front().settings_changes.clear();
    follow(
        &mut world,
        person,
        vec![Step::Press { find: Find::exactly_named(Role::Button, b"Raise forge.paths_per_subscription to 8") }],
    );
    settle(&mut world);
    let sent = world.front().settings_changes.clone();
    let saved = kept(world.front());
    release(&mut world, person);
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].against, against);
    assert_eq!(sent[0].running, c::Running::Stays);
    assert_eq!(sent[0].items.len(), 1);
    let item = sent[0].items.get(0).expect("limits fixture");
    assert_eq!(item.setting, c::Setting::Application { kind: 22, scope: 0, number: 1 });
    let c::SettingEdit::Set(c::SettingSet { value: c::SettingValue::Application(encoded) }) = &item.edit else {
        panic!("one limits list")
    };
    let value = w::LimitOverrides::decode(&w::CEILINGS, &mut skein_lib::Reader::new(encoded)).expect("limits fixture");
    assert_eq!(value.overrides.len(), 1);
    let raised = value.overrides.get(0).expect("limits fixture");
    assert_eq!(&raised.name, &row.limit);
    assert_eq!(raised.value, 8);
    assert_eq!(raised.reason.as_ref(), format!("raised for task {task}, which needed 5").as_bytes());
    let kept = saved.expect("limits fixture");
    assert!(
        matches!(kept, forge::settings::Edit::Set(forge::settings::SizedValue { value: forge::settings::Value::Limits(values), .. }) if values.len() == 1 && values[0].value == 8)
    );
}
fn release(world: &mut Clients, person: ClientId) {
    // The restart oracle compares stable work; a startup intentionally releases kept limit holds.
    follow(
        world,
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Release task") },
            Step::Press {
                find: Find::exactly_named(Role::Button, b"Release")
                    .within(Box::new([skein_fake_person::Target::Role { role: Role::Dialog, name: Box::new([]) }])),
            },
        ],
    );
    settle(world);
}
fn kept(front: &Joint) -> Option<forge::settings::Edit> {
    let key = engine::Key::Forge(forge::Key::Setting(forge::settings::SettingKey {
        kind: 22,
        scope: 0,
        number: 1,
        slot: forge::settings::SettingSlot::Change,
    }));
    let engine::Record::Forge { row } = front.engine.store.rows.get(&key)? else { panic!("forge setting") };
    let forge::Stored::Setting(row) = row.as_ref() else { panic!("setting row") };
    Some(row.edit.clone())
}
#[test]
fn a_project_member_reads_that_an_owner_can_raise_a_limit() {
    let (mut world, task) = held(Reader::Member);
    let caller = world.front().caller;
    let person = world.client(caller, fixtures::inbox());
    settle(&mut world);
    let owner_words = world
        .tree(person)
        .nodes()
        .iter()
        .any(|node| node.name.as_deref() == Some(b"A deployment owner can raise it. You can ask temper."));
    let no_raise = world
        .tree(person)
        .nodes()
        .iter()
        .all(|node| node.name.as_deref() != Some(b"Raise forge.paths_per_subscription to 8"));
    let before = kept(world.front());
    world.event(
        person,
        Event::Act { action: Action::Pages(Act::LimitSettings(limit_settings::Act::Raise { task, revision: 1 })) }
            .into(),
    );
    settle(&mut world);
    let after = kept(world.front());
    release(&mut world, person);
    assert!(owner_words);
    assert!(no_raise);
    assert_eq!(after, before);
}

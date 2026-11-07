//! Catalogue switches through both roots (temper/docs/design/client/README.md, 3.5).
use jig_client_kit::harness::{Settings, World};
use jig_domain_core::settings::{CatalogueModel, ModelSource};
use jig_domain_people::InitialOwner;
use skein_fake_person::{Find, Step};
use skein_ui::{Role, State};
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain_world::{store::Store, walking};

fn configuration() -> temper_engine_domain::Config {
    let mut config = walking::config(fixtures::SEED);
    config.core.owners = Box::new([
        config.core.owners[0].clone(),
        InitialOwner { project: 0, identity: config.core.owners[0].identity.clone() },
    ]);
    let mut opus = config.core.settings.models[0].clone();
    opus.name = b"claude-opus-5-5".as_slice().into();
    let mut sonnet = opus.clone();
    sonnet.name = b"claude-sonnet-5-5".as_slice().into();
    config.core.settings.models = Box::new([opus.clone(), sonnet.clone()]);
    config.core.defaults.charters[0].policy.model = opus.clone();
    config.core.defaults.charters[0].policy.alternatives = Box::new([sonnet.clone()]);
    config.core.defaults.models = Box::new([
        CatalogueModel {
            number: 1,
            model: opus,
            shown: Some(b"Opus 5.5".as_slice().into()),
            description: None,
            picker: true,
            withdrawn: false,
            source: ModelSource::Stock,
            estimated: false,
        },
        CatalogueModel {
            number: 2,
            model: sonnet,
            shown: Some(b"Sonnet 5.5".as_slice().into()),
            description: None,
            picker: true,
            withdrawn: false,
            source: ModelSource::Stock,
            estimated: false,
        },
    ]);
    config
}

#[test]
fn an_owner_turns_a_model_off_and_it_leaves_the_picker() {
    let mut limits = Joint::new(fixtures::SEED, fixtures::domain_limits()).engine.env.limits;
    limits.people.initial_owners = 2;
    let front = Joint::with_engine(
        fixtures::SEED,
        &fixtures::domain_limits(),
        Store::new(),
        std::array::from_fn(|_| configuration()),
        &limits,
        1,
    );
    let mut world = World::<Temper, Joint>::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(2, 0, 0);
    let chat = world.client(fixtures::PARTY, fixtures::chats_page());
    super::model::follow(
        &mut world,
        chat,
        vec![
            super::model::see(Role::Link, b"New chat"),
            super::model::press(Role::Link, b"New chat"),
            super::model::see(Role::Button, b"Sonnet 5.5"),
            super::model::press(Role::Button, b"Sonnet 5.5"),
            Step::Type {
                find: Find::named(Role::TextArea, b"What do you want done?"),
                words: b"Hello".as_slice().into(),
            },
            Step::Send { find: Find::named(Role::TextArea, b"What do you want done?") },
            super::model::see(Role::Disclosure, b"Model"),
        ],
    );
    let owner = world.client(fixtures::PARTY, Address::Models { project: 1 });
    super::model::follow(&mut world, owner, vec![super::model::see(Role::Check, b"Sonnet 5.5 in the picker")]);
    super::model::follow(
        &mut world,
        owner,
        vec![super::model::press(Role::Check, b"Sonnet 5.5 in the picker"), super::model::see(Role::Text, b"Off")],
    );
    let switch = world
        .tree(owner)
        .nodes()
        .iter()
        .find(|n| n.role == Role::Check && n.name.as_deref() == Some(b"Sonnet 5.5 in the picker"))
        .unwrap();
    assert!(!switch.states.as_slice().contains(&State::Pressed));
    assert!(!switch.states.as_slice().contains(&State::Disabled));
    let root = world.root(chat).person();
    let task = root.pages().chat().unwrap().task(root.client()).unwrap();
    assert_eq!(task.llm.as_ref().unwrap().main.model.bytes(), b"claude-sonnet-5-5");
    let byline = world
        .tree(chat)
        .nodes()
        .iter()
        .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model"))
        .unwrap();
    assert_eq!(byline.description.as_deref(), Some(b"Sonnet 5.5".as_slice()));
    assert!(
        !world.tree(chat).nodes().iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Sonnet 5.5"))
    );
    super::model::follow(
        &mut world,
        chat,
        vec![super::model::press(Role::Button, b"All models"), super::model::see(Role::Button, b"Sonnet 5.5")],
    );
}

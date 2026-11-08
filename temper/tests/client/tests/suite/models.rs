//! Catalogue switches through both roots (temper/docs/design/client/README.md, 3.5).
use jig_client_kit::harness::{Settings, World};
use jig_domain_core::settings::{CatalogueModel, ModelSource};
use jig_domain_people::InitialOwner;
use skein_fake_person::{Find, Step};
use skein_ui::{Role, State};
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain_world::{store::Store, walking};

fn configuration(list: bool) -> temper_engine_domain::Config {
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
            first_read: None,
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
            first_read: None,
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
    if list {
        config.core.settings.endpoints[0].providers = Box::new([1]);
        let provider = &mut config.core.settings.llm_providers.providers[0];
        provider.name = b"openai".as_slice().into();
        provider.list =
            Some(jig_domain_accounts::ProviderList { keep: Box::new([b"*".as_slice().into()]), drop: Box::new([]) });
    }
    config
}

fn model_world(configs: [temper_engine_domain::Config; 2]) -> World<Temper, Joint> {
    let mut limits = Joint::new(fixtures::SEED, fixtures::domain_limits()).engine.env.limits;
    limits.people.initial_owners = 2;
    let mut store = Store::new();
    if configs[0].core.settings.llm_providers.providers[0].list.is_some() {
        store = temper_engine_domain_world::fixtures::restored_store(configuration(true), &limits);
        let mut rows = temper_engine_domain_world::fixtures::account_rows(&configs[0].core.settings.llm_providers);
        for record in &mut rows {
            if let temper_engine_domain::Record::Core(jig_domain_core::Record::Accounts(
                jig_domain_accounts::Stored::Account { holder, .. },
            )) = record
            {
                *holder = jig_domain_accounts::Holder::Party(fixtures::PARTY);
            }
        }
        store.seed(rows, Box::new([]));
    }
    let front = Joint::with_engine(fixtures::SEED, &fixtures::domain_limits(), store, configs, &limits, 1);
    let mut world = World::<Temper, Joint>::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(2, 0, 0);
    world
}

#[test]
fn a_switch_turned_off_and_back_holds_across_a_reload() {
    let mut world = model_world(std::array::from_fn(|_| configuration(false)));
    let chat = world.client(fixtures::PARTY, fixtures::chats_page());
    super::model::follow(
        &mut world,
        chat,
        fixtures::start_change(false)
            .into_iter()
            .chain([
                super::model::see(Role::Button, b"Sonnet 5.5"),
                super::model::press(Role::Button, b"Sonnet 5.5"),
                Step::Type {
                    find: Find::named(Role::TextArea, b"What do you want done?"),
                    words: b"Hello".as_slice().into(),
                },
                Step::Send { find: Find::named(Role::TextArea, b"What do you want done?") },
                super::model::see(Role::Disclosure, b"Model"),
            ])
            .collect(),
    );
    let owner = world.client(fixtures::PARTY, fixtures::chats_page());
    super::model::follow(
        &mut world,
        owner,
        vec![
            super::model::see(Role::Link, b"Deployment"),
            super::model::press(Role::Link, b"Deployment"),
            super::model::see(Role::Link, b"Models"),
            super::model::press(Role::Link, b"Models"),
            super::model::see(Role::Check, b"Sonnet 5.5 in the picker"),
        ],
    );
    super::model::follow(
        &mut world,
        owner,
        vec![super::model::press(Role::Check, b"Sonnet 5.5 in the picker"), super::model::see(Role::Text, b"Off")],
    );
    // This world's restart drops the address; restore the browser's saved page (client/README.md, 6.10).
    world.reload(owner);
    super::model::follow(
        &mut world,
        owner,
        vec![
            super::model::see(Role::Status, b"Live"),
            Step::Go { address: Address::Models { project: 1 } },
            super::model::see(Role::Text, b"Off"),
        ],
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
    super::model::follow(&mut world, owner, vec![super::model::press(Role::Check, b"Sonnet 5.5 in the picker")]);
    let switch = world
        .tree(owner)
        .nodes()
        .iter()
        .find(|n| n.role == Role::Check && n.name.as_deref() == Some(b"Sonnet 5.5 in the picker"))
        .unwrap();
    assert!(switch.states.as_slice().contains(&State::Pressed));
    super::model::follow(
        &mut world,
        chat,
        vec![super::model::press(Role::Button, "‹ Models".as_bytes()), super::model::see(Role::Button, b"Sonnet 5.5")],
    );
}

#[test]
fn a_model_added_by_name_is_on_the_picker() {
    for terminal in [false, true] {
        let mut world = model_world(std::array::from_fn(|_| {
            let mut config = configuration(false);
            config.core.defaults.charters[0].rest = true;
            config.core.settings.llm_providers.providers[0].name = b"anthropic".as_slice().into();
            config
        }));
        let owner = world.client(fixtures::PARTY, Address::Models { project: 1 });
        if terminal {
            world.at_text_shell(owner);
        }
        super::model::follow(
            &mut world,
            owner,
            vec![
                super::model::see(Role::Heading { level: 2 }, b"Add a model by name"),
                super::model::see(Role::Select, b"Provider"),
                super::model::press(Role::Option, b"Claude"),
                Step::Type {
                    find: Find::named(Role::TextField, "The model’s name".as_bytes()),
                    words: b"claude-haiku-5-5".as_slice().into(),
                },
                if terminal {
                    Step::Send { find: Find::named(Role::TextField, "The model’s name".as_bytes()) }
                } else {
                    super::model::press(Role::Button, b"Add")
                },
                super::model::see(Role::Text, "claude-haiku-5-5 · added by you · estimated prices".as_bytes()),
            ],
        );
        let field = world
            .tree(owner)
            .nodes()
            .iter()
            .find(|n| n.role == Role::TextField && n.name.as_deref() == Some("The model’s name".as_bytes()))
            .unwrap();
        assert!(field.value.as_ref().unwrap().text.is_empty());
        let switch = world
            .tree(owner)
            .nodes()
            .iter()
            .find(|n| n.role == Role::Check && n.name.as_deref() == Some(b"claude-haiku-5-5 in the picker"))
            .unwrap();
        assert!(switch.states.as_slice().contains(&State::Pressed));
        let added = world
            .root(owner)
            .person()
            .pages()
            .me(world.root(owner).person().client())
            .unwrap()
            .projects
            .get(0)
            .unwrap()
            .models
            .iter()
            .find(|m| m.name.bytes() == b"claude-haiku-5-5")
            .unwrap();
        assert_eq!(added.source, jig_client::contract::ModelSource::Person);
        assert!(added.picker && added.estimated && added.efforts.is_empty() && added.effort.is_none());
        super::model::follow(
            &mut world,
            owner,
            vec![
                Step::Go { address: Address::NewChat { draft: false } },
                super::model::see(Role::Button, b"Sonnet 5.5"),
                super::model::press(Role::Button, b"Sonnet 5.5"),
                super::model::see(Role::Button, b"claude-haiku-5-5"),
                super::model::press(Role::Button, b"claude-haiku-5-5"),
                super::model::see(Role::Button, b"claude-haiku-5-5"),
                super::model::press(Role::Button, b"claude-haiku-5-5"),
            ],
        );
        assert!(!world.tree(owner).nodes().iter().any(|n| n.name.as_deref() == Some(b"Effort")));
        let reloaded = world.client(fixtures::PARTY, Address::Models { project: 1 });
        if terminal {
            world.at_text_shell(reloaded);
        }
        super::model::follow(
            &mut world,
            reloaded,
            vec![
                super::model::see(Role::Text, "claude-haiku-5-5 · added by you · estimated prices".as_bytes()),
                Step::Go { address: Address::NewChat { draft: false } },
                super::model::see(Role::Button, b"Sonnet 5.5"),
                super::model::press(Role::Button, b"Sonnet 5.5"),
                super::model::see(Role::Button, b"claude-haiku-5-5"),
            ],
        );
    }
}

// Each row advances the same account and task through the list's lifecycle.
#[test]
#[expect(clippy::too_many_lines, reason = "the table follows one account and task through reads and reloads")]
fn a_models_list_arrives_fails_narrows_and_keeps_the_persons_switch() {
    use jig_domain_link::{ListFailure, ListOutcome, ListedModel};
    use temper_engine_domain_world::kinds;

    let mut world = model_world(std::array::from_fn(|_| configuration(true)));
    let chat = world.client(fixtures::PARTY, fixtures::chats_page());
    super::model::follow(
        &mut world,
        chat,
        fixtures::start_change(false)
            .into_iter()
            .chain([
                super::model::see(Role::Button, b"Sonnet 5.5"),
                super::model::press(Role::Button, b"Sonnet 5.5"),
                Step::Type {
                    find: Find::named(Role::TextArea, b"What do you want done?"),
                    words: b"Hello".as_slice().into(),
                },
                Step::Send { find: Find::named(Role::TextArea, b"What do you want done?") },
                super::model::see(Role::Disclosure, b"Model"),
            ])
            .collect(),
    );
    let task = {
        let person = world.root(chat).person();
        person.pages().chat().unwrap().task(person.client()).unwrap().chip.task
    };
    let choice = world.front().engine.task(task).unwrap().llm.clone();
    let owner = world.client(fixtures::PARTY, fixtures::chats_page());
    super::model::follow(
        &mut world,
        owner,
        vec![
            super::model::see(Role::Link, b"Deployment"),
            super::model::press(Role::Link, b"Deployment"),
            super::model::see(Role::Link, b"Models"),
            super::model::press(Role::Link, b"Models"),
            super::model::see(Role::Check, b"Sonnet 5.5 in the picker"),
        ],
    );
    let stock: &[&[u8]] = &[b"claude-opus-5-5", b"claude-sonnet-5-5", b"gpt-future"];
    let narrowed: &[&[u8]] = &[b"claude-opus-5-5", b"gpt-future"];
    for (at, (names, picker, revision_moves, me_moves)) in [
        (Some(stock), false, true, true),
        (None, false, false, false),
        (Some(narrowed), false, false, true),
        (Some(narrowed), true, false, false),
    ]
    .into_iter()
    .enumerate()
    {
        if picker {
            super::model::follow(
                &mut world,
                owner,
                vec![
                    super::model::press(Role::Check, b"gpt-future in the picker"),
                    super::model::see(Role::Text, b"On"),
                ],
            );
        }
        let revision = kinds::revision(&world.front().engine.store.rows);
        let deliveries = world.front().me_deliveries;
        if at != 0 {
            super::model::follow(
                &mut world,
                owner,
                vec![
                    super::model::press(Role::Button, b"Read again"),
                    super::model::see(Role::Button, "Reading…".as_bytes()),
                ],
            );
        }
        let outcome = names.map_or(ListOutcome::Failed(ListFailure::Unreachable), |names| {
            ListOutcome::Listed(
                names
                    .iter()
                    .map(|name| ListedModel {
                        name: (*name).into(),
                        shown: None,
                        window: None,
                        output: None,
                        efforts: Box::new([]),
                    })
                    .collect(),
            )
        });
        world.front().answer_read(outcome);
        super::model::follow(
            &mut world,
            owner,
            vec![
                Step::See {
                    find: Find::text(if names.is_some() {
                        b"Read from OpenAI"
                    } else {
                        b"Couldn't read OpenAI's list"
                    }),
                    within: fixtures::WITHIN,
                },
                super::model::see(Role::Text, "gpt-future · estimated prices · new".as_bytes()),
                super::model::see(Role::Button, b"Read again"),
            ],
        );
        assert_eq!(
            kinds::revision(&world.front().engine.store.rows),
            revision + u32::from(revision_moves),
            "read {at}"
        );
        assert_eq!(world.front().me_deliveries > deliveries, me_moves, "read {at}: me changes only with its offer");
        assert_eq!(world.front().engine.task(task).unwrap().llm, choice, "read {at}: a task keeps its choice");
        let person = world.root(chat).person();
        assert_eq!(
            person.pages().chat().unwrap().task(person.client()).unwrap().llm.as_ref().unwrap().main.model.bytes(),
            b"claude-sonnet-5-5"
        );
        for reload in [false, true] {
            if reload {
                world.reload(owner);
                super::model::follow(
                    &mut world,
                    owner,
                    vec![
                        super::model::see(Role::Status, b"Live"),
                        Step::Go { address: Address::Models { project: 1 } },
                        super::model::see(Role::Text, "gpt-future · estimated prices · new".as_bytes()),
                        Step::See {
                            find: Find::text(if names.is_some() {
                                b"Read from OpenAI"
                            } else {
                                b"Couldn't read OpenAI's list"
                            }),
                            within: fixtures::WITHIN,
                        },
                    ],
                );
            }
            let switch = world
                .tree(owner)
                .nodes()
                .iter()
                .find(|node| node.role == Role::Check && node.name.as_deref() == Some(b"gpt-future in the picker"))
                .unwrap();
            assert_eq!(switch.states.as_slice().contains(&State::Pressed), picker, "read {at}, reload {reload}");
            let person = world.root(owner).person();
            let models = &person.pages().me(person.client()).unwrap().projects.get(0).unwrap().models;
            assert_eq!(models.len(), if at < 2 { 3 } else { 2 });
            assert_eq!(models.iter().any(|model| model.name.bytes() == b"claude-sonnet-5-5"), at < 2);
            let new = models.iter().find(|model| model.name.bytes() == b"gpt-future").unwrap();
            assert_eq!(new.source, jig_client::contract::ModelSource::List);
            assert!(new.estimated && new.new_until.is_some());
            assert_eq!(new.picker, picker);
            super::model::follow(
                &mut world,
                owner,
                vec![
                    Step::Go { address: Address::NewChat { draft: false } },
                    super::model::see(Role::Button, b"Opus 5.5"),
                    super::model::press(Role::Button, b"Opus 5.5"),
                ],
            );
            assert_eq!(
                world
                    .tree(owner)
                    .nodes()
                    .iter()
                    .any(|node| { node.role == Role::Button && node.name.as_deref() == Some(b"gpt-future") }),
                picker,
                "read {at}, reload {reload}: picker"
            );
            if !picker {
                super::model::follow(
                    &mut world,
                    owner,
                    vec![
                        super::model::press(Role::Button, b"All models"),
                        super::model::see(Role::Button, b"gpt-future"),
                        super::model::see(Role::Text, b"new"),
                    ],
                );
            }
            super::model::follow(
                &mut world,
                owner,
                vec![
                    Step::Go { address: Address::Models { project: 1 } },
                    super::model::see(Role::Check, b"gpt-future in the picker"),
                ],
            );
        }
    }
}

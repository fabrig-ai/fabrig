//! Owner model acts decide against the current catalogue (domain/engine/settings.md, 11).
use crate::settings_support::*;
use jig_core_world::{
    client_peer::ClientPeer,
    fixtures, peers,
    world::{Boot, Setup, World},
};
use jig_domain_core::documents;
use jig_domain_core::{self as core, settings as s};
use jig_domain_people as people;
use jig_test_domain as root;
use s::catalogue::ModelAct;
use skein_lib::{ReplyTo, Token};

fn configured(count: u32) -> (root::Config, root::Limits) {
    let (mut config, mut limits) = configuration();
    limits.core.settings.alternatives = 63;
    config.core.defaults.charters[0].rest = true;
    let first = config.core.defaults.models[0].clone();
    config.core.defaults.models = (1..=count)
        .map(|number| {
            let mut row = first.clone();
            row.number = number;
            if number != 1 {
                row.model.name = format!("stock-{number}").into_bytes().into();
            }
            row
        })
        .collect();
    config.core.settings.models = config.core.defaults.models.iter().map(|row| row.model.clone()).collect();
    peers::journal(&mut limits);
    (config, limits)
}
fn start_with(config: root::Config, limits: &root::Limits) -> (World, u64) {
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits: *limits,
        boot: Boot::Empty,
        ..Setup::default()
    });
    assert!(world.domain.ready(), "{:?}", world.domain.start_failure());
    world.client_peer = Some(ClientPeer::default());
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    (world, signed)
}
fn setup(count: u32) -> (World, u64) {
    let (config, limits) = configured(count);
    start_with(config, &limits)
}
fn entry(world: &World, number: u32) -> s::CatalogueModel {
    let Some(s::Value::Model(value)) = world.domain.settings().value(s::Setting::Model(number)) else {
        panic!("catalogue model")
    };
    value.as_ref().clone()
}
fn act(world: &mut World, signed: u64, key: u8, act: ModelAct) -> people::Reply {
    world.send(root::Event::Core(core::Event::ChangeModel {
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [key; 16],
        digest: [key; 32],
        act,
    }));
    let (_, root::Delivery::Core(core::Held::PeopleReply { reply, .. })) =
        world.client_peer.as_mut().expect("client peer").delivered.pop().expect("model act delivery")
    else {
        panic!("model act reply")
    };
    reply
}
fn add(provider: u32, name: &[u8]) -> ModelAct {
    ModelAct::Add { provider, name: name.into() }
}
fn fault(reply: &people::Reply, wanted: s::Fault) {
    assert!(
        matches!(reply, people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
        why: people::Malformed::Settings { fault, .. }, ..
    })) if *fault == wanted),
        "{reply:?}"
    );
}

#[test]
fn turning_a_model_off_keeps_the_rest_of_its_value() {
    let (mut world, signed) = setup(1);
    let reply = ask(
        &mut world,
        signed,
        90,
        people::Ask::Create {
            project: 1,
            batch: Box::new([jig_test_profile::fixtures::people_member(b"held model", 20)]),
        },
    );
    let people::Reply::Outcome(people::Outcome::Created { first: task, .. }) = reply else { panic!("{reply:?}") };
    let held = world.domain.core_mut().charter_of(task).clone();
    // A separate settings decision changes the row after the client could have read it.
    let mut newest = entry(&world, 1);
    newest.description = Some(b"newest description".as_slice().into());
    newest.model.input_price = 0;
    changed(
        &change(
            &mut world,
            signed,
            21,
            1,
            Box::new([s::Item {
                setting: s::Setting::Model(1),
                edit: s::Edit::Set(s::Value::Model(Box::new(newest.clone()))),
            }]),
        ),
        2,
    );
    changed(&act(&mut world, signed, 22, ModelAct::Picker { number: 1, on: false }), 3);
    // A retry keeps its original answer even after the catalogue revision has moved.
    changed(&act(&mut world, signed, 22, ModelAct::Picker { number: 1, on: false }), 3);
    newest.picker = false;
    assert_eq!(entry(&world, 1), newest);
    assert_eq!(world.domain.core_mut().charter_of(task), &held);
    assert_eq!(fixtures::live_row(&world, task).settings.task, 1);
    super::models::host(&mut world);
    let me = frame(&mut world, signed);
    let catalogue = jig_protocol_engine_clients::translate::llm::catalogue(&[newest]);
    let offer = jig_protocol_engine_clients::translate::llm::models(
        me.projects.get(0).unwrap().offer.as_slice(),
        &catalogue,
        &jig_client_test_records::limits(),
    )
    .unwrap();
    assert!(!offer.get(0).unwrap().picker());
    assert_eq!(
        act(&mut world, signed, 23, ModelAct::Picker { number: 1, on: false }),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::SettingsUnchanged
        }))
    );
    assert_eq!(world.domain.settings().newest(), 3);
    fixtures::finish_chat(&mut world, 0, b"Done");
}

#[test]
fn a_committed_picker_change_refreshes_a_live_me_watch() {
    let (mut world, signed) = setup(1);
    super::models::host(&mut world);
    let watcher = Token::new(45);
    world.send(root::Event::Core(core::Event::Watch {
        watcher,
        sign_in: signed,
        subject: documents::Watched::Frame,
        window: documents::Window { rows: 8, after: None },
    }));
    let initial = std::mem::take(&mut world.client_peer.as_mut().unwrap().delivered);
    assert_eq!(initial.len(), 1);
    world.send(root::Event::Core(core::Event::Delivered { watcher }));
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    world.send(root::Event::Core(core::Event::ChangeModel {
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [41; 16],
        digest: [41; 32],
        act: ModelAct::Picker { number: 1, on: false },
    }));
    assert!(world.client_peer.as_ref().unwrap().delivered.is_empty(), "me waits for the commit");
    world.release_commits();
    let delivered = std::mem::take(&mut world.client_peer.as_mut().unwrap().delivered);
    assert_eq!(delivered.len(), 2, "the committed answer and refreshed me arrive together");
    for (commit, delivery) in delivered {
        assert!(commit <= world.store.applied);
        match delivery {
            root::Delivery::Core(core::Held::PeopleReply { reply, .. }) => changed(&reply, 2),
            root::Delivery::Core(core::Held::Delivery { watcher: got, document }) => {
                assert_eq!(got, watcher);
                let documents::Document::Frame(me) = *document else { panic!("me watch") };
                let catalogue = jig_protocol_engine_clients::translate::llm::catalogue(&[entry(&world, 1)]);
                let offer = jig_protocol_engine_clients::translate::llm::models(
                    me.projects.get(0).unwrap().offer.as_slice(),
                    &catalogue,
                    &jig_client_test_records::limits(),
                )
                .unwrap();
                assert!(!offer.get(0).unwrap().picker(), "the live me translates the changed catalogue");
            }
            other => panic!("unexpected picker publication: {other:?}"),
        }
    }
    world.send(root::Event::Core(core::Event::Delivered { watcher }));
    let same =
        s::Item { setting: s::Setting::Model(1), edit: s::Edit::Set(s::Value::Model(Box::new(entry(&world, 1)))) };
    assert_eq!(
        change(&mut world, signed, 42, 2, Box::new([same])),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::SettingsUnchanged,
        }))
    );
    assert!(world.client_peer.as_ref().unwrap().delivered.is_empty(), "an equivalent setting marks no me");
    changed(&act(&mut world, signed, 41, ModelAct::Picker { number: 1, on: false }), 2);
    assert!(world.client_peer.as_ref().unwrap().delivered.is_empty(), "a keyed retry marks no me");
    fault(&act(&mut world, signed, 43, ModelAct::Picker { number: 64, on: false }), s::Fault::Reference);
    assert!(world.client_peer.as_ref().unwrap().delivered.is_empty(), "a refused act marks no me");
    frame(&mut world, signed);
    assert!(world.client_peer.as_ref().unwrap().delivered.is_empty(), "reading me marks no watch");
}

#[test]
fn adding_a_model_by_name_makes_a_persons_model_on_the_picker() {
    let (mut config, limits) = configured(3);
    // Equal most_costly totals select the lower number, even with a different token price.
    config.core.defaults.models[1].model.input_price = 3;
    config.core.defaults.models[1].model.efforts = Box::new([b"high".as_slice().into()]);
    config.core.defaults.models[1].model.effort = Some(b"high".as_slice().into());
    config.core.defaults.models[2].model = config.core.defaults.models[1].model.clone();
    config.core.defaults.models[2].model.name = b"stock-3".as_slice().into();
    config.core.defaults.models[2].model.window = 4096;
    config.core.defaults.models[2].model.input_price = 6;
    let mut other = config.core.settings.llm_providers.providers[0].clone();
    other.number = 2;
    other.name = b"Other".as_slice().into();
    config.core.settings.llm_providers.providers =
        Box::new([config.core.settings.llm_providers.providers[0].clone(), other]);
    let expected = config.core.defaults.models[1].model.clone();
    let (mut world, signed) = start_with(config, &limits);
    changed(&act(&mut world, signed, 21, add(1, b"my-model")), 2);
    let value = entry(&world, 64);
    let mut expected = expected;
    expected.name = b"my-model".as_slice().into();
    expected.efforts = Box::default();
    expected.effort = None;
    assert_eq!(
        value,
        s::CatalogueModel {
            first_read: None,
            number: 64,
            model: expected,
            shown: None,
            description: None,
            picker: true,
            withdrawn: false,
            source: s::ModelSource::Person,
            estimated: true
        }
    );
    super::models::host(&mut world);
    let me = frame(&mut world, signed);
    let offered = me.projects.get(0).unwrap().offer.as_slice().iter().find(|row| row.model == 63).unwrap();
    assert!(!offered.accounts.is_empty());
    for account in &offered.accounts {
        assert_eq!(world.domain.accounts().account(*account).unwrap().provider(), 1);
    }
    changed(&act(&mut world, signed, 22, add(1, b"another-model")), 3);
    assert_eq!(entry(&world, 63).model.name.as_ref(), b"another-model");
}

#[test]
fn a_name_out_of_shape_is_refused_as_shape() {
    let (mut world, signed) = setup(1);
    for (at, name) in [b"".as_slice(), b"bad/name", b"bad name", b"bad\nname", "mødel".as_bytes(), &[b'x'; 129]]
        .into_iter()
        .enumerate()
    {
        fault(&act(&mut world, signed, u8::try_from(at + 21).unwrap(), add(1, name)), s::Fault::Shape);
        assert_eq!(world.domain.settings().newest(), 1);
    }
    changed(&act(&mut world, signed, 30, add(1, b"Good-model_2.0:main")), 2);
    changed(&act(&mut world, signed, 31, add(1, &[b'x'; 128])), 3);
}
#[test]
fn a_name_the_catalogue_holds_is_refused_as_shape() {
    let (mut world, signed) = setup(1);
    let name = entry(&world, 1).model.name;
    fault(&act(&mut world, signed, 21, add(1, &name)), s::Fault::Shape);
    assert_eq!(world.domain.settings().newest(), 1);
}
#[test]
fn adding_a_withdrawn_name_brings_it_back() {
    let (mut world, signed) = setup(2);
    let mut original = entry(&world, 2);
    changed(
        &change(
            &mut world,
            signed,
            21,
            1,
            Box::new([s::Item { setting: s::Setting::Model(2), edit: s::Edit::Removed }]),
        ),
        2,
    );
    assert!(entry(&world, 2).withdrawn);
    changed(&act(&mut world, signed, 22, add(1, &original.model.name)), 3);
    original.withdrawn = false;
    assert_eq!(entry(&world, 2), original);
}
#[test]
fn a_full_catalogue_refuses_as_count() {
    let (mut world, signed) = setup(64);
    fault(&act(&mut world, signed, 21, add(1, b"no-room")), s::Fault::Count);
    assert_eq!(world.domain.settings().newest(), 1);
}
#[test]
fn a_provider_with_no_stock_takes_its_endpoint_and_the_dearest_figures() {
    let (mut config, limits) = configured(2);
    config.core.defaults.models[1].model.input_price = 7;
    let mut other = config.core.settings.llm_providers.providers[0].clone();
    other.number = 2;
    other.name = b"Other".as_slice().into();
    config.core.settings.llm_providers.providers =
        Box::new([config.core.settings.llm_providers.providers[0].clone(), other]);
    config.core.settings.endpoints = Box::new([
        config.core.settings.endpoints[0].clone(),
        core::EndpointFacts { direct: Box::new([]), number: 2, dialect: 4, program: None, providers: Box::new([2]) },
    ]);
    let mut expected = config.core.defaults.models[1].model.clone();
    expected.provider = 2;
    expected.endpoint = 2;
    expected.dialect = 4;
    expected.name = b"local-model".as_slice().into();
    let listed = s::catalogue::new_model(
        2,
        b"listed-model".as_slice().into(),
        s::ModelSource::List,
        &config.core.defaults.models,
        &[core::EndpointFacts {
            direct: Box::new([]),
            number: 2,
            dialect: 4,
            program: Some(b"backend".as_slice().into()),
            providers: Box::new([2]),
        }],
    )
    .unwrap();
    assert_eq!(listed.model.program.as_deref(), Some(b"backend".as_slice()));
    assert_eq!(listed.source, s::ModelSource::List);
    let (mut world, signed) = start_with(config, &limits);
    changed(&act(&mut world, signed, 21, add(2, b"local-model")), 2);
    assert_eq!(entry(&world, 64).model, expected);
}
#[test]
fn an_unknown_provider_is_a_reference() {
    let (mut world, signed) = setup(1);
    fault(&act(&mut world, signed, 21, add(99, b"missing-provider")), s::Fault::Reference);
    assert_eq!(world.domain.settings().newest(), 1);
}
#[test]
fn a_party_who_isnt_a_deployment_owner_is_not_theirs() {
    let (mut world, signed) = setup(1);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Member }]),
    })));
    for (key, request) in
        [(21, ModelAct::Picker { number: 1, on: false }), (22, add(1, b"not-mine")), (23, add(1, b"bad/name"))]
    {
        let reply = act(&mut world, signed, key, request);
        assert!(
            matches!(
                reply,
                people::Reply::Refused(people::Refusal::Role { held: Some(people::Role::Member) })
                    | people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role {
                        held: Some(people::Role::Member)
                    }))
            ),
            "{reply:?}"
        );
    }
    assert_eq!(world.domain.settings().newest(), 1);
}

//! Stock creation offers fit their receivers (jig/docs/design/client/domain/contract/documents.md, 3).
use jig_client_protocol_wire::Record as _;
use jig_domain_core::{
    Record,
    documents::{self, Document, Frame},
    engine::Engine,
};
use jig_domain_people as people;
use skein_lib::{Env, Reader, Time, Wall, Writer};

fn cluster_engine(name: &str) -> jig_protocol_config::Engine {
    let root = std::path::PathBuf::from(format!("/fixture/{name}"));
    let cluster = super::cluster_documents(&root);
    let value = temper_playground::documents::Configuration::from_jig(super::cluster_document(&cluster, "engine"));
    configured_engine(&value)
}

fn configured_engine(value: &temper_playground::documents::Configuration) -> jig_protocol_config::Engine {
    let mut value = value.clone();
    crate::support::configuration::use_stock(&mut value);
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &super::DOCUMENT).expect("engine document");
    temper_service::startup::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        &temper_service::stock::Given::Stock,
        &mut value.jig,
    )
    .expect("stock engine")
}

fn stock_frame(engine: &jig_protocol_config::Engine) -> Frame {
    let mut engine = engine.clone();
    let policy = engine.policies.iter().find(|policy| policy.number == 1).expect("stock policy");
    engine.projects = Box::new([jig_protocol_config::Project {
        number: 1,
        name: "Project 1".into(),
        roles: policy.roles.clone(),
        policy: policy.policy.clone(),
        templates: policy.templates.clone(),
    }]);
    let config = jig_protocol_config::core(&engine, [1; 16], 7);
    let mut rows = jig_test_profile::fixtures::seeded_project_rows(&config);
    rows.extend([
        Record::People(people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 1, subject: b"owner-a".as_slice().into() },
                login: b"owner".as_slice().into(),
                name: b"Owner".as_slice().into(),
            },
        }),
        Record::People(people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        }),
    ]);
    let limits = engine.limits.core;
    let mut declarations =
        skein_lib::List::with_capacity(u32::try_from(config.connectors.len()).expect("connector count"));
    for connector in &config.connectors {
        declarations
            .push(jig_domain_contract::Declaration::empty(jig_domain_contract::Connector(*connector)))
            .expect("connector declaration");
    }
    let mut jig = Engine::new(config, declarations.as_slice(), &limits).expect("stock core");
    jig.core_mut().start_begin(false, Wall::EPOCH);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    for row in rows {
        assert!(!matches!(jig.core_mut().restore(&env, row), jig_domain_core::Restore::Refused));
    }
    let document = documents::make(
        jig.core(),
        &limits,
        1,
        &documents::Subject::Frame { party: 1 },
        &documents::Window { rows: 1, after: None },
        Wall::EPOCH,
    )
    .expect("stock me document");
    let Document::Frame(frame) = document else { panic!("me document") };
    frame
}

fn offered(frame: &Frame) -> &[u32] {
    let creating = frame.projects.get(0).expect("stock project").creating.as_ref().expect("complete creating offer");
    let jig_domain_core::acts::Bounds::Create { charters, .. } = &creating.within else { panic!("creating bounds") };
    charters.as_slice()
}

fn browser_frame(frame: &Frame, models: &[jig_protocol_engine_clients::Model]) -> jig_client::contract::Me {
    use jig_client_protocol_wire as w;
    let wire = jig_protocol_engine_clients::translate::owner::frame(
        frame,
        Wall::from_nanos(86_400_000_000_000),
        models,
        &w::CEILINGS,
    )
    .expect("stock wire offer");
    let mut writer = Writer::new(
        usize::try_from(wire.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("me bytes"),
    );
    wire.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("encoded me");
    let client = temper_client_shell_web::limits::client().client;
    let limits = jig_client_protocol_link::Limits::new(client.domain, client.exchanges).expect("browser limits");
    let bytes = writer.finish();
    let wire = w::Me::decode(limits.wire(), &mut Reader::new(&bytes)).expect("browser receives stock me");
    jig_client_protocol_link::translate::documents::frame(wire, &limits).expect("browser decodes stock offer")
}

#[test]
fn the_cluster_offers_every_stock_charter_but_the_trial() {
    let mut engine = cluster_engine("cluster-creating");
    let frame = stock_frame(&engine);
    let mut expected: Vec<_> =
        engine.charters.iter().filter(|charter| charter.number != 10).map(|charter| charter.number).collect();
    expected.sort_unstable();
    assert_eq!(expected.len(), 11, "stock agents except the trial, including the standing roles");
    assert_eq!(offered(&frame), expected);
    let received = browser_frame(&frame, &jig_protocol_config::clients::models(&engine.llm));
    let words = jig_client::contract::Text::new(b"Hello".as_slice().into(), engine.limits.core.people.words)
        .expect("chat words");
    let ask = jig_client::shapes::chat(&received, 1, words).expect("stock chat is offered");
    let jig_client::contract::Ask::Create { project, tasks } = ask else { panic!("create chat") };
    assert_eq!(project, 1);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks.get(0).expect("chat task").executor, jig_client::contract::Executor::Agent { charter: 1 });
    engine.limits.core.documents.lines = 5;
    assert!(stock_frame(&engine).projects.get(0).expect("project").creating.is_none(), "overflow refuses whole");
}

#[test]
fn the_browser_receives_a_complete_stock_me_offer() {
    let mut engine = cluster_engine("browser-creating");
    // Isolate the receiver from the cluster's independently checked offer capacity.
    engine.limits.core.documents.lines = 12;
    let frame = stock_frame(&engine);
    assert_eq!(offered(&frame).len(), engine.charters.iter().filter(|charter| charter.number != 10).count());
    let received = browser_frame(&frame, &jig_protocol_config::clients::models(&engine.llm));
    let creating = received.projects.get(0).expect("project").creating.as_ref().expect("creating");
    assert_eq!(creating.within.charters.as_slice(), offered(&frame));
}

#[test]
fn the_forgejo_fixture_offers_every_stock_charter_but_the_trial() {
    use crate::support;
    let recipe = support::recipe(std::path::Path::new("/fixture/forgejo-creating"), &super::fixture_forgejo());
    let value = temper_playground::documents::Configuration::from_jig(recipe.start_document());
    let engine = configured_engine(&value);
    assert_eq!(engine.charters.len(), 12);
    let mut expected: Vec<_> =
        engine.charters.iter().filter(|charter| charter.number != 10).map(|charter| charter.number).collect();
    expected.sort_unstable();
    assert_eq!(offered(&stock_frame(&engine)), expected);
}

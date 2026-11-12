//! Stock creation offers fit their receivers (jig/docs/design/client/domain/contract/documents.md, 3).
use jig_domain_core::{
    Core, Record,
    documents::{self, Document, Frame},
};
use jig_domain_people as people;
use skein_lib::{Env, Reader, Time, Wall, Writer};

fn cluster_engine(name: &str) -> jig_protocol_config::Engine {
    let scratch = crate::support::Scratch::startup(name);
    let cluster = super::cluster_documents(scratch.path());
    let value = super::cluster_document(&cluster, "engine");
    let engine = configured_engine(&value);
    scratch.remove();
    engine
}

fn configured_engine(value: &serde_json::Value) -> jig_protocol_config::Engine {
    let mut value = value.clone();
    crate::support::configuration::use_stock(&mut value);
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &super::DOCUMENT).expect("engine document");
    temper_service::startup::engine(&document, 0, &jig_protocol_config::Directories::working()).expect("stock engine")
}

fn stock_frame(engine: &jig_protocol_config::Engine) -> Frame {
    let config = jig_protocol_config::core(engine, [1; 16], 7);
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
    let mut core = Core::new(config, &limits).expect("stock core");
    core.start_begin(false, Wall::EPOCH);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    for row in rows {
        assert!(!matches!(core.restore(&env, row), jig_domain_core::Restore::Refused));
    }
    let document = documents::make(
        &core,
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
    use jig_client_protocol_wire::v20 as w;
    let wire =
        jig_protocol_engine_clients::translate::owner::frame(frame, models, &w::CEILINGS).expect("stock wire offer");
    let mut writer = Writer::new(usize::try_from(wire.measure()).expect("me bytes"));
    wire.encode(&mut writer).expect("encoded me");
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
    assert_eq!(expected.len(), 8, "stock agents except the trial, including the standing role");
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
    engine.limits.core.documents.lines = 9;
    let frame = stock_frame(&engine);
    assert_eq!(offered(&frame).len(), engine.charters.iter().filter(|charter| charter.number != 10).count());
    let received = browser_frame(&frame, &jig_protocol_config::clients::models(&engine.llm));
    let creating = received.projects.get(0).expect("project").creating.as_ref().expect("creating");
    assert_eq!(creating.within.charters.as_slice(), offered(&frame));
}

#[test]
fn the_forgejo_fixture_offers_every_stock_charter_but_the_trial() {
    use crate::support::{self, Forge};
    let scratch = support::Scratch::startup("forgejo-creating");
    let forge = Forge::Throwaway {
        forgejo: std::path::Path::new("/fixture/forgejo"),
        api: "127.0.0.1:3000".parse().expect("fixture address"),
        git: "http://127.0.0.1:3000/".into(),
    };
    support::configuration(scratch.path(), &forge);
    let bytes = std::fs::read(scratch.path().join("temper.json")).expect("process configuration");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
    let engine = configured_engine(&value);
    assert_eq!(engine.charters.len(), 9);
    let mut expected: Vec<_> =
        engine.charters.iter().filter(|charter| charter.number != 10).map(|charter| charter.number).collect();
    expected.sort_unstable();
    assert_eq!(offered(&stock_frame(&engine)), expected);
    scratch.remove();
}

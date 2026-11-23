use crate::support;
use temper_box_shell::directories::jig_directories;
use temper_client_protocol_wire::v1::Record as CodecRecord;
#[path = "creating.rs"]
mod creating;
#[path = "configuration/turn_landing.rs"]
mod turn_landing;

use support::configuration_fixture::DOCUMENT;

fn process_engine(case: &str) -> jig_protocol_config::Engine {
    let scratch = support::Scratch::startup(case);
    let mut value = support::lay_configuration(scratch.path(), &support::Forge::Unreached);
    // Forge::Unreached omits Forgejo's overrides; include its whole-authority bounds here.
    value.jig_over("authority.grants", 15);
    value.jig_over("tasks.authority_grants", 15);
    // The standing charters, closer procedure and close job are delegation targets.
    value.jig_over("authority.executors", 22);
    // The standing charters, closer procedure and close job are delegation targets.
    value.jig_over("tasks.executor_kinds", 22);
    value.jig_over("notes.scopes", 18);
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    // Parsing a fixture without a deployment; only its limits are used.
    let engine = temper_service::startup::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        &temper_service::stock::Given::Stock,
        &mut value.jig,
    )
    .expect("engine configuration");
    scratch.remove();
    engine
}

#[test]
fn the_process_document_holds_the_agent_after_the_store_restores_stock() {
    use skein_lib::{List, Secret};
    use temper_service::{
        settings::{Files, Seeds},
        startup,
    };

    let scratch = support::Scratch::startup("process-stock-turn");
    support::lay_configuration(scratch.path(), &support::Forge::Unreached);
    let bytes =
        support::read_start(&scratch.path().join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone)
            .document;
    let document = serde_json::from_slice(&bytes).expect("deployment JSON");
    // Compose stock agents(), not the fixture's pre-serve agents.setting.
    let configuration = serde_json::to_vec(&temper_agent_fixtures::configuration(&document)).expect("stock agent JSON");
    let agent = temper_worker_shell::agent::prepare_bytes(configuration.into()).expect("stock agent").agent;
    let directories = jig_directories(&support::configuration::directories(scratch.path()));
    let named = startup::named(&bytes, &directories).expect("named fixture files");
    let mut entries = List::with_capacity(u32::try_from(named.files.len()).expect("file count"));
    for file in named.files {
        entries.push((file.path, Secret::new(Box::new([b'7'; 64])))).expect("named file count");
    }
    let seeds = Seeds {
        link_secret: None,
        local_secret: None,
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
    };
    startup::check(&bytes, &directories, &Files { entries }, &agent, &seeds, None, List::with_capacity(0), None)
        .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&startup::lines(&why))));
    scratch.remove();
}

#[test]
fn the_process_workers_figures_follow_the_agents_setting() {
    use temper_client_protocol_wire::v1 as w;

    for (case, forge) in [
        (
            "forgejo",
            support::Forge::Throwaway {
                forgejo: std::path::Path::new("/fixture/forgejo"),
                api: "127.0.0.1:3000".parse().expect("fixture forge"),
                git: "http://127.0.0.1:3000/".into(),
            },
        ),
        ("unreached", support::Forge::Unreached),
    ] {
        let scratch = support::Scratch::startup(&format!("process-worker-figures-{case}"));
        support::lay_configuration(scratch.path(), &forge);
        let path = scratch.path().join(temper_config_shell::FILE);
        let mut document = support::start_document(&path, temper_config_shell::Shape::Standalone);
        let mut agent = temper_agent_fixtures::read(scratch.path());
        for payload in [63_740, 8192] {
            agent["profile"]["declared"]["tool_payload"] = payload.into();
            temper_agent_fixtures::apply(&mut document, scratch.path(), &agent);
            let setting = std::fs::read(scratch.path().join("agents.setting")).expect("agents setting");
            let agents =
                w::AgentFigures::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&setting)).expect("agents figures");
            let part = temper_service::startup::box_worker(
                &serde_json::to_vec(&document).expect("deployment JSON"),
                &jig_directories(&support::configuration::directories(scratch.path())),
                &temper_service::stock::Given::Stock,
                &agents,
                4,
                &[],
            )
            .expect("box composition")
            .expect("box worker");
            let bytes = temper_worker_shell::agent::figures::configuration(
                part.figures.as_ref().expect("composed figures"),
                None,
                scratch.path(),
                &[],
            )
            .expect("agent document");
            let agent = temper_worker_shell::agent::prepare_bytes(bytes).expect("composed agent").agent;
            // A smaller agent also needs a smaller temper-limits tool_payload;
            // otherwise the engine's stock payload wins the carrier maximum.
            let (mut jig, own) = temper_service::limits::base(
                temper_service::limits::stock().live_goals,
                payload,
                temper_service_core::derived::Sources::agent(&agent.declaration()),
            )
            .expect("derived engine figures");
            jig.core.people.words = jig.core.people.words.min(payload - 2);
            let part = temper_service::startup::box_worker_with_limits(
                &serde_json::to_vec(&document).expect("deployment JSON"),
                &jig_directories(&support::configuration::directories(scratch.path())),
                &temper_service::stock::Given::Stock,
                &agents,
                4,
                &[],
                Some(&agent.declaration()),
                (&jig, &own),
            )
            .expect("derived box composition")
            .expect("box worker");
            let limits = temper_service::worker_settings::root_limits(&part.worker, &agent, &part.limits);
            let mut problems = temper_worker_service::startup::Problems::empty();
            temper_worker_service::startup::worker::worker(&limits, &mut problems);
            assert!(
                problems.is_empty(),
                "{case}, payload {payload}: {}",
                String::from_utf8_lossy(&temper_worker_service::startup::lines(&problems))
            );
        }
        scratch.remove();
    }
}

#[test]
fn the_native_forge_fixture_fits_its_store() {
    let scratch = support::Scratch::startup("native-forge-store");
    let value = support::lay_configuration(
        scratch.path(),
        &support::Forge::Throwaway {
            forgejo: std::path::Path::new("/fixture/forgejo"),
            api: "127.0.0.1:3000".parse().expect("fixture forge"),
            git: "http://127.0.0.1:3000/".into(),
        },
    );
    let document =
        support::read_start(&scratch.path().join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone)
            .document;
    let agent = temper_agent_fixtures::prepared(scratch.path()).agent;
    let store = temper_service::startup::store_limits_with_limits(
        &document,
        &jig_protocol_config::Directories::working(),
        Some(&agent.declaration()),
        &value.jig,
        &value.limits,
    )
    .unwrap_or_else(|why| panic!("fixture store: {}", String::from_utf8_lossy(&temper_service::startup::lines(&why))));
    eprintln!("native forge map value: {} bytes", store.store.store.value);
    scratch.remove();
}

#[test]
fn the_process_http_mount_fits_the_native_clients_burst() {
    let engine = process_engine("native-http-calls");
    let native = temper_client_shell_terminal::limits::native();
    // jig/docs/design/protocol/client/default-http.md, 5: streams occupy mounted calls too.
    let calls = native.connections.exchanges + engine.limits.clients_http.streams;
    assert!(
        engine.limits.clients_http.calls >= calls,
        "the process fixture must admit the native client's exchanges beside its streams"
    );
}

#[test]
fn the_process_and_client_world_documents_fit_their_clients() {
    let engine = process_engine("native-document-limits");
    let client = temper_client_shell_terminal::limits::native().client;
    assert_documents_fit(&engine.limits.core, client.client.domain);
    let client_world = temper_client_world::fixtures::domain_limits();
    let front =
        temper_client_world::front::Joint::with_change_landing(temper_client_world::fixtures::SEED, &client_world);
    assert_documents_fit(&temper_engine_domain::core_limits(&front.engine.env.limits), client_world.client.domain);
}
fn assert_documents_fit(engine: &jig_domain_core::Limits, native: jig_client::DomainLimits) {
    assert!(engine.documents.rows <= native.rows, "the client can receive every document row");
    assert!(engine.documents.lines <= native.rows, "the client can receive every document's short list");
    let paths = engine.authority.segments.max(engine.tasks.hold_segments).max(engine.tasks.authority_segments);
    assert!(paths <= native.segments, "the client can receive every document path");
    assert!(engine.documents.text_bytes <= native.text, "the client can receive every document text");
    assert!(engine.documents.grants <= native.authority, "every grant fits");
    assert!(engine.documents.executors <= native.authority, "every executor fits");
    assert!(engine.documents.implications <= native.authority, "every implication fits");
    assert!(temper_engine_domain::WATCHES_PER_STREAM >= native.watches, "the stream fits the client's watches");
    assert!(temper_engine_domain::VIEWS.watchers_per_party >= native.watches, "the party fits the client's watches");
    assert!(temper_engine_domain::VIEWS.watchers >= native.watches, "the engine fits the client's watches");
}

#[test]
fn the_clusters_description_is_the_labs_canonical_form() {
    let bytes =
        std::fs::read(skein_scratch::paths::tree().join("bin/temper-box/tests/fixtures/cluster/experiment.json"))
            .expect("cluster description");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("cluster JSON");
    // Sort explicitly, including if a dependency enables serde_json's preserve_order.
    value.sort_all_objects();
    assert_eq!(bytes, (serde_json::to_string(&value).expect("canonical JSON") + "\n").as_bytes());
    let mut remaining = vec![&value];
    while let Some(item) = remaining.pop() {
        match item {
            serde_json::Value::Number(number) => assert!(
                number.as_i64().is_some_and(|n| (-(1_i64 << 53) + 1..=(1_i64 << 53) - 1).contains(&n)),
                "cluster integers must be within ±(2^53−1): {number}"
            ),
            serde_json::Value::Array(items) => remaining.extend(items),
            serde_json::Value::Object(items) => remaining.extend(items.values()),
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::String(_) => {}
        }
    }
    let scratch = support::Scratch::startup("cluster-canonical-description");
    let cluster = cluster_documents(scratch.path());
    for name in ["engine", "w1", "w2"] {
        let document = cluster_document(&cluster, name);
        assert!(document["engine"].get("memory_bytes").is_none());
        if let Some(workers) = document["workers"].as_array() {
            for worker in workers {
                assert!(worker.get("memory_bytes").is_none());
            }
        }
    }
    scratch.remove();
}

fn cluster_documents(scratch: &std::path::Path) -> support::cluster::Cluster {
    support::cluster::Cluster::lay_documents(
        scratch,
        &[("forge", "address", "127.0.0.1:3"), ("llm", "address", "127.0.0.1:1"), ("engine", "workers", "127.0.0.1:2")],
    )
}

fn cluster_document(cluster: &support::cluster::Cluster, process: &str) -> serde_json::Value {
    serde_json::from_slice(
        &support::read_start(
            &cluster.dir(process).join("config").join(temper_config_shell::FILE),
            if process == "engine" {
                temper_config_shell::Shape::EngineAlone
            } else {
                temper_config_shell::Shape::WorkerAlone
            },
        )
        .document,
    )
    .expect("cluster document JSON")
}

fn cluster_engine(cluster: &support::cluster::Cluster) -> jig_protocol_config::Engine {
    let value = cluster_document(cluster, "engine");
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    temper_service::startup::engine(&document, 0, &jig_protocol_config::Directories::working())
        .expect("initial stock engine")
}

#[test]
fn the_clusters_documents_check_as_their_shapes() {
    use temper_service::startup::{self, Shape};
    let scratch = support::Scratch::startup("cluster-documents");
    let cluster = cluster_documents(scratch.path());
    let engine = cluster_document(&cluster, "engine");
    assert!(engine["engine"].get("limits").is_none());
    let engine_limits = cluster_engine(&cluster).limits;
    assert_eq!(engine_limits.core.fleet.grace.as_millis(), support::cluster::fleet_grace_ms());
    for (process, expected) in [("engine", Shape::EngineAlone), ("w1", Shape::WorkerAlone), ("w2", Shape::WorkerAlone)]
    {
        let root = cluster.dir(process);
        let directories = temper_real_loop_world::cluster::directories_in([
            &root.join("config"),
            &root.join("data"),
            &root.join("cache"),
        ]);
        let bytes = support::read_start(
            &root.join("config").join(temper_config_shell::FILE),
            if process == "engine" {
                temper_config_shell::Shape::EngineAlone
            } else {
                temper_config_shell::Shape::WorkerAlone
            },
        )
        .document;
        let shape = startup::shape(&bytes, &jig_directories(&directories))
            .unwrap_or_else(|why| panic!("{process}: {}", String::from_utf8_lossy(&startup::lines(&why))));
        assert_eq!(shape, expected, "{process}");
        match shape {
            Shape::EngineAlone => {
                startup::named(&bytes, &jig_directories(&directories)).expect("checked engine alone");
            }
            Shape::WorkerAlone => {
                let mut part = startup::worker_part(&bytes, &jig_directories(&directories), None, &[])
                    .expect("checked worker alone");
                temper_worker_service::figures::apply(&mut part, support::cluster::figures(&engine))
                    .expect("applied engine figures");
                let worker = &part.worker.jig;
                let stop = jig_protocol_config::stop_bound(
                    &worker.graces,
                    worker.limits.as_ref().expect("configured worker limits").link.ping,
                    worker.limits.as_ref().expect("configured worker limits").link.silence,
                )
                .unwrap_or_else(|| {
                    panic!(
                        "bin/temper-box/tests/fixtures/cluster/{process}.json: workers[0].graces stop_bound overflows"
                    )
                });
                assert!(
                    stop < skein_lib::Duration::from_millis(support::cluster::fleet_grace_ms()),
                    "bin/temper-box/tests/fixtures/cluster/{process}.json: workers[0].graces stop_bound must be below engine.json: engine.limits.overrides.fleet.grace_ms"
                );
            }
            Shape::Standalone => panic!("cluster members are separate"),
        }
    }
    scratch.remove();
}

#[test]
fn the_clusters_documents_can_list_every_offered_charter() {
    let scratch = support::Scratch::startup("cluster-creating-charters");
    let cluster = cluster_documents(scratch.path());
    let engine = cluster_engine(&cluster);
    let offered = engine.charters.iter().filter(|charter| charter.number != 10).count();
    assert!(
        engine.limits.core.documents.lines >= u32::try_from(offered).expect("offered charter count"),
        "the cluster's creating document must list all {offered} eligible stock charters before offering a chat"
    );
    scratch.remove();
}

#[test]
fn the_clusters_workers_admit_the_initial_stock() {
    let scratch = support::Scratch::startup("cluster-stock-admission");
    let cluster = cluster_documents(scratch.path());
    let mut engine = cluster_engine(&cluster);
    engine.limits.core.tasks.message_bytes = 4094;
    engine.limits.core.tasks.result_bytes = 128;
    engine.limits.core.tasks.inbox_bytes = 4096;
    engine.limits.core.tasks.run_inbox = 8;
    let core = jig_protocol_config::core(&engine, [0; 16], 1);
    let needs = jig_domain_core::settings::fleet_needs::configured(&core, &engine.limits.core);
    for process in ["w1", "w2"] {
        let profile = support::owned::deployment_profile(
            &cluster.dir(process),
            &format!("bin/temper-box/tests/fixtures/cluster/{process}/{}", temper_config_shell::FILE),
        );
        assert_eq!(
            jig_domain_fleet::shortfall(&needs, &profile.agent.bounds, jig_domain_fleet::HostKind::Worker),
            None,
            "{process} must admit startup stock before the story applies its settings"
        );
    }
    scratch.remove();
}

#[test]
fn the_turn_story_profile_declares_prepare_and_fits_its_store() {
    let scratch = support::Scratch::startup("turn-story-store");
    let mut value = support::lay_configuration(scratch.path(), &support::Forge::Unreached);
    support::batch_landing::message_room(scratch.path(), &mut value);
    let document = serde_json::to_vec(&value).expect("deployment JSON");
    let agent = temper_agent_fixtures::prepared(scratch.path()).agent;
    let _store = temper_service::startup::store_limits_with_limits(
        &document,
        &jig_protocol_config::Directories::working(),
        Some(&agent.declaration()),
        &value.jig,
        &value.limits,
    )
    .unwrap_or_else(|why| panic!("turn store: {}", String::from_utf8_lossy(&temper_service::startup::lines(&why))));
    let document = jig_protocol_config::document(&document, &DOCUMENT).expect("document");
    let at = jig_protocol_config::read::Object::new(&document, 0).unwrap().member(&document, b"engine").unwrap();
    let engine = temper_service::startup::engine_over(
        &document,
        at,
        &jig_protocol_config::Directories::working(),
        &temper_service::stock::Given::Stock,
        &mut value.jig,
    )
    .expect("engine");
    assert_eq!(temper_service::limits::prepare_jobs(&engine.limits).len(), 1);
    scratch.remove();
}

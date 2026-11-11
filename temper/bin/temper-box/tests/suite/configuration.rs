use crate::support;
use temper_box_shell::directories::jig_directories;
#[path = "creating.rs"]
mod creating;

// The process configuration reader admits a MiB, 64K tokens and 32 levels.
const DOCUMENT: jig_protocol_config::DocumentLimits =
    jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };

fn process_engine(case: &str) -> jig_protocol_config::Engine {
    let scratch = support::Scratch::startup(case);
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let bytes = std::fs::read(scratch.path().join("temper.json")).expect("process configuration");
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).expect("deployment JSON");
    // Forge::Unreached omits Forgejo's overrides; include its whole-authority bounds here.
    value["engine"]["limits"]["overrides"]["authority.grants"] = 14.into();
    value["engine"]["limits"]["overrides"]["tasks.authority_grants"] = 14.into();
    // The standing charter, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["authority.executors"] = 18.into();
    // The standing charter, closer procedure and close job are delegation targets.
    value["engine"]["limits"]["overrides"]["tasks.executor_kinds"] = 18.into();
    value["engine"]["limits"]["overrides"]["notes.scopes"] = 17.into();
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    // Parsing a fixture without a deployment; only its limits are used.
    let engine = temper_service::startup::engine(&document, 0, &jig_protocol_config::Directories::working())
        .expect("engine configuration");
    scratch.remove();
    engine
}

#[test]
fn the_browsers_client_bounds_are_the_terminals() {
    assert_eq!(
        temper_client_shell_web::limits::client(),
        temper_client_shell_terminal::limits::native().client,
        "the browser receives what the terminal does (client/README.md, 2)"
    );
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
fn the_owned_example_fits_the_native_client() {
    let value: serde_json::Value = serde_json::from_slice(support::owned::EXAMPLE).expect("owned example JSON");
    let bytes = serde_json::to_vec(&value["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &DOCUMENT).expect("engine document");
    // Parsing a fixture without a deployment; only its limits are used.
    let engine = temper_service::startup::engine(&document, 0, &jig_protocol_config::Directories::working())
        .expect("engine configuration");
    let native = temper_client_shell_terminal::limits::native().client.client.domain;
    assert!(engine.limits.core.documents.rows <= native.rows);
    assert!(engine.limits.core.documents.lines <= native.rows);
    let paths = engine
        .limits
        .core
        .authority
        .segments
        .max(engine.limits.core.tasks.hold_segments)
        .max(engine.limits.core.tasks.authority_segments);
    assert!(paths <= native.segments);
    assert!(engine.limits.core.documents.text_bytes <= native.text);
    assert!(engine.limits.core.authority.grants <= native.authority);
    assert!(engine.limits.core.tasks.authority_grants <= native.authority);
    assert_owned_limits(&value["engine"]["limits"]);
}
fn assert_owned_limits(limits: &serde_json::Value) {
    let mut expected = serde_json::json!({"engine":{"limits":{"profile":"standard","overrides":{
        "documents.rows":2,"documents.text_bytes":512,
        "clients.streams":8,"clients.exchanges":12,
        "clients_http.streams":8,"clients_http.calls":12,
        "fleet.configured_workers":8,"fleet.workers":2,
        "tasks.inbox_bytes":874_120,"tasks.message_bytes":10_660,"tasks.result_bytes":10_660
    }}}});
    let agent: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/owned/agent.json")).expect("owned agent");
    support::configuration::turn_bounds::apply(&mut expected, support::configuration::turn_bounds::turn_body(&agent));
    assert_eq!(limits, &expected["engine"]["limits"]);
}

#[test]
fn the_retained_cluster_overrides_name_no_charter_documents() {
    // Two charters, the chat template and the deployment landing rule.
    const ITEMS: usize = 4;
    let bytes = std::fs::read(skein_scratch::paths::tree().join("bin/temper-box/tests/fixtures/cluster/settings.json"))
        .expect("retained cluster overrides");
    // The process story fills whole settings from the owner's reads before decoding.
    let overrides: serde_json::Value = serde_json::from_slice(&bytes).expect("cluster overrides JSON");
    let items = overrides["items"].as_array().expect("cluster setting overrides");
    assert_eq!(items.len(), ITEMS);
    for (index, number) in [(0, 1), (1, 2)] {
        let item = &items[index];
        assert_eq!(item["setting"], serde_json::json!({"kind":"charter","number":number}));
        assert_eq!(item["set"]["documents"], serde_json::json!([]));
    }
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
    for process in value["processes"].as_array().expect("cluster processes") {
        let name = process["name"].as_str().expect("process name");
        let memory = match name {
            "forge" | "llm" => serde_json::Value::Null,
            "engine" => cluster_document(&cluster, name)["engine"]["memory_bytes"].clone(),
            "w1" | "w2" => cluster_document(&cluster, name)["workers"][0]["memory_bytes"].clone(),
            _ => panic!("unknown cluster process {name}"),
        };
        assert_eq!(process["memory_bytes"], memory, "{name}'s bound must match its document");
    }
    scratch.remove();
}

fn cluster_documents(scratch: &std::path::Path) -> support::cluster::Cluster {
    support::cluster::Cluster::documents(
        scratch,
        &[("forge", "address", "127.0.0.1:3"), ("llm", "address", "127.0.0.1:1"), ("engine", "workers", "127.0.0.1:2")],
    )
}

fn cluster_document(cluster: &support::cluster::Cluster, process: &str) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(cluster.dir(process).join("config/temper.json")).expect("cluster document"))
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
    let engine_limits = &engine["engine"]["limits"]["overrides"];
    assert_eq!(
        engine_limits["fleet.grace_ms"],
        support::cluster::FLEET_GRACE_MS,
        "bin/temper-box/tests/fixtures/cluster/engine.json: engine.limits.overrides.fleet.grace_ms"
    );
    for (process, expected) in [("engine", Shape::EngineAlone), ("w1", Shape::WorkerAlone), ("w2", Shape::WorkerAlone)]
    {
        let root = cluster.dir(process);
        let directories = temper_real_loop_world::cluster::directories_in([
            &root.join("config"),
            &root.join("data"),
            &root.join("cache"),
        ]);
        let bytes = std::fs::read(root.join("config/temper.json")).expect("cluster document");
        let shape = startup::shape(&bytes, &jig_directories(&directories))
            .unwrap_or_else(|why| panic!("{process}: {}", String::from_utf8_lossy(&startup::lines(&why))));
        assert_eq!(shape, expected, "{process}");
        match shape {
            Shape::EngineAlone => {
                startup::named(&bytes, &jig_directories(&directories)).expect("checked engine alone");
            }
            Shape::WorkerAlone => {
                let part = startup::worker_part(&bytes, &jig_directories(&directories)).expect("checked worker alone");
                let worker = &part.worker.jig;
                let stop = jig_protocol_config::stop_bound(
                    &worker.graces,
                    worker.limits.link.ping,
                    worker.limits.link.silence,
                )
                .unwrap_or_else(|| {
                    panic!(
                        "bin/temper-box/tests/fixtures/cluster/{process}.json: workers[0].graces stop_bound overflows"
                    )
                });
                assert!(
                    stop < skein_lib::Duration::from_millis(support::cluster::FLEET_GRACE_MS),
                    "bin/temper-box/tests/fixtures/cluster/{process}.json: workers[0].graces stop_bound must be below engine.json: engine.limits.overrides.fleet.grace_ms"
                );
                let value: serde_json::Value = serde_json::from_slice(&bytes).expect("worker JSON");
                for key in ["link.ping_ms", "link.silence_ms"] {
                    assert_eq!(
                        value["workers"][0]["limits"]["overrides"][key], engine_limits[key],
                        "bin/temper-box/tests/fixtures/cluster/{process}.json: workers[0].limits.overrides.{key} must equal engine.json: engine.limits.overrides.{key}"
                    );
                }
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
    let engine = cluster_engine(&cluster);
    let core = jig_protocol_config::core(&engine, [0; 16], 1);
    let needs = jig_domain_core::settings::fleet_needs::configured(&core, &engine.limits.core);
    for process in ["w1", "w2"] {
        let profile = support::owned::deployment_profile(
            &cluster.dir(process),
            &format!("bin/temper-box/tests/fixtures/cluster/{process}.json"),
        );
        assert_eq!(
            jig_domain_fleet::shortfall(&needs, &profile.agent.bounds, jig_domain_fleet::HostKind::Worker),
            None,
            "{process} must admit startup stock before the story applies its settings"
        );
    }
    scratch.remove();
}

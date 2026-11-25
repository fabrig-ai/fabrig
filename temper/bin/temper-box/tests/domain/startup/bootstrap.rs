//! Admit the subprocess bootstrap and worker opening without a sitting
//! (temper/docs/design/testing/box/README.md, 2.1).
#[path = "opening.rs"]
mod opening;

use crate::{configuration, support, support_tests};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use support::{
    cluster::Cluster,
    limit_settings::{Bootstrap, figures_bytes},
};
use temper_client_protocol_wire::v1::AgentFigures;
use temper_playground::documents::Configuration;
use temper_service::startup::{self, LimitInputs, Resolved};

struct Fixture {
    name: &'static str,
    inputs: LimitInputs,
    bootstrap: Bootstrap,
}

struct EngineFixture {
    fixture: Fixture,
    cluster: Cluster,
    workers: Vec<Value>,
    agents: AgentFigures,
}

const ROOT: &str = "/fixture/bootstrap";

fn fixtures() -> Vec<Fixture> {
    let root = Path::new(ROOT);
    let forge = configuration::fixture_forgejo();
    [boxed("base", &root.join("base"), &support::Forge::Unreached), boxed("Forgejo", &root.join("forgejo"), &forge)]
        .into_iter()
        .chain(engine_fixtures().into_iter().map(|engine| engine.fixture))
        .collect()
}

fn engine_fixtures() -> Vec<EngineFixture> {
    let root = Path::new(ROOT);
    let cluster = configuration::cluster_documents(&root.join("cluster"));
    let turn = support_tests::cluster_turn_description(&root.join("turn"));
    let github = super::live_github_cluster(&root.join("github"));
    vec![
        engine("cluster", cluster, |_, _, _| {}),
        engine("cluster-turn", turn, |value, data, agents| {
            let providers = temper_agent_fixtures::providers::values(&value["world_providers"]);
            let agent = temper_agent_fixtures::with_figures(value, agents, data, &providers);
            let configured = support::cluster_turn::compose_engine(value.clone(), &agent);
            *value = configured.configuration;
            value["world_providers"] = temper_agent_fixtures::providers::flat(&configured.agents.providers);
            *agents = configured.agents.figures;
        }),
        engine("GitHub", github, |value, _, _| super::configure_live_github(value)),
    ]
}

fn boxed(name: &'static str, root: &Path, forge: &support::Forge<'_>) -> Fixture {
    let recipe = support::recipe(root, forge);
    let agent = recipe.agent.agent.declaration();
    let dirs = temper_service_shell::jig_directories(&support::configuration::directories(root));
    let inputs = LimitInputs::new(
        &recipe.read_start().document,
        &dirs,
        &recipe.files,
        Some(&agent),
        &support::admission::seeds(),
        &recipe.given,
        true,
        None,
    );
    let bootstrap = Bootstrap::compose(&figures_bytes(&recipe.configuration), Some(&agent)).expect(name);
    Fixture { name, inputs, bootstrap }
}

fn engine(
    name: &'static str,
    cluster: Cluster,
    edit: impl FnOnce(&mut Configuration, &Path, &mut AgentFigures),
) -> EngineFixture {
    let rendered = cluster.render_files("engine");
    let mut value = rendered.engine_configuration();
    let root = cluster.dir("engine");
    let data = root.join("data");
    let mut agents = temper_service::stock::agents();
    edit(&mut value, &data, &mut agents);
    let providers = temper_agent_fixtures::providers::take(&mut value).expect("fixture providers");
    let given = temper_agent_fixtures::providers::given_with(providers, None);
    let paths = [root.join("config"), data, root.join("cache")];
    let dirs = temper_real_loop_world::cluster::directories_in(paths.each_ref().map(PathBuf::as_path));
    let dirs = temper_service_shell::jig_directories(&dirs);
    let document = serde_json::to_vec(&value).expect("engine startup JSON");
    let named = startup::named_with(&document, &dirs, &given).expect("engine named at stock");
    let mut rendered = rendered.files;
    let settings = cluster.settings_file();
    let workers = rendered
        .iter()
        .find(|(path, _)| path == &settings)
        .map_or_else(Vec::new, |(_, bytes)| support::cluster::worker_settings_from(bytes));
    rendered.push((cluster.dir("llm").join("root.der"), skein_tls_kit::pki::ROOT.into()));
    let files = super::rendered_files(named.files, named.asked_files, &rendered);
    let inputs = LimitInputs::new(&document, &dirs, &files, None, &support::admission::seeds(), &given, false, None);
    let bootstrap = Bootstrap::compose(&figures_bytes(&value), None).expect(name);
    EngineFixture { fixture: Fixture { name, inputs, bootstrap }, cluster, workers, agents }
}

fn resolve_held(fixture: &Fixture) -> Resolved {
    let held = temper_engine_shell::testing::parse_held(Path::new("/fixture/limits-held"), &fixture.bootstrap.held())
        .expect("held file parses");
    startup::resolve_limits(&fixture.inputs, &held.jig, &held.temper)
        .unwrap_or_else(|why| panic!("{}: {}", fixture.name, String::from_utf8_lossy(&startup::lines(&why))))
}

#[test]
fn subprocess_bootstraps_hold_limits_the_served_engine_admits() {
    for fixture in fixtures() {
        let _ = resolve_held(&fixture);
    }
}

#[test]
fn subprocess_bootstraps_change_a_fresh_deployments_settings() {
    let reads = support_tests::limit_reads(&json!([]), &json!([]));
    for fixture in fixtures() {
        let mut file = None;
        let call = support::ctl::limit_settings_apply_with(
            Path::new("/fixture/bootstrap"),
            &fixture.bootstrap.overrides(),
            &reads,
            None,
            |_, bytes| file = Some(serde_json::from_slice::<serde_json::Value>(bytes).expect("owner change JSON")),
        );
        assert!(call.is_some(), "{} requires an owner change", fixture.name);
        let change = file.expect("one owner change");
        assert!(
            change["items"]
                .as_array()
                .expect("change items")
                .iter()
                .any(|item| { item["set"].as_array().is_some_and(|figures| !figures.is_empty()) }),
            "{} must set figures that are not already held: {change}",
            fixture.name,
        );
    }
}

#[test]
fn stock_without_owner_edits_needs_no_bootstrap() {
    let stock = Configuration::stock(serde_json::Value::Null);
    let bootstrap = Bootstrap::compose(&figures_bytes(&stock), None).expect("stock carriers");
    assert!(!bootstrap.needs_sitting(None, false, false));
    let reads = support_tests::limit_reads(&json!([]), &json!([]));
    let call = support::ctl::limit_settings_apply_with(
        Path::new("/fixture/bootstrap"),
        &bootstrap.overrides(),
        &reads,
        None,
        |_, _| panic!("no change file for stock"),
    );
    assert!(call.is_none());
}

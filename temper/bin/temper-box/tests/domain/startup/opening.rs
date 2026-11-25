//! Workers' admission on the subprocess bootstrap figures (temper/docs/design/testing/box/README.md, 2.1).
use super::{EngineFixture, engine_fixtures, resolve_held};
use crate::support;
use jig_domain_core::settings::{WorkerSetting, fleet_needs};
use jig_domain_fleet as fleet;
use jig_domain_link::Hello;
use serde_json::{Value, json};
use skein_lib::{Env, List, Queue, Time, Token, Wall};
use std::path::PathBuf;
use temper_engine_service::figures::Composer;
use temper_worker_service::{figures, startup};

#[test]
fn subprocess_workers_open_on_the_figures_their_engine_serves() {
    for EngineFixture { fixture, cluster, workers, agents } in engine_fixtures() {
        let held = resolve_held(&fixture);
        let inputs = &fixture.inputs;
        let engine = temper_engine_service::startup::checked_start_with_resolved(
            &inputs.document,
            &inputs.directories,
            &inputs.files,
            None,
            &inputs.seeds,
            List::with_capacity(0),
            None,
            &inputs.given,
            &held,
        )
        .unwrap_or_else(|why| {
            let lines = temper_engine_service::startup::lines(&why);
            panic!("{}: {}", fixture.name, String::from_utf8_lossy(&lines));
        })
        .engine;
        let mut composer = Composer::new(&engine.jig, &engine.limits);
        composer.take_limits(&held.jig, &held.temper);
        let served = composer.compose(&agents);

        // Use the fresh restore's configured needs, with the description's admitted worker settings.
        let core = temper_engine_service::startup::core(&engine.jig, inputs.seeds.deployment, inputs.seeds.seed);
        let limits = temper_engine_domain::core_limits(&held.root);
        let needs = fleet_needs::configured(&core, &limits);
        let mut fleet = fleet::Domain::new(&limits.fleet, limits.settings.versions, needs.clone());
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.fleet };
        let members = members(fixture.name, &workers);
        fleet.configure_workers(jig_domain_core::worker_membership(&members, &limits).expect("fixture membership"));
        let mut out = Queue::uncapped();
        fleet::step(&mut fleet, &env, fleet::Event::Needs(needs), &mut out);
        assert!(!members.is_empty(), "{}: no worker settings", fixture.name);
        assert_eq!(cluster.worker_names().count(), members.len(), "{}: membership of rendered workers", fixture.name);

        for name in cluster.worker_names() {
            let label = format!("{} {name}", fixture.name);
            let setting = members
                .iter()
                .find(|member| member.name.as_ref() == name.as_bytes())
                .unwrap_or_else(|| panic!("{label}: no worker setting"));
            let worker = setting.number.checked_sub(1).expect("positive worker setting number");
            let channel = Token::new(u64::from(setting.number));
            let mut figures = served.clone();
            figures.places = setting.places;
            let hello = worker_hello(&label, &cluster, name, figures);
            assert_eq!(hello.slots, setting.places, "{label}: worker's hello slots");

            fleet::step(&mut fleet, &env, fleet::Event::Hello { channel, worker, hello }, &mut out);
            while let Some(request) = out.pop() {
                if let fleet::Request::Refuse { hello, .. } = request {
                    panic!("{label}: fleet refused hello: {hello:?}");
                }
            }
            assert_eq!(fleet.worker_channel(worker), Some(channel), "{label}: fleet did not admit channel");
        }
    }
}

fn members(fixture: &str, workers: &[Value]) -> Vec<WorkerSetting> {
    use settings_form::view::{SettingEdit, SettingValue};
    let bytes = serde_json::to_vec(&json!({"items": workers})).expect("worker settings JSON");
    let items = settings_form::file::decode(&bytes, &settings_form::Limits::standard())
        .unwrap_or_else(|why| panic!("{fixture}: {}", String::from_utf8_lossy(&why)));
    items
        .iter()
        .map(|item| {
            let SettingEdit::Set(set) = &item.edit else { panic!("{fixture}: expected a worker setting") };
            let SettingValue::Worker(value) = &set.value else { panic!("{fixture}: expected a worker value") };
            WorkerSetting {
                number: value.number,
                name: value.name.clone(),
                places: value.places,
                digest: value.digest,
                withdrawn: value.withdrawn,
            }
        })
        .collect()
}

fn worker_hello(
    label: &str,
    cluster: &support::cluster::Cluster,
    name: &str,
    figures: jig_protocol_worker_wire::Figures,
) -> Hello {
    let rendered = cluster.render_files(name);
    let root = cluster.dir(name);
    let paths = [root.join("config"), root.join("data"), root.join("cache")];
    let dirs = temper_real_loop_world::cluster::directories_in(paths.each_ref().map(PathBuf::as_path));
    let dirs = temper_service_shell::jig_directories(&dirs);
    let document = rendered.start.unwrap_or_else(|| panic!("{label}: missing worker document")).document;
    let mut part = startup::worker_part(&document, &dirs, None, &[]).unwrap_or_else(|why| refused(label, &why));
    let files = super::super::read_rendered_files(core::mem::take(&mut part.files), Box::new([]), &rendered.files)
        .unwrap_or_else(|why| panic!("{label}: {why}"));
    let _unknown = figures::apply(&mut part, figures).unwrap_or_else(|why| refused(label, &why));
    let bytes = temper_worker_shell::agent::figures::configuration(
        part.figures.as_ref().expect("applied figures"),
        None,
        &paths[1],
        &part.environment,
    )
    .unwrap_or_else(|why| panic!("{label}: the engine's figures: {why}"));
    let agent =
        support::try_prepare_agent(bytes).unwrap_or_else(|why| panic!("{label}: the engine's figures: {why}")).agent;
    let mut service = startup::worker_alone(part, &files, &agent, support::admission::worker_runtime(), 7)
        .unwrap_or_else(|why| refused(label, &why));
    temper_worker_service::testing::startup_hello(&mut service)
        .unwrap_or_else(|| panic!("{label}: worker sent no hello"))
}

fn refused(label: &str, why: &startup::Problems) -> ! {
    panic!("{label}: {}", String::from_utf8_lossy(&startup::lines(why)))
}

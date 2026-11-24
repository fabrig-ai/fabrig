//! Subprocess fixtures apply their figures through ctl before restarting
//! (temper/docs/design/protocol/configuration/limit-settings.md, 8).
use jig_client_protocol_wire::Record as _;
use serde_json::{Value, json};
use std::{fs, path::Path};
use temper_playground::documents::Configuration;

/// Keep figures beside, never inside, the deployment document while composing a fixture.
pub fn remember(directory: &Path, configuration: &Configuration) {
    if let Some(providers) = configuration.get("world_providers") {
        temper_agent_fixtures::providers::write(&temper_agent_fixtures::data(directory), providers);
    }
    remember_figures(directory, configuration);
}

/// Keep only figures; the browser world commits its prospective providers through ctl.
pub fn remember_figures(directory: &Path, configuration: &Configuration) {
    let stock = Configuration::stock(Value::Null);
    let jig: Vec<_> = jig_protocol_config::names::NAMES
        .iter()
        .filter_map(|row| {
            let value = (row.value)(&configuration.jig);
            (value != (row.value)(&stock.jig))
                .then(|| json!({"name":row.name,"value":value,"reason":"subprocess fixture"}))
        })
        .collect();
    let own: Vec<_> = temper_service::limits::NAMES
        .iter()
        .filter_map(|row| {
            let value = (row.value)(&configuration.limits);
            (value != (row.value)(&stock.limits))
                .then(|| json!({"name":row.name,"value":value,"reason":"subprocess fixture"}))
        })
        .collect();
    super::write_private(
        &directory.join("fixture-limits.json"),
        &serde_json::to_vec(&json!({"limits":jig,"temper-limits":own})).expect("fixture limits"),
    );
}

/// Continue a fixture edit without losing its non-document figures.
#[must_use]
pub fn configuration(directory: &Path) -> Configuration {
    configuration_at(directory, super::read_file_shape(&directory.join(temper_config_shell::FILE)))
}

#[must_use]
pub fn configuration_at(directory: &Path, shape: temper_config_shell::Shape) -> Configuration {
    let mut value = Configuration::from_jig(super::start_document(&directory.join(temper_config_shell::FILE), shape));
    value["world_providers"] = temper_agent_fixtures::providers::read(&temper_agent_fixtures::data(directory));
    let path = directory.join("fixture-limits.json");
    if path.exists() {
        let figures: Value = serde_json::from_slice(&fs::read(path).expect("fixture limits")).expect("limits JSON");
        for item in figures["limits"].as_array().expect("jig figures") {
            jig_protocol_config::overrides::apply(
                &mut value.jig,
                item["name"].as_str().expect("name").as_bytes(),
                item["value"].as_u64().expect("figure"),
            )
            .expect("jig figure");
        }
        for item in figures["temper-limits"].as_array().expect("temper figures") {
            temper_service::limits::apply(
                &mut value.limits,
                item["name"].as_str().expect("name").as_bytes(),
                item["value"].as_u64().expect("figure"),
            )
            .expect("temper figure");
        }
    }
    value
}

/// A subprocess's places are its worker setting, never its file's slots.
pub fn local_places(root: &Path, places: u32) {
    use jig_client_protocol_wire::WorkerSetting;
    let local = WorkerSetting { number: 1, name: b"local".as_slice().into(), places, digest: None, withdrawn: false };
    let mut writer = skein_lib::Writer::new(
        usize::try_from(local.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("local bound"),
    );
    local.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("local setting");
    super::write_private(&root.join("local.setting"), &writer.finish());
    let item = json!({"setting":{"kind":"worker","number":1},"set":{"name":"local","places":places,"digest":null}});
    super::write_private(
        &root.join("local-worker-settings.json"),
        &serde_json::to_vec(&json!({"items":[item]})).expect("local settings JSON"),
    );
}

/// Install the composed fixture once through the shipped settings command.
pub fn install(directory: &Path) {
    install_at(&super::configuration::directories(directory));
}

pub fn install_paths(paths: &[std::path::PathBuf; 3]) {
    install_paths_with(paths, Vec::new());
}

/// Install a fresh fixture's additional owner settings before the bootstrap drains.
pub(crate) fn install_paths_with(paths: &[std::path::PathBuf; 3], extra: Vec<super::ctl::Step>) {
    use std::os::unix::ffi::OsStrExt;
    let mut directories = super::configuration::directories(&paths[0]);
    directories.data = paths[1].as_os_str().as_bytes().into();
    directories.cache = paths[2].as_os_str().as_bytes().into();
    install_at_with(&directories, extra);
}

pub fn install_at(directories: &temper_box_shell::directories::Directories) {
    install_at_with(directories, Vec::new());
}

fn install_at_with(directories: &temper_box_shell::directories::Directories, extra: Vec<super::ctl::Step>) {
    use std::os::unix::ffi::OsStrExt;
    let directory = Path::new(std::ffi::OsStr::from_bytes(&directories.config));
    let data = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    let pending = directory.join("fixture-limits.json");
    if !pending.exists() {
        assert!(extra.is_empty(), "additional bootstrap settings require fixture limits");
        return;
    }
    let bytes = fs::read(&pending).expect("fixture limits");
    let applied = data.join("fixture-limits-applied.json");
    if fs::read(&applied).ok().as_deref() == Some(bytes.as_slice()) {
        assert!(extra.is_empty(), "additional bootstrap settings require a fresh fixture");
        return;
    }
    let mut value = configuration(directory);
    let is_box = value["workers"].as_array().is_some_and(|workers| !workers.is_empty());
    let sources = if is_box {
        let agent = temper_agent_fixtures::read(directory);
        value.limits.tool_payload =
            u32::try_from(agent["profile"]["declared"]["tool_payload"].as_u64().expect("agent payload"))
                .expect("payload");
        temper_service_core::derived::Sources {
            turn: super::configuration::turn_bounds::turn_body(&agent),
            payload: value.limits.tool_payload,
            inbox: u32::try_from(agent["profile"]["policy"]["inbox"].as_u64().unwrap_or(82)).expect("inbox"),
            unacknowledged: u32::try_from(agent["profile"]["policy"]["unacknowledged"].as_u64().unwrap_or(8))
                .expect("unacknowledged"),
        }
    } else {
        temper_service_core::derived::Sources {
            payload: value.limits.tool_payload,
            ..temper_service_core::derived::STOCK
        }
    };
    let (floor, own_floor) = temper_service::limits::base(value.limits.live_goals, value.limits.tool_payload, sources)
        .expect("fixture derivation");
    for name in [
        "tasks.message_bytes",
        "tasks.result_bytes",
        "tasks.inbox_bytes",
        "core.call_answer_bytes",
        "io.output",
        "hub.turn_bytes",
        "hub.transcript_bytes",
        "hub.turn_queue_bytes",
        "core.resume_bytes",
        "hub.outcome_bytes",
    ] {
        let row = jig_protocol_config::names::find(name.as_bytes()).expect("carrier");
        let (old, least) = ((row.value)(&value.jig), (row.value)(&floor));
        if old < least {
            eprintln!("fixture carrier {name}: {old} -> {least}");
            jig_protocol_config::overrides::apply(&mut value.jig, name.as_bytes(), least).expect("carrier floor");
        }
    }
    for name in [
        "forge.client.op_bytes",
        "forge.client.answer_bytes",
        "root.journal.transcript_bytes",
        "root.loads.transcript_bytes",
        "root.loads.bytes",
        "root.loads.reply_bytes",
        "root.loads.read_bytes",
    ] {
        let row = temper_service::limits::find(name.as_bytes()).expect("carrier");
        let (old, least) = ((row.value)(&value.limits), (row.value)(&own_floor));
        if old < least {
            eprintln!("fixture carrier {name}: {old} -> {least}");
            temper_service::limits::apply(&mut value.limits, name.as_bytes(), least).expect("carrier floor");
        }
    }
    if extra.is_empty() {
        remember(directory, &value);
    } else {
        // These owner edits, not fixture copies, supply any provider changes.
        remember_figures(directory, &value);
    }
    let figures: Value =
        serde_json::from_slice(&fs::read(&pending).expect("normalized figures")).expect("figures JSON");
    let mut overrides = Vec::new();
    for kind in ["limits", "temper-limits"] {
        for item in figures[kind].as_array().expect("kind figures") {
            let name = item["name"].as_str().expect("name");
            overrides.push((kind, name, item["value"].as_u64().expect("figure"), "subprocess fixture"));
        }
    }
    apply_with(directories, &overrides, extra);
    super::write_private(&applied, &fs::read(pending).expect("applied figures"));
}

/// Write the operator's bootstrap file for the same figures the setting takes.
pub fn hold(directory: &Path, overrides: &[(&str, &str, u64, &str)]) {
    use std::fmt::Write;
    let mut lines = String::new();
    for (_, name, value, _) in overrides {
        writeln!(&mut lines, "{name} {value}").expect("held line");
    }
    super::write_private(&directory.join("limits-held"), lines.as_bytes());
}

/// Bootstrap a fresh sitting with limits-held, apply the same figures as
/// settings, drain, and remove the file. The next serve takes the settings.
fn apply_with(
    directories: &temper_box_shell::directories::Directories,
    overrides: &[(&str, &str, u64, &str)],
    extra: Vec<super::ctl::Step>,
) {
    use super::ctl::{Call, Step, Story};
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let directory = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    hold(directory, overrides);
    let mut steps = vec![Step::Run(Call::sign_in().in_directory("limit-owner"))];
    steps.extend(super::ctl::limit_settings(directory, overrides).into_iter().chain(extra).map(|step| match step {
        Step::Run(call) => Step::Run(call.in_directory("limit-owner")),
        Step::Then(next) => Step::Then(Box::new(move |calls| next(calls).in_directory("limit-owner"))),
        _ => unreachable!("bootstrap settings steps are calls"),
    }));
    steps.push(Step::Run(Call::new(&["sign-out"]).in_directory("limit-owner")));
    let command = skein_world::end_to_end::Command {
        program: skein_scratch::paths::binary("temper-box").with_file_name(
            match super::read_file_shape(
                &Path::new(std::ffi::OsStr::from_bytes(&directories.config)).join(temper_config_shell::FILE),
            ) {
                temper_config_shell::Shape::Standalone => "temper-box",
                temper_config_shell::Shape::EngineAlone => "temper-engine",
                temper_config_shell::Shape::WorkerAlone => panic!("a worker alone has no limit settings store"),
            },
        ),
        arguments: std::iter::once("serve".into())
            .chain(directories.flags().into_vec().into_iter().map(|word| std::ffi::OsString::from_vec(word.into_vec())))
            .collect(),
        directory: directory.into(),
        environment: Vec::new(),
    };
    let serve = skein_world::end_to_end::Binary::start(command, skein_world::end_to_end::Mode::Pipes, 65_536)
        .expect("limit-setting sitting");
    let seen = super::ctl::run_with_serve(Story::new(directory, steps), serve);
    assert!(seen.mismatches.is_empty(), "{}", seen.mismatches.join("\n"));
    for call in seen.calls {
        assert_eq!(
            call.exit,
            Some(skein_io::kernel::Exit::Code(0)),
            "bootstrap settings: {}",
            String::from_utf8_lossy(&call.stderr)
        );
    }
    fs::remove_file(directory.join("limits-held")).expect("release limit settings");
}

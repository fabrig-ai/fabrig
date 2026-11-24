//! Subprocess fixtures apply their figures through ctl before restarting
//! (temper/docs/design/protocol/configuration/limit-settings.md, 8).
use jig_client_protocol_wire::Record as _;
use jig_domain_core::settings::Override;
use serde_json::{Value, json};
use std::{fs, path::Path};
use temper_engine_service::startup::{self, LimitKind};
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
    super::write_private(&directory.join("fixture-limits.json"), &figures_bytes(configuration));
}

/// Serialize the fixture figures in the same form the native fixture retains.
#[must_use]
pub fn figures_bytes(configuration: &Configuration) -> Vec<u8> {
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
    serde_json::to_vec(&json!({"limits":jig,"temper-limits":own})).expect("fixture limits")
}

/// The one composition used for a held start and its subsequent owner change.
#[derive(Debug)]
pub struct Bootstrap {
    jig: Vec<Override>,
    temper: Vec<Override>,
}
impl Bootstrap {
    /// `agent` is the local worker's declaration, not the declarations of remote workers.
    pub fn compose(figures: &[u8], agent: Option<&temper_service_core::agent::Declaration>) -> Result<Self, String> {
        let figures: Value = serde_json::from_slice(figures).expect("fixture figures JSON");
        let mut jig = overrides(&figures["limits"]);
        let mut temper = overrides(&figures["temper-limits"]);
        let floors = startup::carrier_floors(agent, &jig, &temper)
            .map_err(|why| String::from_utf8_lossy(&startup::lines(&why)).into_owned())?;
        for (kind, name, needs) in &floors {
            let overrides = match kind {
                LimitKind::Jig => &mut jig,
                LimitKind::Temper => &mut temper,
            };
            raise_to(overrides, name, *needs);
        }
        jig.sort_by(|left, right| left.name.cmp(&right.name));
        temper.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(Self { jig, temper })
    }

    #[must_use]
    pub fn held(&self) -> String {
        held_text(&self.overrides())
    }

    #[must_use]
    pub fn overrides(&self) -> Vec<(&str, &str, u64, &str)> {
        let mut overrides = Vec::new();
        for (kind, items) in [(LimitKind::Jig, &self.jig), (LimitKind::Temper, &self.temper)] {
            for item in items {
                overrides.push((word(kind.name()), word(&item.name), item.value, word(&item.reason)));
            }
        }
        overrides
    }

    fn bytes(&self) -> Vec<u8> {
        let items = |overrides: &[Override]| -> Vec<_> {
            overrides
                .iter()
                .map(|item| json!({"name":word(&item.name),"value":item.value,"reason":word(&item.reason)}))
                .collect()
        };
        serde_json::to_vec(&json!({"limits":items(&self.jig),"temper-limits":items(&self.temper)}))
            .expect("figures JSON")
    }

    /// A fresh store with empty lists needs no sitting unless another edit needs it.
    #[must_use]
    pub fn needs_sitting(&self, local: Option<&[u8]>, extra: bool, previously_applied: bool) -> bool {
        !self.jig.is_empty() || !self.temper.is_empty() || local.is_some() || extra || previously_applied
    }
}

fn word(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("fixture JSON text")
}

fn overrides(items: &Value) -> Vec<Override> {
    items
        .as_array()
        .expect("fixture override list")
        .iter()
        .map(|item| Override {
            name: item["name"].as_str().expect("limit name").as_bytes().into(),
            value: item["value"].as_u64().expect("limit figure"),
            reason: item["reason"].as_str().expect("limit reason").as_bytes().into(),
        })
        .collect()
}

fn raise_to(items: &mut Vec<Override>, name: &[u8], needs: u64) {
    if let Some(item) = items.iter_mut().find(|item| item.name.as_ref() == name) {
        item.value = item.value.max(needs);
    } else {
        items.push(Override { name: name.into(), value: needs, reason: b"subprocess fixture".as_slice().into() });
    }
}

/// Continue a fixture edit without losing its non-document figures.
#[must_use]
pub fn configuration(directory: &Path) -> Configuration {
    configuration_at(directory, super::read_file_shape(&directory.join(temper_config_shell::FILE)))
}

#[must_use]
pub fn configuration_at(directory: &Path, shape: temper_config_shell::Shape) -> Configuration {
    let document = super::start_document(&directory.join(temper_config_shell::FILE), shape);
    let providers = temper_agent_fixtures::providers::read(&temper_agent_fixtures::data(directory));
    let path = directory.join("fixture-limits.json");
    let figures = path.exists().then(|| fs::read(path).expect("fixture limits"));
    configuration_over(document, providers, figures.as_deref())
}

/// Read the typed configuration from the start, provider copy and optional retained figures.
#[must_use]
pub fn configuration_over(document: Value, providers: Value, figures: Option<&[u8]>) -> Configuration {
    let mut value = Configuration::from_jig(document);
    value["world_providers"] = providers;
    if let Some(bytes) = figures {
        let figures: Value = serde_json::from_slice(bytes).expect("limits JSON");
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

/// Encode the worker setting copy; the document's slots do not set its places.
#[must_use]
pub fn local_copy(places: u32) -> Box<[u8]> {
    use jig_client_protocol_wire::WorkerSetting;
    let local = WorkerSetting { number: 1, name: b"local".as_slice().into(), places, digest: None, withdrawn: false };
    let mut writer = skein_lib::Writer::new(
        usize::try_from(local.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("local bound"),
    );
    local.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("local setting");
    writer.finish()
}

/// A subprocess's places are its worker setting, never its file's slots.
pub fn local_places(root: &Path, places: u32) {
    super::write_private(&root.join("local.setting"), &local_copy(places));
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
    let previous = fs::read(&applied).ok();
    if previous.as_deref() == Some(bytes.as_slice()) {
        assert!(extra.is_empty(), "additional bootstrap settings require a fresh fixture");
        return;
    }
    let path = directory.join(temper_config_shell::FILE);
    let shape = super::read_file_shape(&path);
    let agent = if shape == temper_config_shell::Shape::Standalone {
        let document = super::start_document(&path, shape);
        document["workers"].as_array().filter(|workers| !workers.is_empty()).map(|_| {
            let providers = temper_service_shell::providers::read(
                &data.join("providers.setting"),
                temper_service::stock::providers(),
                &mut Vec::new(),
            );
            let agent = temper_agent_fixtures::with_figures(
                &document,
                &temper_agent_fixtures::agent_figures(data),
                data,
                &providers,
            );
            super::prepare_agent(serde_json::to_vec(&agent).expect("fixture agent JSON").into_boxed_slice())
                .agent
                .declaration()
        })
    } else {
        None
    };
    let bootstrap = Bootstrap::compose(&bytes, agent.as_ref()).expect("fixture carrier derivation");
    let bytes = bootstrap.bytes();
    let local = data.join("local-worker-settings.json");
    let local = local.exists().then(|| fs::read(local).expect("local worker settings"));
    super::write_private(&pending, &bytes);
    if previous.as_deref() == Some(bytes.as_slice()) {
        assert!(extra.is_empty(), "additional bootstrap settings require a fresh fixture");
        return;
    }
    if bootstrap.needs_sitting(local.as_deref(), !extra.is_empty(), previous.is_some()) {
        apply_with(directories, shape, &bootstrap, local, extra);
    }
    super::write_private(&applied, &bytes);
}

/// Write the operator's bootstrap file for the same figures the setting takes.
pub fn hold(directory: &Path, overrides: &[(&str, &str, u64, &str)]) {
    super::write_private(&directory.join("limits-held"), held_text(overrides).as_bytes());
}

/// The bootstrap file keeps the operator's override order across both limit kinds.
#[must_use]
pub fn held_text(overrides: &[(&str, &str, u64, &str)]) -> String {
    use std::fmt::Write;
    let mut lines = String::new();
    for (_, name, value, _) in overrides {
        writeln!(&mut lines, "{name} {value}").expect("held line");
    }
    lines
}

/// Bootstrap a fresh sitting with limits-held, apply the same figures as
/// settings, drain, and remove the file. The next serve takes the settings.
fn apply_with(
    directories: &temper_box_shell::directories::Directories,
    shape: temper_config_shell::Shape,
    bootstrap: &Bootstrap,
    local: Option<Vec<u8>>,
    extra: Vec<super::ctl::Step>,
) {
    use super::ctl::{Call, Step, Story};
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    let directory = Path::new(std::ffi::OsStr::from_bytes(&directories.data));
    super::write_private(&directory.join("limits-held"), bootstrap.held().as_bytes());
    let mut steps = vec![Step::Run(Call::sign_in().in_directory("limit-owner"))];
    steps.extend(super::ctl::limit_settings(directory, &bootstrap.overrides(), local).into_iter().chain(extra).map(
        |step| match step {
            Step::Run(call) => Step::Run(call.in_directory("limit-owner")),
            Step::Then(next) => Step::Then(Box::new(move |calls| next(calls).in_directory("limit-owner"))),
            Step::ThenIf(next) => {
                Step::ThenIf(Box::new(move |calls| next(calls).map(|call| call.in_directory("limit-owner"))))
            }
            _ => unreachable!("bootstrap settings steps are calls"),
        },
    ));
    steps.push(Step::Run(Call::new(&["sign-out"]).in_directory("limit-owner")));
    let command = skein_world::end_to_end::Command {
        program: skein_scratch::paths::binary("temper-box").with_file_name(match shape {
            temper_config_shell::Shape::Standalone => "temper-box",
            temper_config_shell::Shape::EngineAlone => "temper-engine",
            temper_config_shell::Shape::WorkerAlone => panic!("a worker alone has no limit settings store"),
        }),
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

//! Inline startup fixtures share settings and limits without entering a world loop.
//! Contract: smith/docs/design/testing.md, section 2.1.

use crate::settings::{self, Layer, Source};
use std::path::PathBuf;

pub(crate) fn trust() -> PathBuf {
    skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der")
}

pub(crate) fn ready<T>(result: Result<T, settings::Refusal>) -> T {
    result.unwrap_or_else(|why| panic!("{}", why.line()))
}

pub(crate) fn source(path: PathBuf, text: String) -> Source {
    Source { layer: Layer::User, path, text }
}

pub(crate) fn settings(toml: &str) -> settings::Settings {
    ready(settings::merge(
        &crate::command::Shared::default(),
        "/workspace".into(),
        &[source("fixture.toml".into(), toml.into())],
    ))
}

#[expect(clippy::zero_sized_map_values, reason = "The fixture retains the configured program names.")]
pub(crate) fn resolved(workspace: PathBuf) -> settings::resolve::Resolved {
    use std::collections::BTreeMap;
    let (local, service) = smith_local_process_world::process::configuration(skein_io::kernel::Fd::new(70), false);
    settings::resolve::Resolved {
        settings: settings::Settings { paths: settings::Paths::default(), workspace, values: BTreeMap::new() },
        accounts: Vec::new(),
        agent: smith_agent_shell::config::Configuration {
            programs: BTreeMap::new(),
            endpoint_programs: BTreeMap::new(),
            service,
            memory: u64::MAX,
            trace: None,
            profile: smith_agent_service::profile::standard(),
            declarations: smith_agent_process_world::declarations(0),
        },
        limits: local.limits.local,
        local: local.local,
        outside_models: BTreeMap::new(),
    }
}

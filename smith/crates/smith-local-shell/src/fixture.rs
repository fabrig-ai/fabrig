//! Inline startup fixtures share settings and limits without entering a world loop.
//! Contract: smith/docs/design/testing-tiers.md, section 1.

use crate::settings::{self, Layer, Source};
use skein_scratch::Scratch;
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

/// Commands find the test process's rg beside the system shell and git.
pub(crate) fn commands_path() -> String {
    use std::os::unix::ffi::OsStrExt;
    let path = std::env::var_os("PATH");
    let rg = smith_agent_shell::config::find_rg(path.as_ref().map(|path| path.as_bytes()))
        .expect("check fixtures require rg on the test process's PATH");
    std::env::join_paths([
        rg.parent().expect("rg directory"),
        std::path::Path::new("/usr/bin"),
        std::path::Path::new("/bin"),
    ])
    .expect("fixture PATH")
    .into_string()
    .expect("fixture PATH is UTF-8")
}

/// Writes a stand-in for tests that check the program's path but do not run it.
pub(crate) fn program(scratch: &Scratch) -> PathBuf {
    let path = scratch.path().join("claude");
    fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("write the scratch program");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("make the scratch program executable");
    path
}

#[test]
fn the_stand_in_is_an_executable_script_inside_its_scratch() {
    let scratch = Scratch::new("smith-program");
    let path = program(&scratch);
    assert!(path.is_absolute() && path.starts_with(scratch.path()), "absolute path inside the scratch");
    let metadata = fs::symlink_metadata(&path).expect("scratch program metadata");
    assert!(metadata.is_file(), "regular scratch program");
    assert_ne!(metadata.permissions().mode() & 0o111, 0, "executable scratch program");
    assert_eq!(fs::read(&path).expect("scratch program bytes"), b"#!/bin/sh\nexit 0\n", "no-op shell script");
}

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

pub(crate) fn resolved(workspace: PathBuf) -> settings::resolve::Resolved {
    use std::collections::BTreeMap;
    let (local, service) = smith_local_process_world::process::configuration(skein_io::kernel::Fd::new(70), false);
    settings::resolve::Resolved {
        settings: settings::Settings { paths: settings::Paths::default(), workspace, values: BTreeMap::new() },
        accounts: Vec::new(),
        agent: smith_agent_shell::config::Configuration {
            program: None,
            program_endpoints: std::collections::BTreeSet::new(),
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

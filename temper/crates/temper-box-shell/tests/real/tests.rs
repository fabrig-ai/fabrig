#[cfg(target_os = "linux")]
use temper_box_kit::configuration;
#[cfg(target_os = "linux")]
use temper_box_shell::testing::{scripted, serve};

mod box_kinds;
mod box_link_form;
mod upgrade;

// Linux's passwd entry is the reference for macOS's command lookup.
#[cfg(target_os = "linux")]
#[test]
fn id_login_agrees_with_the_effective_users_passwd_entry() {
    let user = skein_shell::effective_user();
    let expected = skein_shell::login_passwd(user).expect("effective user's passwd login");
    assert_eq!(temper_box_shell::testing::login(user).expect("Linux login"), expected);
    assert_eq!(skein_shell::login_id().expect("macOS login lookup"), expected);
}

// This check names open files through /proc/self/fd links; macOS's /dev/fd has no links.
#[cfg(target_os = "linux")]
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "the comparison retains typed limits and the scripted listener for both startup routes"
)]
fn the_standalone_operations_check_retains_no_descriptors() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let directory = std::env::temp_dir().join(format!("temper-operations-{}", std::process::id()));
    fs::create_dir(&directory).expect("operations scratch");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private scratch");
    for name in ["accounts", "secrets", "state"] {
        fs::create_dir_all(directory.join(name)).expect("fixture parents");
    }
    let write = |name: &str, bytes: &[u8]| {
        let path = directory.join(name);
        fs::write(&path, bytes).expect("fixture bytes");
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture");
    };
    for (name, bytes) in jig_channel_kit::fixtures::files(false).entries.into_boxed() {
        if name.starts_with(b"charters/") || name.as_ref() == b"agent.json" {
            continue;
        }
        write(std::str::from_utf8(&name).expect("fixture path"), bytes.expose());
    }
    for (name, bytes) in [
        ("worker.secret", b"worker-secret".as_slice()),
        ("secrets/forge-1.user", b"42\n".as_slice()),
        ("secrets/forge-1.api-token", b"forge-1-api-token-value".as_slice()),
        ("secrets/forge-1.git-token", b"forge-1-git-token-value".as_slice()),
        ("root.der", skein_tls_kit::pki::ROOT),
    ] {
        write(name, bytes);
    }
    let listener = scripted::choose().expect("scripted listener");
    let environment = [(
        Box::from(scripted::ADDRESS),
        listener.local_addr().expect("scripted address").to_string().into_bytes().into_boxed_slice(),
    )];
    let (agent, mut config) = configuration::typed_documents(&directory, "/fixture/temper");
    let file = temper_config_shell::render(&configuration::file(), temper_config_shell::Shape::Standalone);
    let named_file = directory.join("worker.secret");
    write(temper_config_shell::FILE, &file);
    temper_agent_fixtures::install(&directory, &serde_json::from_slice(&agent).expect("agent JSON"));
    let composed = temper_agent_fixtures::at(&config, &directory);
    configuration::turn_bounds::apply(&mut config, configuration::turn_bounds::turn_body(&composed));
    let document = serde_json::to_vec(&config).expect("deployment document");
    assert_eq!(configuration::flags(&directory).len(), 6);
    let operations = temper_box_shell::operations_in_with_limits(
        "temper",
        &environment,
        b"/",
        &configuration::directories(&directory),
        &document,
        &temper_service::stock::Given::Stock,
        &config.jig,
        &config.limits,
    )
    .expect("standalone operations");
    assert_no_descriptors_in(&directory);
    assert!(operations > 0);
    assert!(!directory.join("workspaces").exists(), "counting credit must not pin the agent or open roots");
    let mut owned = Vec::new();
    let (mut service, _, _) = serve::prepare_with_limits(
        "temper",
        &environment,
        b"/",
        &configuration::directories(&directory),
        &document,
        &mut owned,
        false,
        false,
        None,
        None,
        &temper_service::stock::Given::Stock,
        None,
        true,
        &[],
        Some((&config.jig, &config.limits)),
        &temper_config_shell::Programs::default(),
    )
    .expect("same standalone preparation");
    assert_eq!(operations, service.operations());
    temper_engine_shell::serve::close_limits_store(service.engine_component().store()).expect("close opened store");
    drop(service);
    temper_service_shell::close(owned);
    assert_no_descriptors_in(&directory);
    let binary = std::env::current_exe().expect("current binary");
    assert_eq!(
        fs::metadata(directory.join("workspaces/agent").join(binary.file_name().expect("binary name")))
            .expect("pinned program")
            .len(),
        fs::metadata(&binary).expect("running program").len()
    );
    let prepared = serve::configured_agent(
        &document,
        &configuration::directories(&directory),
        &environment,
        &temper_config_shell::Programs::default(),
    )
    .expect("derived agent")
    .expect("local agent");
    assert_eq!(
        fs::read(directory.join("workspaces/agent/agent.json")).expect("pinned configuration"),
        prepared.bytes.as_ref()
    );
    fs::remove_file(&named_file).expect("remove named file");
    assert_eq!(
        temper_box_shell::operations_in(&environment, b"/", &configuration::directories(&directory), &document)
            .expect("count without named file"),
        operations
    );
    write(temper_config_shell::FILE, &file);
    let why = temper_box_shell::prepare_in(&environment, b"/", &configuration::directories(&directory))
        .err()
        .expect("missing file refused");
    assert!(why.starts_with("temper serve: "), "{why}");
    assert!(why.contains(named_file.to_str().expect("named path")), "{why}");
    fs::remove_dir_all(directory).expect("remove operations scratch");
}

// Only the Linux descriptor-link check calls this helper.
#[cfg(target_os = "linux")]
#[expect(clippy::unnecessary_debug_formatting, reason = "escape non-printing bytes in descriptor paths")]
fn assert_no_descriptors_in(directory: &std::path::Path) {
    for entry in std::fs::read_dir("/proc/self/fd").expect("open descriptor list") {
        let path = entry.expect("descriptor entry").path();
        if let Ok(target) = std::fs::read_link(&path) {
            assert!(!target.starts_with(directory), "descriptor {path:?} still holds {target:?}");
        }
    }
}

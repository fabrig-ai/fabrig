use crate::{listeners, support};
use jig_client_protocol_wire::Record as _;
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};
use support::bundle_fixture::BUNDLE_FILES;

#[test]
fn a_stock_policy_past_its_limit_refuses_before_listening() {
    let scratch = support::Scratch::startup("stock-policy-floor");
    let output = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("init")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    fs::write(
        scratch.path().join("limits-held"),
        "core.policy_bytes 100
",
    )
    .expect("held limit");
    let output = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("serve")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("serve");
    assert_eq!(output.status.code(), Some(1));
    // Each executor place in four policy authorities adds 32 bytes.
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "temper-box serve: limits-held line 1: core.policy_bytes (setting limits): core.policy_bytes is 100; the stock's policy 1 needs 6455 bytes; raise it in engine.limits.overrides\n",
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stdout.is_empty(), "no sign-in link: {}", String::from_utf8_lossy(&output.stdout));
    scratch.remove();
}

#[test]
fn a_held_login_return_hook_is_idempotent_and_refuses_unsafe_metadata_without_reading_the_file() {
    let scratch = support::Scratch::startup("rung-return-hook");
    let root = scratch.path();
    let account = root.join("synthetic-record");
    // This is a synthetic file, never an account or a real login. A command
    // that should never be needed makes an accidental export fail the test.
    support::write_private(&account, b"synthetic fixture");
    support::held_login::return_hook(Path::new("/must-not-export"), root, root, &account).expect("install return hook");
    let run = || std::process::Command::new(root.join("give-back")).output().expect("return hook");
    assert!(run().status.success(), "already returned file is accepted by stat");
    assert!(run().status.success(), "the launcher's second return is harmless");
    assert!(root.join("returning").is_file(), "cleanup prevents an import starting afterwards");
    fs::set_permissions(&account, fs::Permissions::from_mode(0o644)).expect("unsafe mode");
    assert!(!run().status.success(), "public metadata is refused");
    support::write_private(&account, b"");
    assert!(!run().status.success(), "empty file is refused");
    fs::remove_file(&account).expect("remove fixture");
    symlink("missing", &account).expect("fixture link");
    assert!(!run().status.success(), "a link never counts as a returned record");
    fs::remove_file(&account).expect("remove fixture link");
    scratch.remove();
    assert_held_record_export();
}

fn assert_held_record_export() {
    let scratch = support::Scratch::startup("held-login-export");
    let root = scratch.path();
    let account = root.join("synthetic-record");
    let run = || std::process::Command::new(root.join("give-back")).output().expect("return hook");
    let stub = held_record_exporter(root);
    support::held_login::return_hook(&stub, root, &root.join("return-client"), &account).expect("install return hook");
    assert!(run().status.success(), "export selects the actual account number from public ctl metadata");
    let returned = fs::symlink_metadata(&account).expect("returned synthetic record");
    assert!(returned.is_file() && returned.len() > 0);
    assert_eq!(returned.permissions().mode() & 0o777, 0o600);
    scratch.remove();
}

fn held_record_exporter(root: &Path) -> std::path::PathBuf {
    let stub = root.join("ctl-stub's path");
    support::write_private(
        &stub,
        br#"#!/bin/sh
set -eu
root=$(dirname "$0")
test "$2" = "$root/return-client"
case "$4" in
list) printf '%s\n' '{"accounts":{"party":[{"provider":1,"plan":true,"number":7}]}}' ;;
export) test "$5" = 7; umask 077; printf '%s' synthetic > "$6" ;;
*) exit 1 ;;
esac
"#,
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable synthetic ctl");
    stub
}

fn held_import_echo(root: &Path) -> std::path::PathBuf {
    let stub = root.join("ctl-stub's path");
    support::write_private(
        &stub,
        br#"#!/bin/sh
printf '%s\n' "$@"
"#,
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable ctl stub");
    stub
}

#[test]
fn a_held_login_import_waits_for_the_rust_file_lock() {
    use std::os::unix::fs::OpenOptionsExt;
    let scratch = support::Scratch::startup("held-login-lock");
    let root = scratch.path();
    let stub = held_import_echo(root);
    let _call = support::held_login::import_guard(&stub, root, root, &[]);
    let lock = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(root.join("login.lock"))
        .expect("login lock");
    lock.lock().expect("hold login lock in Rust");
    let mut guard = support::parent::Guard::new(
        std::process::Command::new(root.join("import-held"))
            .env_clear()
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("start import wrapper"),
    );
    // One bounded observation while Rust holds the lock; ctl exits as soon as it starts.
    std::thread::sleep(std::time::Duration::from_millis(100));
    let blocked = guard.child.as_mut().expect("import child").try_wait().expect("import status").is_none();
    drop(lock);
    let output = guard.child.take().expect("import child").wait_with_output().expect("import after unlock");
    assert!(blocked, "the import wrapper must not reach ctl while Rust holds login.lock");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).expect("ctl arguments"), format!("ctl\n{}\n", root.display()));
    scratch.remove();
}

#[test]
fn a_held_login_import_refuses_after_returning_without_starting_ctl() {
    let scratch = support::Scratch::startup("held-login-import");
    let root = scratch.path();
    let stub = held_import_echo(root);
    let arguments = ["account", "import", "synthetic-record", "--key", "connect-plan"];
    let call = support::held_login::import_guard(&stub, root, root, &arguments);
    let ended = support::ctl::alone(root, call.clone());
    assert_eq!(ended.exit, Some(skein_io::kernel::Exit::Code(0)));
    assert_eq!(
        String::from_utf8(ended.stdout).expect("stub arguments"),
        format!("ctl\n{}\naccount\nimport\nsynthetic-record\n--key\nconnect-plan\n", root.display())
    );
    support::write_private(&root.join("returning"), b"");
    let ended = support::ctl::alone(root, call);
    assert_eq!(ended.exit, Some(skein_io::kernel::Exit::Code(1)));
    assert!(ended.stdout.is_empty(), "refusal never starts ctl");
    scratch.remove();
}

/// temper/docs/design/testing/live-tier.md: terminal cleanup permits recovery, keeping imports closed.
#[test]
fn a_failed_terminal_return_allows_recovery_without_reopening_import() {
    let scratch = support::Scratch::startup("held-terminal-recovery");
    let root = scratch.path();
    let account = root.join("synthetic-record");
    let client = root.join("return-client");
    let stub = held_record_exporter(root);
    support::held_login::return_hook(Path::new("/bin/false"), root, root, &account).expect("install return hook");
    let run = || std::process::Command::new(root.join("give-back")).output().expect("return hook");
    assert!(!run().status.success(), "terminal return fails with no box up");
    assert!(!account.exists());
    support::held_login::recovery_hook(&stub, root, &client, &account).expect("install return hook");
    assert!(root.join("terminal-return").is_file(), "terminal state survives recovery setup");
    assert!(root.join("returning").is_file(), "recovery never reopens import");
    let arguments = ["account", "import", account.to_str().expect("record path"), "--key", "restart-plan"];
    let imported = support::ctl::alone(root, support::held_login::import_guard(&stub, root, &client, &arguments));
    assert_eq!(imported.exit, Some(skein_io::kernel::Exit::Code(1)));
    assert!(imported.stdout.is_empty(), "excluded import never starts ctl");
    assert!(run().status.success(), "recovery uses its fresh client to export");
    let metadata = fs::symlink_metadata(&account).expect("returned synthetic record");
    assert!(metadata.is_file() && metadata.len() > 0);
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
    let installed = support::held_login::return_hook(&stub, root, root, &account);
    assert_eq!(installed, Err("terminal return excludes another import".into()));
    scratch.remove();
}

/// temper/docs/design/testing/live-tier.md: the sitting's hook returns once.
#[test]
fn a_lent_record_returns_through_the_box_still_up_and_once() {
    let scratch = support::Scratch::startup("lent-record-hook");
    let root = scratch.path();
    let account = root.join("synthetic-record");
    let stub = root.join("ctl-stub's path");
    support::write_private(
        &stub,
        br#"#!/bin/sh
set -eu
printf '%s\n' "$4" >> seen
case "$4" in
list) printf '%s\n' '{"accounts":{"party":[{"provider":1,"plan":true,"number":7}]}}' ;;
export) test "$5" = 7; umask 077; printf '%s' synthetic > "$6" ;;
*) exit 1 ;;
esac
"#,
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable synthetic ctl");
    support::held_login::return_hook(&stub, root, root, &account).expect("install return hook");
    support::write_private(&account, b"synthetic fixture");
    fs::remove_file(&account).expect("record moved in");
    let run = || std::process::Command::new(root.join("give-back")).current_dir(root).output().expect("return hook");
    let returned = run();
    assert!(!root.join("return-client").exists(), "a box still up needs no new client");
    assert!(!root.join("box.out").exists(), "the installed hook runs before any new box");
    assert!(returned.status.success(), "{}", String::from_utf8_lossy(&returned.stderr));
    let metadata = fs::symlink_metadata(&account).expect("returned synthetic record");
    assert!(metadata.is_file() && metadata.len() > 0);
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
    assert!(run().status.success(), "the second hook call owes no export");
    assert_eq!(fs::read_to_string(root.join("seen")).expect("ctl calls"), "list\nexport\n");
    scratch.remove();
}

#[test]
fn a_worker_root_reached_through_a_link_is_refused() {
    let scratch = support::Scratch::startup("linked-root");
    let directory = scratch.path();
    support::lay_configuration(directory, &support::Forge::Unreached);
    fs::create_dir(directory.join("real-root")).expect("root directory");
    symlink(directory.join("real-root"), directory.join("workspaces")).expect("linked root");
    assert_eq!(
        temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
            .err()
            .expect("refused"),
        "temper serve: application.workers[0].root: must be a private directory of this user's, not reached through a link\n"
    );
    scratch.remove();
}

fn typed_prepare(
    directory: &Path,
    figures: &temper_playground::documents::Configuration,
) -> Result<temper_box_shell::Service, String> {
    let mut service = temper_box_shell::prepare_in_with_limits(
        "temper",
        &[],
        b"/",
        &support::configuration::directories(directory),
        &temper_service::stock::Given::Stock,
        &[],
        &figures.jig,
        &figures.limits,
    )?;
    // These tests inspect a prepared start without driving its root.
    temper_engine_shell::serve::close_limits_store(service.engine_component().store())?;
    Ok(service)
}
#[test]
fn a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant() {
    let scratch = support::Scratch::startup("box-fresh-init");
    let init = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("init")
        .arg("--box")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("init command");
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    // init's ports are fixed when free; tests run side by side, so each listener moves to a drawn one.
    support::own_listener_files(scratch.path());
    let ended = support::boxed::run(scratch.path(), None, support::boxed::Plan::Stop, support::boxed::BOX_DEADLINE);
    let diagnostic = String::from_utf8_lossy(&ended.stderr);
    let link = support::printed_link(&ended.stdout).unwrap_or_else(|| panic!("serve's link: {diagnostic}"));
    assert!(skein_lib::bytes::find(link, b"/sign-in?secret=").is_some(), "{diagnostic}");
    assert!(support::boxed::serve_listening(scratch.path()), "serve listens: {diagnostic}");
    assert!(!diagnostic.contains("application.setup"), "{diagnostic}");
    assert_eq!(ended.exit, Some(skein_io::kernel::Exit::Code(0)), "{diagnostic}");
    assert_eq!(ended.left, Some(Vec::new()), "no process outlived the box: {diagnostic}");
    // Forge 3 is asked: its group never ran, so it has no directory and nothing was fetched.
    for absent in ["forge-3", "forgejo", "forgejo-runner"] {
        assert!(!scratch.path().join(absent).exists(), "{absent}: {diagnostic}");
    }
    scratch.remove();
}

#[test]
fn a_box_adopts_an_inherited_clients_listener_without_binding_again() {
    let scratch = support::Scratch::startup("box-inherited-client");
    let init = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("init")
        .arg("--box")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("init command");
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    support::own_listener_files(scratch.path());
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("held listener");
    let address = listener.local_addr().expect("client address");
    let dirs = support::configuration::directories(scratch.path());
    let mut file = temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").file;
    file.listen = Some(address);
    support::write_private(
        &scratch.path().join(temper_config_shell::FILE),
        &temper_config_shell::render(&file, temper_config_shell::Shape::Standalone),
    );
    let mut command = listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
    command.args(support::configuration::flags(scratch.path()));
    let ended = listeners::probe(command, address, "secret=");
    assert!(ended.ready.is_some(), "box ready: {:?}", ended.stderr);
    assert!(ended.reply.expect("clients answer").starts_with(b"HTTP/1.1 "));
    assert!(ended.status.success(), "box drains: {:?}", ended.stderr);
    scratch.remove();
}

#[test]
fn a_box_refuses_an_unmatched_inherited_listener_naming_its_address() {
    let scratch = support::Scratch::startup("box-unmatched-listener");
    support::lay_configuration(scratch.path(), &support::Forge::Unreached);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("held listener");
    let address = listener.local_addr().expect("unmatched address");
    let mut command = listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
    command.args(support::configuration::flags(scratch.path()));
    let ended = command.output().expect("box refusal");
    assert_eq!(ended.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(ended.stderr).expect("diagnostic"),
        format!(
            "temper-box: {}: inherited listener fd 3 at {address}: no step names this listener\n",
            scratch.path().join(temper_config_shell::FILE).display()
        )
    );
    assert!(ended.stdout.is_empty(), "no step started");
    scratch.remove();
}

fn owned_example(case: &str) -> support::Scratch {
    let scratch = support::Scratch::startup(case);
    let owned = support::owned::Owned::example();
    support::owned::lay(scratch.path(), &owned);
    fs::write(
        scratch.path().join(temper_config_shell::FILE),
        temper_config_shell::render(&support::owned::example_file(&owned), temper_config_shell::Shape::Standalone),
    )
    .expect("example configuration");
    scratch
}
#[test]
fn the_owned_example_starts_without_credential_files() {
    let scratch = owned_example("owned-example-records");
    let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()));
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    assert!(!scratch.path().join("forge-1").exists(), "startup writes no forge files");
    scratch.remove();
}

#[test]
fn serve_prepares_once_on_the_owned_example() {
    let scratch = owned_example("owned-example-prepared");
    let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()));
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    scratch.remove();
}
/// The owned example with typed engine figures for eight places and ten workspaces.
fn eight_slots(case: &str) -> (support::Scratch, temper_playground::documents::Configuration) {
    let scratch = owned_example(case);
    let path = scratch.path().join(temper_config_shell::FILE);
    let mut document = temper_playground::documents::Configuration::from_jig(support::start_document(
        &path,
        temper_config_shell::Shape::Standalone,
    ));
    document.jig_over("hub.slots", 8);
    document.jig_over("fleet.slots", 8);
    document.temper_over("checkout.workspaces", 10);
    document.temper_over("git_component.operations", 10);
    support::limit_settings::local_places(scratch.path(), 8);
    (scratch, document)
}

#[test]
fn serve_prepares_eight_slots_and_ten_workspaces_on_the_owned_example() {
    let (scratch, document) = eight_slots("eight-slots");
    support::admission(scratch.path(), &document).expect("eight-place startup");
    assert!(document["engine"].get("limits").is_none());
    scratch.remove();
}

#[test]
fn serve_sizes_its_worker_from_eight_setting_places_not_document_slots() {
    use jig_client_protocol_wire::{CEILINGS, WorkerSetting};
    {
        let (scratch, mut document) = eight_slots("eight-setting-places");
        support::admission(scratch.path(), &document).expect("complete figures");
        let local = fs::read(scratch.path().join("local.setting")).expect("local setting");
        let local = WorkerSetting::decode(&CEILINGS, &mut skein_lib::Reader::new(&local)).expect("worker setting");
        assert_eq!(local.places, 8);
        assert_eq!(document["workers"][0]["slots"], 1, "the file carries no places");
        document.jig_over("hub.slots", 7);
        document.temper_over("checkout.workspaces", 7);
        document.temper_over("git_component.operations", 7);
        let part = temper_service::startup::box_worker_with_limits(
            &serde_json::to_vec(&document).expect("deployment JSON"),
            &jig_protocol_config::Directories::working(),
            &temper_service::stock::Given::Stock,
            &temper_service::stock::agents(),
            local.places,
            &[],
            None,
            (&document.jig, &document.limits),
        )
        .expect("engine figures")
        .expect("worker");
        assert_eq!(part.worker.jig.slots, 8);
        assert_eq!(part.worker.jig.limits.expect("applied figures").hub.slots, 8);
        assert_eq!(part.limits.checkout.workspaces, 8);
        assert_eq!(part.limits.git_component.operations, 8);
        typed_prepare(scratch.path(), &document).expect("derived capacities prepare");
        scratch.remove();
    }
}

#[test]
fn serve_prepares_once_on_a_bare_init() {
    let scratch = support::Scratch::startup("bare-init-prepared");
    let output = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("init")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("bare init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()));
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    assert!(!scratch.path().join("demo").exists(), "bare init creates no demo");
    scratch.remove();
}
#[test]
fn serve_refuses_an_explicit_call_bound_below_the_agents_call_input() {
    let scratch = owned_example("owned-call-input-bound");
    let path = scratch.path().join(temper_config_shell::FILE);
    let mut document = temper_playground::documents::Configuration::from_jig(support::start_document(
        &path,
        temper_config_shell::Shape::Standalone,
    ));
    // The owned agent declares tool_payload 63740; one below refuses, equality prepares.
    for (bound, expected) in [
        (
            63739,
            Some(concat!(
                "engine.limits.overrides.core.call_answer_bytes: 63739 bytes, short of the agent's call input (profile.declared.tool_payload), 63740 bytes; raise it to at least 63740 bytes, or omit it to follow the agent's call input\n",
                "engine.limits.overrides.core.call_answer_bytes: call input 63740 bytes; wire answer 63739 bytes; smith host_reply_bytes 63740 bytes\n",
            )),
        ),
        (63740, None),
    ] {
        document.jig_over("core.call_answer_bytes", bound);
        let prepared = support::admission(scratch.path(), &document);
        match expected {
            Some(lines) => assert_eq!(prepared.expect_err("call input refused"), lines),
            None => assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default()),
        }
    }
    scratch.remove();
}

#[test]
fn serve_checks_the_owned_agents_admission_at_each_bound() {
    let scratch = owned_example("owned-admission-bounds");
    let bytes =
        support::read_start(&scratch.path().join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone)
            .document;
    let mut figures = temper_playground::documents::Configuration::stock(serde_json::from_slice(&bytes).expect("JSON"));
    figures.jig_over("tasks.message_bytes", 10660);
    figures.jig_over("tasks.result_bytes", 10660);
    figures.jig_over("tasks.inbox_bytes", 874_120);
    figures.limits.tool_payload = 10662;
    temper_agent_fixtures::providers::remember(scratch.path(), &temper_service::stock::providers());
    let original = temper_agent_fixtures::read(scratch.path());
    for (member, value, expected) in [
        (
            "max_turns",
            7,
            Some("engine.charters[0].turns: needs 64 turns; the agent's profile.policy.max_turns allows 7\n"),
        ),
        (
            "max_turns",
            15,
            Some("engine.charters[0].turns: needs 64 turns; the agent's profile.policy.max_turns allows 15\n"),
        ),
        (
            "max_turns",
            799,
            Some("engine.charters[1].turns: needs 800 turns; the agent's profile.policy.max_turns allows 799\n"),
        ),
        ("max_turns", 800, None),
        (
            "check_timeout",
            300_001,
            Some(
                "engine.charters[1].reserve_time_ms: needs checks within 300000 ms; the agent's profile.policy.check_timeout is 300001 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n",
            ),
        ),
        ("check_timeout", 300_000, None),
        ("check_timeout", 60_000, None),
        (
            "tool_payload",
            63739,
            Some(
                "workers[0].limits.overrides.hub.event_bytes: 63738 bytes, past agents setting declared.tool_payload less the 2-byte separator of 63737\n",
            ),
        ),
        ("tool_payload", 63740, None),
        (
            "tool_payload",
            600,
            Some(
                "engine.limits: needs 10660-byte messages (tasks.message_bytes); the agent offers 598 (tool_payload less 2)\ntemper serve: workers[0].agent.profile.declared.tool_payload: 600 bytes, short of temper's documentation answer (read_docs), 2048 bytes\n",
            ),
        ),
    ] {
        let mut agent = original.clone();
        let section = if member == "tool_payload" { "declared" } else { "policy" };
        agent["profile"][section][member] = value.into();
        temper_agent_fixtures::install(scratch.path(), &agent);
        support::configuration::turn_bounds::apply(
            &mut figures,
            support::configuration::turn_bounds::turn_body(&agent),
        );
        let result = typed_prepare(scratch.path(), &figures);
        match expected {
            Some(line) if member != "tool_payload" => {
                let mut service = result.expect("charter shortfalls warn");
                assert_eq!(
                    service.notes().pop(),
                    Some(temper_service::Note::Engine(temper_service::EngineNote::StartupWarning {
                        line: line.trim_end().as_bytes().into()
                    })),
                    "{member}={value}"
                );
                assert_eq!(service.notes().pop(), None);
            }
            Some(line) => assert_eq!(
                result.err().expect("admission refused"),
                format!("temper serve: {line}"),
                "{member}={value}"
            ),
            None => assert!(result.is_ok(), "{member}={value}: {}", result.err().unwrap_or_default()),
        }
    }
    let mut agent = original.clone();
    agent["profile"]["policy"]["max_turns"] = 5.into();
    agent["profile"]["policy"]["check_timeout"] = 300_000.into();
    agent["profile"]["declared"]["tool_payload"] = 600.into();
    temper_agent_fixtures::install(scratch.path(), &agent);
    support::configuration::turn_bounds::apply(&mut figures, support::configuration::turn_bounds::turn_body(&agent));
    assert_eq!(
        typed_prepare(scratch.path(), &figures).err().expect("old tuple refused"),
        "temper serve: engine.limits: needs 10660-byte messages (tasks.message_bytes); the agent offers 598 (tool_payload less 2)\ntemper serve: workers[0].agent.profile.declared.tool_payload: 600 bytes, short of temper's documentation answer (read_docs), 2048 bytes\n"
    );
    scratch.remove();
}

fn configuration_document(value: &serde_json::Value) -> skein_json::Document {
    jig_protocol_config::document(
        &serde_json::to_vec(value).expect("configuration JSON"),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .expect("configuration document")
}

#[test]
fn the_shipped_chat_is_ended_by_its_requester() {
    let scratch = support::Scratch::startup("owned-ending-authority");
    let bounded = support::owned::bounded(scratch.path());
    let value: serde_json::Value = serde_json::from_slice(&bounded.document).expect("deployment JSON");
    let engine = temper_service::startup::engine(
        &configuration_document(&value["engine"]),
        0,
        &jig_protocol_config::Directories::working(),
    )
    .expect("engine configuration");
    assert_eq!(engine.templates[0].number, 1);
    assert_eq!(engine.templates[0].flags.ended_by, jig_domain_tasks::EndedBy::Requester);
    assert_eq!(engine.templates[1].number, 2);
    assert_eq!(engine.templates[1].flags.ended_by, jig_domain_tasks::EndedBy::Result);
    scratch.remove();
}

#[test]
fn shipped_bounds_fit_the_configured_charters_and_worker() {
    use jig_protocol_config as config;
    use temper_service as service;

    let scratch = support::Scratch::startup("owned-admission");
    let bounded = support::owned::bounded(scratch.path());
    let value: serde_json::Value = serde_json::from_slice(&bounded.document).expect("deployment JSON");
    let engine =
        temper_service::startup::engine(&configuration_document(&value["engine"]), 0, &config::Directories::working())
            .expect("engine configuration");
    let agent = &bounded.agent;
    let core = config::core(&engine, [0; 16], 1);
    let needs = jig_domain_core::settings::fleet_needs::configured(&core, &engine.limits.core);
    assert_eq!(
        jig_domain_fleet::shortfall(&needs, &agent.bounds.clone(), jig_domain_fleet::HostKind::Worker,),
        None,
        "the shipped worker is admitted by the canonical fleet check"
    );
    // The minimum Smith payload is the engine's message requirement plus
    // Smith's rendered separator; its advertised message excludes that separator.
    let payload = u64::from(engine.limits.core.tasks.message_bytes)
        .checked_add(temper_worker_domain::SEPARATOR)
        .expect("rendered message");
    assert!(agent.host_limits.message_bytes >= payload);
    let named = service::named(&bounded.document, &config::Directories::working()).expect("owned configuration");
    let mut jig = config::worker(
        &configuration_document(&value["workers"][0]),
        0,
        &config::Directories::working(),
        &temper_service::startup::fixed_worker(),
    )
    .expect("worker configuration");
    jig.limits = Some(engine.limits);
    jig.agent = Some(temper_playground::documents::pinned_agent());
    let worker = &service::settings::Worker {
        jig,
        root: named.worker.expect("standalone worker").root,
        git: Some(b"/usr/bin/git".as_slice().into()),
    };
    let own = &temper_playground::documents::Configuration::stock(value.clone()).limits;
    assert!(
        worker.jig.limits.as_ref().expect("configured worker limits").hub.event_bytes
            >= u64::from(agent.bounds.message_bytes)
    );
    assert!(u64::from(own.checkout.message_bytes) >= agent.host_limits.call_bytes);
    let prepared = service::worker_settings::prepare(worker, agent, &service::limits::worker(own))
        .expect("complete worker bounds");
    // Each receiver one byte below its sender's bound fails its own rule.
    let mut short = prepared.root.clone();
    short.host.event_bytes = u64::from(agent.bounds.message_bytes).checked_sub(1).expect("nonempty message");
    assert_eq!(temper_worker_domain::unfit(&short).as_slice(), &[temper_worker_domain::Unfit::BoundsMessage]);
    let mut short = prepared.root.clone();
    short.checkout.message_bytes = u32::try_from(agent.host_limits.call_bytes)
        .expect("call fits checkout bound")
        .checked_sub(1)
        .expect("nonempty call");
    assert_eq!(temper_worker_domain::unfit(&short).as_slice(), &[temper_worker_domain::Unfit::CallBytes]);
    scratch.remove();
}

#[test]
fn every_command_checks_setup_before_reading_files() {
    let scratch = owned_example("every-command-setup");
    support::owned::setup(
        scratch.path(),
        &support::owned::Names {
            forgejo: Path::new("/opt/forgejo"),
            runner: None,
            port: 3000,
            bot: "temper",
            person: "ada",
        },
    );
    let path = scratch.path().join(temper_config_shell::FILE);
    support::write_private(&path, b"limits = {}\n");
    let line = format!(
        "{}:1: limits: moved: limits are settings; ask temper, or use temper ctl settings apply\n",
        temper_config_shell::FILE
    );
    support::owned::refused(scratch.path(), "prepare", format!("temper-box forge prepare: forge 1: {line}").as_bytes());
    scratch.remove();
}

fn fixture_bundle(directory: &std::path::Path) {
    for (name, bytes) in BUNDLE_FILES {
        let file = directory.join("bundle").join(name);
        fs::create_dir_all(file.parent().expect("bundle parent")).expect("bundle directory");
        fs::write(file, bytes).expect("bundle file");
    }
}
fn bundle_configuration(directory: &std::path::Path) -> temper_playground::documents::Configuration {
    let figures = support::lay_configuration(directory, &support::Forge::Unreached);
    let document = directory.join(temper_config_shell::FILE);
    let mut file = support::read_file(&document, temper_config_shell::Shape::Standalone);
    file.internal.bundle = Some(directory.join("bundle"));
    support::write_file(&document, &file, temper_config_shell::Shape::Standalone);
    figures
}
#[test]
fn a_bundle_with_a_forge_sign_in_entry_starts_before_any_forge_is_kept() {
    let scratch = support::Scratch::startup("bundle-forge-sign-in");
    let directory = scratch.path();
    let figures = bundle_configuration(directory);
    fixture_bundle(directory);
    fs::write(directory.join("bundle/index.html"), b"<!doctype html><head></head><body>temper</body>")
        .expect("page with a head");
    support::forge_sign_in::configure(directory, "127.0.0.1:3000", "127.0.0.1:7100", "127.0.0.1:7200");
    support::forge_sign_in::success(&support::forge_sign_in::command(directory).output().expect("sign-in key"));
    let prepared = typed_prepare(directory, &figures);
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    scratch.remove();
}

#[test]
fn programs_directories_are_checked_at_the_name_before_startup_effects() {
    for row in ["missing", "private", "link", "mode", "file"] {
        let scratch = support::Scratch::startup(&format!("programs-{row}"));
        let directory = scratch.path();
        support::lay_configuration(directory, &support::Forge::Unreached);
        let target = directory.join("programs");
        if row != "missing" {
            if row == "file" {
                fs::write(&target, b"not a directory").expect("file");
            } else {
                fs::create_dir(&target).expect("programs directory");
                fs::set_permissions(&target, fs::Permissions::from_mode(if row == "mode" { 0o750 } else { 0o700 }))
                    .expect("directory mode");
            }
        }
        if row == "link" {
            let real = directory.join("real-programs");
            fs::rename(&target, &real).expect("move private program directory");
            symlink(&real, &target).expect("leaf link");
        }
        let path = target.clone();
        let mut agent = temper_agent_fixtures::read(directory);
        agent["programs"] = serde_json::json!({"claude_code":{"path":"/usr/bin/true", "directory":path}});
        temper_agent_fixtures::install(directory, &agent);
        // A later missing secret proves that the preflight neither creates nor inspects a login.
        fs::remove_file(directory.join("worker.secret")).expect("absent peer");
        let refused = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
            .err()
            .expect("refused");
        if ["missing", "private"].contains(&row) {
            assert!(refused.contains("workers[0].secret"), "{row}: {refused}");
            assert!(!refused.contains("programs.claude_code.directory"), "{row}: {refused}");
        } else {
            let reason = match row {
                "link" => "symbolic link",
                "mode" => "nonprivate mode bits 0o50",
                "file" => "not a directory",
                _ => unreachable!(),
            };
            assert_eq!(
                refused,
                format!("temper serve: programs.claude_code.directory: {}: {reason}\n", path.display())
            );
        }
        assert!(!directory.join("workspaces").exists(), "{row}: no worker root");
        assert!(!target.exists() || row != "missing", "missing programs stay missing");
        scratch.remove();
    }
}

#[test]
fn program_login_refuses_invalid_input_before_creation() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    for row in ["malformed", "wrong-type", "oversized", "bad-name"] {
        let scratch = support::Scratch::startup(&format!("login-{row}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        configuration(
            directory,
            (row != "no-program").then_some(programs.as_path()),
            std::path::Path::new("/usr/bin/true"),
        );
        match row {
            "malformed" => fs::write(directory.join(temper_config_shell::FILE), b"[").expect("malformed TOML"),
            "wrong-type" => fs::write(directory.join(temper_config_shell::FILE), b"owner = 42\n").expect("wrong type"),
            "oversized" => fs::File::create(directory.join(temper_config_shell::FILE))
                .expect("configuration")
                .set_len(1_048_577)
                .expect("past bound"),
            "bad-agent" => fs::write(directory.join("agent.json"), b"{}").expect("bad agent"),
            _ => {}
        }
        let why = refused(directory, if row == "bad-name" { "../escape" } else { "source" }.as_ref());
        assert!(why.starts_with("temper-box program-login: "), "{row}: {why}");
        assert_eq!(why.lines().count(), 1, "{row}: {why}");
        assert!(!why.contains("exec:"), "{row}: {why}");
        if row == "no-program" {
            assert!(why.contains("programs.claude_code: not configured"), "{why}");
        }
        if row == "oversized" {
            assert!(why.contains("past 1048576 bytes"), "{why}");
        }
        if row == "bad-name" {
            assert!(why.contains("source name must be"), "{why}");
        }
        assert!(!programs.exists());
        no_deployment_resources(directory);
        scratch.remove();
    }
}

#[test]
fn program_login_checks_both_directory_names_before_creating_anything() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    for row in
        ["program-link", "program-mode", "program-file", "source-link", "source-mode", "source-file", "source-dangling"]
    {
        let scratch = support::Scratch::startup(&format!("login-{row}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        let source = programs.join("source");
        if row.starts_with("source-") {
            fs::create_dir(&programs).expect("programs");
            fs::set_permissions(&programs, fs::Permissions::from_mode(0o700)).expect("private");
        }
        let target = if row.starts_with("source-") { &source } else { &programs };
        let expected = if row.ends_with("link") || row.ends_with("dangling") {
            symlink(directory.join("missing"), target).expect("linked leaf");
            "symbolic link"
        } else if row.ends_with("mode") {
            fs::create_dir(target).expect("directory");
            fs::set_permissions(target, fs::Permissions::from_mode(0o750)).expect("unsafe mode");
            "nonprivate mode bits 0o50"
        } else {
            fs::write(target, b"file").expect("wrong kind");
            "not a directory"
        };
        configuration(directory, Some(&programs), std::path::Path::new("/usr/bin/true"));
        let why = refused(directory, "source".as_ref());
        assert!(why.contains(expected), "{row}: {why}");
        assert!(why.contains(&target.display().to_string()), "{row}: {why}");
        assert!(!directory.join("missing").exists(), "linked peer stays absent");
        if row.starts_with("program-") {
            assert!(fs::symlink_metadata(&source).is_err(), "missing source stays absent");
        }
        no_deployment_resources(directory);
        scratch.remove();
    }
}

#[test]
fn program_login_prepares_private_directories_through_each_supported_parent() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    for row in ["missing", "private"] {
        let scratch = support::Scratch::startup(&format!("login-create-{row}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        if row == "private" {
            fs::create_dir(&programs).expect("programs");
            fs::set_permissions(&programs, fs::Permissions::from_mode(0o700)).expect("private");
        }
        let configured = if row == "linked-parent" {
            symlink(directory, directory.join("parent")).expect("linked parent");
            directory.join("parent/programs")
        } else {
            programs.clone()
        };
        let program = directory.join("claude");
        fs::write(&program, b"#!/nonexistent/interpreter\n").expect("unlaunchable executable");
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).expect("executable mode");
        configuration(directory, Some(&configured), &program);
        let why = refused(directory, "source".as_ref());
        assert!(why.starts_with("temper-box program-login: exec: "), "{row}: {why}");
        assert_eq!(why.lines().count(), 1);
        for path in [&programs, &programs.join("source")] {
            assert_eq!(fs::metadata(path).expect("created directory").permissions().mode() & 0o777, 0o700);
        }
        no_deployment_resources(directory);
        scratch.remove();
    }
}

#[test]
fn serve_box_refuses_closed_or_non_pipe_descriptors_before_starting() {
    for (inherited, fd, redirections) in [(false, 3, "3>&- 4>&-"), (true, 4, "3<&0 4>/dev/null 5>&-")] {
        let script =
            format!("exec \"$1\" internal serve --config /unused --data /unused --cache /unused --box {redirections}");
        let mut command = if inherited {
            listeners::command(std::net::TcpListener::bind("127.0.0.1:0").expect("listener"), &script)
        } else {
            let mut command = std::process::Command::new("/bin/sh");
            command
                .args([
                    "-c",
                    &script,
                    "serve-box-test",
                    skein_scratch::paths::binary("temper-box").to_str().expect("a UTF-8 path"),
                ])
                .env_clear()
                .stdin(std::process::Stdio::piped());
            command
        };
        let output = command.output().expect("serve child");
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stdout.as_slice(), b"");
        assert_eq!(
            String::from_utf8(output.stderr).expect("UTF-8 diagnostic"),
            format!("temper-box serve: --box: fd {fd} is not an open pipe\n")
        );
    }
}

#[test]
fn serve_claims_its_clients_listener_and_box_pipes_follow_it() {
    for boxed in [false, true] {
        let scratch = support::Scratch::startup(if boxed { "inherited-boxed" } else { "inherited-serve" });
        support::lay_configuration(scratch.path(), &support::Forge::Unreached);
        support::stock_agent(scratch.path());
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("held listener");
        let address = listener.local_addr().expect("address");
        let path = scratch.path().join(temper_config_shell::FILE);
        let mut file = support::read_file(&path, temper_config_shell::Shape::Standalone);
        file.listen = Some(address);
        support::write_file(&path, &file, temper_config_shell::Shape::Standalone);
        let script = if boxed { "exec \"$@\" 3<&0 4>&1 5<&2 0</dev/null 2>&1" } else { "exec \"$@\" 3<&0 0</dev/null" };
        let mut command = listeners::command(listener, script);
        if boxed {
            command.arg("internal");
        }
        command.arg("serve").args(support::configuration::flags(scratch.path()));
        let (answers, keep_answers_open) = std::io::pipe().expect("box answers");
        if boxed {
            command.arg("--box").stderr(std::process::Stdio::from(answers));
        }
        let ended = listeners::probe(command, address, "secret=");
        drop(keep_answers_open);
        assert!(ended.ready.is_some(), "serve ready: {:?} {}", ended.stdout, String::from_utf8_lossy(&ended.stderr));
        assert!(ended.reply.expect("clients answer").starts_with(b"HTTP/1.1 "));
        assert!(ended.status.success(), "serve drains: {:?} {:?}", ended.stdout, ended.stderr);
        scratch.remove();
    }
}

#[test]
fn machine_startup_leaves_unconnected_forge_handover_files_unread() {
    for row in ["absent", "exact", "over", "newline", "public", "link", "parent-link", "directory", "unreadable"] {
        let scratch = owned_example(&format!("handover-{row}"));
        let directory = scratch.path();
        let forge = directory.join("forge-1");
        fs::create_dir(&forge).expect("forge directory");
        fs::set_permissions(&forge, fs::Permissions::from_mode(0o700)).expect("handover fixture");
        let token = forge.join("admin_token");
        // Every other old file is deliberately unsafe and must stay unread.
        for name in ["api_token", "user", "git_token", "webhook_secret"] {
            fs::write(forge.join(name), b"unused").expect("handover fixture");
            fs::set_permissions(forge.join(name), fs::Permissions::from_mode(0o644)).expect("handover fixture");
        }
        match row {
            "absent" => {}
            "directory" => fs::create_dir(&token).expect("handover fixture"),
            "link" => symlink(forge.join("api_token"), &token).expect("handover fixture"),
            "parent-link" => {
                fs::rename(&forge, directory.join("actual-forge")).expect("handover fixture");
                symlink(directory.join("actual-forge"), &forge).expect("handover fixture");
                support::write_private(&token, b"test-admin");
            }
            "exact" | "over" => support::write_private(&token, &vec![b'x'; if row == "exact" { 4096 } else { 4097 }]),
            "newline" => support::write_private(&token, b"test-admin\n"),
            "public" | "unreadable" => {
                support::write_private(&token, b"test-admin");
                fs::set_permissions(&token, fs::Permissions::from_mode(if row == "public" { 0o644 } else { 0o000 }))
                    .expect("handover fixture");
            }
            _ => unreachable!(),
        }
        let answer = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory));
        assert!(answer.is_ok(), "unconnected machine leaves handover unread: {row}: {:?}", answer.err());
        if row == "unreadable" {
            fs::set_permissions(&token, fs::Permissions::from_mode(0o600)).expect("handover fixture");
        }
        scratch.remove();
    }
}
#[test]
fn the_live_github_configuration_passes_startup_admission() {
    use std::os::unix::ffi::OsStrExt;
    use support::live_github::configure_engine;

    let templates = skein_scratch::paths::package().join("tests/fixtures/github");
    let api = "127.0.0.1:31001".parse().expect("synthetic API socket");
    let git = "127.0.0.1:31002".parse().expect("synthetic git socket");
    let workers = "127.0.0.1:31003".parse().expect("synthetic worker socket");
    let owner = "fabrig-ai";
    let scratch = support::Scratch::startup(&format!("live-github-admission-{owner}"));
    let mut cluster = support::cluster::Cluster::lay_description_from(scratch.path(), &templates);
    for (name, value) in [
        ("proc.forge.ready.address", "127.0.0.1:31002"),
        ("proc.llm.ready.address", "127.0.0.1:31004"),
        ("proc.engine.ready.workers", "127.0.0.1:31003"),
        ("story.clients", "127.0.0.1:31005"),
    ] {
        cluster.bind(name, value);
    }
    cluster.prepare("engine");
    cluster.prepare("w1");
    let engine_path = cluster.dir("engine").join("config").join(temper_config_shell::FILE);
    let mut engine = temper_playground::documents::Configuration::from_jig(support::start_document(
        &engine_path,
        temper_config_shell::Shape::EngineAlone,
    ));
    configure_engine(&mut engine, owner, api, git, workers);
    let mut file = support::read_file(&engine_path, temper_config_shell::Shape::EngineAlone);
    file.workers = Some(workers);
    file.test.github = Some(temper_config_shell::TestGitHub {
        api: api.to_string(),
        transport: temper_config_shell::Transport::Plaintext,
        remote: format!("http://{git}"),
        web: format!("http://{git}"),
        git: None,
    });
    support::write_file(&engine_path, &file, temper_config_shell::Shape::EngineAlone);
    for member in ["engine", "w1"] {
        let directory = cluster.dir(member);
        let config = directory.join("config");
        let data = directory.join("data");
        let cache = directory.join("cache");
        let args: Vec<Box<[u8]>> = [
            b"--config".as_slice(),
            config.as_os_str().as_bytes(),
            b"--data",
            data.as_os_str().as_bytes(),
            b"--cache",
            cache.as_os_str().as_bytes(),
        ]
        .into_iter()
        .map(Box::from)
        .collect();
        let (directories, _) = temper_box_shell::directories::resolve(&args, &[], b"/").expect("member directories");
        let bytes = support::read_start(
            &config.join(temper_config_shell::FILE),
            if member == "engine" {
                temper_config_shell::Shape::EngineAlone
            } else {
                temper_config_shell::Shape::WorkerAlone
            },
        )
        .document;
        let operations = if member == "w1" {
            let mut part = temper_worker_service::startup::worker_part(
                &bytes,
                &temper_box_shell::directories::jig_directories(&directories),
                None,
                &[],
            )
            .expect("worker part");
            part.named.program = skein_scratch::paths::binary("temper-box").as_os_str().as_bytes().into();
            part.named.arguments = Box::new([part.named.agent.clone()]);
            let files =
                temper_service_shell::read_files(core::mem::take(&mut part.files), skein_shell::effective_user())
                    .expect("worker files");
            let mut descriptors = Vec::new();
            let service = temper_worker_shell::worker::from_figures(
                part,
                support::cluster::figures_with_limits(&engine),
                &files,
                &directories,
                &mut descriptors,
                &[],
            )
            .expect("worker figures admission");
            let operations = service.operations();
            temper_service_shell::close(descriptors);
            operations
        } else {
            crate::admission(&directories, &bytes).unwrap_or_else(|why| panic!("{owner}/{member}: {why}"))
        };
        eprintln!("{owner}/{member}: startup admission passed; operations={operations}");
    }
    scratch.remove();
}

#[test]
fn forge_run_refuses_a_listener_at_the_wrong_address_before_taking_the_lock() {
    let scratch = support::Scratch::startup("forge-wrong-listener");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
    let found = listener.local_addr().expect("address");
    support::owned::configuration(
        scratch.path(),
        &support::owned::Names {
            forgejo: std::path::Path::new("/not-run/forgejo"),
            runner: None,
            port: 9,
            bot: "temper",
            person: "ada",
        },
    );
    let mut command = listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
    command.args(["forge", "run"]).args(support::configuration::flags(scratch.path())).args(["1", "forgejo"]);
    let ended = command.output().expect("run");
    assert_eq!(ended.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(ended.stderr).expect("diagnostic"),
        format!("temper-box forge run: forge 1: fd 3 listens at {found}, expected owned.listen 127.0.0.1:9\n")
    );
    assert!(!scratch.path().join("forge-1/forgejo.lock").exists());
    scratch.remove();
}

#[test]
fn forge_run_keeps_fd_three_through_the_version_spawn_and_exec() {
    use skein_forge::forgejo::pin::VERSION;
    let scratch = support::Scratch::startup("forge-listener-exec");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let binary = scratch.path().join("forgejo");
    fs::write(
        &binary,
        format!(
            r#"#!/bin/sh
if [ "$1" = --version ]; then
    test -z "${{LISTEN_FDS+x}}" || exit 20
    test ! -S /dev/fd/3 || exit 21
    printf 'forgejo version {VERSION}\n'
    exit 0
fi
test "$LISTEN_FDS" = 1 || exit 22
test -S /dev/fd/3 || exit 23
printf 'inherited fd 3 after exec\n'
"#
        ),
    )
    .expect("fake Forgejo");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("executable");
    support::owned::configuration(
        scratch.path(),
        &support::owned::Names { forgejo: &binary, runner: None, port: address.port(), bot: "temper", person: "ada" },
    );
    support::owned::finished(scratch.path());
    let state_path = scratch.path().join("forge-1/state");
    let mut state = temper_forge_setup::state::decode(&fs::read(&state_path).expect("state")).expect("prepared");
    state.runner = None;
    support::write_private(&state_path, temper_forge_setup::state::encode(&state).expose());
    let mut command = listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
    command.args(["forge", "run"]).args(support::configuration::flags(scratch.path())).args(["1", "forgejo"]);
    let ended = command.output().expect("run");
    assert!(ended.status.success(), "{} {:?}", ended.status, String::from_utf8_lossy(&ended.stderr));
    assert_eq!(ended.stdout, b"inherited fd 3 after exec\n");
    scratch.remove();
}

#[cfg(target_os = "linux")]
#[test]
fn parent_guard_observes_its_recorded_process_after_reaping() {
    use std::process::{Command, Stdio};
    let spawn = || Command::new("/bin/cat").stdin(Stdio::piped()).stdout(Stdio::null()).spawn().expect("owned child");
    let mut guard = support::parent::Guard::new(spawn());
    let mut recorded = spawn();
    guard.record(recorded.id(), std::process::id());
    recorded.kill().expect("kill recorded child");
    recorded.wait().expect("reap recorded child");
    guard.wait_for(recorded.id());
    // Cleanup must keep using that exited identity after its PID is released.
    drop(guard);
}

#[cfg(target_os = "linux")]
#[test]
fn observing_parent_descendants_leaves_cleanup_with_their_world() {
    use std::process::{Command, Stdio};
    let mut owner = support::parent::Guard::new(
        Command::new("/bin/cat").stdin(Stdio::piped()).stdout(Stdio::null()).spawn().expect("owned child"),
    );
    let observer = support::parent::Guard::observing_test();
    assert!(observer.pids().contains(&owner.child.as_ref().expect("child").id()));
    drop(observer);
    assert!(owner.child.as_mut().expect("child").try_wait().expect("child status").is_none());
}

#[cfg(target_os = "linux")]
#[test]
fn forge_parent_listener_helper() {
    if temper_test_env::value(&temper_test_env::FORGE_PARENT_HELPER).is_some() {
        support::forge_parent::serve();
    }
}

#[cfg(target_os = "linux")]
#[test]
fn forge_run_ignores_its_childs_term_relays_the_listener_and_dies_with_its_status() {
    forge_parent_shutdown(false);
}

#[cfg(target_os = "linux")]
#[test]
fn killing_forge_run_alone_kills_its_forgejo_child() {
    forge_parent_shutdown(true);
}

#[cfg(target_os = "linux")]
fn forge_parent_shutdown(kill: bool) {
    use support::parent::{Guard, signal, wait};
    let scratch = support::Scratch::startup(&format!("forge-parent-{kill}"));
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let binary = scratch.path().join("forgejo");
    let pid_file = scratch.path().join("fake.pid");
    let release = scratch.path().join("parent.fifo");
    assert!(
        std::process::Command::new("/usr/bin/mkfifo").arg(&release).status().expect("parent release FIFO").success()
    );
    let test = std::env::current_exe().expect("startup binary");
    fs::write(
        &binary,
        format!(
            r#"#!/bin/sh
if [ "$1" = --version ]; then
    printf 'forgejo version {}\n'
    exit 0
fi
test "$LISTEN_FDS" = 1 || exit 22
test -S /proc/self/fd/3 || exit 23
export {parent_helper}=1
'{}' --exact startup::forge_parent_listener_helper --nocapture &
helper=$!
trap 'printf term > "{pid}.term"; wait "$helper"; exit 37' TERM
printf '%s' "$helper" > '{pid}.helper'
printf '%s' "$$" > '{pid}.tmp' && mv '{pid}.tmp' '{pid}'
IFS= read -r parent < '{release}' || exit 24
test "$parent" -gt 1 || exit 25
kill -TERM "$parent"
printf sent > '{pid}.sent'
wait "$helper"
exit 38
"#,
            skein_forge::forgejo::pin::VERSION,
            test.display(),
            pid = pid_file.display(),
            release = release.display(),
            parent_helper = temper_test_env::FORGE_PARENT_HELPER.name
        ),
    )
    .expect("fake Forgejo");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("executable");
    support::owned::configuration(
        scratch.path(),
        &support::owned::Names { forgejo: &binary, runner: None, port: address.port(), bot: "temper", person: "ada" },
    );
    support::owned::finished(scratch.path());
    let state_path = scratch.path().join("forge-1/state");
    let mut state = temper_forge_setup::state::decode(&fs::read(&state_path).expect("state")).expect("prepared");
    state.runner = None;
    support::write_private(&state_path, temper_forge_setup::state::encode(&state).expose());
    let mut command = listeners::command(listener, "exec \"$@\" 3<&0 0</dev/null");
    command.args(["forge", "run"]).args(support::configuration::flags(scratch.path())).args(["1", "forgejo"]);
    let mut guard = Guard::new(command.spawn().expect("forge run"));
    wait(&mut guard, |_| pid_file.exists());
    let fake = fs::read_to_string(&pid_file).expect("fake pid").parse().expect("pid");
    guard.record(fake, guard.child.as_ref().expect("parent").id());
    // Record the helper while its shell is held at the release FIFO. Send only
    // our owned parent's identity, which stays unreaped until after this TERM.
    let helper = fs::read_to_string(pid_file.with_extension("pid.helper")).expect("helper pid").parse().expect("pid");
    guard.record(helper, fake);
    fs::write(&release, format!("{}\n", guard.child.as_ref().expect("parent").id())).expect("release child's TERM");
    // The inherited listener is already bound: one connection waits for the
    // fake's response instead of polling HTTP with a separate wall clock.
    let reply = listeners::request(address).expect("inherited listener response");
    assert!(reply.starts_with(b"HTTP/1.1 200 "), "{}", String::from_utf8_lossy(&reply));
    wait(&mut guard, |_| pid_file.with_extension("pid.sent").exists());
    // Standard signals coalesce while pending. The child's TERM must have
    // reached signalfd before a separate external TERM can test forwarding.
    wait(&mut guard, |guard| support::parent::term_consumed(guard.child.as_ref().expect("parent")));
    assert!(!pid_file.with_extension("pid.term").exists(), "the child's TERM was not relayed");
    let parent = guard.child.as_mut().expect("parent");
    assert!(parent.try_wait().expect("status").is_none(), "Forgejo's SIGTERM leaves forge run running");
    eprintln!("parent story: child's TERM consumed; listener answered");
    signal(parent, if kill { "KILL" } else { "TERM" });
    if !kill {
        // The trap records the relayed TERM; the owner signals the helper by
        // pidfd so even a helper reaped early cannot turn into another target.
        wait(&mut guard, |_| pid_file.with_extension("pid.term").exists());
        eprintln!("parent story: external TERM relayed");
        guard.signal(helper, skein_io::kernel::Signal::Terminate);
    }
    eprintln!("parent story: waiting for parent exit");
    guard.child.as_mut().expect("parent").wait().expect("parent exit");
    eprintln!("parent story: parent exited");
    if kill {
        // Real Forgejo has no script helper; the guard ends this test's helper.
        guard.wait_for(fake);
    } else {
        guard.wait_all();
    }
    let ended = guard.child.take().expect("parent").wait_with_output().expect("reaped");
    if kill {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(ended.status.signal(), Some(9));
    } else {
        assert_eq!(ended.status.code(), Some(37), "{}", String::from_utf8_lossy(&ended.stderr));
        assert!(String::from_utf8_lossy(&ended.stdout).contains("fake connected"));
        assert!(String::from_utf8_lossy(&ended.stderr).contains("fake ready on fd 3"));
    }
    scratch.remove();
}

#[test]
fn forge_run_refuses_other_listener_counts_and_any_for_the_runner_before_opening_files() {
    let ended = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .args(["forge", "run", "--config", "/no-document", "--data", "/unused", "--cache", "/unused", "1", "forgejo"])
        .env_clear()
        .env("LISTEN_FDS", "0")
        .output()
        .expect("run");
    assert_eq!(ended.status.code(), Some(1));
    let diagnostic = String::from_utf8(ended.stderr).expect("diagnostic");
    assert!(diagnostic.contains("LISTEN_FDS") && diagnostic.contains("fd 3"), "{diagnostic}");
    assert!(!diagnostic.contains("no-document"), "refuses before document read");
}

#[test]
fn listener_refusals_name_the_fd_before_serve_or_a_fake_opens_files() {
    for args in [
        &["serve", "--config", "/no-document", "--data", "/unused", "--cache", "/unused"][..],
        &["fake", "llm", "--document", "/no-document"],
        &["fake", "forge", "--document", "/no-document"],
    ] {
        let ended = std::process::Command::new("/bin/sh")
            .args([
                "-c",
                "exec \"$@\" 3>/dev/null",
                "bad-listener",
                skein_scratch::paths::binary("temper-box").to_str().expect("a UTF-8 path"),
            ])
            .args(args)
            .env_clear()
            .env("LISTEN_FDS", "1")
            .output()
            .expect("run");
        assert_eq!(ended.status.code(), Some(1));
        let diagnostic = String::from_utf8(ended.stderr).expect("diagnostic");
        assert!(diagnostic.contains("fd 3 is not a socket"), "{diagnostic}");
    }
}

#[test]
fn forge_child_refuses_a_wrong_parent_or_missing_program_without_reading_a_document() {
    for (parent, program) in [(0, "/bin/true"), (std::process::id(), "/no-such-forge-child-program")] {
        let ended = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
            .args(["internal", "forge-child", "--parent", &parent.to_string(), "--signal", "kill", "--", program])
            .env_clear()
            .output()
            .expect("child refusal");
        assert_eq!(ended.status.code(), Some(1));
        assert_eq!((ended.stdout).len(), 0);
        let why = String::from_utf8(ended.stderr).expect("diagnostic");
        assert!(why.starts_with("temper internal forge-child: "), "{why}");
        assert_eq!(why.lines().count(), 1, "{why}");
        assert!(!why.contains("document"), "{why}");
    }
}

#[test]
fn a_fresh_box_offers_chatgpt_and_claude_with_no_claude_on_path() {
    let scratch = support::Scratch::startup("box-both-models");
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    // The shipped box has an empty environment. Its own init writes both documents.
    let seen = support::boxed::run_story(
        scratch.path(),
        support::configuration::flags(scratch.path())
            .into_iter()
            .map(|flag| flag.into_string().expect("flag"))
            .collect(),
        scratch.path(),
        support::person::welcome_model_offers(),
        support::boxed::BOX_DEADLINE,
        || (),
    );
    eprintln!("fresh box Model step: {} ms", clock.now().now.as_nanos().saturating_sub(began.as_nanos()) / 1_000_000);
    assert!(seen.finished, "{}", String::from_utf8_lossy(&seen.stderr));
    let agent = temper_agent_fixtures::read(scratch.path());
    assert_eq!(agent["programs"]["claude_code"]["path"], "fetched");
    scratch.remove();
}

#[cfg(all(any(target_arch = "x86_64", target_arch = "aarch64"), any(target_os = "linux", target_os = "macos")))]
#[test]
fn a_fetched_program_runs_through_its_launcher() {
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use temper_fetch::claude_code::{self, Libc};
    let scratch = support::Scratch::startup("fetched-program-launcher");
    let directory = scratch.path();
    let temper = skein_scratch::paths::binary("temper-box");
    let init = std::process::Command::new(&temper)
        .args(["init", "--box"])
        .args(support::configuration::flags(directory))
        .env_clear()
        .output()
        .unwrap();
    assert!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let agent = temper_agent_fixtures::read(directory);
    assert_eq!(agent["programs"]["claude_code"]["path"], "fetched");
    let build = claude_code::build(
        if cfg!(target_os = "macos") {
            skein_forge::forgejo::pin::Platform::Macos
        } else {
            skein_forge::forgejo::pin::Platform::Linux
        },
        std::env::consts::ARCH,
        if cfg!(target_env = "musl") { Libc::Musl } else { Libc::Gnu },
    )
    .unwrap();
    let file = directory.join("claude-code/2.1.311/claude");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, format!("#!/bin/sh\nprintf '%s\\000' \"$@\" > '{}/arguments'\nprintf '%s %s' \"$DISABLE_AUTOUPDATER\" \"$DISABLE_UPDATES\" > '{}/updater'\nprintf '%s' \"$CLAUDE_CONFIG_DIR\" > '{}/source'\nprintf '%s' \"$ANTHROPIC_CONFIG_DIR\" > '{}/anthropic-source'\nprintf 'command stdout\\n'\nexit 0\n", directory.display(), directory.display(), directory.display(), directory.display())).unwrap();
    fs::OpenOptions::new().write(true).open(&file).unwrap().set_len(build.bytes).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(directory.join("claude-code"), fs::Permissions::from_mode(0o700)).unwrap();
    let login = std::process::Command::new(&temper)
        .arg("program-login")
        .args(support::configuration::flags(directory))
        .arg("some-name")
        .env_clear()
        .output()
        .unwrap();
    assert!(login.status.success(), "{}", String::from_utf8_lossy(&login.stderr));
    assert_eq!(login.stdout, b"command stdout\n");
    assert!(!String::from_utf8_lossy(&login.stderr).contains("fetching"));
    assert_eq!(fs::read(directory.join("arguments")).unwrap(), b"auth\0login\0");
    assert_eq!(
        fs::read(directory.join("source")).unwrap(),
        directory.join("programs/some-name").as_os_str().as_bytes()
    );
    assert_eq!(
        fs::read(directory.join("anthropic-source")).unwrap(),
        directory.join("programs/some-name/anthropic").as_os_str().as_bytes()
    );
    assert_eq!(fs::read(directory.join("updater")).unwrap(), b"1 1");
    let launcher = directory.join("claude-code/claude");
    assert_eq!(fs::read_link(&launcher).unwrap(), temper);
    assert_eq!(fs::read_link(directory.join("claude-code/cache")).unwrap(), directory);
    assert_eq!(fs::metadata(launcher.parent().unwrap()).unwrap().permissions().mode() & 0o777, 0o700);
    let odd = [std::ffi::OsString::from_vec(b"one \xff\n'".to_vec()), "--two=$x".into()];
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let output = std::process::Command::new(&launcher).args(&odd).env_clear().output().unwrap();
    let elapsed_ns = clock.now().now.as_nanos().saturating_sub(began.as_nanos());
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, b"command stdout\n");
    assert_eq!((output.stderr).len(), 0);
    assert_eq!(fs::read(directory.join("arguments")).unwrap(), b"one \xff\n'\0--two=$x\0");
    assert_eq!(fs::read(directory.join("updater")).unwrap(), b"1 1");
    let began = clock.now().now;
    let direct = std::process::Command::new(&file).args(&odd).env_clear().output().unwrap();
    let direct_ns = clock.now().now.as_nanos().saturating_sub(began.as_nanos());
    assert!(direct.status.success());
    eprintln!(
        "cached launcher: {elapsed_ns} ns; direct: {direct_ns} ns; added: {} ns",
        elapsed_ns.saturating_sub(direct_ns)
    );
    assert!(!file.with_extension("part").exists());
    assert!(!file.with_extension("lock").exists(), "a cache hit opens no lock");
    scratch.remove();
}

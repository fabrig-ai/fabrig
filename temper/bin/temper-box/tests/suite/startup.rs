use crate::{listeners, support};
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};

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

#[test]
fn programs_directories_are_checked_at_the_name_before_startup_effects() {
    let scratch = support::Scratch::startup("programs-link");
    let directory = scratch.path();
    support::lay_configuration(directory, &support::Forge::Unreached);
    let target = directory.join("programs");
    let real = directory.join("real-programs");
    fs::create_dir(&real).expect("programs directory");
    fs::set_permissions(&real, fs::Permissions::from_mode(0o700)).expect("directory mode");
    symlink(&real, &target).expect("leaf link");
    let mut agent = temper_agent_fixtures::read(directory);
    agent["programs"] = serde_json::json!({"claude_code":{"path":"/usr/bin/true", "directory":target}});
    temper_agent_fixtures::install(directory, &agent);
    // The link refusal precedes the missing secret.
    fs::remove_file(directory.join("worker.secret")).expect("absent peer");
    let refused = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
        .err()
        .expect("refused");
    assert_eq!(refused, format!("temper serve: programs.claude_code.directory: {}: symbolic link\n", target.display()));
    assert!(!directory.join("workspaces").exists(), "no worker root");
    scratch.remove();
}

#[test]
fn program_login_prepares_private_directories_through_each_supported_parent() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    let scratch = support::Scratch::startup("login-create-missing");
    let directory = scratch.path();
    let programs = directory.join("programs");
    let program = directory.join("claude");
    fs::write(&program, b"#!/nonexistent/interpreter\n").expect("unlaunchable executable");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).expect("executable mode");
    configuration(directory, Some(&programs), &program);
    let why = refused(directory, "source".as_ref());
    assert!(why.starts_with("temper-box program-login: exec: "), "{why}");
    assert_eq!(why.lines().count(), 1);
    for path in [&programs, &programs.join("source")] {
        assert_eq!(fs::metadata(path).expect("created directory").permissions().mode() & 0o777, 0o700);
    }
    no_deployment_resources(directory);
    scratch.remove();
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

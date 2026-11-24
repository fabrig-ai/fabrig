use crate::support;

fn launcher(directory: &std::path::Path, argument: &str, login: Option<&std::path::Path>) -> std::process::Command {
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg(skein_scratch::paths::tree().join("scripts/proof-live.sh"))
        .arg(argument)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", directory)
        .env("TMPDIR", directory)
        .env(temper_test_env::LIVE_STORE.name, directory.join("store"))
        .env(temper_test_env::LIVE_KEEP.name, directory.join("evidence"));
    if let Some(login) = login {
        command.env(temper_test_env::LIVE_CLAUDE.name, login);
    }
    command
}

#[test]
fn the_claude_launcher_refuses_invalid_held_directories() {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };
    let scratch = support::Scratch::new("live-claude-refusals");
    let root = scratch.path();
    fs::write(root.join("file"), b"opaque").expect("file fixture");
    for (name, mode) in
        [("public", 0o755), ("empty", 0o700), ("Bad-name", 0o700), ("abcdefghijklmnopqrstuvwxyz1234567", 0o700)]
    {
        fs::create_dir(root.join(name)).expect("directory fixture");
        fs::set_permissions(root.join(name), fs::Permissions::from_mode(mode)).expect("mode");
    }
    symlink(root.join("empty"), root.join("link")).expect("link fixture");
    for (argument, login, message) in [
        ("forge-land-claude", None, "TEMPER_TEST_LIVE_CLAUDE is required for forge-land-claude"),
        ("forge-land-claude", Some("missing"), "TEMPER_TEST_LIVE_CLAUDE names no directory"),
        ("forge-land-claude", Some("file"), "TEMPER_TEST_LIVE_CLAUDE is not a directory or is a link"),
        ("forge-land-claude", Some("link"), "TEMPER_TEST_LIVE_CLAUDE is not a directory or is a link"),
        ("forge-land-claude", Some("link/"), "TEMPER_TEST_LIVE_CLAUDE is not a directory or is a link"),
        ("forge-land-claude", Some("public"), "TEMPER_TEST_LIVE_CLAUDE is not 0700"),
        ("forge-land-claude", Some("empty"), "TEMPER_TEST_LIVE_CLAUDE is empty"),
        (
            "forge-land-claude",
            Some("Bad-name"),
            "TEMPER_TEST_LIVE_CLAUDE has no valid source name (1 to 32 lowercase letters, digits or hyphens)",
        ),
        (
            "forge-land-claude",
            Some("abcdefghijklmnopqrstuvwxyz1234567"),
            "TEMPER_TEST_LIVE_CLAUDE has no valid source name (1 to 32 lowercase letters, digits or hyphens)",
        ),
        ("forge-land", Some("empty"), "TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude"),
        ("", Some("empty"), "TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude"),
    ] {
        let path = login.map(|name| root.join(name));
        let output = launcher(root, argument, path.as_deref()).output().expect("offline launcher checks");
        assert_eq!(output.status.code(), Some(2), "{argument}: {login:?}");
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), message, "{argument}: {login:?}");
    }
    scratch.remove();
}

#[test]
fn a_claude_configuration_starts_on_inits_limits() {
    assert_live_startup("live-claude-startup", configure_claude);
}

fn assert_live_startup(case: &str, configure: fn(&[std::path::PathBuf; 3])) {
    use skein_io::kernel::Exit;
    use skein_lib::Duration;
    use skein_world::end_to_end::Command;
    use support::{boxed, forge_land};
    let (scratch, paths) = initialized_deployment(case);
    configure(&paths);
    // Stop at the sign-in line: no person signs in or starts the playground.
    // There is no login and the configured executable does not exist.
    let outcome = boxed::run_command(
        Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: std::iter::once("serve".into())
                .chain(forge_land::deployment_flags(&paths).into_iter().map(Into::into))
                .collect(),
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        None,
        boxed::Plan::Stop,
        Duration::from_secs(10), // Startup and pipe drain only, within the focused suite's 30 s.
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert!(support::printed_link(&outcome.stdout).is_some(), "serve reaches its sign-in line: {stderr}");
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "serve drains: {stderr}");
    assert_eq!(outcome.left, Some(vec![]), "no surviving children: {stderr}");
    if paths[1].join("programs").exists() {
        assert!(std::fs::read_dir(paths[1].join("programs")).expect("private programs").next().is_none());
    }
    scratch.remove();
}

fn initialized_deployment(case: &str) -> (support::Scratch, [std::path::PathBuf; 3]) {
    let scratch = support::Scratch::new(case);
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    support::forge_land::initialize(&paths);
    support::own_listener_files(&paths[0]);
    (scratch, paths)
}

fn configure_claude(paths: &[std::path::PathBuf; 3]) {
    use std::{fs, os::unix::fs::DirBuilderExt, path::Path};
    fs::DirBuilder::new().mode(0o700).create(paths[1].join("programs")).expect("fresh private programs");
    support::forge_land::configure_live_model(
        &paths[0],
        support::forge_land::LiveModel::ClaudeCode { login: &paths[1].join("programs/claude-login") },
        Path::new("/usr/bin/true"),
    );
}

fn stubbed_launcher(case: &str, body: &str, source: &str) -> (support::Scratch, std::process::Output) {
    stubbed_story_launcher(case, body, source, "forge-land-claude", false)
}

fn stubbed_story_launcher(
    case: &str,
    body: &str,
    source: &str,
    story: &str,
    default_store: bool,
) -> (support::Scratch, std::process::Output) {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new(case);
    let login = scratch.path().join(source);
    fs::create_dir(&login).expect("synthetic held directory");
    fs::set_permissions(&login, fs::Permissions::from_mode(0o700)).expect("private login fixture");
    fs::create_dir(login.join("opaque-entry")).expect("listing fixture only");
    let cargo = scratch.path().join("cargo");
    fs::write(&cargo, format!("#!/bin/sh\n{body}\n")).expect("offline build stub");
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).expect("executable build stub");
    let mut command = launcher(scratch.path(), story, if story == "forge-land-claude" { Some(&login) } else { None });
    command.env("PATH", format!("{}:/usr/bin:/bin", scratch.path().display()));
    if story == "forge-land-claude-key" {
        let key = scratch.path().join("synthetic-key");
        support::write_private(&key, b"opaque fixture");
        command.env(temper_test_env::LIVE_ANTHROPIC_KEY.name, key);
    }
    if default_store {
        command.env_remove(temper_test_env::LIVE_STORE.name);
    }
    let output = command.output().expect("stubbed launcher");
    (scratch, output)
}

#[test]
fn the_launcher_returns_the_login_after_build_failure() {
    // SourceName's inclusive lower and upper lengths (protocol/credentials-llm-accounts.md, 1).
    for source in ["a", "abcdefghijklmnopqrstuvwxyz123456"] {
        let (scratch, output) = stubbed_launcher("live-claude-build-failure", "exit 17", source);
        assert_eq!(output.status.code(), Some(17));
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("CLAUDE LOGIN BACK: box stopped; same private directory, inode and owner; used in place")
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("Approve the printed ChatGPT"));
        scratch.remove();
    }
}

#[test]
fn the_launcher_returns_the_login_after_a_signal() {
    let (scratch, output) = stubbed_launcher("live-claude-signal", "kill -TERM \"$PPID\"", "claude-login");
    assert_eq!(output.status.code(), Some(143));
    assert!(String::from_utf8_lossy(&output.stdout).contains("CLAUDE LOGIN BACK:"));
    scratch.remove();
}

#[test]
fn the_launcher_refuses_changed_login_metadata_on_exit() {
    let (scratch, output) =
        stubbed_launcher("live-claude-changed", "chmod 755 \"$TEMPER_TEST_LIVE_CLAUDE\"\nexit 17", "claude-login");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("CLAUDE LOGIN NOT BACK:"));
    scratch.remove();
}

#[test]
fn the_launcher_runs_only_the_claude_story_then_returns() {
    let body = "printf '%s\\n' \"$@\" >> \"$HOME/arguments\"\nexit 0";
    let (scratch, output) = stubbed_launcher("live-claude-pass", body, "claude-login");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("CLAUDE LOGIN BACK:"));
    let arguments = std::fs::read_to_string(scratch.path().join("arguments")).expect("stub arguments");
    assert!(arguments.contains("test(=a_person_asks_claude_code_to_land_a_change_on_the_owned_forge)"));
    scratch.remove();
}

/// The launcher exports the stores' directory and keeps its story's store beneath it.
#[test]
fn the_launcher_gives_each_landing_story_its_own_store() {
    let body = "printf '%s\\n' \"$TEMPER_TEST_LIVE_STORE\" \"$@\" >> \"$HOME/seen\"\nexit 0";
    for (story, test, default_store) in [
        ("forge-land", "a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge", false),
        ("forge-land-claude", "a_person_asks_claude_code_to_land_a_change_on_the_owned_forge", false),
        ("forge-land-claude-key", "a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge", false),
        ("forge-land", "a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge", true),
    ] {
        let (scratch, output) =
            stubbed_story_launcher("live-launcher-stores", body, "claude-login", story, default_store);
        assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
        let home = std::fs::canonicalize(scratch.path()).expect("resolved home");
        let stores = home.join(if default_store { "src/rust/tmp/live-store" } else { "store" });
        let stores = stores.to_str().expect("UTF-8 stores");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.lines().any(|line| line == format!("kept store {stores}/{story}")), "{stdout}");
        if story == "forge-land-claude-key" {
            assert!(!stdout.contains("Approve the printed ChatGPT"), "{stdout}");
        }
        let seen = std::fs::read_to_string(scratch.path().join("seen")).expect("stub arguments and stores");
        assert_eq!(
            seen.lines().collect::<Vec<_>>(),
            [
                stores,
                "nextest",
                "run",
                "-p",
                "temper-box",
                "--profile",
                "live-temper",
                "--no-run",
                stores,
                "nextest",
                "run",
                "-p",
                "temper-box",
                "--profile",
                "live-temper",
                "--ignore-default-filter",
                "--run-ignored",
                "only",
                "--no-capture",
                "-E",
                &format!("binary(=live) & test(={test})"),
            ],
            "{story}, default_store={default_store}"
        );
        scratch.remove();
    }
}

/// `TEMPER_TEST_WRAPPER` names one command that runs each of the launcher's builds, cargo's
/// own arguments following it.
#[test]
fn the_launcher_runs_its_builds_through_the_wrapper_it_is_given() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("live-launcher-wrapper");
    let root = scratch.path();
    for (name, body) in [("cargo", "exit 99"), ("wrap", "printf '%s\\n' \"$@\" >> \"$HOME/seen\"\nexit 0")] {
        let stub = root.join(name);
        support::write_private(&stub, format!("#!/bin/sh\n{body}\n").as_bytes());
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable build stub");
    }
    let output = launcher(root, "forge-land", None)
        .env("PATH", format!("{}:/usr/bin:/bin", root.display()))
        .env(temper_test_env::WRAPPER.name, "wrap")
        .output()
        .expect("wrapped launcher");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let seen = fs::read_to_string(root.join("seen")).expect("wrapper arguments");
    assert_eq!(
        seen.lines().collect::<Vec<_>>(),
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "temper-box",
            "--profile",
            "live-temper",
            "--no-run",
            "cargo",
            "nextest",
            "run",
            "-p",
            "temper-box",
            "--profile",
            "live-temper",
            "--ignore-default-filter",
            "--run-ignored",
            "only",
            "--no-capture",
            "-E",
            "binary(=live) & test(=a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge)",
        ]
    );
    scratch.remove();
}

/// testing/process-tier.md: the offline driver proofs run in the process tier.
#[test]
fn the_offline_rung_runs_its_proofs_in_the_process_profile() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("offline-rung-wrapper");
    let root = scratch.path();
    let stub = root.join("wrap");
    support::write_private(
        &stub,
        b"#!/bin/sh\n[ \"$NEXTEST_DOUBLE_SPAWN\" = 0 ] || exit 98\nprintf '%s\\n' \"$@\" > \"$HOME/seen\"\nexit 0\n",
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable wrapper stub");
    let output = launcher(root, "rung-b0-offline", None)
        .env("PATH", format!("{}:/usr/bin:/bin", root.display()))
        .env(temper_test_env::WRAPPER.name, "wrap")
        .output()
        .expect("wrapped offline launcher");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let seen = fs::read_to_string(root.join("seen")).expect("wrapper arguments");
    assert_eq!(
        seen.lines().collect::<Vec<_>>(),
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "temper-box",
            "--test",
            "proof_live",
            "--profile",
            "process-temper",
            "--no-fail-fast"
        ]
    );
    scratch.remove();
}

struct HeldBox {
    scratch: support::Scratch,
    paths: [std::path::PathBuf; 3],
    record: std::path::PathBuf,
    stub: std::path::PathBuf,
    starts: usize,
    returned_before_stop: Vec<bool>,
}

impl HeldBox {
    fn new(case: &str) -> Self {
        use std::{fs, os::unix::fs::PermissionsExt};
        let scratch = support::Scratch::new(case);
        let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
        let record = scratch.path().join("record");
        support::write_private(&record, b"synthetic fixture");
        let stub = scratch.path().join("ctl");
        support::write_private(
            &stub,
            br#"#!/bin/sh
set -eu
root=$(dirname "$0")
test -e "$root/running"
test "$2" = "$(cat "$root/running")/client"
case "$4" in
import) mv "$5" "$root/inside" ;;
list) printf '%s\n' '{"accounts":{"party":[{"provider":1,"plan":true,"number":7}]}}' ;;
export) test "$5" = 7; mv "$root/inside" "$6" ;;
*) exit 1 ;;
esac
"#,
        );
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable fake ctl");
        Self { scratch, paths, record, stub, starts: 0, returned_before_stop: vec![] }
    }

    fn import(plan: &support::forge_land::LivePlan<'_>, stub: &std::path::Path, record: &std::path::Path) {
        let (commands, _) = support::forge_land::live_commands(plan.first, plan.model);
        let import = commands
            .iter()
            .find(|args| args.starts_with(&["account".into(), "import".into()]))
            .expect("held record is imported for each box");
        let words = import.iter().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(words.last(), Some(&if plan.first { "connect-plan" } else { "restart-plan" }));
        let imported = support::ctl::alone(plan.client, plan.held_import(stub, &words));
        assert_eq!(imported.exit, Some(skein_io::kernel::Exit::Code(0)));
        assert!(!record.exists(), "import moves the source into the box");
    }

    fn run(&mut self, first: bool) {
        use support::{
            forge_land::{LiveModel, LivePlan},
            held_login::{End, Ending},
        };
        let root = self.scratch.path();
        let client = root.join(if first { "first" } else { "second" });
        std::fs::create_dir(&client).expect("fake box client");
        support::write_private(&root.join("running"), client.as_os_str().as_encoded_bytes());
        self.starts += 1;
        let plan = LivePlan {
            paths: &self.paths,
            client: &client,
            first,
            kept: false,
            model: LiveModel::ChatGpt { account: Some(&self.record) },
            address: None,
            tasks: None,
            baseline: None,
            roots: None,
            words: "",
            line: "",
        };
        plan.install_return_hook(&self.stub).expect("install return hook");
        Self::import(&plan, &self.stub, &self.record);
        let now = skein_lib::Time::ZERO;
        let mut ending = Ending::new(true, first, false, now);
        let mut exit = None;
        loop {
            match ending.next(now, true, exit) {
                End::Return => {
                    exit = Some(support::ctl::alone(&client, plan.return_call()).exit);
                }
                End::Stop(failure) => {
                    use std::os::unix::fs::PermissionsExt;
                    self.returned_before_stop.push(std::fs::symlink_metadata(&self.record).is_ok_and(|file| {
                        file.is_file() && file.len() > 0 && file.permissions().mode() & 0o777 == 0o600
                    }));
                    std::fs::remove_file(root.join("running")).expect("stop fake box");
                    assert_eq!(failure, None);
                    return;
                }
                End::Wait => panic!("fake ctl already ended"),
            }
        }
    }
}

/// The generated hooks import, return and reopen the record before either box stops.
#[test]
fn a_held_login_returns_on_restart_and_failures_in_either_box() {
    let mut boxes = HeldBox::new("held-restart-endings");
    boxes.run(true);
    boxes.run(false);
    assert_eq!(boxes.returned_before_stop, [true, true]);
    assert!(boxes.record.is_file());
    boxes.scratch.remove();
}

/// A terminal give-back wins over hook installation and every later import.
#[test]
fn terminal_login_cleanup_excludes_a_restart_import() {
    use support::forge_land::{LiveModel, LivePlan};
    let mut boxes = HeldBox::new("held-terminal-return");
    boxes.run(true);
    let root = boxes.scratch.path();
    let terminal = support::ctl::alone(root, support::ctl::Call::command(&root.join("give-back"), &[]));
    assert_eq!(terminal.exit, Some(skein_io::kernel::Exit::Code(0)));
    let client = root.join("second");
    let plan = LivePlan {
        paths: &boxes.paths,
        client: &client,
        first: false,
        kept: false,
        model: LiveModel::ChatGpt { account: Some(&boxes.record) },
        address: None,
        tasks: None,
        baseline: None,
        roots: None,
        words: "",
        line: "",
    };
    let installed = plan.install_return_hook(&boxes.stub);
    assert_eq!(installed, Err("terminal return excludes another import".into()));
    let words = ["account", "import", boxes.record.to_str().expect("record path"), "--key", "restart-plan"];
    let imported = support::ctl::alone(root, plan.held_import(&boxes.stub, &words));
    assert_eq!(imported.exit, Some(skein_io::kernel::Exit::Code(1)));
    assert!(boxes.record.is_file());
    assert_eq!(boxes.starts, 1);
    boxes.scratch.remove();
}

fn held_launcher(case: &str, story: &str, body: &str) -> (support::Scratch, std::process::Output) {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new(case);
    let root = scratch.path();
    let account = root.join("synthetic-record");
    support::write_private(&account, b"synthetic fixture");
    let cargo = root.join("cargo");
    support::write_private(
        &cargo,
        format!(
            r#"#!/bin/sh
set -eu
printf '%s :: %s\n' "$TEMPER_TEST_LIVE_STORE" "$*" >> "$HOME/cargo-calls"
{body}
"#
        )
        .as_bytes(),
    );
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).expect("executable build stub");
    let output = launcher(root, story, None)
        .env(temper_test_env::LIVE_ACCOUNT.name, &account)
        .env("PATH", format!("{}:/usr/bin:/bin", root.display()))
        .output()
        .expect("held-login launcher");
    (scratch, output)
}

#[test]
fn the_launcher_returns_the_ordinary_held_login_after_build_failure() {
    for story in ["forge-land", "rung-b0"] {
        let body = r#"mkdir -p "$HOME/STORY_STORE"
printf '#!/bin/sh\ntouch "$HOME/hook-ran"\n' > "$HOME/STORY_STORE/give-back"
chmod 700 "$HOME/STORY_STORE/give-back"
exit 17"#;
        let body = body.replace("STORY_STORE", if story == "forge-land" { "store/forge-land" } else { "store" });
        let (scratch, output) = held_launcher("held-build-failure", story, &body);
        assert_eq!(output.status.code(), Some(17), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("LOGIN BACK: regular file mode=600"));
        let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
        assert_eq!(calls.lines().count(), 1, "a record at rest needs no return test");
        assert!(scratch.path().join("synthetic-record").is_file());
        assert!(!scratch.path().join("hook-ran").exists(), "a record at rest needs no hook");
        scratch.remove();
    }
}

#[test]
fn the_launcher_runs_the_ordinary_return_hook_before_stopping_after_a_signal() {
    for story in ["forge-land", "rung-b0"] {
        // The first cargo call represents the build; the second imports a
        // synthetic record and installs the same hook contract as the story.
        let body = r#"case " $* " in *" --no-run "*) exit 0 ;; esac
mkdir -p "$TEMPER_TEST_LIVE_STORE"
mv "$TEMPER_TEST_LIVE_ACCOUNT" "$TEMPER_TEST_LIVE_STORE/synthetic-import"
cat > "$TEMPER_TEST_LIVE_STORE/give-back" <<'HOOK'
#!/bin/sh
set -eu
test ! -d "/proc/$(cat "$TEMPER_TEST_LIVE_STORE/runner")"
kill -0 "$(cat "$TEMPER_TEST_LIVE_STORE/box")"
mv "$TEMPER_TEST_LIVE_STORE/synthetic-import" "$TEMPER_TEST_LIVE_ACCOUNT"
printf '%s\n' 'HOOK RETURNED'
HOOK
chmod 700 "$TEMPER_TEST_LIVE_STORE/give-back"
printf '%s\n' "$$" > "$TEMPER_TEST_LIVE_STORE/runner"
for directory in "$TMPDIR/owned" "$TEMPER_TEST_LIVE_STORE/owned"; do
    mkdir -p "$directory"
    (cd "$directory"; exec sleep 30) >/dev/null 2>&1 &
    printf '%s\n' "$!" >> "$HOME/owned-children"
    printf '%s\n' "$!" > "$TEMPER_TEST_LIVE_STORE/box"
done
kill -TERM "$PPID"
# The launcher must end and reap this PID before running the hook.
exec sleep 30"#;
        let body = body.replace(
            "$TEMPER_TEST_LIVE_STORE",
            if story == "forge-land" { "$TEMPER_TEST_LIVE_STORE/forge-land" } else { "$TEMPER_TEST_LIVE_STORE" },
        );
        let (scratch, output) = held_launcher("held-signal", story, &body);
        assert_eq!(output.status.code(), Some(143), "{}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("HOOK RETURNED"), "{stdout}");
        assert!(stdout.contains("LOGIN BACK: regular file mode=600"), "{stdout}");
        let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
        assert_eq!(calls.lines().count(), 2, "the installed hook needs no new box");
        assert_launcher_children_stopped(scratch.path());
        scratch.remove();
    }
}

fn assert_launcher_children_stopped(root: &std::path::Path) {
    let children = std::fs::read_to_string(root.join("owned-children")).expect("owned stub children");
    let mut survivors = Vec::new();
    for pid in children.lines() {
        let running = std::process::Command::new("kill")
            .args(["-0", pid])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("owned stub status")
            .success();
        if running {
            let _cleanup = std::process::Command::new("kill").args(["-KILL", pid]).status();
            survivors.push(pid);
        }
    }
    assert!(survivors.is_empty(), "the sitting stops its scratch and store descendants: {survivors:?}");
}

#[test]
fn the_launcher_refuses_changed_ordinary_held_login_metadata() {
    use std::os::unix::fs::PermissionsExt;
    for story in ["forge-land", "rung-b0"] {
        let body = r#"mkdir -p "$HOME/STORY_STORE"
printf '#!/bin/sh\ntouch "$HOME/hook-ran"\n' > "$HOME/STORY_STORE/give-back"
chmod 700 "$HOME/STORY_STORE/give-back"
chmod 644 "$TEMPER_TEST_LIVE_ACCOUNT"
exit 17"#;
        let body = body.replace("STORY_STORE", if story == "forge-land" { "store/forge-land" } else { "store" });
        let (scratch, output) = held_launcher("held-metadata", story, &body);
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("LOGIN NOT BACK:"), "{stderr}");
        assert!(!stderr.contains("synthetic-record"), "only the store is named");
        let metadata = std::fs::symlink_metadata(scratch.path().join("synthetic-record")).expect("unchanged record");
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o644);
        assert_eq!(metadata.len(), u64::try_from(b"synthetic fixture".len()).expect("fixture length"));
        assert!(!scratch.path().join("hook-ran").exists());
        let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
        assert_eq!(calls.lines().count(), 1, "unsafe metadata runs no return test");
        scratch.remove();
    }
}

#[test]
fn the_launcher_records_a_failed_ordinary_return_hook() {
    for story in ["forge-land", "rung-b0"] {
        for returns in [true, false] {
            let body = r#"case " $* " in *" --no-run "*) exit 0 ;; esac
case " $* " in
*the_held_record_returns_on_a_box_of_its_own*)
    test "$TEMPER_TEST_LIVE_STORE" = "$HOME/STORY_STORE"
    test -f "$TEMPER_TEST_LIVE_STORE/hook-ran"
    test "$TMPDIR" != "$HOME"
    mkdir -p "$TMPDIR/return-box"
    (cd "$TMPDIR/return-box"; exec sleep 30) >/dev/null 2>&1 &
    printf '%s\n' "$!" >> "$HOME/owned-children"
    RETURN
    ;;
esac
mkdir -p "$HOME/STORY_STORE"
mv "$TEMPER_TEST_LIVE_ACCOUNT" "$HOME/STORY_STORE/synthetic-import"
cat > "$HOME/STORY_STORE/give-back" <<'HOOK'
#!/bin/sh
touch "$HOME/STORY_STORE/hook-ran"
exit 9
HOOK
chmod 700 "$HOME/STORY_STORE/give-back"
for directory in "$TMPDIR/owned" "$HOME/STORY_STORE/owned"; do
    mkdir -p "$directory"
    (cd "$directory"; exec sleep 30) >/dev/null 2>&1 &
    printf '%s\n' "$!" >> "$HOME/owned-children"
done
exit 17"#;
            let story_store = if story == "forge-land" { "store/forge-land" } else { "store" };
            let body = body.replace("STORY_STORE", story_store).replace(
                "    RETURN",
                if returns {
                    "    mv \"$TEMPER_TEST_LIVE_STORE/synthetic-import\" \"$TEMPER_TEST_LIVE_ACCOUNT\"; exit 0"
                } else {
                    "    exit 19"
                },
            );
            let (scratch, output) = held_launcher("held-return-failure", story, &body);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(if returns { 17 } else { 1 }), "{stdout}\n{stderr}");
            let store = scratch.path().join(story_store);
            let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
            assert_eq!(calls.lines().count(), 3, "build, story, then the return: {calls}");
            let returned = calls.lines().nth(2).expect("return invocation");
            assert!(returned.starts_with(&format!("{} :: ", store.display())), "{returned}");
            assert!(returned.contains("--ignore-default-filter --run-ignored only"), "{returned}");
            assert!(
                returned.ends_with("binary(=live) & test(=the_held_record_returns_on_a_box_of_its_own)"),
                "{returned}"
            );
            if returns {
                assert!(stdout.contains("LOGIN BACK: regular file mode=600"), "{stdout}");
            } else {
                assert!(stderr.contains(&support::held_login::not_back("final stat failed", &store)), "{stderr}");
                assert!(
                    !stdout.contains("synthetic-record") && !stderr.contains("synthetic-record"),
                    "only the store is named"
                );
                assert!(!scratch.path().join("synthetic-record").exists(), "no record created");
                assert!(store.join("synthetic-import").is_file(), "import retained");
            }
            assert!(store.join("give-back").is_file(), "store retained for recovery");
            assert_launcher_children_stopped(scratch.path());
            scratch.remove();
        }
    }
}

#[test]
fn a_held_chatgpt_record_is_admitted_only_as_a_private_regular_file_never_named() {
    use support::forge_land::held_account;
    let scratch = support::Scratch::new("held-account");
    let file = scratch.path().join("record");
    support::write_private(&file, b"synthetic fixture");
    let link = scratch.path().join("link");
    std::os::unix::fs::symlink(&file, &link).expect("link to a private regular file");
    assert_eq!(held_account(&link), Err("TEMPER_TEST_LIVE_ACCOUNT is not a regular file"));
    scratch.remove();
}

/// testing/live-tier.md: one `TEMPER_TEST_LIVE_STORE` holds a store per landing story.
#[test]
fn each_landing_story_takes_its_own_store_under_one_live_store() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::{LiveModel, story_store};
    let scratch = support::Scratch::new("live-story-stores");
    let stores = scratch.path().join("parent/stores");
    let chatgpt = LiveModel::ChatGpt { account: None };
    let chatgpt_store = stores.join("forge-land");
    assert_eq!(story_store(&stores, chatgpt), Ok((chatgpt_store.clone(), false)));
    for directory in [stores.parent().expect("stores parent"), &stores, &chatgpt_store] {
        let metadata = fs::metadata(directory).expect("private directory");
        assert!(metadata.is_dir());
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
    }
    scratch.remove();
}

/// testing/live-tier.md: the held record's import and give-back name the sitting's
/// client directory, never the run's root.
#[test]
fn the_held_records_import_and_give_back_name_the_sittings_client_directory() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::{LiveModel, LivePlan};
    let scratch = support::Scratch::new("held-login-client");
    let store = scratch.path().join("stores/forge-land");
    let paths = ["config", "data", "cache"].map(|name| store.join(name));
    let run = store.join("first");
    let client = run.join("client");
    let held = scratch.path().join("held");
    for directory in [store.parent().expect("stores"), &store, &paths[0], &paths[1], &paths[2], &run, &client, &held] {
        fs::create_dir(directory).expect("fixture directory");
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).expect("private fixture directory");
    }
    let record = held.join("record");
    support::write_private(&record, b"synthetic fixture");
    let stub = scratch.path().join("ctl-stub's path");
    let seen = scratch.path().join("seen");
    support::write_private(
        &stub,
        br#"#!/bin/sh
set -eu
printf '%s\n' "$@" >> "$(dirname "$0")/seen"
case "$4" in
import) ;;
list) printf '%s\n' '{"accounts":{"party":[{"provider":1,"plan":true,"number":7}]}}' ;;
export) test "$5" = 7; umask 077; printf '%s' synthetic > "$6" ;;
*) exit 1 ;;
esac
"#,
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable synthetic ctl");
    let plan = LivePlan {
        paths: &paths,
        client: &run,
        first: true,
        kept: false,
        model: LiveModel::ChatGpt { account: Some(&record) },
        address: None,
        tasks: None,
        baseline: None,
        roots: None,
        words: "",
        line: "",
    };
    let arguments = ["account", "import", record.to_str().expect("record path"), "--key", "connect-plan"];
    let imported = support::ctl::alone(&run, plan.held_import(&stub, &arguments));
    assert_eq!(imported.exit, Some(skein_io::kernel::Exit::Code(0)));
    plan.install_return_hook(&stub).expect("install return hook");
    fs::remove_file(&record).expect("synthetic import removes the record");
    let returned = std::process::Command::new(store.join("give-back"))
        .stdin(std::process::Stdio::null())
        .output()
        .expect("return hook");
    assert!(returned.status.success(), "{}", String::from_utf8_lossy(&returned.stderr));
    assert_eq!(
        fs::read_to_string(&seen).expect("import and give-back arguments"),
        format!(
            "ctl\n{client}\n{import}\nctl\n{client}\naccount\nlist\n--json\nctl\n{client}\naccount\nexport\n7\n{record}\n",
            import = arguments.join("\n"),
            client = client.display(),
            record = record.display(),
        )
    );
    let returned = fs::symlink_metadata(&record).expect("returned synthetic record");
    assert!(returned.is_file() && returned.len() > 0);
    assert_eq!(returned.permissions().mode() & 0o777, 0o600);
    scratch.remove();
}

#[test]
fn an_anthropic_key_is_admitted_by_metadata_without_naming_its_path() {
    use support::forge_land::anthropic_key;
    let scratch = support::Scratch::new("live-key-admission");
    let file = scratch.path().join("synthetic-key");
    support::write_private(&file, b"opaque fixture");
    let link = scratch.path().join("link");
    std::os::unix::fs::symlink(&file, &link).expect("link to good fixture");
    assert_eq!(
        anthropic_key(Some(link.into())),
        Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link")
    );
    scratch.remove();
}

fn configure_key(paths: &[std::path::PathBuf; 3]) {
    use std::path::Path;
    support::forge_land::configure_live_model(
        &paths[0],
        support::forge_land::LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") },
        Path::new(""),
    );
}

#[test]
fn a_key_configuration_changes_only_anthropics_list_policy_and_starts() {
    assert_live_startup("live-key-startup", configure_key);
}

#[test]
fn the_key_launcher_refuses_invalid_files_and_other_held_inputs() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("live-key-launcher-refusals");
    let root = scratch.path();
    let good = root.join("good");
    support::write_private(&good, b"opaque fixture");
    support::write_private(&root.join("public"), b"opaque fixture");
    fs::set_permissions(root.join("public"), fs::Permissions::from_mode(0o644)).expect("public fixture");
    support::write_private(&root.join("empty"), b"");
    std::os::unix::fs::symlink(&good, root.join("link")).expect("link to good fixture");
    for (file, extra, message) in [
        (None, None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is required for forge-land-claude-key"),
        (Some("missing"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY names no file"),
        (Some("public"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is not 0600"),
        (Some("empty"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is empty"),
        (Some("link"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link"),
        (Some("."), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link"),
        (Some("good"), Some(temper_test_env::LIVE_ACCOUNT.name), "TEMPER_TEST_LIVE_ACCOUNT is for forge-land"),
        (Some("good"), Some(temper_test_env::LIVE_CLAUDE.name), "TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude"),
    ] {
        let mut command = launcher(root, "forge-land-claude-key", None);
        if let Some(file) = file {
            command.env(temper_test_env::LIVE_ANTHROPIC_KEY.name, root.join(file));
        }
        if let Some(extra) = extra {
            command.env(extra, &good);
        }
        let output = command.output().expect("offline key launcher refusal");
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), message);
    }
    scratch.remove();
}

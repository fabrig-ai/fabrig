use crate::{listeners, support};
use std::{
    fmt::Write,
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
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
    document["engine"]["projects"] = serde_json::json!([]);
    document["engine"]["limits"]["overrides"]["core.policy_bytes"] = 100.into();
    document["application"]["demo"] = serde_json::Value::Null;
    document["application"]["landing"] = serde_json::json!({"projects": []});
    fs::write(&path, serde_json::to_vec(&document).expect("JSON")).expect("deployment");
    let output = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("serve")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("serve");
    assert_eq!(output.status.code(), Some(1));
    // Each executor place in four policy authorities adds 32 bytes.
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "temper serve: engine.limits.overrides.core.policy_bytes: core.policy_bytes is 100; the stock's policy 1 needs 5982 bytes; raise it in engine.limits.overrides\n",
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(output.stdout.is_empty(), "no sign-in link: {}", String::from_utf8_lossy(&output.stdout));
    scratch.remove();
}

#[test]
fn a_rung_reads_created_chats_waited_changes_and_both_child_sets_in_ctls_forms() {
    use serde_json::json;
    use support::forge_land::rung::{child_rows, created_chat, plan_question, task_digest, waited};
    // The chat-start document supplies the created range; the top-level task
    // names an operation target and is null when creating a chat.
    let created = json!({"operation":"chat","task":null,"to":null,
        "outcome":{"done":{"document":{"created":{"first":7,"last":7,"given":{"task":7,"amount":207_552}}}}}});
    assert_eq!(created_chat(&created), Ok(7));
    assert!(created_chat(&json!({"task":7})).is_err(), "an operation target is not creation evidence");
    let panel = json!({"task":3,"pull":{"number":2},"head":"abc","gates":[]});
    let change = json!({"change":panel,"met":true});
    assert_eq!(waited(&["change", "show", "3"], change), Ok(panel));
    assert!(waited(&["change", "show", "3"], json!({"met":true})).is_err());
    assert!(waited(&["change", "show", "3"], json!({"met":false})).is_err());
    let task = json!({"task":{"chip":{"task":2},"digest":"abc"},"met":true});
    assert_eq!(waited(&["task", "show", "2"], task.clone()), Ok(task));
    assert_eq!(task_digest(&json!({"chip":{"task":2},"digest":"plain"})), Ok("plain"));
    assert_eq!(task_digest(&json!({"task":{"digest":"compound"},"tree":{}})), Ok("compound"));
    assert!(task_digest(&json!({"chip":{"task":2}})).is_err());
    let accounts = json!({"accounts":{"party":[]},"met":true});
    assert_eq!(waited(&["account", "list"], accounts.clone()), Ok(accounts));
    let live = json!({"tasks":{"rows":[{"chip":{"task":3}}],"more":false,"digest":"abc"},"read_on":null});
    let ended = json!({"live":[],"live_after":null,
        "ended":{"rows":{"rows":[{"chip":{"task":4}}],"after":null,"cut":false}}});
    let rows = child_rows(&live, &ended).expect("both sets");
    assert_eq!(rows, vec![json!({"chip":{"task":3}}), json!({"chip":{"task":4}})]);
    assert_eq!(child_rows(&live, &json!({"ended":"none"})).expect("no ended children").len(), 1);
    let mut cut = ended.clone();
    cut["ended"]["rows"]["cut"] = true.into();
    assert!(child_rows(&live, &cut).is_err(), "a partial set cannot prove no change");
    let mut more = live;
    more["tasks"]["more"] = true.into();
    assert!(child_rows(&more, &ended).is_err(), "an unread live child cannot disappear");
    let question = json!({"event":{"owner":{"source":{"line":{"words":{
        "kind":"question","by":{"task":2},"message":7,"words":"May I go ahead with this plan?"
    }}}}}});
    assert_eq!(plan_question(std::slice::from_ref(&question), 2, 7), Ok("May I go ahead with this plan?"));
    assert!(plan_question(std::slice::from_ref(&question), 3, 7).is_err(), "actual author, not just any question");
    assert!(plan_question(std::slice::from_ref(&question), 2, 8).is_err(), "the exact question in the person's inbox");
}

#[test]
fn a_rung_recognizes_the_exact_delivered_approval_in_explicit_message_activity() {
    use serde_json::json;
    use support::forge_land::rung::approval_at;
    let mut delivered = json!({"position":9,"at":42,"event":{"owner":{"source":{"line":{"words":{
        "message":3,"by":{"task":1},"kind":{"answer":{"question":1}},
        "words":"Go ahead. The CSV's first line must be the header fruit,count."
    }}}}}});
    assert_eq!(approval_at(std::slice::from_ref(&delivered), 1), Some(42));
    assert_eq!(approval_at(std::slice::from_ref(&delivered), 2), None, "the chat must send it");
    delivered["event"]["owner"]["source"]["line"]["words"]["words"] = "Go ahead.".into();
    assert_eq!(approval_at(&[delivered], 1), None, "the header condition must reach the work too");
}

#[test]
fn rung_b0s_live_configuration_is_admitted_without_an_account_or_backend() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = support::Scratch::startup("rung-b0-admission");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    support::forge_land::rung::deployment(&paths);
    let mut directories = support::configuration::directories(scratch.path());
    directories.config = paths[0].as_os_str().as_bytes().into();
    directories.data = paths[1].as_os_str().as_bytes().into();
    directories.cache = paths[2].as_os_str().as_bytes().into();
    let result = temper_box_shell::boxed_operations_in(
        &[],
        b"/",
        &directories,
        &fs::read(paths[0].join("temper.json")).expect("configuration"),
    );
    assert!(result.is_ok(), "B0's real configuration admits before any live account: {:?}", result.err());
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
    support::held_login::return_hook(Path::new("/must-not-export"), root, root, &account);
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
    support::held_login::return_hook(&stub, root, &root.join("return-client"), &account);
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

#[test]
fn a_held_login_import_refuses_after_returning_without_starting_ctl() {
    let scratch = support::Scratch::startup("held-login-import");
    let root = scratch.path();
    let stub = root.join("ctl-stub's path");
    support::write_private(
        &stub,
        br#"#!/bin/sh
printf '%s\n' "$@"
"#,
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable ctl stub");
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
    support::held_login::return_hook(Path::new("/bin/false"), root, root, &account);
    let run = || std::process::Command::new(root.join("give-back")).output().expect("return hook");
    assert!(!run().status.success(), "terminal return fails with no box up");
    assert!(!account.exists());
    support::held_login::recovery_hook(&stub, root, &client, &account);
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
    let installed = std::panic::catch_unwind(|| support::held_login::return_hook(&stub, root, root, &account));
    assert!(installed.is_err(), "recovery cannot admit a later sitting");
    scratch.remove();
}

/// temper/docs/design/testing/live-tier.md: stat disarms the held record's guard.
#[test]
fn a_held_record_at_its_path_owes_the_guard_nothing() {
    let scratch = support::Scratch::startup("held-record-at-rest");
    let root = scratch.path();
    let paths = ["config", "data", "cache"].map(|name| root.join(name));
    let account = root.join("synthetic-record");
    let binary = Path::new("/must-not-export");
    support::write_private(&account, b"synthetic fixture");
    support::held_login::return_hook(binary, root, root, &account);
    drop(support::held_login::Lent::new(binary, &paths, &account));
    for name in ["returning", "return-client", "box.out", "call-0.out"] {
        assert!(!root.join(name).exists(), "nothing owed: no {name}");
    }
    fs::set_permissions(&account, fs::Permissions::from_mode(0o644)).expect("unsafe mode");
    let before = fs::symlink_metadata(&account).expect("fixture metadata");
    let refused = std::panic::catch_unwind(|| drop(support::held_login::Lent::new(binary, &paths, &account)))
        .expect_err("unsafe metadata fails the test");
    let message = refused.downcast_ref::<String>().expect("guard's message");
    assert!(message.starts_with("LOGIN NOT BACK: "), "{message}");
    assert!(!message.contains(account.to_str().expect("fixture path")), "only the store is named");
    let after = fs::symlink_metadata(&account).expect("fixture left alone");
    assert_eq!(after.permissions().mode(), before.permissions().mode());
    assert_eq!(after.len(), before.len());
    for name in ["returning", "return-client", "box.out", "call-0.out"] {
        assert!(!root.join(name).exists(), "unsafe metadata runs no {name}");
    }
    scratch.remove();
}

/// temper/docs/design/testing/live-tier.md: the sitting's hook returns once.
#[test]
fn a_lent_record_returns_through_the_box_still_up_and_once() {
    let scratch = support::Scratch::startup("lent-record-hook");
    let root = scratch.path();
    let paths = ["config", "data", "cache"].map(|name| root.join(name));
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
    support::held_login::return_hook(&stub, root, root, &account);
    support::write_private(&account, b"synthetic fixture");
    fs::remove_file(&account).expect("record moved in");
    let returned = std::panic::catch_unwind(|| drop(support::held_login::Lent::new(&stub, &paths, &account)));
    assert!(!root.join("return-client").exists(), "a box still up needs no new client");
    assert!(!root.join("box.out").exists(), "the installed hook runs before any new box");
    returned.expect("the sitting's box returned the record");
    let metadata = fs::symlink_metadata(&account).expect("returned synthetic record");
    assert!(metadata.is_file() && metadata.len() > 0);
    assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
    drop(support::held_login::Lent::new(&stub, &paths, &account));
    assert_eq!(fs::read_to_string(root.join("seen")).expect("ctl calls"), "list\nexport\n");
    scratch.remove();
}

/// temper/docs/design/testing/live-tier.md: a failed return preserves the story's panic.
#[test]
fn a_lent_record_not_back_keeps_its_store_and_the_storys_panic() {
    let scratch = support::Scratch::startup("lent-record-not-back");
    let root = scratch.path();
    let paths = ["config", "data", "cache"].map(|name| root.join(name));
    let account = root.join("synthetic-record");
    let stub = root.join("temper-stub");
    support::write_private(&stub, b"#!/bin/sh\nexit 1\n");
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable failing ctl");
    support::held_login::return_hook(&stub, root, root, &account);
    let failed = std::panic::catch_unwind(|| {
        let _lent = support::held_login::Lent::new(&stub, &paths, &account);
        panic!("the story's own failure");
    })
    .expect_err("story failed");
    assert_eq!(failed.downcast_ref::<&str>(), Some(&"the story's own failure"));
    assert!(root.is_dir() && stub.is_file() && root.join("give-back").is_file(), "store retained");
    let failed = std::panic::catch_unwind(|| drop(support::held_login::Lent::new(&stub, &paths, &account)))
        .expect_err("an otherwise passing test fails when the record is not back");
    let message = failed.downcast_ref::<String>().expect("guard's message");
    assert!(message.starts_with("LOGIN NOT BACK: "), "{message}");
    assert!(message.ends_with(&format!("; store {} retained for recovery", root.display())), "{message}");
    assert!(!message.contains(account.to_str().expect("fixture path")), "only the store is named");
    assert!(root.is_dir() && stub.is_file() && root.join("give-back").is_file(), "store retained again");
    assert!(!account.exists(), "failed return creates no record");
    scratch.remove();
}

#[test]
fn a_rungs_setup_keeps_the_final_json_answer_after_progress_and_refuses_a_broken_stream() {
    use support::forge_land::rung::ctl_json;
    assert_eq!(ctl_json(b"{\"setting_up\":3}\n{\"project\":2}\n"), Ok(serde_json::json!({"project":2})));
    assert!(ctl_json(b"{\"setting_up\":3}\nnot JSON\n{\"project\":2}").is_err());
    assert!(ctl_json(b"").is_err());
}

#[test]
fn a_configuration_file_past_its_bound_is_refused() {
    let scratch = support::Scratch::startup("configuration-bound");
    let directory = scratch.path();
    fs::File::create(directory.join("temper.json"))
        .expect("configuration file")
        .set_len(1_048_577)
        .expect("one byte past bound");
    assert_eq!(
        temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
            .err()
            .expect("refused"),
        format!("temper serve: configuration {}: past 1048576 bytes\n", directory.join("temper.json").display())
    );
    scratch.remove();
}
#[test]
fn a_secret_file_readable_by_others_refuses_the_start_naming_it() {
    let scratch = support::Scratch::startup("public-token");
    let directory = scratch.path();
    support::configuration(directory, &support::Forge::Unreached);
    fs::set_permissions(directory.join("worker.secret"), fs::Permissions::from_mode(0o644))
        .expect("public permissions");
    let refused = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
        .err()
        .expect("start refused");
    assert_eq!(
        refused,
        format!(
            "temper serve: workers[0].secret: {}: readable by others or reached through a link\n",
            scratch.path().join("worker.secret").display()
        )
    );
    assert!(!refused.contains("forge-1-git-token-value"), "no secret in a diagnostic");
    scratch.remove();
}
#[test]
fn a_worker_root_reached_through_a_link_is_refused() {
    let scratch = support::Scratch::startup("linked-root");
    let directory = scratch.path();
    support::configuration(directory, &support::Forge::Unreached);
    fs::create_dir(directory.join("real-root")).expect("root directory");
    symlink(directory.join("real-root"), directory.join("worker-root")).expect("linked root");
    assert_eq!(
        temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
            .err()
            .expect("refused"),
        "temper serve: application.workers[0].root: must be a private directory of this user's, not reached through a link\n"
    );
    scratch.remove();
}
#[test]
fn problems_reach_stderr_one_line_each() {
    let scratch = support::Scratch::startup("problem-lines");
    let directory = scratch.path();
    fs::write(directory.join("temper.json"), b"{}").expect("configuration");
    assert_eq!(
        temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
            .err()
            .expect("refused"),
        "temper serve: engine: missing\ntemper serve: workers: missing\ntemper serve: application: missing\n"
    );
    scratch.remove();
}

#[test]
fn a_document_charter_is_refused_naming_temper_ctl_settings() {
    let scratch = support::Scratch::startup("stored-charters");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("configuration bytes")).expect("configuration JSON");
    document["engine"]["charters"] = serde_json::json!([]);
    support::write_private(&path, &serde_json::to_vec(&document).expect("configuration JSON"));
    let ended = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("serve")
        .args(support::configuration::flags(scratch.path()))
        .env_clear()
        .output()
        .expect("serve refusal");
    assert_eq!(ended.status.code(), Some(1));
    assert!(ended.stdout.is_empty());
    assert_eq!(
        ended.stderr,
        b"temper serve: engine.charters: settings live in the store; read them with temper ctl DIR settings show\n"
    );
    scratch.remove();
}

#[test]
fn an_inline_seed_policy_is_refused_naming_its_path() {
    let scratch = support::Scratch::startup("inline-seed-policy");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("configuration bytes")).expect("configuration JSON");
    let original: serde_json::Value =
        serde_json::from_slice(&jig_channel_kit::fixtures::engine_document()).expect("jig fixture");
    document["engine"]["projects"][0]["policy"] = original["engine"]["projects"][0]["policy"].clone();
    support::write_private(&path, &serde_json::to_vec(&document).expect("configuration JSON"));
    let ended = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("serve")
        .args(support::configuration::flags(scratch.path()))
        .env_clear()
        .output()
        .expect("serve refusal");
    assert_eq!(ended.status.code(), Some(1));
    assert!(ended.stdout.is_empty());
    assert_eq!(
        ended.stderr,
        b"temper serve: engine.projects[0].policy: name a policy, policy: 1; change it with temper ctl settings apply\n"
    );
    scratch.remove();
}

#[test]
fn an_address_with_no_token_in_the_directory_is_not_signed_in() {
    for (row, token, expected) in [
        ("absent", None, "temper open: not signed in: give the link temper serve printed\n"),
        ("empty", Some(b"".as_slice()), "temper open: not signed in: give the link temper serve printed\n"),
    ] {
        let scratch = support::Scratch::startup(&format!("open-{row}"));
        if let Some(token) = token {
            fs::write(scratch.path().join("token"), token).expect("token file");
            fs::set_permissions(scratch.path().join("token"), fs::Permissions::from_mode(0o600))
                .expect("private token");
        }
        assert_eq!(
            temper_box_shell::open::check_token(scratch.path()).expect_err("not signed in").line(scratch.path()),
            expected,
            "{row}"
        );
        scratch.remove();
    }
}

#[test]
fn a_setup_limit_past_its_check_refuses_temper_serve_naming_both_limits() {
    let scratch = support::Scratch::startup("setup-limits");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("fixture")).expect("JSON");
    value["application"]["limits"]["overrides"]["setup.backoff_ms"] = 6000.into();
    support::write_private(
        &scratch.path().join("temper.json"),
        &serde_json::to_vec(&value).expect("configuration JSON"),
    );
    assert_eq!(
        temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()))
            .err()
            .expect("configuration refused"),
        "temper serve: application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n"
    );
    scratch.remove();
}
fn box_document(scratch: &support::Scratch) -> serde_json::Value {
    support::configuration(scratch.path(), &support::Forge::Unreached);
    serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("configuration")).expect("JSON")
}
fn refused_box(scratch: &support::Scratch, value: &serde_json::Value) -> String {
    support::write_private(&scratch.path().join("temper.json"), &serde_json::to_vec(value).expect("JSON"));
    temper_box_shell::run(
        temper_box_shell::Command::Box {
            directories: support::configuration::directories(scratch.path()),
            echo: temper_box_machine::host::Echo::None,
            front: false,
            record: false,
        },
        &[],
        b"/",
        b"temper-box",
        "0.0.0",
        "0123456789ab",
    )
    .expect_err("box refused")
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
    assert!(skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some(), "{diagnostic}");
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
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
    document["application"]["clients"]["listener"] = serde_json::json!({"socket":address.to_string()});
    support::write_private(&path, &serde_json::to_vec(&document).expect("JSON"));
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
    support::configuration(scratch.path(), &support::Forge::Unreached);
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
            scratch.path().join("temper.json").display()
        )
    );
    assert!(ended.stdout.is_empty(), "no step started");
    scratch.remove();
}
#[test]
fn a_nonempty_box_configuration_without_a_document_is_refused_naming_it() {
    let scratch = support::Scratch::startup("box-absent-document");
    fs::write(scratch.path().join("other"), b"kept").expect("nonempty configuration");
    assert_eq!(
        temper_box_shell::run(
            temper_box_shell::Command::Box {
                directories: support::configuration::directories(scratch.path()),
                echo: temper_box_machine::host::Echo::None,
                front: false,
                record: false,
            },
            &[],
            b"/",
            b"temper-box",
            "0.0.0",
            "0123456789ab",
        ),
        Err(format!(
            "temper-box: configuration {}: absent; temper init writes one\n",
            scratch.path().join("temper.json").display()
        ))
    );
    scratch.remove();
}
#[test]
fn a_box_configuration_missing_members_is_refused_one_line_each() {
    let scratch = support::Scratch::startup("box-members");
    assert_eq!(
        refused_box(&scratch, &serde_json::json!({})),
        "temper-box: engine: missing\ntemper-box: workers: missing\ntemper-box: application: missing\n"
    );
    scratch.remove();
}
#[test]
fn a_setup_naming_no_forge_refuses_the_box_before_it_starts() {
    let scratch = support::Scratch::startup("box-forge");
    let mut value = box_document(&scratch);
    value["application"]["setup"] = serde_json::json!([{ "forge":7 }]);
    value["application"]["setup"][0]["forge"] = 7.into();
    assert_eq!(refused_box(&scratch, &value), "temper-box: application.setup: unknown member\n");
    assert!(!scratch.path().join("forge-7").exists());
    scratch.remove();
}
#[test]
fn a_box_maximum_backoff_below_its_base_is_refused() {
    let scratch = support::Scratch::startup("box-backoff");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 100.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper-box: application.limits.overrides.box.backoff_ms: must agree with box.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_maximum_backoff_reaching_its_window_is_refused() {
    let scratch = support::Scratch::startup("box-window");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 300_000.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper-box: application.limits.overrides.box.backoff_max_ms: must agree with box.window_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_restart_count_past_its_bound_is_refused() {
    let scratch = support::Scratch::startup("box-restarts");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 65.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper-box: application.limits.overrides.box.restarts: past its bound\n"
    );
    scratch.remove();
}
#[test]
fn a_box_restart_count_outside_its_type_is_refused() {
    let scratch = support::Scratch::startup("box-restarts-type");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 4_294_967_296_u64.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper-box: application.limits.overrides.box.restarts: outside its type\n"
    );
    scratch.remove();
}
#[test]
fn an_unknown_box_limit_is_refused() {
    let scratch = support::Scratch::startup("box-unknown");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.nope"] = 1.into();
    assert_eq!(
        refused_box(&scratch, &value),
        "temper-box: application.limits.overrides.box.nope: no limit of temper's has this name\n"
    );
    scratch.remove();
}
#[test]
fn a_box_limit_past_its_check_refuses_temper_serve_too() {
    let scratch = support::Scratch::startup("box-serve-refused");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.backoff_max_ms"] = 100.into();
    support::write_private(&scratch.path().join("temper.json"), &serde_json::to_vec(&value).expect("JSON"));
    assert_eq!(
        temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()))
            .err()
            .expect("serve refused"),
        "temper serve: application.limits.overrides.box.backoff_ms: must agree with box.backoff_max_ms\n"
    );
    scratch.remove();
}
#[test]
fn a_box_limit_within_its_check_is_accepted_by_temper_serve() {
    let scratch = support::Scratch::startup("box-serve-accepted");
    let mut value = box_document(&scratch);
    value["application"]["limits"]["overrides"]["box.restarts"] = 3.into();
    support::write_private(&scratch.path().join("temper.json"), &serde_json::to_vec(&value).expect("JSON"));
    assert!(temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path())).is_ok());
    scratch.remove();
}

fn owned_example(case: &str) -> support::Scratch {
    let scratch = support::Scratch::startup(case);
    let owned = support::owned::Owned::example();
    support::owned::lay(scratch.path(), &owned);
    fs::write(scratch.path().join("temper.json"), support::owned::example_document(scratch.path(), &owned))
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
/// The owned example, raised by what a worker of 8 slots and 10 workspaces needs.
fn eight_slots(case: &str, leave: Option<&str>) -> support::Scratch {
    let scratch = owned_example(case);
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
    document["workers"][0]["slots"] = 8.into();
    for (part, name, value) in [
        ("worker", "hub.slots", 8),
        ("engine", "fleet.slots", 8),
        ("application", "checkout.workspaces", 10),
        ("application", "git_component.operations", 10),
    ] {
        if leave != Some(name) {
            let limits = if part == "worker" { &mut document["workers"][0] } else { &mut document[part] };
            limits["limits"]["overrides"][name] = value.into();
        }
    }
    fs::write(path, serde_json::to_vec(&document).expect("JSON")).expect("deployment");
    scratch
}

#[test]
fn serve_prepares_eight_slots_and_ten_workspaces_on_the_owned_example() {
    let scratch = eight_slots("eight-slots", None);
    let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()));
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    scratch.remove();
}

#[test]
fn serve_names_what_to_raise_for_eight_slots() {
    for (name, line) in [
        (
            "hub.slots",
            "workers[0].slots: past this worker's limits.overrides.hub.slots, which must be at least its slots",
        ),
        ("fleet.slots", "workers[0].slots: must agree with engine.limits.overrides.fleet.slots, at least the slots"),
        (
            "checkout.workspaces",
            "workers[0].slots: must agree with application.limits.overrides.checkout.workspaces, at least the slots",
        ),
        (
            "git_component.operations",
            "application.limits.overrides.git_component.operations: must agree with checkout.workspaces",
        ),
    ] {
        let scratch = eight_slots(&format!("eight-slots-{name}"), Some(name));
        let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()));
        assert_eq!(prepared.err().expect("short limit refuses"), format!("temper serve: {line}\n"), "{name}");
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
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    // The owned agent declares tool_payload 10662; one below refuses, equality prepares.
    for (bound, expected) in [
        (
            10661,
            Some(concat!(
                "temper serve: workers[0].limits.overrides.core.call_answer_bytes: 10661 bytes, short of the agent's call input (profile.declared.tool_payload), 10662 bytes; raise it to at least 10662 bytes, or omit it to follow the agent's call input\n",
                "temper serve: workers[0].limits.overrides.core.call_answer_bytes: call input 10662 bytes; wire answer 10661 bytes; smith host_reply_bytes 10662 bytes\n",
            )),
        ),
        (10662, None),
    ] {
        document["workers"][0]["limits"]["overrides"]["core.call_answer_bytes"] = bound.into();
        support::write_private(&path, &serde_json::to_vec(&document).expect("configuration JSON"));
        let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()));
        match expected {
            Some(lines) => assert_eq!(prepared.err().expect("call input refused"), lines),
            None => assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default()),
        }
    }
    scratch.remove();
}

#[test]
fn serve_checks_the_owned_agents_admission_at_each_bound() {
    let scratch = owned_example("owned-admission-bounds");
    let path = scratch.path().join("owned/agent.json");
    let original: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("agent fixture")).expect("agent JSON");
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
            60001,
            Some(
                "engine.charters[1].reserve_time_ms: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 60001 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n",
            ),
        ),
        ("check_timeout", 60000, None),
        (
            "check_timeout",
            300_000,
            Some(
                "engine.charters[1].reserve_time_ms: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 300000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n",
            ),
        ),
        (
            "tool_payload",
            10661,
            Some(
                "engine.limits: needs 10660-byte messages (tasks.message_bytes); the agent offers 10659 (tool_payload less 2)\n",
            ),
        ),
        ("tool_payload", 10662, None),
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
        support::write_private(&path, &serde_json::to_vec(&agent).expect("agent JSON"));
        let result = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()));
        match expected {
            Some(line) if member != "tool_payload" => {
                let temper_box_shell::Service::Joined(mut service) = result.expect("charter shortfalls warn") else {
                    panic!("standalone service")
                };
                assert_eq!(
                    service.notes().pop(),
                    Some(temper_service::Note::StartupWarning { line: line.trim_end().as_bytes().into() }),
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
    support::write_private(&path, &serde_json::to_vec(&agent).expect("agent JSON"));
    assert_eq!(
        temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()))
            .err()
            .expect("old tuple refused"),
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
    let worker = &service::settings::Worker {
        jig: config::worker(&configuration_document(&value["workers"][0]), 0, &config::Directories::working())
            .expect("worker configuration"),
        root: named.worker.expect("standalone worker").root,
        git: Some(b"/usr/bin/git".as_slice().into()),
    };
    let mut own = service::limits::profile(service::limits::Profile::Small);
    for (name, number) in value["application"]["limits"]["overrides"].as_object().expect("overrides") {
        service::limits::apply(&mut own, name.as_bytes(), number.as_u64().expect("limit number"))
            .expect("application override");
    }
    let own = &own;
    assert!(worker.jig.limits.hub.event_bytes >= u64::from(agent.bounds.message_bytes));
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
fn the_owned_example_is_named_without_a_problem() {
    // Only naming a fixture document; no deployment or files are opened.
    let named = temper_service::named(support::owned::EXAMPLE, &jig_protocol_config::Directories::working())
        .expect("owned example named");
    assert_eq!(named.worker.as_ref().expect("standalone worker").agent.as_ref(), b"/home/me/temper/owned/agent.json");
    assert_eq!(named.worker.as_ref().expect("standalone worker").root.as_ref(), b"workspaces");
    assert_eq!(named.deployment.as_ref(), b"state");
    for path in [b"forge-1/api_token".as_slice(), b"forge-1/user", b"forge-1/webhook_secret", b"forge-1/git_token"] {
        assert_eq!(
            named.files.iter().find(|file| file.path.as_ref() == path).map(|file| file.private),
            None,
            "records-only owned input names no file: {path:?}"
        );
    }
}

#[test]
fn every_command_checks_setup_before_reading_files() {
    let scratch = owned_example("every-command-setup");
    let path = scratch.path().join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["limits"]["overrides"]["setup.backoff_ms"] = 6000.into();
    let bytes = serde_json::to_vec(&value).expect("configuration JSON");
    support::write_private(&path, &bytes);
    let line = "application.limits.overrides.setup.backoff_ms: must agree with application.limits.overrides.setup.backoff_max_ms\n";
    for (command, refused) in [
        (
            "serve",
            temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(scratch.path()))
                .err()
                .expect("serve refused"),
        ),
        (
            "temper-box",
            temper_box_shell::run(
                temper_box_shell::Command::Box {
                    directories: support::configuration::directories(scratch.path()),
                    echo: temper_box_machine::host::Echo::None,
                    front: false,
                    record: false,
                },
                &[],
                b"/",
                b"temper-box",
                "0.0.0",
                "0123456789ab",
            )
            .expect_err("box refused"),
        ),
    ] {
        assert_eq!(
            refused,
            format!("{}: {line}", if command == "serve" { "temper serve" } else { command }),
            "{command}"
        );
    }
    for verb in ["prepare", "run"] {
        support::owned::refused(scratch.path(), verb, format!("temper forge {verb}: forge 1: {line}").as_bytes());
    }
    let runner = support::owned::command(scratch.path(), "runner").output().expect("runner refuses");
    assert_eq!(runner.status.code(), Some(1));
    assert_eq!(runner.stderr, format!("temper forge run: forge 1: {line}").as_bytes());
    scratch.remove();
}

const PAGE: &[u8] = b"<!doctype html><title>t</title>";
const DIGEST: &str = "bundle/0123456789abcdef";

/// A refusal's row: its name, what spoils the bundle, and the start's refusal.
type Spoiled<'a> = (&'a str, fn(&std::path::Path), &'a str);
/// A limit's row: its name, the configuration's change, and the start's refusal.
type Limited<'a> = (&'a str, fn(&mut serde_json::Value), &'a str);

const BUNDLE_FILES: &[(&str, &[u8])] = &[
    ("index.html", PAGE),
    ("0123456789abcdef/boot.js", b"boot();"),
    ("0123456789abcdef/files/robots.txt", b"User-agent: *\nDisallow:\n"),
];
// The page, boot.js and robots.txt: three files, 31 + 7 + 24 bytes.
const BUNDLE_COUNT: u32 = 3;
const BUNDLE_BYTES: u32 = 62;
// A .js path at the service's 1024-byte bound.
const PATH_STEM_BOUND: usize = temper_service::clients::bundle::NAME_BYTES as usize - b".js".len();
const BUNDLE_LIMITS: temper_service::limits::Clients =
    temper_service::limits::Clients { bundle_files: 128, bundle_bytes: 16 << 20 };
fn fixture_bundle(directory: &std::path::Path) {
    for (name, bytes) in BUNDLE_FILES {
        let file = directory.join("bundle").join(name);
        fs::create_dir_all(file.parent().expect("bundle parent")).expect("bundle directory");
        fs::write(file, bytes).expect("bundle file");
    }
}
fn bundle_configuration(directory: &std::path::Path) {
    support::configuration(directory, &support::Forge::Unreached);
    let document = directory.join("temper.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&document).expect("configuration")).expect("JSON");
    value["application"]["clients"]["bundle"] = directory.join("bundle").to_str().expect("bundle path").into();
    support::write_private(&document, &serde_json::to_vec(&value).expect("JSON"));
}
#[test]
fn a_bundle_directory_is_read_with_its_one_digest_directory() {
    let scratch = support::Scratch::startup("bundle-read");
    let directory = scratch.path();
    fixture_bundle(directory);
    let limits = temper_service::limits::Clients { bundle_files: 128, bundle_bytes: 16 << 20 };
    let bundle = temper_box_shell::bundle::read(directory.join("bundle").as_os_str().as_encoded_bytes(), limits)
        .expect("bundle read");
    assert_eq!(bundle.page.as_ref(), PAGE);
    assert_eq!(bundle.etag.as_ref(), b"\"edaa28e315690734\"");
    assert_eq!(bundle.digest.as_ref(), b"0123456789abcdef");
    assert_eq!(bundle.files.len(), 2);
    let script = bundle.files.get(b"boot.js".as_slice()).expect("script");
    assert_eq!((script.kind, script.bytes.as_ref()), (b"text/javascript".as_slice(), b"boot();".as_slice()));
    let robots = bundle.files.get(b"files/robots.txt".as_slice()).expect("root file");
    assert_eq!(
        (robots.kind, robots.bytes.as_ref()),
        (b"text/plain; charset=utf-8".as_slice(), b"User-agent: *\nDisallow:\n".as_slice())
    );
    scratch.remove();
}
#[test]
fn embedded_and_directory_files_make_the_same_bundle() {
    let scratch = support::Scratch::startup("bundle-embedded");
    fixture_bundle(scratch.path());
    // Root files beside index.html are ignored in both reads.
    fs::write(scratch.path().join("bundle/ignored.bin"), b"ignored").expect("root file");
    let mut files = BUNDLE_FILES.to_vec();
    files.push(("ignored.bin", b"ignored"));
    let directory =
        temper_box_shell::bundle::read(scratch.path().join("bundle").as_os_str().as_encoded_bytes(), BUNDLE_LIMITS)
            .expect("directory");
    let embedded = temper_box_shell::bundle::embedded(&files, BUNDLE_LIMITS).expect("embedded");
    assert_eq!(embedded.page, directory.page);
    assert_eq!(embedded.etag, directory.etag);
    assert_eq!(embedded.digest, directory.digest);
    assert_eq!(embedded.files.len(), directory.files.len());
    for (name, _) in BUNDLE_FILES.iter().skip(1) {
        let (_, relative) = name.split_once('/').expect("digest path");
        let from_directory = directory.files.get(relative.as_bytes()).expect("directory file");
        let from_embedded = embedded.files.get(relative.as_bytes()).expect("embedded file");
        assert_eq!(from_embedded.kind, from_directory.kind);
        assert_eq!(from_embedded.bytes, from_directory.bytes);
    }
    scratch.remove();
}

// Four compressible files and their forms, a root file, and the page: 10 files, 73 bytes.
const FORMED_COUNT: u32 = 10;
const FORMED_BYTES: u32 = 73;
const FORMED_FILES: &[(&str, &[u8])] = &[
    ("index.html", PAGE),
    // Forms deliberately precede files in the embedded input.
    ("0123456789abcdef/boot.js.gz", b"gz js"),
    ("0123456789abcdef/a/module.wasm.gz", b"gz wasm"),
    ("0123456789abcdef/a/style.css.gz", b"gz css"),
    ("0123456789abcdef/image.svg.gz", b"gz svg"),
    ("0123456789abcdef/boot.js", b"js"),
    ("0123456789abcdef/a/module.wasm", b"wasm"),
    ("0123456789abcdef/a/style.css", b"css"),
    ("0123456789abcdef/image.svg", b"svg"),
    ("0123456789abcdef/files/robots.txt", b"robots"),
];
fn formed_reads(
    case: &str,
    files: &[(&str, &[u8])],
    limits: temper_service::limits::Clients,
) -> [Result<temper_service::clients::bundle::Bundle, String>; 2] {
    let scratch = support::Scratch::startup(case);
    for (name, bytes) in files {
        let path = scratch.path().join("bundle").join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        fs::write(path, bytes).expect("file");
    }
    let directory =
        temper_box_shell::bundle::read(scratch.path().join("bundle").as_os_str().as_encoded_bytes(), limits);
    let embedded = temper_box_shell::bundle::embedded(files, limits);
    scratch.remove();
    [directory, embedded]
}
#[test]
fn a_bundle_reads_each_form_onto_its_file() {
    let limits = temper_service::limits::Clients { bundle_files: FORMED_COUNT, bundle_bytes: FORMED_BYTES };
    for read in formed_reads("bundle-read-forms", FORMED_FILES, limits) {
        let bundle = read.expect("both forms read at their bounds");
        assert_eq!(bundle.files.len(), 5);
        for (name, plain, gzip) in [
            (b"boot.js".as_slice(), b"js".as_slice(), b"gz js".as_slice()),
            (b"a/module.wasm", b"wasm", b"gz wasm"),
            (b"a/style.css", b"css", b"gz css"),
            (b"image.svg", b"svg", b"gz svg"),
        ] {
            let file = bundle.files.get(name).expect("plain file");
            assert_eq!(file.bytes.as_ref(), plain);
            assert_eq!(file.gzip.as_deref(), Some(gzip));
        }
        assert_eq!(bundle.files.get(b"files/robots.txt".as_slice()).expect("root file").gzip, None);
        assert!(bundle.files.get(b"boot.js.gz".as_slice()).is_none());
    }
}
#[test]
fn the_compressed_forms_count_against_the_bounds() {
    use temper_service::limits::Clients;
    for (limits, why) in [
        (Clients { bundle_files: FORMED_COUNT - 1, ..BUNDLE_LIMITS }, "past clients.bundle_files"),
        (Clients { bundle_bytes: FORMED_BYTES - 1, ..BUNDLE_LIMITS }, "past clients.bundle_bytes"),
    ] {
        for read in formed_reads("bundle-form-limits", FORMED_FILES, limits) {
            assert!(read.expect_err("form exceeds bound").contains(why));
        }
    }
}
#[test]
fn a_compressed_form_with_no_file_refuses_the_start() {
    let files = [("index.html", PAGE), ("0123456789abcdef/absent.js.gz", b"gz".as_slice())];
    for read in formed_reads("bundle-orphan-form", &files, BUNDLE_LIMITS) {
        assert!(read.expect_err("orphan form").contains("a compressed form with no file"));
    }
}
#[test]
fn a_gz_of_another_type_refuses_the_start() {
    let files = [("index.html", PAGE), ("0123456789abcdef/x.txt.gz", b"gz".as_slice())];
    for read in formed_reads("bundle-form-type", &files, BUNDLE_LIMITS) {
        assert!(read.expect_err("type off closed list").contains("file type is not on the closed list"));
    }
}

#[test]
fn embedded_files_refuse_each_spoiled_bundle_and_limit() {
    use temper_service::limits::Clients;
    let long = format!("0123456789abcdef/{}.js", "a".repeat(PATH_STEM_BOUND + 1)); // 1025 bytes below the digest.
    let rows = [
        ("no page", BUNDLE_FILES[1..].to_vec(), BUNDLE_LIMITS, "index.html: absent".to_owned()),
        ("no digest", BUNDLE_FILES[..1].to_vec(), BUNDLE_LIMITS, "no digest directory".to_owned()),
        (
            "two digests",
            vec![("index.html", PAGE), ("0123456789abcdef/a.js", b""), ("fedcba9876543210/a.js", b"")],
            BUNDLE_LIMITS,
            "fedcba9876543210: more than one digest directory".to_owned(),
        ),
        (
            "bad digest",
            vec![("index.html", PAGE), ("0123456789abcdeF/a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdeF: digest must be 16 lowercase hex digits".to_owned(),
        ),
        (
            "short digest",
            vec![("index.html", PAGE), ("abc/a.js", b"")],
            BUNDLE_LIMITS,
            "abc: digest must be 16 lowercase hex digits".to_owned(),
        ),
        (
            "parent path",
            vec![("index.html", PAGE), ("0123456789abcdef/../a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/../a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "dot path",
            vec![("index.html", PAGE), ("0123456789abcdef/./a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/./a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "empty component",
            vec![("index.html", PAGE), ("0123456789abcdef//a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef//a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "long path",
            vec![("index.html", PAGE), (long.as_str(), b"")],
            BUNDLE_LIMITS,
            format!("{long}: path must be plain and at most 1024 bytes"),
        ),
        (
            "closed list",
            vec![("index.html", PAGE), ("0123456789abcdef/data.bin", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/data.bin: file type is not on the closed list".to_owned(),
        ),
        (
            "files",
            BUNDLE_FILES.to_vec(),
            Clients { bundle_files: BUNDLE_COUNT - 1, ..BUNDLE_LIMITS },
            "0123456789abcdef/files/robots.txt: past clients.bundle_files".to_owned(),
        ),
        (
            "bytes",
            BUNDLE_FILES.to_vec(),
            Clients { bundle_bytes: BUNDLE_BYTES - 1, ..BUNDLE_LIMITS },
            "0123456789abcdef/files/robots.txt: past clients.bundle_bytes".to_owned(),
        ),
    ];
    for (name, files, limits, why) in rows {
        assert_eq!(
            temper_box_shell::bundle::embedded(&files, limits).expect_err("refused"),
            format!("application.clients.bundle: the binary's bundle: {why}"),
            "{name}"
        );
    }
}

#[test]
fn embedded_files_at_each_bound_are_admitted() {
    let bound = format!("0123456789abcdef/{}.js", "a".repeat(PATH_STEM_BOUND));
    for (name, files, limits) in [
        (
            "files and bytes",
            BUNDLE_FILES.to_vec(),
            temper_service::limits::Clients { bundle_files: BUNDLE_COUNT, bundle_bytes: BUNDLE_BYTES },
        ),
        ("path", vec![("index.html", PAGE), (bound.as_str(), b"")], BUNDLE_LIMITS),
    ] {
        temper_box_shell::bundle::embedded(&files, limits).unwrap_or_else(|why| panic!("{name}: {why}"));
    }
}

#[test]
fn a_bundle_with_a_forge_sign_in_entry_starts_before_any_forge_is_kept() {
    let scratch = support::Scratch::startup("bundle-forge-sign-in");
    let directory = scratch.path();
    bundle_configuration(directory);
    fixture_bundle(directory);
    fs::write(directory.join("bundle/index.html"), b"<!doctype html><head></head><body>temper</body>")
        .expect("page with a head");
    support::forge_sign_in::configure(directory, "127.0.0.1:3000", "127.0.0.1:7100", "127.0.0.1:7200");
    support::forge_sign_in::success(&support::forge_sign_in::command(directory).output().expect("sign-in key"));
    let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory));
    assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
    scratch.remove();
}

#[test]
fn a_bundle_page_without_a_head_refuses_the_start_for_its_metadata() {
    let scratch = support::Scratch::startup("bundle-no-head");
    let directory = scratch.path();
    bundle_configuration(directory);
    fixture_bundle(directory);
    let why = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
        .err()
        .expect("page refused");
    assert_eq!(
        why,
        format!(
            "temper serve: application.clients.bundle: {}/bundle: the page has no </head> for its sign-in metadata\n",
            directory.display()
        )
    );
    scratch.remove();
}

#[test]
fn a_directory_that_is_no_bundle_refuses_the_start() {
    let rows: [Spoiled<'_>; 4] = [
        (
            "no-page",
            |directory| fs::remove_file(directory.join("bundle/index.html")).expect("page removed"),
            "temper serve: application.clients.bundle: bundle/index.html: absent\n",
        ),
        (
            "no-digest",
            |directory| fs::remove_dir_all(directory.join(DIGEST)).expect("digest removed"),
            "temper serve: application.clients.bundle: bundle: no digest directory\n",
        ),
        (
            "two-digests",
            |directory| fs::create_dir(directory.join("bundle/fedcba9876543210")).expect("second digest"),
            "temper serve: application.clients.bundle: bundle/fedcba9876543210: more than one digest directory\n",
        ),
        (
            "off-the-list",
            |directory| fs::write(directory.join(DIGEST).join("files/data.bin"), b"x").expect("odd file"),
            "temper serve: application.clients.bundle: bundle/0123456789abcdef/files/data.bin: file type is not on the closed list\n",
        ),
    ];
    for (name, spoil, expected) in rows {
        let scratch = support::Scratch::startup(&format!("bundle-{name}"));
        let directory = scratch.path();
        bundle_configuration(directory);
        fixture_bundle(directory);
        spoil(directory);
        assert_eq!(
            temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
                .err()
                .expect("refused"),
            expected.replace(": bundle", &format!(": {}/bundle", directory.display())),
            "row {name}"
        );
        scratch.remove();
    }
}
#[test]
fn a_bundle_past_a_limit_refuses_the_start() {
    let rows: [Limited<'_>; 2] = [
        (
            "files",
            |value| value["application"]["limits"]["overrides"]["clients.bundle_files"] = 2.into(),
            "temper serve: application.clients.bundle: bundle/0123456789abcdef/files/robots.txt: past clients.bundle_files\n",
        ),
        (
            "bytes",
            |value| value["application"]["limits"]["overrides"]["clients.bundle_bytes"] = 61.into(),
            "temper serve: application.clients.bundle: bundle/0123456789abcdef/files/robots.txt: past clients.bundle_bytes\n",
        ),
    ];
    for (name, limit, expected) in rows {
        let scratch = support::Scratch::startup(&format!("bundle-limit-{name}"));
        let directory = scratch.path();
        bundle_configuration(directory);
        fixture_bundle(directory);
        support::owned::configuration_value(directory, limit);
        assert_eq!(
            temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
                .err()
                .expect("refused"),
            expected.replace(": bundle", &format!(": {}/bundle", directory.display())),
            "row {name}"
        );
        scratch.remove();
    }
}

#[test]
fn programs_directories_are_checked_at_the_name_before_startup_effects() {
    for row in ["missing", "private", "link", "linked-parent", "mode", "file"] {
        let scratch = support::Scratch::startup(&format!("programs-{row}"));
        let directory = scratch.path();
        support::configuration(directory, &support::Forge::Unreached);
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
        let path = match row {
            "link" => {
                let link = directory.join("linked");
                symlink(&target, &link).expect("leaf link");
                link
            }
            "linked-parent" => {
                let parent = directory.join("parent");
                symlink(directory, &parent).expect("parent link");
                parent.join("programs")
            }
            _ => target.clone(),
        };
        let agent_path = directory.join("agent.json");
        let mut agent: serde_json::Value =
            serde_json::from_slice(&fs::read(&agent_path).expect("agent")).expect("JSON");
        agent["programs"] = serde_json::json!({"claude_code":{"path":"/nonexistent/claude", "directory":path}});
        fs::write(agent_path, serde_json::to_vec(&agent).expect("agent JSON")).expect("agent");
        // A later missing secret proves that the preflight neither creates nor inspects a login.
        fs::remove_file(directory.join("worker.secret")).expect("absent peer");
        let refused = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory))
            .err()
            .expect("refused");
        if ["missing", "private", "linked-parent"].contains(&row) {
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
                format!(
                    "temper serve: workers[0].agent.configuration: programs.claude_code.directory: {}: {reason}\n",
                    path.display()
                )
            );
        }
        assert!(!directory.join("worker-root").exists(), "{row}: no worker root");
        assert!(!target.exists() || row != "missing", "missing programs stay missing");
        scratch.remove();
    }
}

#[test]
fn program_login_refuses_invalid_input_before_creation() {
    use support::program_login::{configuration, no_deployment_resources, refused};
    for row in ["no-program", "malformed", "missing-members", "oversized", "bad-agent", "bad-name"] {
        let scratch = support::Scratch::startup(&format!("login-{row}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        configuration(
            directory,
            (row != "no-program").then_some(programs.as_path()),
            std::path::Path::new("/nonexistent/claude"),
        );
        match row {
            "malformed" => fs::write(directory.join("temper.json"), b"{").expect("malformed JSON"),
            "missing-members" => fs::write(directory.join("temper.json"), b"{}").expect("missing members"),
            "oversized" => fs::File::create(directory.join("temper.json"))
                .expect("configuration")
                .set_len(1_048_577)
                .expect("past bound"),
            "bad-agent" => fs::write(directory.join("agent.json"), b"{}").expect("bad agent"),
            _ => {}
        }
        let why = refused(directory, if row == "bad-name" { "../escape" } else { "source" }.as_ref());
        assert!(why.starts_with("temper program-login: "), "{row}: {why}");
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
        configuration(directory, Some(&programs), std::path::Path::new("/nonexistent/claude"));
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
    for row in ["missing", "private", "linked-parent"] {
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
        configuration(directory, Some(&configured), std::path::Path::new("/nonexistent/claude"));
        let why = refused(directory, "source".as_ref());
        assert!(why.starts_with("temper program-login: exec: "), "{row}: {why}");
        assert_eq!(why.lines().count(), 1);
        for path in [&programs, &programs.join("source")] {
            assert_eq!(fs::metadata(path).expect("created directory").permissions().mode() & 0o777, 0o700);
        }
        no_deployment_resources(directory);
        scratch.remove();
    }
}

#[test]
fn init_prints_written_files_account_guidance_and_explicit_start_flags_on_stderr() {
    const NO_GIT: &str =
        "init: no git on PATH: a real forge's repositories need git 2.41 or later, set as workers[].git";
    for (has_git, boxed) in [(true, false), (false, false), (true, true), (false, true)] {
        let scratch = support::Scratch::startup(&format!("init-command-{has_git}-{boxed}"));
        let bin = scratch.path().join("bin");
        fs::create_dir(&bin).expect("bin");
        let git = bin.join("git");
        if has_git {
            fs::write(&git, b"#!/bin/sh\nexit 0\n").expect("git");
            fs::set_permissions(&git, fs::Permissions::from_mode(0o755)).expect("executable");
        }
        let output = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
            .arg("init")
            .args(if boxed { vec!["--box"] } else { vec![] })
            .args(support::configuration::flags(scratch.path()))
            .env_clear()
            .env("PATH", &bin)
            .output()
            .expect("init command");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty());
        let lines = String::from_utf8(output.stderr).expect("init lines");
        let mut expected = String::new();
        let names = if boxed {
            vec!["worker.secret", "worker.digest", "demo/llm.key", "demo/bin/git", "agent.json", "temper.json"]
        } else {
            vec!["worker.secret", "worker.digest", "agent.json", "temper.json"]
        };
        for name in names {
            writeln!(expected, "init: wrote {}", scratch.path().join(name).display()).expect("string");
        }
        assert!(lines.starts_with(&expected), "files are reported in write order");
        assert!(
            lines.contains(
                "init: after starting, continue with ChatGPT or add an OpenAI API key from the accounts page\n"
            )
        );
        let path = scratch.path().display();
        let command = if boxed { "box" } else { "serve" };
        assert!(lines.ends_with(&format!(
            "init: start it with temper {command} --config {path} --data {path} --cache {path}\n"
        )));

        let diagnostics: Vec<_> = lines.lines().filter(|line| line.starts_with("init: no git")).collect();
        assert_eq!(diagnostics, if has_git { vec![] } else { vec![NO_GIT] });
        let document: serde_json::Value =
            serde_json::from_slice(&fs::read(scratch.path().join("temper.json")).expect("configuration"))
                .expect("JSON");
        assert_eq!(document["application"]["workers"][0]["git"].is_null(), !has_git);
        assert_eq!(document["application"]["welcome"].as_bool(), if boxed { Some(true) } else { None });
        assert_eq!(scratch.path().join("demo").exists(), boxed);
        if !boxed {
            assert!(!lines.contains("person is person"), "bare init has no person diagnostic");
        }
        scratch.remove();
    }
}

#[test]
fn box_initializes_only_absent_or_empty_configuration_directories() {
    for row in ["absent", "empty", "nonempty"] {
        let scratch = support::Scratch::startup(&format!("box-directory-{row}"));
        let config = scratch.path().join("config");
        let data = scratch.path().join("data");
        let cache = scratch.path().join("cache");
        let arguments: Vec<Box<[u8]>> = [
            b"--config".as_slice(),
            config.as_os_str().as_encoded_bytes(),
            b"--data",
            data.as_os_str().as_encoded_bytes(),
            b"--cache",
            cache.as_os_str().as_encoded_bytes(),
        ]
        .iter()
        .map(|v| Box::from(*v))
        .collect();
        let (dirs, _) = temper_box_shell::directories::resolve(&arguments, &[], b"/").expect("scratch directories");
        if row != "absent" {
            fs::create_dir(&config).expect("configuration");
        }
        if row == "nonempty" {
            fs::write(config.join("other"), b"kept").expect("entry");
        }
        let result = temper_box_shell::box_command::configuration(&dirs, &[]);
        if row == "nonempty" {
            assert_eq!(
                result.expect_err("missing document"),
                format!("configuration {}: absent; temper init writes one", dirs.document().display())
            );
            assert_eq!(fs::read(config.join("other")).expect("kept"), b"kept");
            assert!(!data.exists());
            assert!(!cache.exists());
        } else {
            let (first, held, _) = result.expect("initialization");
            assert_eq!(held.len(), 5, "init retains its five listeners");
            assert!(data.join("worker.secret").exists());
            assert!(cache.is_dir());
            let (second, reopened, _) = temper_box_shell::box_command::configuration(&dirs, &[]).expect("second read");
            assert_eq!(second, first, "existing configuration is kept");
            assert!(reopened.is_empty(), "reading an existing configuration reserves no listeners");
            for (fd, _) in held {
                skein_shell::keeper::close_keeper_fd(fd);
            }
        }
        scratch.remove();
    }
}

#[test]
fn serve_box_refuses_closed_or_non_pipe_descriptors_before_starting() {
    for inherited in [false, true] {
        for (redirections, offset) in [("{r}>&- {a}>&-", 0), ("{r}>/dev/null {a}>&-", 0), ("{r}>&1 {a}>&-", 1)] {
            let requests = if inherited { 4 } else { 3 };
            let answers = requests + 1;
            let redirections = redirections.replace("{r}", &requests.to_string()).replace("{a}", &answers.to_string());
            let script = format!(
                "exec \"$1\" internal serve --config /unused --data /unused --cache /unused --box {} {redirections}",
                if inherited { "3<&0" } else { "" }
            );
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
            let fd = requests + offset;
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(
                String::from_utf8(output.stderr).expect("UTF-8 diagnostic"),
                format!("temper serve: --box: fd {fd} is not an open pipe\n")
            );
        }
    }
}

#[test]
fn serve_claims_its_clients_listener_and_box_pipes_follow_it() {
    for boxed in [false, true] {
        let scratch = support::Scratch::startup(if boxed { "inherited-boxed" } else { "inherited-serve" });
        support::configuration(scratch.path(), &support::Forge::Unreached);
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("held listener");
        let address = listener.local_addr().expect("address");
        let path = scratch.path().join("temper.json");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("config")).expect("JSON");
        value["application"]["clients"]["listener"] = serde_json::json!({"socket":address.to_string()});
        support::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
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
        assert!(ended.ready.is_some(), "serve ready: {:?} {:?}", ended.stdout, ended.stderr);
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
fn the_store_page_holds_the_templates_largest_turn_frame() {
    let scratch = support::Scratch::startup("turn-store-page");
    let bounded = support::owned::bounded_template(scratch.path());
    let store = temper_service::startup::store_limits(
        &bounded.document,
        &jig_protocol_config::Directories::working(),
        Some(&bounded.agent),
    )
    .expect("template store limits");
    let turn = u64::from(bounded.agent.limits.bodies.turn_body);
    eprintln!("template T={turn}, kv.page.bytes={}, T+64={}", store.store.store.page.bytes, turn + 64);
    assert!(u64::from(store.store.store.page.bytes) >= turn + 64);
    scratch.remove();
}

/// Hold only the limit maps this derivation owns; keep the document's other text.
fn turn_bound_document(path: &std::path::Path, turn: u32, escaped: bool) {
    use support::configuration::turn_bounds;
    let text = fs::read_to_string(path).expect("turn-bound document");
    let plain = if escaped { text.replace("{{", "{").replace("}}", "}") } else { text.clone() };
    let before: serde_json::Value = serde_json::from_str(&plain).expect("document JSON");
    let mut after = before.clone();
    turn_bounds::apply(&mut after, turn);
    let mut maps = Vec::new();
    for pointer in ["/engine/limits/overrides", "/application/limits/overrides"] {
        if let Some(previous) = before.pointer(pointer) {
            maps.push((previous, after.pointer(pointer).expect("same map")));
        }
    }
    if let Some(workers) = before["workers"].as_array() {
        for (index, worker) in workers.iter().enumerate() {
            maps.push((&worker["limits"]["overrides"], &after["workers"][index]["limits"]["overrides"]));
        }
    }
    let write = temper_test_env::rewrite();
    if !write {
        assert_eq!(
            before,
            after,
            "{}: run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p temper-box --test suite -E 'test(turn_bounds)'",
            path.display()
        );
        return;
    }
    let mut result = String::new();
    let mut rest = plain.as_str();
    while let Some(at) = rest.find("\"overrides\"") {
        let start = at + rest[at..].find('{').expect("override map");
        let end = start + rest[start..].find('}').expect("flat numeric overrides") + 1;
        let previous: serde_json::Value = serde_json::from_str(&rest[start..end]).expect("override JSON");
        let (_, next) = maps.iter().find(|(old, _)| **old == previous).expect("known override map");
        result.push_str(&rest[..start]);
        let indent = rest[..at].rsplit_once('\n').map_or("", |(_, line)| line);
        let indent = &indent[..indent.len() - indent.trim_start().len()];
        let map = serde_json::to_string_pretty(next).expect("limit map JSON");
        result.push_str(&map.replace('\n', &format!("\n{indent}")));
        rest = &rest[end..];
    }
    result.push_str(rest);
    if escaped {
        // Only JSON object braces are doubled in the fixture's format; interpolation
        // braces inside quoted strings remain the fixture reader's expressions.
        let mut in_string = false;
        let mut slash = false;
        result = result.chars().fold(String::new(), |mut out, c| {
            if !in_string && matches!(c, '{' | '}') {
                out.push(c);
            }
            out.push(c);
            if c == '"' && !slash {
                in_string = !in_string;
            }
            slash = c == '\\' && !slash;
            out
        });
    }
    fs::write(path, result).expect("write derived turn bounds");
}

#[test]
fn the_live_github_configuration_passes_startup_admission() {
    use std::os::unix::ffi::OsStrExt;
    use support::live_github::{configure_engine, document};

    let templates = skein_scratch::paths::package().join("tests/support/github");
    let api = "127.0.0.1:31001".parse().expect("synthetic API socket");
    let git = "127.0.0.1:31002".parse().expect("synthetic git socket");
    let workers = "127.0.0.1:31003".parse().expect("synthetic worker socket");
    let owner = "fabrig-ai";
    let scratch = support::Scratch::startup(&format!("live-github-admission-{owner}"));
    let mut cluster = support::cluster::Cluster::description_from(scratch.path(), &templates);
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
    let engine_path = cluster.dir("engine").join("config/temper.json");
    let mut engine = document(&engine_path);
    configure_engine(&mut engine, owner, api, git, workers);
    support::write_private(&engine_path, &serde_json::to_vec(&engine).expect("live-shaped engine JSON"));
    let worker_path = cluster.dir("w1").join("config/temper.json");
    let mut worker = document(&worker_path);
    worker["application"]["workers"][0]["git"] = "/usr/bin/git".into();
    support::write_private(&worker_path, &serde_json::to_vec(&worker).expect("worker JSON"));
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
        let bytes = fs::read(config.join("temper.json")).expect("rendered configuration");
        let operations = temper_box_shell::operations_in(&[], b"/", &directories, &bytes)
            .unwrap_or_else(|why| panic!("{owner}/{member}: {why}"));
        eprintln!("{owner}/{member}: startup admission passed; operations={operations}");
    }
    scratch.remove();
}

#[test]
fn the_reference_profile_passes_startup_admission() {
    let scratch = support::Scratch::startup("reference-profile-admission");
    let profile = support::owned::bounded_reference(scratch.path());
    let admission = profile.admission();
    scratch.remove();
    if let Err(why) = admission {
        panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why)));
    }
}

#[test]
fn checked_in_deployment_profiles_pass_startup_admission() {
    let profiles = [
        "docs/examples/owned.json",
        "crates/temper-init/template/temper.json",
        "bin/temper-box/tests/fixtures/cluster/engine.json",
        "bin/temper-box/tests/fixtures/cluster/w1.json",
        "bin/temper-box/tests/fixtures/cluster/w2.json",
    ];
    let mut failures = String::new();
    for (index, relative) in profiles.iter().enumerate() {
        let scratch = support::Scratch::startup(&format!("deployment-profile-{index}"));
        let profile = match *relative {
            "docs/examples/owned.json" => support::owned::bounded(scratch.path()),
            "crates/temper-init/template/temper.json" => support::owned::bounded_template(scratch.path()),
            _ => support::owned::deployment_profile(scratch.path(), relative),
        };
        let turn = profile.agent.limits.bodies.turn_body;
        let record = size_of::<temper_engine_domain::Record>();
        match profile.admission() {
            Ok(()) => eprintln!("{relative}: startup admission passed; T={turn}, Record={record}"),
            Err(why) => {
                writeln!(
                    failures,
                    "{relative}: T={turn}, Record={record}\n{}",
                    String::from_utf8_lossy(&temper_service::startup::lines(&why))
                )
                .expect("diagnostic");
            }
        }
        scratch.remove();
    }
    assert!(failures.is_empty(), "{failures}");
}

#[test]
fn turn_bounds() {
    let root = skein_scratch::paths::tree();
    for (deployment, agent) in [
        ("crates/temper-init/template/temper.json", "crates/temper-init/template/agent.json"),
        ("crates/temper-init/template/bare.json", "crates/temper-init/template/bare-agent.json"),
        ("docs/examples/owned.json", "docs/examples/owned/agent.json"),
    ] {
        let agent: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(agent)).expect("agent document")).expect("agent JSON");
        let turn = support::configuration::turn_bounds::turn_body(&agent);
        turn_bound_document(&root.join(deployment), turn, false);
        eprintln!("{deployment}: T={turn}");
    }
    let directory = root.join("bin/temper-box/tests/fixtures/cluster");
    let agent = fs::read_to_string(directory.join("agent.json")).expect("cluster agent");
    let agent: serde_json::Value =
        serde_json::from_str(&agent.replace("{{", "{").replace("}}", "}")).expect("cluster agent JSON");
    let turn = support::configuration::turn_bounds::turn_body(&agent);
    for name in ["engine.json", "w1.json", "w2.json"] {
        turn_bound_document(&directory.join(name), turn, true);
    }
    eprintln!("cluster fixture: T={turn}");
    let github = root.join("bin/temper-box/tests/support/github");
    let agent = fs::read_to_string(github.join("agent.json")).expect("GitHub agent");
    let agent: serde_json::Value =
        serde_json::from_str(&agent.replace("{{", "{").replace("}}", "}")).expect("GitHub agent JSON");
    let turn = support::configuration::turn_bounds::turn_body(&agent);
    for name in ["engine.json", "w1.json"] {
        turn_bound_document(&github.join(name), turn, true);
    }
}

#[test]
fn the_template_agent_refuses_jigs_small_hub_turn_bound_before_serving() {
    let scratch = support::Scratch::startup("small-hub-turn");
    let output = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("init")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("init");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let path = scratch.path().join("temper.json");
    let mut document: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("deployment")).expect("JSON");
    let hub = jig_protocol_config::profile(jig_protocol_config::Profile::Small).hub;
    let limits = &mut document["engine"]["limits"]["overrides"];
    limits["hub.turn_bytes"] = hub.turn_bytes.into();
    limits["hub.transcript_bytes"] = hub.transcript_bytes.into();
    limits["hub.turn_queue_bytes"] = hub.turn_queue_bytes.into();
    fs::write(&path, serde_json::to_vec(&document).expect("JSON")).expect("deployment");
    let output = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("serve")
        .args(support::configuration::flags(scratch.path()))
        .output()
        .expect("serve");
    let agent: serde_json::Value =
        serde_json::from_slice(&fs::read(scratch.path().join("agent.json")).expect("agent")).expect("agent JSON");
    let turn = support::configuration::turn_bounds::turn_body(&agent);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).lines().find(|line| line.starts_with("temper serve:")).expect("startup problem").starts_with(&format!(
        "temper serve: engine.limits.overrides.hub.turn_bytes: 65536 bytes, short of the agent's largest turn, {turn} bytes"
    )), "{}", String::from_utf8_lossy(&output.stderr));
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
        format!("temper forge run: forge 1: fd 3 listens at {found}, expected owned.listen 127.0.0.1:9\n")
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
    test ! -S /proc/self/fd/3 || exit 21
    printf 'forgejo version {VERSION}\n'
    exit 0
fi
test "$LISTEN_FDS" = 1 || exit 22
test -S /proc/self/fd/3 || exit 23
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
    for (which, count) in [("forgejo", "0"), ("forgejo", "2"), ("forgejo", "many"), ("runner", "1"), ("runner", "0")] {
        let ended = std::process::Command::new(skein_scratch::paths::binary("temper-box"))
            .args(["forge", "run", "--config", "/no-document", "--data", "/unused", "--cache", "/unused", "1", which])
            .env_clear()
            .env("LISTEN_FDS", count)
            .output()
            .expect("run");
        assert_eq!(ended.status.code(), Some(1));
        let diagnostic = String::from_utf8(ended.stderr).expect("diagnostic");
        assert!(diagnostic.contains("LISTEN_FDS") && diagnostic.contains("fd 3"), "{diagnostic}");
        assert!(!diagnostic.contains("no-document"), "refuses before document read");
    }
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
        assert!(ended.stdout.is_empty());
        let why = String::from_utf8(ended.stderr).expect("diagnostic");
        assert!(why.starts_with("temper internal forge-child: "), "{why}");
        assert_eq!(why.lines().count(), 1, "{why}");
        assert!(!why.contains("document"), "{why}");
    }
}

#[test]
fn box_logs_must_hold_one_whole_record_and_tails_at_least_one_line() {
    for (name, value) in [("log_bytes", 4117), ("tail_lines", 0)] {
        let scratch = support::Scratch::startup(&format!("box-{name}"));
        let mut document = box_document(&scratch);
        document["application"]["limits"]["overrides"][format!("box.{name}")] = value.into();
        assert_eq!(
            refused_box(&scratch, &document),
            format!("temper-box: application.limits.overrides.box.{name}: past its bound\n")
        );
        scratch.remove();
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
    let agent: serde_json::Value =
        serde_json::from_slice(&fs::read(scratch.path().join("agent.json")).expect("agent document"))
            .expect("agent JSON");
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
    let agent: serde_json::Value = serde_json::from_slice(&fs::read(directory.join("agent.json")).unwrap()).unwrap();
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
    fs::write(&file, format!("#!/bin/sh\nprintf '%s\\000' \"$@\" > '{}/arguments'\nprintf '%s' \"$DISABLE_AUTOUPDATER\" > '{}/updater'\nprintf '%s' \"$CLAUDE_CONFIG_DIR\" > '{}/source'\nprintf 'command stdout\\n'\nexit 0\n", directory.display(), directory.display(), directory.display())).unwrap();
    fs::OpenOptions::new().write(true).open(&file).unwrap().set_len(build.bytes).unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();
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
    assert_eq!(fs::read(directory.join("updater")).unwrap(), b"1");
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
    assert!(output.stderr.is_empty());
    assert_eq!(fs::read(directory.join("arguments")).unwrap(), b"one \xff\n'\0--two=$x\0");
    assert_eq!(fs::read(directory.join("updater")).unwrap(), b"1");
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

#[test]
fn program_login_places_both_launcher_links_only_for_a_fetched_program() {
    use std::os::unix::fs::MetadataExt;
    for fetched in [false, true] {
        let scratch = support::Scratch::startup(&format!("login-links-{fetched}"));
        let directory = scratch.path();
        let programs = directory.join("programs");
        support::program_login::configuration(
            directory,
            Some(&programs),
            if fetched { Path::new("fetched") } else { Path::new("/missing/claude") },
        );
        let mut command = std::process::Command::new(skein_scratch::paths::binary("temper-box"));
        command.arg("program-login").args(support::configuration::flags(directory)).arg("source").env_clear();
        // A wrong-sized cached build refuses before any network request.
        let build = directory.join("claude-code/2.1.311/claude");
        fs::create_dir_all(build.parent().unwrap()).unwrap();
        fs::write(&build, b"wrong size").unwrap();
        let launcher = directory.join("claude-code/claude");
        let cache = directory.join("claude-code/cache");
        if fetched {
            symlink("/stale/temper", &launcher).unwrap();
            symlink("/stale/cache", &cache).unwrap();
        }
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        if fetched {
            assert_eq!(fs::read_link(&launcher).unwrap(), skein_scratch::paths::binary("temper-box"));
            assert_eq!(fs::read_link(&cache).unwrap(), directory);
            let before = fs::symlink_metadata(&launcher).unwrap();
            let again = command.output().unwrap();
            assert_eq!(again.status.code(), Some(1));
            assert_eq!(before.ino(), fs::symlink_metadata(&launcher).unwrap().ino(), "right links stay untouched");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("temper: claude: "),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            assert!(fs::symlink_metadata(&launcher).is_err());
            assert!(fs::symlink_metadata(&cache).is_err());
        }
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fetching"));
        scratch.remove();
    }
}

#[test]
fn serve_and_a_worker_alone_place_links_only_for_a_fetched_program() {
    for worker in [false, true] {
        for fetched in [false, true] {
            let scratch = support::Scratch::startup(&format!("serve-launcher-{worker}-{fetched}"));
            let directory = scratch.path();
            support::program_login::configuration(
                directory,
                Some(&directory.join("programs")),
                if fetched { Path::new("fetched") } else { Path::new("/found/claude") },
            );
            for (name, bytes) in jig_channel_kit::fixtures::files(false).entries.into_boxed() {
                if !name.starts_with(b"charters/") {
                    let path = directory.join(std::str::from_utf8(&name).unwrap());
                    fs::create_dir_all(path.parent().unwrap()).unwrap();
                    support::write_private(&path, bytes.expose());
                }
            }
            support::write_private(&directory.join("worker.secret"), b"worker-secret");
            if worker {
                let document = directory.join("temper.json");
                let mut input: serde_json::Value = serde_json::from_slice(&fs::read(&document).unwrap()).unwrap();
                input.as_object_mut().unwrap().remove("engine");
                input["workers"][0]["engine"] = serde_json::json!({"socket":"127.0.0.1:7100"});
                input["application"] = serde_json::json!({"workers":input["application"]["workers"],"limits":input["application"]["limits"]});
                fs::write(&document, serde_json::to_vec(&input).unwrap()).unwrap();
            }
            let prepared = temper_box_shell::prepare_in(&[], b"/", &support::configuration::directories(directory));
            assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default());
            let launcher = directory.join("claude-code/claude");
            let cache = directory.join("claude-code/cache");
            if fetched {
                assert_eq!(fs::read_link(&launcher).unwrap(), skein_scratch::paths::binary("temper-box"));
                assert_eq!(fs::read_link(&cache).unwrap(), directory);
            } else {
                assert!(fs::symlink_metadata(launcher).is_err());
                assert!(fs::symlink_metadata(cache).is_err());
            }
            assert!(!directory.join("claude-code/2.1.311").exists(), "startup fetches nothing");
            scratch.remove();
        }
    }
}

#[test]
fn serve_listening_requires_the_clients_listener_in_this_runs_log() {
    let scratch = support::Scratch::startup("serve-listening");
    assert!(!support::boxed::serve_listening(scratch.path()), "an absent log is not listening");
    fs::create_dir_all(scratch.path().join("box/logs")).expect("logs");
    let log = scratch.path().join("box/logs/serve.log");
    fs::write(&log, b"listening workers 127.0.0.1:9000\n").expect("workers listen");
    assert!(!support::boxed::serve_listening(scratch.path()), "another listener is not clients");
    fs::write(&log, b"2026-11-06T00:00:00Z listening clients 127.0.0.1:8367\n").expect("clients listen");
    assert!(support::boxed::serve_listening(scratch.path()), "clients have listened");
    scratch.remove();
}

#[test]
fn ctl_link_sign_in_waits_for_serve_before_starting_its_bound() {
    support::ctl::serve_gate_tests::link_sign_in_waits_for_serve_before_starting_its_bound();
}

#[test]
fn native_sign_in_keeps_its_start_bound_until_serve_listens() {
    support::person::serve_gate_tests::native_sign_in_keeps_its_start_bound_until_serve_listens();
}

#[test]
fn a_story_purse_is_priced_from_the_init_template() {
    assert!(support::first_repository::run_spend(5).is_power_of_two());
}

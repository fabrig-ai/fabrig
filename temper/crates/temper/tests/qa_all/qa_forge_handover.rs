//! The engine owns the handover (temper/docs/design/protocol/forge.md, 2.1–2.2;
//! temper/docs/design/protocol/credentials.md, 2.1).
//! Forges are records (temper/docs/design/draft/playground-reset.md, 3).
//! These tests inspect processes, files, the forge API and the client's rows.
use crate::support;

use skein_io::kernel::Exit;
use std::{
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink},
    path::{Path, PathBuf},
};
use support::{Scratch, ctl, owned};

const FORGE: u16 = 1;
// Configuration.md 6: ordinary private credentials have no group or other bits.
const PRIVATE: u32 = 0o600;
// configuration.md 6, 7: an explicit records.value_bytes admission bound.
const RECORD_BYTES: usize = 4096;

fn directory(scratch: &Path, flag: &str) -> PathBuf {
    let flags = support::configuration::flags(scratch);
    let pair = flags.windows(2).find(|pair| pair[0] == flag).expect("directory flag");
    PathBuf::from(&pair[1])
}

fn document(scratch: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(directory(scratch, "--config").join("temper.json")).expect("fixture document"))
        .expect("fixture JSON")
}

fn forge_directory(scratch: &Path) -> PathBuf {
    directory(scratch, "--data").join("forge-1")
}

fn machine_configured(case: &str, person: &str) -> Scratch {
    let scratch = Scratch::new(case);
    let (forgejo, _) = owned::binaries();
    let [address] = support::free_addresses(&[]);
    owned::configuration(
        scratch.path(),
        &owned::Names { forgejo: &forgejo, runner: None, port: address.port(), bot: "temper", person },
    );
    owned::configuration_value(scratch.path(), |value| {
        value["application"]["machine"].as_object_mut().expect("machine").remove("fake");
        // The first repository's temper/ branch prefix needs eight authority segments.
        value["engine"]["limits"]["overrides"]["authority.segments"] = 8.into();
    });
    scratch
}

fn configured(case: &str) -> Scratch {
    configured_person(case, "ada")
}

fn configured_person(case: &str, person: &str) -> Scratch {
    let scratch = machine_configured(case, person);
    // A failed preparation keeps the new forge's record without minting a
    // handover. The test then prepares it separately to control its file.
    let binary = document(scratch.path())["application"]["machine"]["owned"]["forgejo"].clone();
    owned::configuration_value(scratch.path(), |value| {
        value["application"]["machine"]["owned"]["forgejo"] = "missing-forgejo".into();
    });
    let client = directory(scratch.path(), "--data").join("record-client");
    fs::create_dir_all(&client).expect("record client");
    let outcome = support::boxed::run_ctl_with_exits(
        scratch.path(),
        box_flags(scratch.path()),
        &client,
        vec![
            ctl::Step::Run(ctl::Call::sign_in()),
            ctl::Step::Run(ctl::Call::new(&["forge", "setup"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
        ],
        support::boxed::BOX_DEADLINE,
        || (),
        &[(1, 1)],
    );
    assert_eq!(
        outcome.exit,
        Some(Exit::Code(0)),
        "record-making box drained: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert!(outcome.left.as_ref().is_some_and(Vec::is_empty), "record-making box left processes");
    assert_eq!(outcome.calls.len(), 3, "forge record made");
    assert_eq!(forge_row(&ctl::json(&outcome.calls[2]))["state"], "set_up_failed");
    let mut value = document(scratch.path());
    value["application"]["machine"]["owned"]["forgejo"] = binary;
    private_write(
        &directory(scratch.path(), "--config").join("temper.json"),
        &serde_json::to_vec(&value).expect("restored machine JSON"),
    );
    scratch
}

fn private_write(path: &Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(PRIVATE)
        .open(path)
        .expect("private test file");
    file.write_all(bytes).expect("private test bytes");
    fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE)).expect("private test mode");
}

fn secrets(scratch: &Path) -> Vec<Vec<u8>> {
    let root = forge_directory(scratch);
    ["admin_token", "person_password", "runner/secret"]
        .iter()
        .filter_map(|name| fs::read(root.join(name)).ok())
        .map(|mut bytes| {
            if bytes.last() == Some(&b'\n') {
                bytes.pop();
            }
            bytes
        })
        .filter(|bytes| !bytes.is_empty())
        .collect()
}

fn no_secrets(bytes: &[u8], credentials: &[Vec<u8>]) {
    for credential in credentials {
        assert!(
            !bytes.windows(credential.len()).any(|part| part == credential),
            "a generated credential escaped in observable output (redacted)"
        );
    }
}

fn inspect(scratch: &Path, credentials: &[Vec<u8>]) -> (ctl::Seen, serde_json::Value) {
    let seen = ctl::run(ctl::Story::new(
        scratch,
        vec![
            ctl::Step::Run(ctl::Call::sign_in()),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "settled", "--timeout", "15s"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
        ],
    ));
    no_secrets(&seen.serve.stdout, credentials);
    no_secrets(&seen.serve.stderr, credentials);
    for call in &seen.calls {
        no_secrets(&call.stdout, credentials);
        no_secrets(&call.stderr, credentials);
    }
    assert!(seen.serve.exit == Some(Exit::Code(0)), "serve did not drain successfully");
    assert_eq!(seen.calls.len(), 3, "client did not finish inspecting forges");
    for call in &seen.calls {
        assert!(call.exit == Some(Exit::Code(0)), "client inspection failed");
    }
    let rows = ctl::json(&seen.calls[2]);
    (seen, rows)
}

fn prepared(case: &str) -> (Scratch, Vec<Vec<u8>>, owned::Running) {
    let scratch = configured(case);
    let output = owned::prepare(scratch.path(), FORGE);
    let credentials = secrets(scratch.path());
    no_secrets(&output.stdout, &credentials);
    no_secrets(&output.stderr, &credentials);
    assert!(output.status.success(), "owned preparation failed (output redacted)");
    let running = start_prepared(scratch.path());
    (scratch, credentials, running)
}

#[test]
fn owned_handover_is_removed_and_restart_uses_records() {
    // protocol/setup.md 6.5: TakeOver, persist, remove, reconcile, Connect.
    let (scratch, credentials, running) = prepared("qa-handover-records");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let (seen, first) = inspect(scratch.path(), &credentials);
    assert!(
        !handover.exists(),
        "successful TakeOver left its handover behind: {}",
        String::from_utf8_lossy(&seen.serve.stderr)
    );
    connected(&first);
    let (_, second) = inspect(scratch.path(), &credentials);
    connected(&second);
    assert!(first == second, "restart from records changed the forge row (values redacted)");
    for old in ["api_token", "git_token", "user", "webhook_secret"] {
        assert!(!forge_directory(scratch.path()).join(old).exists(), "owned credentials escaped to legacy files");
    }
    running.stop();
    scratch.remove();
}

#[test]
#[ignore = "backlog: an unsafe handover at start refuses serve; setup.md 6.5 and configuration.md 6 isolate the forge; off the happy path (prepare writes 0600 within bound)"]
fn nonprivate_handover_blocks_connect_without_stopping_serve() {
    // configuration.md 6; setup.md 6.5: failed private read leaves the file.
    let (scratch, credentials, running) = prepared("qa-handover-mode");
    let handover = forge_directory(scratch.path()).join("admin_token");
    fs::set_permissions(&handover, fs::Permissions::from_mode(0o644)).expect("adversarial mode");
    // Restore the deliberately unsafe mode even when the process harness
    // reports an early exit, so retained test credentials remain private.
    let inspection = std::panic::catch_unwind(|| inspect(scratch.path(), &credentials));
    fs::set_permissions(&handover, fs::Permissions::from_mode(PRIVATE)).expect("restore private handover mode");
    let (_, rows) = match inspection {
        Ok(result) => result,
        Err(failure) => std::panic::resume_unwind(failure),
    };
    assert!(handover.exists(), "unsafe handover was consumed");
    assert!(forge_row(&rows)["state"] != "connected", "unsafe handover connected");
    running.stop();
    scratch.remove();
}

#[test]
#[ignore = "backlog: an unsafe handover at start refuses serve; setup.md 6.5 and configuration.md 6 isolate the forge; off the happy path (prepare writes 0600 within bound)"]
fn linked_handover_is_refused_and_target_is_preserved() {
    // configuration.md 6: a linked private file is refused, even at 0600.
    let (scratch, credentials, running) = prepared("qa-handover-link");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let target = forge_directory(scratch.path()).join("private-handover-target");
    fs::rename(&handover, &target).expect("move test handover");
    symlink(&target, &handover).expect("linked test handover");
    let (_, rows) = inspect(scratch.path(), &credentials);
    assert!(
        fs::symlink_metadata(&handover).expect("handover retained").file_type().is_symlink(),
        "linked handover was consumed"
    );
    assert!(target.exists(), "linked handover's target was removed");
    assert!(forge_row(&rows)["state"] != "connected", "linked handover connected");
    running.stop();
    scratch.remove();
}

fn refused_handover(case: &str) -> (Scratch, Vec<Vec<u8>>, owned::Running) {
    let (scratch, mut credentials, running) = prepared(case);
    let handover = forge_directory(scratch.path()).join("admin_token");
    let mut invalid = fs::read(&handover).expect("test handover");
    invalid[0] = if invalid[0] == b'a' { b'b' } else { b'a' };
    private_write(&handover, &invalid);
    credentials.push(invalid);
    (scratch, credentials, running)
}

#[test]
fn refused_takeover_keeps_handover_and_redacts_its_bytes() {
    // setup.md 6.5 and skein forge-setup.md 4.7: a stop never removes it.
    let (scratch, credentials, running) = refused_handover("qa-handover-refused");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let (_, rows) = inspect(scratch.path(), &credentials);
    assert!(handover.exists(), "refused TakeOver removed its handover");
    assert!(forge_row(&rows)["state"] != "connected", "refused TakeOver connected");
    running.stop();
    scratch.remove();
}

fn start_prepared(scratch: &Path) -> owned::Running {
    let running = owned::run(scratch, "forgejo");
    let value = document(scratch);
    let address: std::net::SocketAddr = value["application"]["machine"]["owned"]["listen"]["socket"]
        .as_str()
        .expect("owned listener")
        .parse()
        .expect("loopback socket");
    let _ = owned::await_answer(address.port(), "/api/v1/version", None);
    running
}

fn forge_row(rows: &serde_json::Value) -> &serde_json::Value {
    rows["forges"]
        .as_array()
        .expect("forges document")
        .iter()
        .find(|row| row["number"].as_u64() == Some(u64::from(FORGE)))
        .expect("configured forge row")
}

fn connected(rows: &serde_json::Value) {
    let row = forge_row(rows);
    let encoded = row.to_string();
    assert!(row["state"] == "connected", "forge did not reach connected (values redacted)");
    assert!(encoded.contains("\"temper\""), "connected identity is not the configured bot (values redacted)");
}

#[test]
fn fresh_handover_replaces_kept_record_and_revokes_old_admin() {
    // setup.md 6.3, 6.5; skein forge-setup.md 4.7: present beats kept.
    let (scratch, mut credentials, running) = prepared("qa-handover-fresh");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let old = fs::read(&handover).expect("first handover");
    let (_, first) = inspect(scratch.path(), &credentials);
    connected(&first);
    running.stop();
    let output = owned::prepare(scratch.path(), FORGE);
    credentials.extend(secrets(scratch.path()));
    no_secrets(&output.stdout, &credentials);
    no_secrets(&output.stderr, &credentials);
    assert!(output.status.success(), "second preparation failed");
    let fresh = fs::read(&handover).expect("fresh handover");
    assert!(old != fresh, "second preparation did not mint a fresh handover");
    let running = start_prepared(scratch.path());
    let (_, rows) = inspect(scratch.path(), &credentials);
    connected(&rows);
    assert!(!handover.exists(), "fresh handover remained beside a kept record");
    let value = document(scratch.path());
    let address: std::net::SocketAddr = value["application"]["machine"]["owned"]["listen"]["socket"]
        .as_str()
        .expect("listener")
        .parse()
        .expect("socket");
    let old_token = std::str::from_utf8(&old).expect("token text").trim_end();
    let fresh_token = std::str::from_utf8(&fresh).expect("token text").trim_end();
    let old_user = owned::answer(address.port(), "/api/v1/user", Some(old_token));
    assert!(old_user.ok().and_then(|user| user.get("id").cloned()).is_none(), "old admin token still authenticates");
    let user = owned::answer(address.port(), "/api/v1/user", Some(fresh_token)).expect("fresh admin check");
    assert!(user["is_admin"] == true, "taken token no longer authenticates as site admin");
    let tokens = owned::answer(address.port(), "/api/v1/admin/users/temper-admin/tokens", Some(fresh_token))
        .expect("admin token listing");
    assert_eq!(tokens.as_array().expect("token list").len(), 1, "TakeOver left older admin tokens valid");
    running.stop();
    scratch.remove();
}

#[test]
fn repeated_handover_of_same_token_is_harmless() {
    // setup.md 6.5 explicitly permits taking the same token again.
    let (scratch, credentials, running) = prepared("qa-handover-repeat");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let token = fs::read(&handover).expect("test handover");
    let (_, first) = inspect(scratch.path(), &credentials);
    connected(&first);
    private_write(&handover, &token);
    let (_, second) = inspect(scratch.path(), &credentials);
    connected(&second);
    assert!(!handover.exists(), "repeated successful handover was left behind");
    running.stop();
    scratch.remove();
}

#[test]
fn bad_fresh_handover_never_falls_back_to_kept_admin() {
    // setup.md 6.5: a present handover is always taken, even with records.
    let (scratch, mut credentials, running) = prepared("qa-handover-no-fallback");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let mut token = fs::read(&handover).expect("test handover");
    let (_, first) = inspect(scratch.path(), &credentials);
    connected(&first);
    token[0] = if token[0] == b'a' { b'b' } else { b'a' };
    private_write(&handover, &token);
    credentials.push(token);
    let (_, rows) = inspect(scratch.path(), &credentials);
    assert!(handover.exists(), "bad fresh handover was discarded");
    assert!(forge_row(&rows)["state"] != "connected", "kept access bypassed a failed fresh takeover");
    running.stop();
    scratch.remove();
}

fn bounded_handover(case: &str, size: usize, operation: &[u8]) {
    // configuration.md 6, 7: file reads are within records.value_bytes.
    // A Forgejo token is shorter, so at the byte bound TakeOver must refuse it;
    // a file one byte past the bound must instead fail the private read.
    let (scratch, mut credentials, running) = prepared(case);
    owned::configuration_value(scratch.path(), |value| {
        value["application"]["limits"]["overrides"]["records.value_bytes"] = serde_json::json!(RECORD_BYTES);
    });
    let handover = forge_directory(scratch.path()).join("admin_token");
    let mut bytes = fs::read(&handover).expect("test handover");
    bytes.resize(size, b'a');
    private_write(&handover, &bytes);
    credentials.push(bytes);
    let (seen, rows) = inspect(scratch.path(), &credentials);
    assert!(handover.exists(), "failed bounded handover was consumed");
    assert!(forge_row(&rows)["state"] != "connected", "invalid bounded handover connected");
    assert!(
        seen.serve.stderr.windows(operation.len()).any(|part| part == operation),
        "failed bounded read named the wrong operation"
    );
    running.stop();
    scratch.remove();
}

#[test]
fn handover_at_read_bound_reaches_takeover_and_stays_private() {
    bounded_handover("qa-handover-at-bound", RECORD_BYTES, b"taking the handover");
}

#[test]
#[ignore = "backlog: an unsafe handover at start refuses serve; setup.md 6.5 and configuration.md 6 isolate the forge; off the happy path (prepare writes 0600 within bound)"]
fn handover_past_read_bound_blocks_only_its_forge() {
    bounded_handover("qa-handover-past-bound", RECORD_BYTES + 1, b"admin_token");
}

fn box_flags(scratch: &Path) -> Vec<String> {
    support::configuration::flags(scratch)
        .iter()
        .map(|arg| arg.to_str().expect("test directory UTF-8").to_owned())
        .collect()
}

#[test]
fn unprepared_owned_forge_is_not_setup_without_box() {
    // setup.md 6.5; domain/forge.md 4.1: no access, and an actionable refusal.
    let scratch = machine_configured("qa-handover-unprepared", "ada");
    let seen = ctl::run(ctl::Story::new(
        scratch.path(),
        vec![
            ctl::Step::Run(ctl::Call::sign_in()),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "setup"])),
        ],
    ));
    assert!(seen.serve.exit == Some(Exit::Code(0)), "unprepared forge prevented a healthy engine stop");
    assert_eq!(seen.calls.len(), 3, "missing client calls");
    assert!(seen.calls[1].exit == Some(Exit::Code(0)), "forges read failed");
    let row = ctl::json(&seen.calls[1]);
    assert!(row["forges"].as_array().expect("forge list").is_empty(), "fresh deployment has no forge record");
    assert!(seen.calls[2].exit == Some(Exit::Code(2)), "setup without a box was not refused");
    let refusal = &seen.calls[2].stderr;
    assert!(
        refusal.windows(b"not under the box".len()).any(|part| part == b"not under the box"),
        "setup refusal did not name the missing box"
    );
    assert!(!forge_directory(scratch.path()).exists(), "unprepared owned forge wrote files");
    scratch.remove();
}

#[test]
fn reconcile_stop_keeps_admin_record_but_never_connects() {
    // setup.md 6.5, skein forge-setup.md 4.3: never take over a person's
    // unmarked account. Preparation made temper as a person, so the fixed
    // bot login names an existing unmarked user.
    let scratch = configured_person("qa-handover-reconcile-stop", "temper");
    let output = owned::prepare(scratch.path(), FORGE);
    assert!(output.status.success(), "preparation with an unmarked person failed");
    let credentials = secrets(scratch.path());
    let running = start_prepared(scratch.path());
    let handover = forge_directory(scratch.path()).join("admin_token");
    let (first, rows) = inspect(scratch.path(), &credentials);
    assert!(!handover.exists(), "TakeOver's completed file was kept after reconcile stopped");
    assert!(forge_row(&rows)["state"] != "connected", "unmarked account was connected as a bot");
    assert!(
        first.serve.stderr.windows(b"Unmarked".len()).any(|part| part == b"Unmarked"),
        "operator did not see reconcile's reason"
    );
    let (_, restored) = inspect(scratch.path(), &credentials);
    assert!(forge_row(&restored)["state"] != "connected", "restart bypassed failed reconcile");
    running.stop();
    scratch.remove();
}

#[test]
fn takeover_stop_at_start_leaves_the_asked_record_failed_with_its_reason() {
    // playground-reset.md 3, 6.4: the record keeps its asked setup across
    // restart; a failed handover names the unfinished step on its row.
    let (scratch, credentials, running) = refused_handover("qa-handover-diagnostic");
    let (_, rows) = inspect(scratch.path(), &credentials);
    let row = forge_row(&rows);
    assert_eq!(row["state"], "set_up_failed", "a refused TakeOver leaves the asked setup failed");
    let why = row["why"].as_str().expect("failed setup's reason");
    assert!(why.contains("taking the handover") && why.contains("401"), "the row names TakeOver's refusal");
    running.stop();
    scratch.remove();
}

#[test]
fn asked_owned_setup_connects_and_box_record_restarts_it() {
    // setup.md 6.5, 7.1, 13.4: the client starts the real owned group;
    // the next box uses its durable record, and takes a fresh handover.
    let scratch = machine_configured("qa-handover-asked", "ada");
    let first_client = directory(scratch.path(), "--data").join("qa-first-client");
    fs::create_dir_all(&first_client).expect("private client root");
    let first = support::boxed::run_ctl(
        scratch.path(),
        box_flags(scratch.path()),
        &first_client,
        vec![
            ctl::Step::Run(ctl::Call::sign_in()),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "setup"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "settled", "--timeout", "30s"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
        ],
        support::boxed::BOX_DEADLINE,
        |_binary| (forge_directory(scratch.path()).join("admin_token").exists(), secrets(scratch.path())),
    );
    let (left_handover, credentials) = first.checked.as_ref().expect("first box's evidence");
    no_secrets(&first.stdout, credentials);
    no_secrets(&first.stderr, credentials);
    for call in &first.calls {
        no_secrets(&call.stdout, credentials);
        no_secrets(&call.stderr, credentials);
        assert!(call.exit == Some(Exit::Code(0)), "asked owned setup's client call failed");
    }
    assert!(first.exit == Some(Exit::Code(0)), "first asked box did not drain");
    assert!(first.left.as_ref().is_some_and(Vec::is_empty), "first box left processes");
    assert!(!left_handover, "asked setup left the handover");
    assert!(
        ctl::json(&first.calls[1])["forges"].as_array().expect("fresh forges").is_empty(),
        "no forge exists before the client's request"
    );
    connected(&ctl::json(first.calls.last().expect("connected forges row")));
    let record = directory(scratch.path(), "--data").join("box/started/1");
    assert!(support::private(&record), "box's durable started record is absent or not private");
    for old in ["api_token", "git_token", "user", "webhook_secret"] {
        assert!(!forge_directory(scratch.path()).join(old).exists(), "asked setup wrote legacy credentials");
    }
    let second_client = directory(scratch.path(), "--data").join("qa-second-client");
    fs::create_dir_all(&second_client).expect("second private client root");
    let second = support::boxed::run_ctl(
        scratch.path(),
        box_flags(scratch.path()),
        &second_client,
        vec![
            ctl::Step::Run(ctl::Call::sign_in()),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "settled", "--timeout", "30s"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
        ],
        support::boxed::BOX_DEADLINE,
        |_binary| forge_directory(scratch.path()).join("admin_token").exists(),
    );
    no_secrets(&second.stdout, credentials);
    no_secrets(&second.stderr, credentials);
    for call in &second.calls {
        no_secrets(&call.stdout, credentials);
        no_secrets(&call.stderr, credentials);
        assert!(call.exit == Some(Exit::Code(0)), "restored box's client call failed");
    }
    assert!(second.exit == Some(Exit::Code(0)), "restored box did not drain");
    assert!(second.left.as_ref().is_some_and(Vec::is_empty), "restored box left processes");
    assert!(second.checked == Some(false), "fresh handover was left beside kept records");
    connected(&ctl::json(second.calls.last().expect("restored forges row")));
    scratch.remove();
}

#[test]
fn removal_failure_blocks_reconcile_then_next_start_recovers() {
    // setup.md 6.5: even a successful TakeOver must not reconcile until
    // its file is removed. A read-only private parent makes unlink fail.
    let (scratch, credentials, running) = prepared("qa-handover-removal");
    let root = forge_directory(scratch.path());
    let handover = root.join("admin_token");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o500)).expect("read-only private forge directory");
    let (_, rows) = inspect(scratch.path(), &credentials);
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("restore forge directory permissions");
    assert!(handover.exists(), "failed removal discarded the handover");
    assert!(forge_row(&rows)["state"] != "connected", "forge connected despite failed handover removal");
    let value = document(scratch.path());
    let address: std::net::SocketAddr = value["application"]["machine"]["owned"]["listen"]["socket"]
        .as_str()
        .expect("listener")
        .parse()
        .expect("socket");
    let bot = owned::answer(address.port(), "/api/v1/users/temper", None);
    assert!(bot.ok().and_then(|user| user.get("id").cloned()).is_none(), "reconcile ran before removal succeeded");
    let (_, restored) = inspect(scratch.path(), &credentials);
    connected(&restored);
    assert!(!handover.exists(), "retry failed to remove the handover");
    running.stop();
    scratch.remove();
}

#[test]
fn asked_prepare_failure_is_visible_and_can_be_started_over() {
    // playground-reset.md 6.4: a failed setup names its step; Start over
    // removes it and makes a new forge, with a fresh number (section 3).
    let scratch = machine_configured("qa-handover-asked-failure", "ada");
    owned::configuration_value(scratch.path(), |value| {
        value["application"]["machine"]["owned"]["forgejo"] = serde_json::json!("qa-missing-forgejo");
    });
    let client = directory(scratch.path(), "--data").join("qa-failure-client");
    fs::create_dir_all(&client).expect("private client root");
    let outcome = support::boxed::run_ctl_with_exits(
        scratch.path(),
        box_flags(scratch.path()),
        &client,
        vec![
            ctl::Step::Run(ctl::Call::sign_in()),
            ctl::Step::Run(ctl::Call::new(&["forge", "setup"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "settled", "--timeout", "15s"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "remove", "1"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "setup"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--until", "settled", "--timeout", "15s"])),
            ctl::Step::Run(ctl::Call::new(&["forge", "list", "--json"])),
        ],
        support::boxed::BOX_DEADLINE,
        || directory(scratch.path(), "--data").join("box/started/1").exists(),
        &[(1, 1), (5, 1)],
    );
    for (index, call) in outcome.calls.iter().enumerate() {
        if [1, 5].contains(&index) {
            assert_eq!(call.exit, Some(Exit::Code(1)), "forge-setup waits for the prepare failure");
            assert!(String::from_utf8_lossy(&call.stderr).contains("prepare"), "the failed step is named");
        } else {
            assert_eq!(call.exit, Some(Exit::Code(0)), "retry after failed prepare was refused");
        }
    }
    assert_eq!(outcome.calls.len(), 8, "retry did not finish");
    assert!(outcome.exit == Some(Exit::Code(0)), "box failed beside its dormant forge");
    assert!(outcome.left.as_ref().is_some_and(Vec::is_empty), "failed group left processes");
    assert!(outcome.checked == Some(false), "failed preparation wrote a started record");
    for index in [3, 7] {
        let rows = ctl::json(&outcome.calls[index]);
        let text = rows.to_string();
        assert!(text.contains("set_up_failed"), "prepare failure was absent from its forge's row");
        assert!(text.contains("prepare"), "prepare's failure did not name the failed step");
    }
    scratch.remove();
}

#[test]
fn takeover_stop_names_its_step_and_reason_for_operator() {
    // setup.md 6.5: a plan's stop names its step and reason; a path is
    // named only for a file operation. Never the file's bytes.
    let (scratch, credentials, running) = refused_handover("qa-handover-operator");
    let (seen, _) = inspect(scratch.path(), &credentials);
    let stderr = String::from_utf8_lossy(&seen.serve.stderr);
    assert!(
        stderr.lines().any(|line| line.contains("taking the handover stopped at") && line.contains("refused 401")),
        "operator diagnostic omitted TakeOver's step and reason: {stderr}"
    );
    running.stop();
    scratch.remove();
}

#[test]
fn final_newline_in_handover_is_not_part_of_token() {
    // configuration.md 6: one final newline is excluded from a secret.
    let (scratch, credentials, running) = prepared("qa-handover-newline");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let mut token = fs::read(&handover).expect("test handover");
    if token.last() != Some(&b'\n') {
        token.push(b'\n');
    }
    private_write(&handover, &token);
    let (_, rows) = inspect(scratch.path(), &credentials);
    connected(&rows);
    assert!(!handover.exists(), "newline-terminated handover was not consumed");
    running.stop();
    scratch.remove();
}

#[test]
fn embedded_newlines_cannot_turn_handover_into_http_headers() {
    // setup.md 6.5 and skein forge-setup.md 4.7: the entire file is the
    // credential. An HTTP-looking suffix cannot authenticate its valid prefix.
    let (scratch, mut credentials, running) = prepared("qa-handover-control-bytes");
    let handover = forge_directory(scratch.path()).join("admin_token");
    let mut token = fs::read(&handover).expect("test handover");
    let original = token.clone();
    token.extend_from_slice(b"\r\nX-QA-Handover: rejected\r\n");
    private_write(&handover, &token);
    credentials.push(token);
    let (_, rows) = inspect(scratch.path(), &credentials);
    assert!(handover.exists(), "malformed handover authenticated using only its prefix");
    assert!(forge_row(&rows)["state"] != "connected", "malformed handover connected");
    let value = document(scratch.path());
    let address: std::net::SocketAddr = value["application"]["machine"]["owned"]["listen"]["socket"]
        .as_str()
        .expect("listener")
        .parse()
        .expect("socket");
    let original = std::str::from_utf8(&original).expect("token text").trim_end();
    let user = owned::answer(address.port(), "/api/v1/user", Some(original)).expect("original admin check");
    assert!(user["is_admin"] == true, "malformed handover altered the valid admin token");
    running.stop();
    scratch.remove();
}

#[test]
fn empty_handover_stops_takeover_instead_of_looking_unprepared() {
    // skein forge-setup.md 4.3, 4.7: an empty given token stops Missing.
    let (scratch, credentials, running) = prepared("qa-handover-empty");
    let handover = forge_directory(scratch.path()).join("admin_token");
    private_write(&handover, b"");
    let (seen, rows) = inspect(scratch.path(), &credentials);
    assert!(handover.exists(), "empty handover was removed without TakeOver Done");
    assert!(forge_row(&rows)["state"] != "connected", "empty handover connected");
    assert!(
        String::from_utf8_lossy(&seen.serve.stderr).to_lowercase().contains("missing"),
        "empty handover did not name Missing"
    );
    running.stop();
    scratch.remove();
}

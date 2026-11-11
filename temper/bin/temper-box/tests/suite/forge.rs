use crate::support;
use std::{fmt::Write as _, fs, os::unix::fs::PermissionsExt, process::Command};
use temper_forge_setup_shell::process;
#[test]
fn prepare_uses_the_owned_machine_without_a_group_file() {
    let scratch = support::Scratch::new("forge_prepare_no_group");
    let _programs = support::owned::scripted(scratch.path());
    fs::remove_dir_all(scratch.path().join("box")).expect("no box files");
    let output = support::owned::prepare(scratch.path(), 1);
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert!(scratch.path().join("forge-1/forgejo/app.ini").is_file());
    assert!(!scratch.path().join("box").exists());
    scratch.remove();
}

#[test]
fn run_uses_the_owned_machine_without_a_group_file() {
    let scratch = support::Scratch::new("forge_run_no_group");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    fs::remove_dir_all(scratch.path().join("box")).expect("no box files");
    for program in ["forgejo", "runner"] {
        let mut run = support::owned::command(scratch.path(), program).spawn().expect("run starts");
        let marker = programs.marker(program, &mut run, support::WITHIN);
        run.kill().and_then(|()| run.wait()).expect("stopped");
        assert_eq!(marker.pid, run.id());
        assert!(!scratch.path().join("box").exists());
    }
    scratch.remove();
}

#[test]
fn a_second_preparation_is_refused_while_the_first_holds_the_lock() {
    let scratch = support::Scratch::new("forge_prepare_held");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let _lock = process::lock(&scratch.path().join("forge-1/forgejo.lock")).expect("first holds lock");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge prepare: forge 1: forgejo is running or being prepared already\n");
    scratch.remove();
}
#[test]
fn a_preparation_cut_short_is_begun_again_from_an_empty_layout() {
    let scratch = support::Scratch::new("forge_cut_short");
    let programs = support::owned::scripted(scratch.path());
    programs.fail("forgejo", "admin");
    let first = support::owned::prepare(scratch.path(), 1);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stderr, b"temper forge prepare: forge 1: CreateAdmin ended with status 1: broken\n");
    fs::write(scratch.path().join("forge-1/forgejo/left-over"), b"x").expect("left-over");
    programs.mend("forgejo");
    let second = support::owned::prepare(scratch.path(), 1);
    assert_eq!(second.status.code(), Some(0), "stderr: {}", String::from_utf8_lossy(&second.stderr));
    assert!(!scratch.path().join("forge-1/forgejo/left-over").exists());
    let state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"));
    assert!(state.expect("a state").prepared, "state ends last");
    scratch.remove();
}
#[test]
fn forgejo_files_with_no_state_are_refused_naming_the_directory() {
    let scratch = support::Scratch::new("forge_no_state");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    fs::remove_file(scratch.path().join("forge-1/state")).expect("no state");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        format!(
            "temper forge prepare: forge 1: {}: Forgejo files have no preparation state\n",
            scratch.path().join("forge-1").display()
        )
    );
    scratch.remove();
}
#[test]
fn a_later_preparation_takes_forgejos_lock_alone() {
    let scratch = support::Scratch::new("forge_later_lock");
    let _programs = support::owned::scripted(scratch.path());
    let first = support::owned::prepare(scratch.path(), 1);
    assert_eq!(first.status.code(), Some(0), "{}", String::from_utf8_lossy(&first.stderr));
    let _lock = process::lock(&scratch.path().join("forge-1/runner.lock")).expect("runner runs");
    let later = support::owned::prepare(scratch.path(), 1);
    assert_eq!(later.status.code(), Some(0), "{}", String::from_utf8_lossy(&later.stderr));
    scratch.remove();
}
#[test]
fn a_first_preparation_refuses_a_running_runner() {
    let scratch = support::Scratch::new("forge_first_runner_lock");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"))
        .expect("record");
    state.prepared = false;
    support::write_private(&scratch.path().join("forge-1/state"), temper_forge_setup::state::encode(&state).expose());
    let _lock = process::lock(&scratch.path().join("forge-1/runner.lock")).expect("runner runs");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge prepare: forge 1: runner is running or being prepared already\n");
    scratch.remove();
}
#[test]
fn a_forge_directory_open_to_others_is_refused_naming_it() {
    let scratch = support::Scratch::new("forge_directory_open");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    fs::set_permissions(scratch.path().join("forge-1"), fs::Permissions::from_mode(0o755)).expect("open directory");
    let refused = support::owned::prepare(scratch.path(), 1);
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        format!(
            "temper forge prepare: forge 1: {}: must be a private directory of this user's, not reached through a link\n",
            scratch.path().join("forge-1").display()
        )
    );
    scratch.remove();
}
#[test]
fn a_termination_signal_kills_the_commands_group_and_ends_the_preparation_interrupted() {
    let scratch = support::Scratch::new("forge_interrupted");
    let programs = support::owned::scripted(scratch.path());
    programs.hang("forgejo", "migrate");
    let mut run = Command::new(skein_scratch::paths::binary("temper-box"))
        .args(["forge", "prepare"])
        .args(support::configuration::flags(scratch.path()))
        .arg("1")
        .current_dir(scratch.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("preparation");
    let sleep = programs.hung(&mut run, support::WITHIN);
    assert!(
        Command::new("/bin/sh").args(["-c", &format!("kill -TERM {}", run.id())]).status().expect("signal").success()
    );
    let ended = run.wait_with_output().expect("preparation ends");
    assert_eq!(ended.status.code(), Some(1));
    assert_eq!(ended.stderr, b"temper forge prepare: forge 1: interrupted\n");
    let clock = skein_shell::Clock::new();
    let deadline = clock.now().now.saturating_add(support::WITHIN);
    let stat = loop {
        let stat = fs::read_to_string(format!("/proc/{sleep}/stat"));
        if stat.is_err()
            || stat.as_ref().is_ok_and(|text| text.rsplit_once(") ").is_some_and(|(_, rest)| rest.starts_with("Z ")))
        {
            break stat;
        }
        assert!(clock.now().now < deadline, "the sleep exits within bound");
        std::thread::yield_now();
    };
    assert!(stat.is_err() || stat.expect("stat").contains(") Z "), "the sleep is dead");
    let state = temper_forge_setup::state::decode(&fs::read(scratch.path().join("forge-1/state")).expect("state"))
        .expect("record");
    assert!(!state.prepared);
    scratch.remove();
}
#[test]
fn a_second_run_is_refused_while_the_first_holds_the_lock() {
    let scratch = support::Scratch::new("forge_run_held");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut first = support::owned::command(scratch.path(), "forgejo").spawn().expect("first run");
    programs.marker("forgejo", &mut first, support::WITHIN);
    let refused = support::owned::command(scratch.path(), "forgejo").output().expect("second run");
    first.kill().and_then(|()| first.wait()).expect("first stopped");
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(refused.stderr, b"temper forge run: forge 1: forgejo is running or being prepared already\n");
    scratch.remove();
}
#[test]
fn the_run_replaces_itself_holding_the_lock_as_standard_input() {
    let scratch = support::Scratch::new("forge_run_exec");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut run = support::owned::command(scratch.path(), "forgejo").spawn().expect("run starts");
    let marker = programs.marker("forgejo", &mut run, support::WITHIN);
    let stdin = fs::read_link(format!("/proc/{}/fd/0", run.id())).expect("stdin");
    run.kill().and_then(|()| run.wait()).expect("stopped");
    assert_eq!(marker.pid, run.id());
    assert_eq!(stdin, programs.lock("forgejo"));
    assert_eq!(
        marker.arguments,
        [
            "--work-path",
            scratch.path().join("forge-1/forgejo").to_str().expect("UTF-8 path"),
            "--config",
            scratch.path().join("forge-1/forgejo/app.ini").to_str().expect("UTF-8 path"),
            "web"
        ]
    );
    scratch.remove();
}
#[test]
fn the_new_process_sees_the_specs_environment_alone() {
    let scratch = support::Scratch::new("forge_run_environment");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut run = support::owned::command(scratch.path(), "forgejo").env("LEAKED", "private").spawn().expect("run");
    let mut marker = programs.marker("forgejo", &mut run, support::WITHIN);
    run.kill().and_then(|()| run.wait()).expect("stopped");
    marker.environment.sort();
    let directory = scratch.path().join("forge-1");
    let mut expected = vec![
        format!("HOME={}/home", directory.display()),
        "PATH=/usr/bin:/bin".into(),
        "TZ=UTC".into(),
        "GIT_CONFIG_NOSYSTEM=1".into(),
        format!("GIT_CONFIG_GLOBAL={}/home/gitconfig", directory.display()),
        format!("PWD={}/forgejo", directory.display()),
    ];
    expected.sort();
    assert_eq!(marker.environment, expected);
    scratch.remove();
}
#[test]
fn the_new_process_holds_no_descriptor_but_the_standard_three() {
    let scratch = support::Scratch::new("forge_run_descriptors");
    let programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let mut run = support::owned::command(scratch.path(), "runner").spawn().expect("run");
    let marker = programs.marker("runner", &mut run, support::WITHIN);
    run.kill().and_then(|()| run.wait()).expect("stopped");
    assert_eq!(marker.descriptors, [0, 1, 2]);
    assert_eq!(
        marker.arguments,
        ["--config", scratch.path().join("forge-1/runner/config.yaml").to_str().expect("UTF-8 path"), "daemon"]
    );
    scratch.remove();
}
#[test]
fn no_secret_of_the_state_reaches_a_refusal() {
    let scratch = support::Scratch::new("forge_run_secret_refusal");
    let _programs = support::owned::scripted(scratch.path());
    support::owned::finished(scratch.path());
    let path = scratch.path().join("forge-1/state");
    let mut state = temper_forge_setup::state::decode(&fs::read(&path).expect("state")).expect("record");
    state.prepared = false;
    support::write_private(&path, temper_forge_setup::state::encode(&state).expose());
    let refused = support::owned::command(scratch.path(), "forgejo").output().expect("refusal");
    assert_eq!(refused.status.code(), Some(1));
    assert_eq!(
        refused.stderr,
        format!("temper forge run: forge 1: its preparation did not end: run temper forge prepare --config {0} --data {0} --cache {0} 1\n", scratch.path().display()).as_bytes()
    );
    for secret in [&state.secrets.secret_key, &state.secrets.internal_token, &state.secrets.jwt_secret] {
        assert_eq!(skein_lib::bytes::find(&refused.stderr, secret.expose()), None);
    }
    scratch.remove();
}

#[test]
fn a_newer_recorded_forgejo_refuses_the_preparation_untouched() {
    use support::owned;
    let scratch = support::Scratch::new("forge_newer_prepare");
    let _programs = owned::scripted(scratch.path());
    assert!(owned::prepare(scratch.path(), 1).status.success());
    owned::record_versions(scratch.path(), Some(b"999.0.0"), None);
    let files = ["state", "forgejo/app.ini"];
    let before: Vec<_> = files
        .iter()
        .map(|name| fs::read(scratch.path().join("forge-1").join(name)).expect("upgrade fixture"))
        .collect();
    owned::refused(scratch.path(), "prepare", b"temper forge prepare: forge 1: forgejo is prepared at 999.0.0, newer than this temper's pin 17.0.1: its migrations only go forward\n");
    let after: Vec<_> = files
        .iter()
        .map(|name| fs::read(scratch.path().join("forge-1").join(name)).expect("upgrade fixture"))
        .collect();
    assert_eq!(before, after);
    assert_eq!(fs::read_dir(scratch.path().join("forge-1/backups")).expect("upgrade fixture").count(), 0);
    scratch.remove();
}
#[test]
fn the_run_refuses_a_newer_recorded_version_in_its_own_words() {
    use support::owned;
    let scratch = support::Scratch::new("forge_newer_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::record_versions(scratch.path(), Some(b"999.0.0"), None);
    owned::refused(scratch.path(), "run", b"temper forge run: forge 1: prepared at 999.0.0 / 13.2.1, this temper pins 17.0.1 / 13.2.1: no command of this temper runs it\n");
    scratch.remove();
}
#[test]
fn the_run_refuses_an_unreadable_recorded_version_in_its_own_words() {
    use support::owned;
    let scratch = support::Scratch::new("forge_unreadable_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::record_versions(scratch.path(), Some(b"private-looking-garbage"), None);
    owned::refused(scratch.path(), "run", b"temper forge run: forge 1: prepared at a version this temper cannot read: no command of this temper runs it\n");
    scratch.remove();
}
#[test]
fn the_run_names_prepare_for_a_runner_added() {
    use support::owned;
    let scratch = support::Scratch::new("forge_runner_added_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::recorded_without_runner(scratch.path());
    owned::refused(scratch.path(), "run", format!("temper forge run: forge 1: prepared without a runner, and the setup has one: run temper forge prepare --config {0} --data {0} --cache {0} 1\n", scratch.path().display()).as_bytes());
    scratch.remove();
}
#[test]
fn the_run_names_prepare_for_a_runner_removed() {
    use support::owned;
    let scratch = support::Scratch::new("forge_runner_removed_run");
    let _programs = owned::scripted(scratch.path());
    owned::finished(scratch.path());
    owned::setup_without_runner(scratch.path());
    owned::refused(scratch.path(), "run", format!("temper forge run: forge 1: prepared with a runner, and the setup has none: run temper forge prepare --config {0} --data {0} --cache {0} 1\n", scratch.path().display()).as_bytes());
    scratch.remove();
}
#[test]
fn an_interrupted_upgrade_leaves_the_state_for_the_next_preparation() {
    use support::owned;
    let scratch = support::Scratch::new("forge_upgrade_interrupted");
    let programs = owned::scripted(scratch.path());
    assert!(owned::prepare(scratch.path(), 1).status.success());
    owned::record_versions(scratch.path(), Some(b"16.0.5"), None);
    let path = owned::file(scratch.path(), 1, temper_forge_setup::place::File::State);
    let before = fs::read(&path).expect("old state");
    programs.hang("forgejo", "dump");
    let mut run = Command::new(skein_scratch::paths::binary("temper-box"))
        .args(["forge", "prepare"])
        .args(support::configuration::flags(scratch.path()))
        .arg("1")
        .current_dir(scratch.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("prepare");
    let _sleep = programs.hung(&mut run, support::WITHIN);
    assert!(
        Command::new("/bin/sh")
            .args(["-c", &format!("kill -TERM {}", run.id())])
            .status()
            .expect("upgrade fixture")
            .success()
    );
    let ended = run.wait_with_output().expect("interrupted");
    assert_eq!(ended.status.code(), Some(1));
    assert_eq!(ended.stderr, b"temper forge prepare: forge 1: interrupted\n");
    assert_eq!(fs::read(path).expect("old state kept"), before);
    assert_eq!(fs::read_dir(scratch.path().join("forge-1/backups")).expect("upgrade fixture").count(), 0);
    scratch.remove();
}

#[test]
fn fake_preparation_writes_only_a_private_token_and_keeps_existing_bytes() {
    for existing in [None, Some(b"existing token\n".as_slice())] {
        let scratch = support::Scratch::new("fake_prepare");
        fake_configuration(scratch.path());
        let directory = scratch.path().join("playground-1");
        let token = directory.join("admin_token");
        if let Some(bytes) = existing {
            fs::create_dir(&directory).expect("directory");
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("private");
            support::write_private(&token, bytes);
        }
        let output = support::owned::prepare(scratch.path(), 1);
        assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        assert_eq!(fs::metadata(&directory).expect("directory").permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::read_dir(&directory).expect("directory").count(), 1);
        let bytes = fs::read(&token).expect("token");
        if let Some(expected) = existing {
            assert_eq!(bytes, expected);
        } else {
            assert_eq!(bytes.len(), 40);
            assert!(bytes.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b)));
            assert_eq!(fs::metadata(&token).expect("token").permissions().mode() & 0o777, 0o600);
        }
        scratch.remove();
    }
}
#[test]
fn fake_preparation_refuses_an_open_directory_and_run_still_refuses_fake() {
    for (verb, names_directory, reason) in [
        ("prepare", true, "must be a private directory of this user's, not reached through a link"),
        ("run", false, "it is fake, not owned"),
    ] {
        let scratch = support::Scratch::new("fake_prepare_refused");
        fake_configuration(scratch.path());
        let directory = scratch.path().join("playground-1");
        fs::create_dir(&directory).expect("directory");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o755)).expect("open directory");
        let location = if names_directory { format!("{}: ", directory.display()) } else { String::new() };
        support::owned::refused(
            scratch.path(),
            verb,
            format!("temper forge {verb}: forge 1: {location}{reason}\n").as_bytes(),
        );
        assert_eq!(fs::read_dir(directory).expect("untouched directory").count(), 0);
        scratch.remove();
    }
}

fn fake_configuration(directory: &std::path::Path) {
    let config = support::configuration::check_document(directory);
    fs::create_dir_all(directory.join("box/groups")).expect("groups");
    support::write_private(&directory.join("box/groups/1"), b"fake\n");
    fs::write(directory.join("temper.json"), serde_json::to_vec(&config).expect("document")).expect("document");
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table of complete fetch command diagnostics and file outcomes")]
fn forge_fetch_command() {
    use skein_lib::sha256::Sha256;
    use support::{configuration, owned};
    for row in ["fake", "paths", "digest", "present", "present-no-group", "missing-roots", "usage"] {
        let scratch = support::Scratch::new(&format!("forge-fetch-{row}"));
        owned::configuration(
            scratch.path(),
            &owned::Names {
                forgejo: std::path::Path::new("/opt/forgejo/forgejo"),
                runner: None,
                port: 3000,
                bot: "temper",
                person: "ada",
            },
        );
        let path = scratch.path().join("temper.json");
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("document")).expect("JSON");
        fs::rename(scratch.path().join("box/groups/1"), scratch.path().join("box/groups/3")).expect("numbered group");
        if row == "present-no-group" {
            fs::remove_dir_all(scratch.path().join("box")).expect("no box files");
        }
        let mut server = None;
        let root = scratch.path().join("root.pem");
        let file = scratch.path().join("forgejo/17.0.1/forgejo");
        match row {
            "fake" => {
                configuration::fake_entry(&mut document["application"]["machine"]["fake"], "127.0.0.1:3001");
                support::write_private(&scratch.path().join("box/groups/3"), b"fake\n");
            }
            "digest" | "present" | "present-no-group" | "missing-roots" => {
                document["application"]["machine"]["owned"]["forgejo"] = "fetched".into();
                if row == "digest" {
                    let peer = support::fetch::server();
                    let origin = format!("https://{}", peer.address);
                    document["application"]["machine"]["owned"]["mirror"] =
                        serde_json::json!({"forgejo":origin,"runner":origin});
                    server = Some(peer);
                }
                if row == "present" || row == "present-no-group" {
                    fs::create_dir_all(file.parent().expect("cache parent")).expect("parents");
                    fs::write(&file, b"mine").expect("person's file");
                }
            }
            "paths" | "usage" => {}
            _ => unreachable!(),
        }
        fs::write(&path, serde_json::to_vec(&document).expect("JSON")).expect("document");
        if row != "missing-roots" {
            support::fetch::roots(&root);
        }
        let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
        command.args(["forge", "fetch"]).args(configuration::flags(scratch.path()));
        if row != "usage" {
            command.arg("3");
        }
        let result = command.env("SSL_CERT_FILE", &root).current_dir(scratch.path()).output().expect("fetch command");
        assert_eq!(result.status.code(), Some(if row == "usage" { 2 } else { 1 }), "{row}");
        let expected_stdout =
            if row == "digest" { "fetched forgejo 0 of 3 bytes\nfetched forgejo 3 of 3 bytes\n" } else { "" };
        assert_eq!(result.stdout, expected_stdout.as_bytes(), "{row}");
        if let Some(peer) = &mut server {
            peer.check();
        }
        let expected = match row {
            "managed" => "temper forge fetch: forge 3: it is managed, not owned\n".into(),
            "fake" => "temper forge fetch: forge 3: it is fake, not owned\n".into(),
            "paths" => "temper forge fetch: forge 3: neither of its binaries is \"fetched\"\n".into(),
            "digest" => {
                let peer = server.as_ref().expect("local peer");
                let mut hash = Sha256::new();
                hash.update(b"abc");
                format!(
                    "temper forge fetch: forge 3: https://{}/forgejo/forgejo/releases/download/v17.0.1/forgejo-17.0.1-linux-{}: digest differs: pinned {}, got {}\n",
                    peer.address,
                    match std::env::consts::ARCH {
                        "x86_64" => "amd64",
                        "aarch64" => "arm64",
                        _ => panic!("test host has pinned architecture"),
                    },
                    skein_forge::forgejo::pin::digest(
                        skein_forge::forgejo::pin::Binary::Forgejo,
                        skein_forge::forgejo::pin::Platform::Linux,
                        match std::env::consts::ARCH {
                            "x86_64" => "amd64",
                            "aarch64" => "arm64",
                            _ => unreachable!(),
                        },
                        skein_forge::forgejo::pin::Form::Binary,
                    )
                    .expect("pin"),
                    String::from_utf8_lossy(&temper_forge_setup::fetch::hex(&hash.finish()))
                )
            }
            "present" | "present-no-group" => {
                format!("temper forge fetch: forge 3: {}: does not match the pinned digest\n", file.display())
            }
            "missing-roots" => format!("temper forge fetch: forge 3: {}: roots: Absent\n", root.display()),
            "usage" => {
                "temper-box forge fetch: invalid arguments\nusage: temper-box forge fetch [DIRECTORIES] FORGE\n".into()
            }
            _ => unreachable!(),
        };
        assert_eq!(result.status.code(), Some(if row == "usage" { 2 } else { 1 }), "{row}");
        assert_eq!(result.stdout, expected_stdout.as_bytes(), "{row}");
        assert_eq!(String::from_utf8_lossy(&result.stderr), expected, "{row}");
        assert!(!file.with_file_name("forgejo.part").exists(), "{row}");
        if row == "present-no-group" {
            assert!(!scratch.path().join("box").exists());
        }
        if row == "present" || row == "present-no-group" {
            assert_eq!(fs::read(&file).expect("person's file kept"), b"mine");
        } else {
            assert!(!file.exists(), "{row}");
        }
        scratch.remove();
    }
    assert_eq!(
        temper_box_shell::TABLE
            .commands
            .iter()
            .find(|row| row.reach == skein_shell::Reach::Words(&["forge", "fetch"]))
            .expect("fetch row")
            .forms,
        ["[DIRECTORIES] FORGE"]
    );
}

#[test]
fn forge_sign_in_refuses_without_openssl_a_named_forge_or_a_provider() {
    use support::forge_sign_in;
    let scratch = support::Scratch::new("forge_sign_in_refusals");
    let directory = scratch.path();
    let _programs = support::owned::scripted(directory);
    forge_sign_in::configure(directory, "127.0.0.1:3000", "127.0.0.1:31000", "127.0.0.1:31999");
    let path = directory.join("temper.json");
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    for (case, expected) in [(0, "openssl"), (1, "application.sign_in.forges"), (2, "engine.clients.provider")] {
        let mut changed = value.clone();
        if case == 1 {
            changed["application"]["sign_in"]["forges"] = serde_json::json!([]);
        }
        if case == 2 {
            changed["engine"]["clients"]["provider"] = serde_json::Value::Null;
        }
        support::write_private(&path, &serde_json::to_vec(&changed).expect("JSON"));
        let output =
            forge_sign_in::command(directory).env("PATH", directory.join("no-programs")).output().expect("refusal");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!directory.join("provider.key").exists());
        assert!(!directory.join("forgejo.secret").exists());
        assert!(
            fs::read_dir(directory).expect("scratch entries").all(|entry| !entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .ends_with(".partial"))
        );
    }
    scratch.remove();
}

#[test]
fn forge_sign_in_prints_the_owned_command_with_public_forge_and_temper_origins() {
    use support::forge_sign_in;
    let scratch = support::Scratch::new("forge_sign_in_public");
    let directory = scratch.path();
    let _programs = support::owned::scripted(directory);
    forge_sign_in::configure(directory, "127.0.0.1:3000", "127.0.0.1:31000", "127.0.0.1:31999");
    let path = directory.join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["application"]["machine"]["owned"]["public"] = "https://git.example.com".into();
    value["application"]["clients"]["public"] = "https://temper.example.com".into();
    value["engine"]["clients"]["provider"]["clients"][0]["redirect_uri"] =
        "https://git.example.com/user/oauth2/temper/callback".into();
    support::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
    let output = forge_sign_in::command(directory).output().expect("sign-in command");
    forge_sign_in::success(&output);
    assert_eq!(
        output.stdout,
        temper_forge_setup::sign_in::instructions(
            b"forgejo",
            None,
            b"forgejo",
            directory.join("forgejo.secret").as_os_str().as_encoded_bytes(),
            b"https://temper.example.com/.well-known/openid-configuration",
            temper_forge_setup::sign_in::FirstSignIn::MakesAccount,
        )
        .as_ref()
    );
    assert!(output.stderr.is_empty());
    scratch.remove();
}

#[test]
fn forge_sign_in_prints_the_owned_command_without_its_private_secret() {
    use support::forge_sign_in;
    let scratch = support::Scratch::new("forge_sign_in_prints");
    let directory = scratch.path();
    let _programs = support::owned::scripted(directory);
    forge_sign_in::configure(directory, "127.0.0.1:3000", "127.0.0.1:31000", "127.0.0.1:31999");
    let output = forge_sign_in::command(directory).output().expect("sign-in command");
    forge_sign_in::success(&output);
    let secret = fs::read(directory.join("forgejo.secret")).expect("scratch secret");
    assert_eq!(secret.len(), 43);
    assert!(secret.iter().all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(byte)));
    for bytes in [&output.stdout, &output.stderr] {
        assert!(!bytes.windows(secret.len()).any(|part| part == secret));
    }
    assert_eq!(
        output.stdout,
        temper_forge_setup::sign_in::instructions(
            b"forgejo",
            None,
            b"forgejo",
            directory.join("forgejo.secret").as_os_str().as_encoded_bytes(),
            b"http://127.0.0.1:31000/.well-known/openid-configuration",
            temper_forge_setup::sign_in::FirstSignIn::MakesAccount,
        )
        .as_ref()
    );
    for name in ["provider.key", "forgejo.secret"] {
        assert!(support::private(&directory.join(name)));
    }
    let repeated = forge_sign_in::command(directory).output().expect("repeat sign-in command");
    forge_sign_in::success(&repeated);
    assert_eq!(repeated.stdout, output.stdout);
    assert!(repeated.stderr.is_empty());
    assert!(fs::read(directory.join("forgejo.secret")).expect("same scratch secret") == secret);
    scratch.remove();
}

#[test]
fn forge_no_registration_follows_the_switch_on_every_preparation_and_print() {
    use support::{forge_sign_in, owned};
    let scratch = support::Scratch::new("forge_no_registration");
    let directory = scratch.path();
    let _programs = owned::scripted(directory);
    forge_sign_in::configure(directory, "127.0.0.1:3000", "127.0.0.1:31000", "127.0.0.1:31999");
    let path = directory.join("forge-1/forgejo/app.ini");
    forge_sign_in::success(&owned::prepare(directory, 1));
    let original = fs::read(&path).expect("initial ini");
    let external = b"[service]\nDISABLE_REGISTRATION = true\nALLOW_ONLY_EXTERNAL_REGISTRATION = true\n[oauth2_client]\nENABLE_AUTO_REGISTRATION = true\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n[mailer]";
    assert!(original.windows(external.len()).any(|part| part == external));
    forge_sign_in::success(&owned::prepare_no_registration(directory, 1));
    let closed = fs::read(&path).expect("closed ini");
    let links = b"[service]\nDISABLE_REGISTRATION = true\n[oauth2_client]\nENABLE_AUTO_REGISTRATION = false\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n[mailer]";
    assert!(closed.windows(links.len()).any(|part| part == links));
    assert!(
        !closed
            .windows(b"ALLOW_ONLY_EXTERNAL_REGISTRATION".len())
            .any(|part| part == b"ALLOW_ONLY_EXTERNAL_REGISTRATION")
    );
    forge_sign_in::success(&owned::prepare_no_registration(directory, 1));
    assert!(fs::read(&path).expect("repeated ini") == closed);
    let empty = support::story_command::standalone(
        owned::prepare_command(directory, 1).env("TEMPER_INTERNAL_FORGE_NO_REGISTRATION", ""),
        "preparation with an empty registration switch",
        support::boxed::BOX_DEADLINE,
    );
    forge_sign_in::success(&empty);
    assert!(fs::read(&path).expect("empty switch ini") == original);
    let output = forge_sign_in::command(directory)
        .env("TEMPER_INTERNAL_FORGE_NO_REGISTRATION", "1")
        .output()
        .expect("sign-in command");
    forge_sign_in::success(&output);
    let settings = temper_forge_setup::sign_in::LINKS_ONLY_SETTINGS;
    assert!(output.stdout.windows(settings.len()).any(|part| part == settings));
    assert!(
        !output
            .stdout
            .windows(b"ALLOW_ONLY_EXTERNAL_REGISTRATION".len())
            .any(|part| part == b"ALLOW_ONLY_EXTERNAL_REGISTRATION")
    );

    // A forge absent from sign_in.forges keeps its settings even with the switch.
    owned::configuration_value(directory, |value| value["application"]["sign_in"]["forges"] = serde_json::json!([]));
    forge_sign_in::success(&owned::prepare(directory, 1));
    let unnamed = fs::read(&path).expect("unnamed forge ini");
    forge_sign_in::success(&owned::prepare_no_registration(directory, 1));
    assert!(fs::read(&path).expect("unnamed closed ini") == unnamed);
    assert!(!unnamed.windows(b"[oauth2_client]".len()).any(|part| part == b"[oauth2_client]"));
    scratch.remove();
}

/// The public client and owner written by init --box; no provider key is read by prepare.
fn source_configuration(directory: &std::path::Path, port: u16) {
    let path = directory.join("temper.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
    value["engine"]["owners"] = serde_json::json!([{"provider":1,"subject":"ada@example.invalid"}]);
    value["engine"]["clients"]["provider"] = serde_json::json!({
        "key":"provider.key", "addresses":[1],
        "clients":[{"id":"forgejo","secret":null,"redirect_uri":"http://127.0.0.1:3000/user/oauth2/temper/callback"}]
    });
    value["application"]["clients"]["listener"] = serde_json::json!({"socket":format!("127.0.0.1:{port}")});
    value["application"]["machine"]["owned"]["sign_in"] = "forgejo".into();
    support::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
}
fn source_script(directory: &std::path::Path, extra: &str) {
    let path = directory.join("forgejo");
    let body = fs::read_to_string(&path).expect("script");
    let body = body.strip_prefix("#!/bin/sh\n").expect("shell program");
    let prefix = format!(
        r#"#!/bin/sh
base='{}'
{}
if [ "$5 $6 $7" = 'admin auth list' ]; then
    printf '%s\n' "$@" >> "$base/source-arguments"
    printf 'ID Name Type Enabled\n27 temper OAuth2 true\n'
    exit 0
fi
if [ "$5 $6 $7" = 'admin auth update-oauth' ]; then
    printf '%s\n' "$@" >> "$base/source-arguments"
    exit 0
fi
"#,
        directory.display(),
        extra
    );
    fs::write(&path, format!("{prefix}{body}")).expect("source script");
}
#[test]
fn a_later_prepare_gives_a_listed_source_the_current_address() {
    let scratch = support::Scratch::new("forge_source_address");
    let directory = scratch.path();
    let _programs = support::owned::scripted(directory);
    source_script(directory, "");
    source_configuration(directory, 31000);
    support::forge_sign_in::success(&support::owned::prepare(directory, 1));
    assert!(!directory.join("source-arguments").exists(), "first preparation has no source");
    let mut expected = String::new();
    for port in [31000, 31001] {
        source_configuration(directory, port);
        let output = support::owned::prepare(directory, 1);
        support::forge_sign_in::success(&output);
        assert!(output.stderr.is_empty(), "successful maintenance is silent");
        let prefix = format!(
            "--work-path\n{0}/forge-1/forgejo\n--config\n{0}/forge-1/forgejo/app.ini\nadmin\nauth\n",
            directory.display()
        );
        write!(expected, "{prefix}list\n{prefix}update-oauth\n--id\n27\n--use-custom-urls\nfalse\n--auto-discover-url\nhttp://127.0.0.1:{port}/.well-known/openid-configuration\n").expect("source arguments");
        assert_eq!(fs::read_to_string(directory.join("source-arguments")).expect("source commands"), expected);
    }
    scratch.remove();
}
#[test]
fn a_refused_update_leaves_prepare_succeeding_with_one_line() {
    let scratch = support::Scratch::new("forge_source_refused");
    let directory = scratch.path();
    let _programs = support::owned::scripted(directory);
    source_configuration(directory, 31000);
    support::forge_sign_in::success(&support::owned::prepare(directory, 1));
    source_script(
        directory,
        r#"if [ "$5 $6 $7" = 'admin auth update-oauth' ]; then printf 'refused\nsecond line\n' >&2; exit 1; fi"#,
    );
    let output = support::owned::prepare(directory, 1);
    support::forge_sign_in::success(&output);
    assert_eq!(output.stderr, b"forge source: discovery address not updated: refused second line \n");
    assert!(output.stdout.is_empty());
    scratch.remove();
}

#[test]
fn source_deadlines_and_spawn_failures_leave_preparation_succeeding() {
    for (command, spawn) in [("list", false), ("update-oauth", false), ("list", true), ("update-oauth", true)] {
        let scratch = support::Scratch::new("forge_source_host_failure");
        let directory = scratch.path();
        let _programs = support::owned::scripted(directory);
        source_configuration(directory, 31000);
        support::forge_sign_in::success(&support::owned::prepare(directory, 1));
        let extra = if spawn && command == "list" {
            format!(
                "if [ \"$1\" = --version ]; then chmod 600 \"$0\"; echo 'forgejo version {}'; exit 0; fi",
                skein_forge::forgejo::pin::VERSION
            )
        } else if spawn {
            r#"if [ "$5 $6 $7" = 'admin auth list' ]; then chmod 600 "$0"; printf '27 temper OAuth2 true\n'; exit 0; fi"#.into()
        } else {
            format!("if [ \"$5 $6 $7\" = 'admin auth {command}' ]; then exec sleep 60; fi")
        };
        source_script(directory, &extra);
        let path = directory.join("temper.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("JSON");
        for (name, number) in [("setup.deadline_ms", 200_u64), ("setup.backoff_ms", 1), ("setup.backoff_max_ms", 100)] {
            value["application"]["limits"]["overrides"][name] = number.into();
        }
        support::write_private(&path, &serde_json::to_vec(&value).expect("JSON"));
        let output = support::owned::prepare(directory, 1);
        support::forge_sign_in::success(&output);
        let line = String::from_utf8(output.stderr).expect("source diagnostic");
        assert_eq!(line.lines().count(), 1, "{command}, spawn={spawn}: {line}");
        assert!(line.starts_with("forge source: discovery address not updated: "));
        assert!(line.contains(if spawn { "spawn:" } else { "source deadline expired" }), "{line}");
        assert!(output.stdout.is_empty());
        scratch.remove();
    }
}
/// Fake HTTP peers share this test's loop with the source command.
fn source_answer(
    case: &str,
    discovery: Option<u16>,
    listed: bool,
    response: &str,
) -> (std::process::Output, String, Vec<String>) {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        process::Stdio,
    };
    let scratch = support::Scratch::new(case);
    let directory = scratch.path();
    let _programs = support::owned::scripted(directory);
    fs::create_dir_all(directory.join("forge-1/forgejo")).expect("source work path");
    let issuer = TcpListener::bind("127.0.0.1:0").expect("issuer peer");
    let forge = TcpListener::bind("127.0.0.1:0").expect("Forgejo peer");
    let issuer_address = issuer.local_addr().expect("issuer address");
    let forge_address = forge.local_addr().expect("Forgejo address");
    source_configuration(directory, issuer_address.port());
    let path = directory.join("temper.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("source configuration")).expect("source JSON");
    value["application"]["machine"]["owned"]["listen"] = serde_json::json!({"socket":forge_address.to_string()});
    value["engine"]["clients"]["provider"]["clients"][0]["redirect_uri"] =
        format!("http://{forge_address}/user/oauth2/temper/callback").into();
    support::write_private(&path, &serde_json::to_vec(&value).expect("source JSON"));
    let program = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nif [ \"$5 $6 $7\" = 'admin auth list' ]; then printf '{}'; fi\n",
        directory.join("source-arguments").display(),
        if listed { "27 temper OAuth2 true\\n" } else { "ID Name Type Enabled\\n" }
    );
    fs::write(directory.join("forgejo"), program).expect("fake Forgejo");
    issuer.set_nonblocking(true).expect("issuer peer polling");
    forge.set_nonblocking(true).expect("Forgejo peer polling");
    let issuer = discovery.map(|status| (issuer, format!("HTTP/1.1 {status} Answer\r\nContent-Length: 2\r\n\r\n{{}}")));
    let forge_response = response.replace("ISSUER", &format!("http://{issuer_address}"));
    let mut peers = Vec::new();
    if let Some(peer) = issuer {
        peers.push(peer);
    }
    peers.push((forge, forge_response));
    let mut child = Command::new(skein_scratch::paths::binary("temper-box"))
        .args(["forge", "source"])
        .args(support::configuration::flags(directory))
        .arg("1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("source command");
    let mut seen = Vec::new();
    let mut deadline = skein_world::Deadline::arm("the source and its fake peers", support::WITHIN);
    deadline.watch_process(child.id());
    loop {
        for (listener, response) in &peers {
            if let Ok((socket, _)) = listener.accept() {
                let mut socket = support::socket::Socket::owned(
                    socket,
                    support::WITHIN,
                    ["the source peer's request", "the source peer's answer"],
                );
                let mut head = Vec::new();
                while !head.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    socket.read_exact(&mut byte).expect("source peer request");
                    head.push(byte[0]);
                    assert!(head.len() <= 16_384, "source request head bound");
                }
                seen.push(String::from_utf8(head).expect("source request text"));
                socket.write_all(response.as_bytes()).expect("source peer answer");
            }
        }
        if child.try_wait().expect("source exit").is_some() {
            break;
        }
        if deadline.passed() {
            support::parent::end_child(&mut child);
            match deadline.expired() {
                skein_world::Expired::Starved(starved) => starved.pass(),
                skein_world::Expired::Failed(figures) => panic!("source did not finish: {figures}"),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let output = child.wait_with_output().expect("source output");
    let commands = fs::read_to_string(directory.join("source-arguments")).unwrap_or_default();
    scratch.remove();
    (output, commands, seen)
}

#[test]
fn source_reads_discovery_before_running_any_forgejo_command() {
    for discovery in [None, Some(502)] {
        let (output, commands, _) = source_answer("forge_source_discovery", discovery, false, "");
        support::forge_sign_in::success(&output);
        assert_eq!(output.stdout, b"source failed temper's discovery did not answer\n");
        assert!(commands.is_empty(), "discovery failure must not run Forgejo: {commands}");
    }
}

#[test]
fn source_adds_an_absent_source_only_after_discovery_answers() {
    let (output, commands, seen) = source_answer("forge_source_absent", Some(200), false, "");
    support::forge_sign_in::success(&output);
    assert_eq!(output.stdout, b"source added\n");
    assert!(commands.contains("add-oauth\n--name\ntemper\n"), "{commands}");
    assert_eq!(seen.len(), 1, "only discovery is read for an absent source");
    assert!(seen[0].starts_with("GET /.well-known/openid-configuration "));
}

#[test]
fn a_listed_source_is_added_only_when_its_sign_in_redirects_to_the_issuer() {
    for (status, location, added) in [
        (307, "ISSUER/oauth/authorize?client_id=forgejo", true),
        (302, "ISSUER/oauth/authorize", true),
        (307, "ISSUER/oauth/authorize#continue", true),
        (307, "", false),
        (307, "/oauth/authorize", false),
        (200, "ISSUER/oauth/authorize", false),
        (500, "", false),
        (307, "https://other.invalid/oauth/authorize", false),
        (307, "ISSUER/oauth/authorize-other", false),
        (307, "ISSUER/oauth/authorize/child", false),
    ] {
        let response = format!("HTTP/1.1 {status} Answer\r\nLocation: {location}\r\nContent-Length: 0\r\n\r\n");
        let (output, commands, seen) = source_answer("forge_source_listed", Some(200), true, &response);
        support::forge_sign_in::success(&output);
        assert_eq!(
            output.stdout,
            if added { b"source added\n".as_slice() } else { b"source failed Forgejo does not serve the source\n" },
            "{status} {location}"
        );
        assert!(!commands.contains("add-oauth"), "listed sources are not added again");
        assert_eq!(seen.len(), 2, "discovery then the source; no redirect followed");
        assert!(seen[0].starts_with("GET /.well-known/openid-configuration "));
        assert!(seen[1].starts_with("GET /user/oauth2/temper "));
    }
}

#[test]
fn a_termination_signal_during_source_maintenance_still_stops_preparation() {
    let scratch = support::Scratch::new("forge_source_interrupted");
    let directory = scratch.path();
    let programs = support::owned::scripted(directory);
    source_configuration(directory, 31000);
    support::forge_sign_in::success(&support::owned::prepare(directory, 1));
    source_script(
        directory,
        r#"if [ "$5 $6 $7" = 'admin auth list' ]; then sleep 60 & echo $! > "$base/hung"; wait; exit; fi"#,
    );
    let mut child = support::owned::prepare_command(directory, 1)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("preparation");
    let _sleep = programs.hung(&mut child, support::WITHIN);
    assert!(
        Command::new("/bin/sh")
            .args(["-c", &format!("kill -TERM {}", child.id())])
            .status()
            .expect("terminate preparation")
            .success()
    );
    let output = child.wait_with_output().expect("interrupted preparation ends");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stderr, b"temper forge prepare: forge 1: interrupted\n");
    scratch.remove();
}

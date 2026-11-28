use crate::support;
use std::{fs, process::Command};
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
    assert_eq!(ended.stderr, b"temper-box forge prepare: forge 1: interrupted\n");
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
fn the_run_replaces_itself_holding_the_lock_as_standard_input() {
    use support::owned;
    let scratch = support::Scratch::new("forge_run_exec");
    let root = scratch.path();
    let programs = owned::scripted(root);
    owned::finished(root);
    fs::remove_dir_all(root.join("box")).expect("no box files");
    let directory = root.join("forge-1");
    let work = directory.join("forgejo");
    let rows = [
        (
            "forgejo",
            vec![
                "--work-path".into(),
                work.display().to_string(),
                "--config".into(),
                work.join("app.ini").display().to_string(),
                "web".into(),
            ],
            vec![
                format!("HOME={}/home", directory.display()),
                "PATH=/usr/bin:/bin".into(),
                "TZ=UTC".into(),
                "GIT_CONFIG_NOSYSTEM=1".into(),
                format!("GIT_CONFIG_GLOBAL={}/home/gitconfig", directory.display()),
                format!("PWD={}", work.display()),
            ],
        ),
        (
            "runner",
            vec!["--config".into(), directory.join("runner/config.yaml").display().to_string(), "daemon".into()],
            vec![
                format!("HOME={}/runner", directory.display()),
                "PATH=/usr/bin:/bin".into(),
                format!("PWD={}", work.display()),
            ],
        ),
    ];
    let launched = rows.map(|(selector, argv, environment)| {
        let mut command = owned::command(root, selector);
        command.env("LEAKED", "private");
        let mut child = support::fixture::spawn(&mut command, skein_io::kernel::Signal::Kill).expect("run starts");
        let marker = programs.marker(selector, &mut child, support::WITHIN);
        let stdin_is_lock = owned::stdin_is(child.id(), &programs.lock(selector));
        let box_exists = root.join("box").exists();
        (selector, child, marker, stdin_is_lock, box_exists, argv, environment)
    });
    let refused_run = owned::command(root, "forgejo").output().expect("second run");
    let refused_prepare = owned::prepare(root, 1);

    for (selector, mut child, mut marker, stdin_is_lock, box_exists, argv, mut environment) in launched {
        child.kill().and_then(|()| child.wait()).expect("stopped");
        marker.environment.sort();
        environment.sort();
        assert_eq!(marker.pid, child.id(), "{selector} replaces itself");
        assert!(stdin_is_lock, "{selector} holds its lock as stdin");
        assert_eq!(marker.arguments, argv, "{selector}");
        assert_eq!(marker.environment, environment, "{selector}");
        assert_eq!(marker.descriptors, [0, 1, 2], "{selector}");
        assert!(!box_exists, "{selector} does not make a box tree");
    }
    assert_eq!(refused_run.status.code(), Some(1));
    assert_eq!(refused_run.stderr, b"temper-box forge run: forge 1: forgejo is running or being prepared already\n");
    assert_eq!(refused_prepare.status.code(), Some(1));
    assert_eq!(
        refused_prepare.stderr,
        b"temper-box forge prepare: forge 1: forgejo is running or being prepared already\n"
    );
    assert!(!root.join("box").exists());
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
    assert_eq!(ended.stderr, b"temper-box forge prepare: forge 1: interrupted\n");
    assert_eq!(fs::read(path).expect("old state kept"), before);
    assert_eq!(fs::read_dir(scratch.path().join("forge-1/backups")).expect("upgrade fixture").count(), 0);
    scratch.remove();
}

#[test]
fn forge_fetch_command() {
    use skein_forge::forgejo::pin::Platform;
    use skein_lib::sha256::Sha256;
    use support::{configuration, owned};
    use temper_forge_setup::{
        fetch::{hex, pins},
        machine::BoxKind,
        setup::Mode,
    };

    let scratch = support::Scratch::new("forge-fetch-digest");
    let directory = scratch.path();
    owned::configuration(
        directory,
        &owned::Names {
            forgejo: std::path::Path::new("/opt/forgejo/forgejo"),
            runner: None,
            port: 3000,
            bot: "temper",
            person: "ada",
        },
    );
    fs::rename(directory.join("box/groups/1"), directory.join("box/groups/3")).expect("numbered group");
    let mut setup = owned::setup_value(directory, 1, "owned");
    let mut peer = support::fetch::server();
    let origin = format!("https://{}", peer.address);
    setup["forgejo"] = "fetched".into();
    setup["mirror"] = serde_json::json!({"forgejo":origin,"runner":origin});
    configuration::setup(directory, 3, setup);
    let directories = temper_service_shell::jig_directories(&configuration::directories(directory));
    let setup = temper_forge_setup_shell::setup_file::read(&directories, 3, BoxKind::Owned).expect("mirrored setup");
    let Mode::Owned(owned) = setup.entry(3, &directories).expect("forge entry").mode else { panic!("owned forge") };
    let platform = if cfg!(target_os = "macos") { Platform::Macos } else { Platform::Linux };
    let pins = pins(&owned, platform, std::env::consts::ARCH).expect("host pins");
    let [pin] = pins.as_ref() else { panic!("only Forgejo is fetched") };

    let root = directory.join("root.pem");
    support::fetch::roots(&root);
    let result = Command::new(skein_scratch::paths::binary("temper-box"))
        .args(["forge", "fetch"])
        .args(configuration::flags(directory))
        .arg("3")
        .env("SSL_CERT_FILE", &root)
        .current_dir(directory)
        .output()
        .expect("fetch command");
    assert_eq!(result.status.code(), Some(1));
    let bytes = peer.bytes;
    assert_eq!(
        result.stdout,
        format!("fetched forgejo 0 of {bytes} bytes\nfetched forgejo {bytes} of {bytes} bytes\n").as_bytes()
    );
    peer.check(pin);

    let pinned = hex(&pin.digest);
    let mut hash = Sha256::new();
    hash.update(b"abc");
    let got = hex(&hash.finish());
    let path = std::str::from_utf8(&pin.path).expect("pinned request path");
    assert_eq!(
        String::from_utf8_lossy(&result.stderr),
        format!(
            "temper-box forge fetch: forge 3: {origin}{path}: digest differs: pinned {}, got {}\n",
            String::from_utf8_lossy(&pinned),
            String::from_utf8_lossy(&got)
        ),
    );
    let file = directory.join("forgejo/17.0.1/forgejo");
    assert!(!file.with_file_name("forgejo.part").exists());
    assert!(!file.with_file_name("forgejo.zst.part").exists());
    assert!(!file.exists());
    scratch.remove();
}

/// The public client and owner written by init --box; no provider key is read by prepare.
fn source_configuration(directory: &std::path::Path, port: u16) {
    let path = directory.join(temper_config_shell::FILE);
    let mut file = support::read_file(&path, temper_config_shell::Shape::Standalone);
    file.owner = Some("ada@example.invalid".into());
    file.openid = Some(temper_config_shell::OpenId {
        key: Some("provider.key".into()),
        addresses: Some(vec![1]),
        clients: vec![temper_config_shell::OpenIdClient {
            id: "forgejo".into(),
            secret: None,
            redirect: "http://127.0.0.1:3000/user/oauth2/temper/callback".into(),
        }],
    });
    file.listen = Some(([127, 0, 0, 1], port).into());
    let mut setup = support::owned::setup_value(directory, 1, "owned");
    setup["sign_in"] = "forgejo".into();
    support::configuration::setup(directory, 1, setup);
    support::write_file(&path, &file, temper_config_shell::Shape::Standalone);
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
    assert_eq!(output.stderr, b"temper-box forge prepare: forge 1: interrupted\n");
    scratch.remove();
}

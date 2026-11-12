mod box_kinds;
mod box_link_form;
mod commands;
mod help;
mod init_clients;
mod limits;
mod reference_settings;
mod settings_fields;
mod tui;

// jig/docs/design/protocol/client/sign-in.md; temper/docs/design/protocol/setup/box.md, 2.
#[test]
fn the_box_footer_hours_agree_with_the_small_profile() {
    use jig_protocol_config::profile::{Profile, profile};
    let span = profile(Profile::Small).clients.link_span;
    let hour = skein_lib::Duration::from_secs(3600);
    let hours = span.as_nanos().div_euclid(hour.as_nanos());
    let words = format!("within {hours} hours");
    let footer = std::str::from_utf8(temper_box_machine::terminal::FOOTER).expect("footer UTF-8");
    assert!(footer.contains(&words), "footer must contain {words:?}: {footer}");
}

// Linux's passwd entry is the reference for macOS's command lookup.
#[cfg(target_os = "linux")]
#[test]
fn id_login_agrees_with_the_effective_users_passwd_entry() {
    let user = skein_shell::effective_user();
    let expected = super::login_passwd(user).expect("effective user's passwd login");
    assert_eq!(super::login(user).expect("Linux login"), expected);
    assert_eq!(super::forge_exec::login_id().expect("macOS login lookup"), expected);
}

#[test]
fn directory_commands_take_flags_after_their_command_words() {
    for (words, valid) in [
        (vec!["init"], true),
        (vec!["init", "--box"], true),
        (vec!["init", "--box", "--box"], false),
        (vec!["init", "extra"], false),
        (vec!["serve"], true),
        (vec![], true),
        (vec!["serve", "old.json"], false),
        (vec!["program-login", "name"], true),
        (vec!["program-login"], false),
        (vec!["forge", "prepare", "1"], true),
        (vec!["forge", "fetch", "1"], true),
        (vec!["forge", "remove", "1"], true),
        (vec!["forge", "run", "1", "runner"], true),
        (vec!["forge", "sign-in", "1"], true),
        (vec!["forge", "source", "1"], true),
        (vec!["unknown"], false),
    ] {
        let mut words: Vec<Box<[u8]>> = words.iter().map(|v| Box::from(v.as_bytes())).collect();
        words
            .extend([b"--config".as_slice(), b"/c", b"--data", b"/d", b"--cache", b"/k"].iter().map(|v| Box::from(*v)));
        assert_eq!(super::configure_with(&words, &[], b"/").is_ok(), valid);
    }
}
const OPEN_LINK: &[u8] =
    b"http://127.0.0.1:7000/jig/v1/sign-in/link?secret=AwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwM".as_slice();
type OpenRow = (&'static str, &'static [u8], Result<(bool, &'static str), &'static str>);
const OPEN_ARGUMENTS: &[OpenRow] = &[
    ("link", OPEN_LINK, Ok((true, "127.0.0.1:7000"))),
    ("address", b"http://127.0.0.1:7000".as_slice(), Ok((false, "127.0.0.1:7000"))),
    ("slash", b"http://127.0.0.1:7000/".as_slice(), Ok((false, "127.0.0.1:7000"))),
    ("IPv6", b"http://[::1]:7000/".as_slice(), Ok((false, "[::1]:7000"))),
    ("loopback range", b"http://127.1.2.3:7000".as_slice(), Ok((false, "127.1.2.3:7000"))),
    ("scheme", b"https://127.0.0.1:7000".as_slice(), Err("not a link or an address: https://127.0.0.1:7000")),
    ("hostname", b"http://localhost:7000".as_slice(), Err("not a link or an address: http://localhost:7000")),
    ("port", b"http://127.0.0.1".as_slice(), Err("not a link or an address: http://127.0.0.1")),
    ("remote", b"http://10.0.0.1:7000".as_slice(), Err("the engine's address must be on loopback")),
    ("path", b"http://127.0.0.1:7000/other".as_slice(), Err("not a link or an address: http://127.0.0.1:7000/other")),
    (
        "short secret",
        b"http://127.0.0.1:7000/jig/v1/sign-in/link?secret=AA".as_slice(),
        Err("not a link or an address: http://127.0.0.1:7000/jig/v1/sign-in/link?secret=AA"),
    ),
];

#[test]
fn each_open_argument_reads_as_a_link_an_address_or_a_refusal() {
    use super::open::{Refused, Target, target};
    for &(row, argument, expected) in OPEN_ARGUMENTS {
        let found = target(argument)
            .map(|target| match target {
                Target::Link { engine, link } => {
                    assert_eq!(link.expose(), argument, "{row}: link bytes");
                    (true, engine.to_string())
                }
                Target::Address { engine } => (false, engine.to_string()),
            })
            .map_err(|why: Refused| why.line(std::path::Path::new("client")));
        assert_eq!(
            found.as_ref().map(|(link, engine)| (*link, engine.as_str())).map_err(String::as_str),
            expected
                .map_err(|why| format!("temper open: {why}\n"))
                .as_ref()
                .map(|(link, engine)| (*link, *engine))
                .map_err(String::as_str),
            "{row}"
        );
    }
}
#[test]
fn open_without_two_arguments_prints_its_usage() {
    for (row, arguments, expected) in [
        ("none", vec![], "temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS\n"),
        ("one", vec!["client".into()], "temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS\n"),
        (
            "three",
            vec!["client".into(), "address".into(), "extra".into()],
            "temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS\n",
        ),
    ] {
        let refused = super::open::start(&arguments, &skein_shell::Clock::new()).expect_err("usage refused");
        assert_eq!(refused.line(std::path::Path::new("client")), expected, "{row}");
    }
}

pub(crate) mod configuration;

#[test]
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
        if name.starts_with(b"charters/") {
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
    let (agent, document) = configuration::documents(&directory, "/fixture/temper");
    write("agent.json", &agent);
    assert_eq!(configuration::flags(&directory).len(), 6);
    let operations = super::operations_in(&[], b"/", &configuration::directories(&directory), &document)
        .expect("standalone operations");
    assert_no_descriptors_in(&directory);
    let mut owned = Vec::new();
    let service = super::prepare(&[], b"/", &configuration::directories(&directory), &document, &mut owned)
        .expect("same standalone preparation");
    assert_eq!(operations, service.operations());
    drop(service);
    super::close(owned);
    assert_no_descriptors_in(&directory);
    fs::remove_dir_all(directory).expect("remove operations scratch");
}
fn assert_no_descriptors_in(directory: &std::path::Path) {
    for entry in std::fs::read_dir("/proc/self/fd").expect("open descriptor list") {
        let path = entry.expect("descriptor entry").path();
        if let Ok(target) = std::fs::read_link(&path) {
            assert!(!target.starts_with(directory), "descriptor {path:?} still holds {target:?}");
        }
    }
}

#[test]
fn ctl_is_configured() {
    for arguments in [vec!["help".into()], vec!["directory".into(), "me".into()]] {
        let words = std::iter::once("ctl".into()).chain(arguments.clone());
        assert_eq!(
            super::configure_with(
                &words.map(|v: std::ffi::OsString| Box::from(v.as_encoded_bytes())).collect::<Vec<_>>(),
                &[],
                b"/"
            ),
            Ok(super::Command::Ctl { arguments })
        );
    }
}

#[test]
fn fixed_sign_in_ports_are_checked_against_owned_and_fake_listeners() {
    // Resolve the fixture's data paths as a deployment does.
    let example = include_bytes!("../../../docs/examples/owned.json");
    let mut value: serde_json::Value = serde_json::from_slice(example).expect("owned JSON");
    value["engine"]["llm"]["providers"] = serde_json::json!([{
        "name":"OpenAI", "number":1, "signed_in_plan":false, "endpoint":"openai", "sources":{
            "sign_in": {"issuer":"https://issuer.example", "client":"client", "scopes":[],
                "authorization":"https://issuer.example/auth", "token":"https://issuer.example/token",
                "keys":"https://issuer.example/keys", "transport":"tls", "server_name":"issuer.example",
                "must_grant":[], "redirect_path":"callback", "redirect_port":3000}
        }
    }]);
    // Keep git concurrency at one so the small profile's 48 sockets reach the
    // redirect-port check after this fixture adds an OAuth listener.
    value["application"]["limits"]["overrides"]["git_component.operations"] = 1.into();
    value["application"]["limits"]["overrides"]["checkout.workspaces"] = 1.into();
    value["workers"][0]["slots"] = 1.into();
    let mut fake = value.clone();
    fake["application"]["machine"].as_object_mut().expect("machine").remove("owned");
    configuration::fake_entry(&mut fake["application"]["machine"]["fake"], "127.0.0.1:3000");
    let mut demo = fake.clone();
    demo["engine"]["llm"]["providers"][0]["sources"]["sign_in"]["redirect_port"] = 8370.into();
    let fixture = configuration::demo_document(std::path::Path::new("/config"), "127.0.0.1:8370");
    for field in ["providers", "endpoints"] {
        let member = fixture["engine"]["llm"][field].as_array().expect("LLM table").last().expect("demo member");
        demo["engine"]["llm"][field].as_array_mut().expect("LLM table").push(member.clone());
    }
    demo["application"]["demo"] = fixture["application"]["demo"].clone();
    let mut bad_demo = demo.clone();
    bad_demo["application"]["demo"]["project"] = 9.into();
    for (document, expected) in [
        (&bad_demo, "application.demo.project: unknown member\n"),
        (
            &demo,
            "engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.demo.llm.listen\n",
        ),
        (
            &value,
            "engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.machine.owned.listen\n",
        ),
        (
            &fake,
            "engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.machine.fake.listen\n",
        ),
    ] {
        assert_eq!(
            super::setup::check(
                &serde_json::to_vec(document).expect("local JSON"),
                &jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").expect("fixture directories")
            )
            .expect_err("local listen conflict"),
            expected
        );
    }
    // The same API on a connected forge belongs to the remote peer, not this process.
    value["application"]["machine"].as_object_mut().expect("machine").remove("owned");
    let deployment = super::setup::check(
        &serde_json::to_vec(&value).expect("connected JSON"),
        &jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").expect("fixture directories"),
    )
    .expect("connected forge");
    assert!(deployment.demo.is_none());
}

#[test]
fn program_login_refuses_arguments_outside_the_required_source_name() {
    for arguments in [
        vec!["program-login"],
        vec!["program-login", "source", "extra"],
        vec!["program-login", "config.json", "source", "extra"],
        vec!["sign-in", "source"],
    ] {
        let expected = if arguments[0] == "sign-in" {
            "temper-box: no command sign-in; temper-box --help lists them\n"
        } else {
            "temper-box program-login: invalid arguments\nusage: temper-box program-login [DIRECTORIES] NAME\n"
        };
        assert_eq!(configure_program_login(&arguments), Err(expected.into()));
    }
}
#[test]
fn program_login_accepts_exactly_the_required_source_name() {
    let directories =
        super::directories::resolve(&[], &[(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))], b"/")
            .expect("default directories")
            .0;
    assert_eq!(
        configure_program_login(&["program-login", "source"]),
        Ok(super::Command::ProgramLogin { directories, name: "source".into() })
    );
}

#[test]
fn forge_child_names_its_death_signal_or_prints_its_usage() {
    use skein_io::kernel::Signal;
    for (word, signal) in [("kill", Signal::Kill), ("terminate", Signal::Terminate)] {
        assert_eq!(
            configure_program_login(&[
                "internal",
                "forge-child",
                "--parent",
                "7",
                "--signal",
                word,
                "--",
                "/bin/forgejo",
                "web"
            ]),
            Ok(super::Command::ForgeChild {
                parent: 7,
                signal,
                program: "/bin/forgejo".into(),
                arguments: vec!["web".into()],
            })
        );
    }
    let usage = "temper-box internal forge-child: invalid arguments\nusage: temper-box internal forge-child --parent PID --signal kill|terminate -- PROGRAM ARGS...\n";
    for arguments in [
        vec!["internal", "forge-child", "--parent", "7", "--", "/bin/forgejo"],
        vec!["internal", "forge-child", "--parent", "7", "--signal", "interrupt", "--", "/bin/forgejo"],
        vec!["internal", "forge-child", "--signal", "kill", "--parent", "7", "--", "/bin/forgejo"],
    ] {
        assert_eq!(configure_program_login(&arguments), Err(usage.into()), "{arguments:?}");
    }
}

fn configure_program_login(arguments: &[&str]) -> Result<super::Command, String> {
    super::configure_with(
        &arguments.iter().map(|value| Box::from(value.as_bytes())).collect::<Vec<_>>(),
        &[(Box::from(b"HOME".as_slice()), Box::from(b"/home/test".as_slice()))],
        b"/",
    )
}

#[test]
fn forge_commands_refuse_the_recorded_fake_kind_for_owned_operations() {
    let scratch = skein_scratch::Scratch::new("forge-not-owned");
    let document = configuration::check_document(scratch.path());
    std::fs::write(scratch.path().join("temper.json"), serde_json::to_vec(&document).expect("JSON")).expect("document");
    std::fs::create_dir_all(scratch.path().join("box/groups")).expect("groups");
    std::fs::write(scratch.path().join("box/groups/1"), b"fake\n").expect("kind");
    let refused = super::forge::start(
        &configuration::directories(scratch.path()),
        &["run".into(), "1".into(), "forgejo".into()],
        None,
        &[],
    )
    .expect_err("fake is not owned");
    assert_eq!(refused.line(), "temper forge run: forge 1: it is fake, not owned\n");
    assert!(!scratch.path().join("forge-1").exists());
}

pub(crate) fn worker_document(directory: &std::path::Path) -> (Vec<u8>, serde_json::Value) {
    let (agent, input) = configuration::documents(directory, "/missing/temper");
    let mut input: serde_json::Value = serde_json::from_slice(&input).expect("fixture document");
    input.as_object_mut().expect("document object").remove("engine");
    input["workers"][0]["engine"] = serde_json::json!({"socket":"127.0.0.1:7100"});
    input["application"] =
        serde_json::json!({"workers":input["application"]["workers"],"limits":input["application"]["limits"]});
    (agent, input)
}
#[test]
fn serve_prepares_a_worker_alone_without_a_deployment_or_setup_host() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let directory = std::env::temp_dir().join(format!("temper-worker-alone-{}", std::process::id()));
    fs::create_dir(&directory).expect("worker-alone fixture");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).expect("worker-alone fixture");
    let (agent, input) = worker_document(&directory);
    fs::write(directory.join("agent.json"), agent).expect("worker-alone fixture");
    fs::write(directory.join("root.der"), skein_tls_kit::pki::ROOT).expect("worker-alone fixture");
    let secret = directory.join("worker.secret");
    fs::write(&secret, b"worker-secret").expect("worker-alone fixture");
    fs::set_permissions(secret, fs::Permissions::from_mode(0o600)).expect("worker-alone fixture");
    let mut owned = Vec::new();
    let (service, store) = super::prepare_with_store(
        &[],
        b"/",
        &configuration::directories(&directory),
        &serde_json::to_vec(&input).expect("worker-alone fixture"),
        &mut owned,
    )
    .expect("worker-alone fixture");
    assert!(store.is_empty(), "no deployment root");
    assert_eq!(owned.len(), 1, "only the worker root");
    let super::Service::Worker(_) = service else { panic!("worker service has no engine or setup host") };
    assert!(!directory.join("state").exists(), "no store directory created");
    super::close(owned);
    fs::remove_dir_all(directory).expect("worker-alone fixture");
}

#[test]
fn box_echo_resolves_its_default_and_flag_without_consuming_other_arguments() {
    use temper_box_machine::host::Echo;
    for (flags, expected) in
        [(vec![], Echo::None), (vec!["--echo", "steps"], Echo::Steps), (vec!["--echo", "none"], Echo::None)]
    {
        let words = ["--config", "/c", "--data", "/d", "--cache", "/k"]
            .into_iter()
            .chain(flags)
            .map(|word| Box::from(word.as_bytes()))
            .collect::<Vec<_>>();
        let super::Command::Box { echo, .. } = super::configure_with(&words, &[], b"/").expect("box settings") else {
            panic!("ordinary box")
        };
        assert_eq!(echo, expected);
    }
    let words =
        ["--config", "/c", "--data", "/d", "--cache", "/k", "--echo", "all"].map(|word| Box::from(word.as_bytes()));
    let why = super::configure_with(&words, &[], b"/").expect_err("echo refused");
    assert!(why.starts_with("temper-box: --echo: expected none or steps"), "{why}");
    for flags in [
        vec!["--echo", "steps", "extra"],
        vec!["--echo", "none", "--unknown"],
        vec!["--echo", "steps", "--echo", "steps"],
    ] {
        let words = ["--config", "/c", "--data", "/d", "--cache", "/k"]
            .into_iter()
            .chain(flags)
            .map(|word| Box::from(word.as_bytes()))
            .collect::<Vec<_>>();
        assert!(super::configure_with(&words, &[], b"/").is_err(), "leftover flags stay refused");
    }
}

// temper/docs/design/protocol/setup/owned.md, 6.
#[test]
fn the_launchers_io_is_the_small_profiles() {
    assert_eq!(temper_launcher::IO, jig_protocol_config::profile(jig_protocol_config::Profile::Small).io,);
}

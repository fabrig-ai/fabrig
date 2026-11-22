mod box_kinds;
mod commands;
mod configuration_file;
mod figures;
mod help;
mod init_clients;
mod limits;
mod reference_settings;
mod settings_fields;

// jig/docs/design/protocol/client/sign-in.md; temper/docs/design/protocol/setup/box.md, 2.
#[test]
fn the_box_footer_hours_agree_with_the_small_profile() {
    let span = jig_protocol_config_fixtures::small().clients.link_span;
    let hour = skein_lib::Duration::from_secs(3600);
    let hours = span.as_nanos().div_euclid(hour.as_nanos());
    let words = format!("within {hours} hours");
    let footer = std::str::from_utf8(temper_box_machine::terminal::FOOTER).expect("footer UTF-8");
    assert!(footer.contains(&words), "footer must contain {words:?}: {footer}");
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
    (
        "page link",
        b"http://127.0.0.1:7000/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        Ok((true, "127.0.0.1:7000")),
    ),
    (
        "IPv6 page link",
        b"http://[::1]:7000/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        Ok((true, "[::1]:7000")),
    ),
    (
        "remote page link",
        b"http://10.0.0.1:7000/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        Err("the engine's address must be on loopback"),
    ),
    (
        "short page secret",
        b"http://127.0.0.1:7000/sign-in?secret=AA",
        Err("not a link or an address: http://127.0.0.1:7000/sign-in?secret=AA"),
    ),
    (
        "page other query",
        b"http://127.0.0.1:7000/sign-in?other=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        Err(
            "not a link or an address: http://127.0.0.1:7000/sign-in?other=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        ),
    ),
    ("page no query", b"http://127.0.0.1:7000/sign-in", Err("not a link or an address: http://127.0.0.1:7000/sign-in")),
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
fn fixed_sign_in_ports_are_checked_against_the_demo_listener() {
    // Resolve the fixture's data paths as a deployment does.
    let example = include_bytes!("../../../docs/examples/owned.json");
    let mut value: serde_json::Value = serde_json::from_slice(example).expect("owned JSON");
    // Keep git concurrency at one so the small profile's 48 sockets reach the
    // redirect-port check after this fixture adds an OAuth listener.
    value["workers"][0]["slots"] = 1.into();
    let directories = jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").expect("directories");
    let mut providers = temper_service::stock::providers();
    providers[0].provider.sign_in.as_mut().unwrap().redirect_port = Some(8370);
    let given = temper_service::stock::Given::Stock
        .with_providers(providers.clone(), Box::new([temper_service::stock::scripted(crate::scripted::PREFERRED)]));
    let bytes = serde_json::to_vec(&value).expect("JSON");
    assert_eq!(
        super::setup::check_with(&bytes, &directories, &given).unwrap_err(),
        "providers.setting.providers[0].sources.sign_in.redirect_port: fixed port conflicts with the box's scripted model\n"
    );
    // A remote peer's port is not a listener of this process.
    providers[0].provider.sign_in.as_mut().unwrap().redirect_port = Some(3000);
    let given = temper_service::stock::Given::Stock
        .with_providers(providers, Box::new([temper_service::stock::scripted(crate::scripted::PREFERRED)]));
    assert!(super::setup::check_with(&bytes, &directories, &given).expect("remote port").demo.is_some());
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
    use std::os::unix::fs::PermissionsExt;
    let scratch = skein_scratch::Scratch::new("forge-not-owned");
    std::fs::set_permissions(scratch.path(), std::fs::Permissions::from_mode(0o700)).expect("private deployment root");
    std::fs::write(
        scratch.path().join(temper_config_shell::FILE),
        temper_config_shell::render(&configuration::file(), temper_config_shell::Shape::Standalone),
    )
    .expect("document");
    std::fs::create_dir_all(scratch.path().join("box/groups")).expect("groups");
    std::fs::write(scratch.path().join("box/groups/1"), b"fake\n").expect("kind");
    configuration::setup(
        scratch.path(),
        1,
        serde_json::json!({"kind":"fake","listen":{"socket":"127.0.0.1:3000"},"remote":"http://127.0.0.1:3000","git":"demo/bin/git"}),
    );
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
    assert_eq!(temper_launcher::IO, jig_protocol_config_fixtures::small().io,);
}

#[test]
fn the_box_builds_its_playground_from_the_scripted_provider_but_an_engine_alone_has_none() {
    let directories = jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").expect("directories");
    let value = configuration::demo_document(std::path::Path::new("/config"), "127.0.0.1:8370");
    assert!(value["application"].get("demo").is_none());
    let bytes = serde_json::to_vec(&value).expect("fixture");
    let boxed = super::setup::check(&bytes, &directories).expect("box");
    let fake = boxed.demo.expect("scripted provider");
    assert_eq!(fake.llm().listen().to_string(), "127.0.0.1:8370");
    let demo = super::setup::playground(&fake);
    assert_eq!(demo.provider, 3);
    assert_eq!(demo.key_file.as_ref(), b"/data/demo/llm.key");
    assert_eq!(
        demo.cues.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        [b"Add CSV export to widget".as_slice(), b"Add a dark theme to widget", b"Fix widget's empty search state",]
    );
    let mut engine = value.clone();
    engine["workers"] = serde_json::json!([]);
    engine["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7200"}]);
    let engine = serde_json::to_vec(&engine).expect("engine fixture");
    assert!(temper_engine_shell::setup::check(&engine, &directories).expect("engine alone").demo.is_none());
    assert!(value["engine"].get("llm").is_none(), "the scripted provider is never a document member");
}

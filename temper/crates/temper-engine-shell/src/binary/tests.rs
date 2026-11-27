use super::{Capture, ENGINE, Reached, Reply, run, serve_command};
use std::path::Path;
use temper_config_shell::{FILE, FORMER, File, Shape};

const VERSION: &str = "0.0.0";
const COMMIT: &str = "0123456789ab";

fn captured(words: &[&str]) -> Capture {
    Capture {
        run_as: ENGINE.binary.as_bytes().into(),
        arguments: words.iter().map(|word| word.as_bytes().into()).collect(),
        environment: Box::new([(b"HOME".as_slice().into(), b"/fixture".as_slice().into())]),
        working: Ok("/fixture".into()),
    }
}

fn printed(words: &[&str]) -> String {
    let Reply::Print(text) = run(&captured(words), VERSION, COMMIT).expect("successful reply") else {
        panic!("{words:?} must print")
    };
    text
}

#[test]
fn engine_table_prints_help_pages_and_the_captured_identity() {
    let overview = ENGINE.help();
    for word in ["-h", "--help", "help"] {
        assert_eq!(printed(&[word]), overview, "{word}");
    }
    assert_eq!(printed(&["--version"]), "temper-engine 0.0.0 (0123456789ab)\n");

    for command in ENGINE.commands {
        let super::Reach::Words(words) = command.reach else { panic!("engine commands are words") };
        let mut arguments = words.to_vec();
        arguments.push("--help");
        let text = printed(&arguments);
        assert_eq!(text, ENGINE.command_help(command), "{arguments:?}");
        assert_eq!(text.starts_with("internal:"), command.set == skein_shell::Set::Internal);
    }
    let serve = printed(&["serve", "--help"]);
    for words in ["engine alone", "temper-worker serve", "temper-box serve"] {
        assert!(serve.contains(words), "{serve}");
    }
    let forge = printed(&["forge", "fetch", "--help"]);
    assert!(
        forge
            .starts_with("internal: not part of temper-engine's public command line; it may change without\nnotice.\n"),
        "{forge}"
    );

    assert_eq!(
        printed(&["help", "pages"]),
        temper_shell::help::answer(ENGINE.binary, &temper_shell::help::Request::Index).expect("index page")
    );
    assert_eq!(printed(&["help", "commands"]), temper_shell::table::commands_page(&ENGINE));
    assert_eq!(printed(&["help", "config"]).as_bytes(), temper_config_shell::help(Shape::EngineAlone).as_ref());
}

#[test]
fn engine_table_refuses_unknown_words_and_missing_pages() {
    for (arguments, word) in [
        (&["box"][..], " box"),
        (&["program-login", "a"][..], " program-login"),
        (&["--version", "extra"][..], " --version"),
        (&[][..], ""),
    ] {
        assert_eq!(
            run(&captured(arguments), VERSION, COMMIT),
            Err((2, format!("temper-engine: no command{word}; temper-engine --help lists them\n")))
        );
    }
    assert_eq!(
        run(&captured(&["help", "nope"]), VERSION, COMMIT),
        Err((1, "temper-engine: no page nope; temper-engine help lists them\n".into()))
    );
}

pub(crate) fn refusal(words: &[&str]) -> (u8, String) {
    run(&captured(words), VERSION, COMMIT).expect_err("command refused")
}

fn config_capture(root: &Path, words: &[&str]) -> Capture {
    let mut words = words.to_vec();
    words.extend(["--config", "config", "--data", "data", "--cache", "cache"]);
    Capture { working: Ok(root.into()), ..captured(&words) }
}

const CONFIG_SOURCE: &[u8] =
    b"# mine\nlisten = '127.0.0.1:7100'\nworkers = '127.0.0.1:7200'\nowner = 'ada@localhost'\n";

#[test]
fn config_upgrade_prints_render_without_writing() {
    let scratch = skein_scratch::Scratch::new("engine-config-upgrade");
    let config = scratch.path().join("config");
    std::fs::create_dir(&config).expect("config directory");
    let path = config.join(FILE);
    std::fs::write(&path, CONFIG_SOURCE).expect("input file");
    let file = File {
        listen: Some("127.0.0.1:7100".parse().expect("listen address")),
        workers: Some("127.0.0.1:7200".parse().expect("workers address")),
        owner: Some("ada@localhost".into()),
        ..File::default()
    };

    let capture = config_capture(scratch.path(), &["config", "upgrade"]);
    let expected = temper_config_shell::render(&file, Shape::EngineAlone);
    assert_eq!(run(&capture, VERSION, COMMIT), Ok(Reply::Print(String::from_utf8_lossy(&expected).into_owned())));
    assert_eq!(std::fs::read(&path).expect("original file"), CONFIG_SOURCE);
    assert_eq!(std::fs::read_dir(&config).expect("config entries").count(), 1, "only the original file remains");
    assert_eq!(std::fs::read_dir(scratch.path()).expect("root entries").count(), 1, "only config exists");
}

#[test]
fn config_upgrade_refusals_match_the_checkpoint_read() {
    let cases: &[&[(&str, &[u8])]] = &[
        &[(FILE, CONFIG_SOURCE), (FORMER, b"not read")],
        &[],
        &[(FILE, b"workers = '127.0.0.1:7200'\nalpha = 1\nbeta = 2\n")],
    ];
    for files in cases {
        let scratch = skein_scratch::Scratch::new("engine-config-upgrade-refusal");
        let config = scratch.path().join("config");
        std::fs::create_dir(&config).expect("config directory");
        for (name, source) in *files {
            std::fs::write(config.join(name), source).expect("input file");
        }
        let checkpoint = config_capture(scratch.path(), &["checkpoint"]);
        let (code, refusal) = run(&checkpoint, VERSION, COMMIT).expect_err("checkpoint refuses unreadable config");
        let upgrade = config_capture(scratch.path(), &["config", "upgrade"]);
        assert_eq!(code, 1, "{refusal}");
        assert_eq!(
            run(&upgrade, VERSION, COMMIT),
            Err((1, refusal.replace("temper-engine checkpoint:", "temper-engine config upgrade:")))
        );
    }
}

#[test]
fn toml_engine_refuses_a_box_file_without_workers() {
    let capture = captured(&["serve", "--config", "/fixture"]);
    let Reached::Command(command, arguments) = ENGINE.reach(&capture.run_as, &capture.arguments) else {
        panic!("serve row")
    };
    assert_eq!(
        serve_command(command, arguments, &capture, |directories, working, record| {
            assert_eq!(directories.config.as_ref(), b"/fixture");
            assert!(!record);
            super::serve::prepare_in_with_read(
                ENGINE.binary,
                &capture.environment,
                working,
                directories,
                &temper_engine_service::stock::Given::Stock,
                |_, shape| {
                    assert_eq!(shape, Shape::EngineAlone);
                    crate::configuration_tests::read_scripted(b"listen = \"127.0.0.1:7100\"\n", shape)
                },
            )
            .map(|_| panic!("standalone file must be refused at the read"))
        }),
        Err((1, format!("temper-engine serve: {FILE}: workers: required in temper-engine's file\n")))
    );
}

#[test]
fn engine_commands_refuse_the_former_file_by_name() {
    let scratch = skein_scratch::Scratch::new("engine-former-file");
    let arguments = [
        b"--config".as_slice(),
        std::os::unix::ffi::OsStrExt::as_bytes(scratch.path().as_os_str()),
        b"--data",
        std::os::unix::ffi::OsStrExt::as_bytes(scratch.path().as_os_str()),
        b"--cache",
        std::os::unix::ffi::OsStrExt::as_bytes(scratch.path().as_os_str()),
    ]
    .map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
    std::fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
    for present in [false, true] {
        if present {
            std::fs::write(scratch.path().join(FILE), b"invalid TOML").expect("current file");
        }
        for (words, command) in [
            (&["serve"][..], "serve"),
            (&["upgrade", "--check", "/unused"][..], "upgrade"),
            (&["upgrade", "--workers", "/unused"][..], "upgrade"),
            (&["upgrade", "--back"][..], "upgrade"),
            (&["checkpoint"][..], "checkpoint"),
            (&["restore", "00000000000000000012-00000000000000ff"][..], "restore"),
        ] {
            let mut arguments: Vec<_> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
            arguments.extend(directories.flags());
            let capture = super::Capture {
                run_as: b"temper-engine".as_slice().into(),
                arguments: arguments.into(),
                environment: Box::new([]),
                working: Ok(scratch.path().into()),
            };
            assert_eq!(
                super::run(&capture, "test", "test"),
                Err((
                    1,
                    format!(
                        "temper-engine {command}: {FORMER}: moved: the configuration is {FILE}; write it from temper-engine help config, then remove {FORMER}\n"
                    )
                ))
            );
        }
    }
}

#[test]
fn versions_print_each_wire_range_then_its_layout() {
    use std::fmt::Write;
    let text = printed(&["versions"]);
    let mut lines = text.lines();
    assert!(lines.by_ref().take(4).eq(["level skein 0", "level smith 0", "level jig 6", "level temper 14"]));
    let format = jig_protocol_store::format::current(temper_protocol_store::STORE_VERSION, [0; 16]);
    assert_eq!(lines.next(), Some(format!("store {}", format.component).as_str()));
    for (family, version) in [
        ("core", u32::from(format.core)),
        ("tasks", u32::from(format.tasks)),
        ("people", u32::from(format.people)),
        ("notes", u32::from(format.notes)),
        ("accounts", u32::from(format.accounts)),
        ("settings", u32::from(format.settings)),
        ("temper", temper_protocol_store::STORE_VERSION),
    ] {
        assert_eq!(lines.next(), Some(format!("family {family} {version}").as_str()));
    }
    let wires = temper_engine_service::wires();
    assert_eq!(wires.len(), 6);
    for (worker, (name, frozen)) in wires.iter().take(5).zip([
        ("jig-worker-wire", 22),
        ("smith-pieces", 1),
        ("smith-charter", 5),
        ("smith-transcript", 8),
        ("temper-worker-wire", 12),
    ]) {
        assert_eq!((worker.name, worker.lowest, worker.highest()), (name, frozen, frozen));
    }
    assert_eq!(wires.get(5).expect("host-tools row").name, "temper-host-tools");
    for wire in &wires {
        let (name, lowest, highest) = (wire.name, wire.lowest, wire.highest());
        assert_eq!(lines.next(), Some(format!("wire {name} {lowest} {highest}").as_str()));
        for version in lowest..=highest {
            let layout = lines.next().expect("one layout follows for each spoken version");
            let fingerprint =
                layout.strip_prefix(&format!("layout {name} {version} ")).expect("layout name and version");
            let mut expected = String::new();
            for byte in wire.layout(version).expect("kept layout") {
                write!(expected, "{byte:02x}").expect("writing into a string");
            }
            assert_eq!(fingerprint, expected);
        }
    }
    assert_eq!(lines.next(), Some("bound stop 602"));
    assert_eq!(lines.next(), Some("bound report 30"));
    assert_eq!(lines.next(), None);
    assert_eq!(
        refusal(&["versions", "extra"]),
        (2, "temper-engine versions: invalid arguments\nusage: temper-engine versions\n".into())
    );
}

#[test]
fn restore_bad_names_are_usage_errors_before_reading_configuration() {
    for arguments in [&["restore"][..], &["restore", "12-ff"]] {
        assert_eq!(
            run(&captured(arguments), VERSION, COMMIT),
            Err((
                2,
                "temper-engine restore: invalid arguments\nusage: temper-engine restore NAME [DIRECTORIES]\n".into()
            ))
        );
    }
}

#[test]
fn engine_serve_accepts_record_once_only() {
    for (words, expected) in [
        (&[][..], Some(false)),
        (&["--record"][..], Some(true)),
        (&["--record", "--record"][..], None),
        (&["--other"][..], None),
    ] {
        let arguments: Vec<_> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
        assert_eq!(super::recording(&arguments), expected);
    }
    let switch = crate::commands::SERVE.switches.iter().find(|switch| switch.name == "record").expect("public switch");
    assert_eq!(switch.set, skein_shell::Set::Public);
    assert_eq!(switch.line, "Have every page served record a journey in its browser, saved by Alt+Shift+S.");
}

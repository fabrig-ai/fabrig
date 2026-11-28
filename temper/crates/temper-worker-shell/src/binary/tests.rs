use super::{Capture, PROGRAM_LOGIN, Reached, Reply, SERVE, WORKER, login_command, run, serve_command};
use std::path::Path;
use temper_config_shell::{FILE, FORMER, File, Shape};
use temper_service_shell::configuration::Read;

const VERSION: &str = "0.0.0";
const COMMIT: &str = "0123456789ab";
const CONFIG: &str = "/fixture/config";

fn captured(words: &[&str]) -> Capture {
    Capture {
        run_as: WORKER.binary.as_bytes().into(),
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
fn worker_table_prints_help_pages_and_the_captured_identity() {
    let overview = WORKER.help();
    for word in ["-h", "--help", "help"] {
        assert_eq!(printed(&[word]), overview, "{word}");
    }
    assert_eq!(printed(&["--version"]), "temper-worker 0.0.0 (0123456789ab)\n");
    let (_, commands) = overview.split_once("\ncommands:\n").expect("commands section");
    let names: Vec<_> = commands
        .split("\n\n")
        .next()
        .expect("command lines")
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(names, ["serve", "config", "program-login", "help"]);
    let words = overview.split_whitespace().collect::<Vec<_>>().join(" ");
    for setting in ["--config CONFIG", "--data DATA", "--cache CACHE", "TEMPER_CONFIG", "TEMPER_DATA", "TEMPER_CACHE"] {
        assert!(words.contains(setting), "{setting}: {overview}");
    }

    for command in WORKER.commands {
        let super::Reach::Words(words) = command.reach else { panic!("worker commands are words") };
        let mut arguments = words.to_vec();
        arguments.push("--help");
        let text = printed(&arguments);
        assert_eq!(text, WORKER.command_help(command), "{arguments:?}");
        assert!(!text.starts_with("internal:"), "{text}");
    }
    let serve = printed(&["serve", "--help"]);
    for words in ["worker alone", "temper-engine serve", "temper-box serve"] {
        assert!(serve.contains(words), "{serve}");
    }
    assert_eq!(
        printed(&["help", "pages"]),
        temper_shell::help::answer(WORKER.binary, &temper_shell::help::Request::Index).expect("index page")
    );
    assert_eq!(printed(&["help", "commands"]), temper_shell::table::commands_page(&WORKER));
    assert_eq!(printed(&["help", "config"]).as_bytes(), temper_config_shell::help(Shape::WorkerAlone).as_ref());
}

#[test]
fn worker_table_refuses_no_command_and_bad_forms() {
    assert_eq!(
        run(&captured(&[]), VERSION, COMMIT),
        Err((2, "temper-worker: no command; temper-worker --help lists them\n".into()))
    );
    for (arguments, command, form) in [
        (&["serve", "extra"][..], "serve", "[DIRECTORIES]"),
        (&["config", "upgrade", "extra"][..], "config upgrade", "[DIRECTORIES]"),
        (&["program-login", "--config", CONFIG], "program-login", "[DIRECTORIES] NAME"),
    ] {
        assert_eq!(
            run(&captured(arguments), VERSION, COMMIT),
            Err((2, format!("temper-worker {command}: invalid arguments\nusage: temper-worker {command} {form}\n"))),
            "{arguments:?}"
        );
    }
}

fn config_capture(root: &Path, words: &[&str]) -> Capture {
    let mut words = words.to_vec();
    words.extend(["--config", "config", "--data", "data", "--cache", "cache"]);
    Capture { working: Ok(root.into()), ..captured(&words) }
}

#[test]
fn config_upgrade_prints_render_without_writing() {
    let scratch = skein_scratch::Scratch::new("worker-config-upgrade");
    let config = scratch.path().join("config");
    std::fs::create_dir(&config).expect("config directory");
    let path = config.join(FILE);
    // Keep the explicit name even though it is the default for this host.
    let name = String::from_utf8(skein_shell::host_name().expect("host name").into_vec()).expect("host name UTF-8");
    let quoted = serde_json::to_string(&name).expect("quoted name");
    let source = format!("# mine\nname = {quoted}\nengine = '127.0.0.1:7200'\n");
    std::fs::write(&path, &source).expect("input file");
    let file =
        File { name: Some(name), engine: Some("127.0.0.1:7200".parse().expect("engine address")), ..File::default() };

    let capture = config_capture(scratch.path(), &["config", "upgrade"]);
    let expected = temper_config_shell::render(&file, Shape::WorkerAlone);
    assert_eq!(run(&capture, VERSION, COMMIT), Ok(Reply::Print(String::from_utf8_lossy(&expected).into_owned())));
    assert_eq!(std::fs::read(&path).expect("original file"), source.as_bytes());
    assert_eq!(std::fs::read_dir(&config).expect("config entries").count(), 1, "only the original file remains");
    assert_eq!(std::fs::read_dir(scratch.path()).expect("root entries").count(), 1, "only config exists");
}

#[test]
fn config_upgrade_refusals_match_the_serve_read() {
    let cases: &[&[(&str, &[u8])]] = &[
        &[(FILE, b"engine = '127.0.0.1:7200'\nname = 'fixture'\n"), (FORMER, b"not read")],
        &[],
        &[(FILE, b"engine = '127.0.0.1:7200'\nalpha = 1\nbeta = 2\n")],
    ];
    for files in cases {
        let scratch = skein_scratch::Scratch::new("worker-config-upgrade-refusal");
        let config = scratch.path().join("config");
        std::fs::create_dir(&config).expect("config directory");
        for (name, source) in *files {
            std::fs::write(config.join(name), source).expect("input file");
        }
        let serve = config_capture(scratch.path(), &["serve"]);
        let (code, refusal) = run(&serve, VERSION, COMMIT).expect_err("serve refuses unreadable config");
        let upgrade = config_capture(scratch.path(), &["config", "upgrade"]);
        assert_eq!(code, 1, "{refusal}");
        assert_eq!(
            run(&upgrade, VERSION, COMMIT),
            Err((1, refusal.replace("temper-worker serve:", "temper-worker config upgrade:")))
        );
    }
}

fn read_scripted(bytes: &[u8], shape: Shape) -> Result<Read, String> {
    let facts = temper_config_shell::Facts { login: "tester".into(), hostname: "fixture".into(), provider_key: false };
    temper_config_shell::parse(bytes, shape, &facts).map(Read::from).map_err(|why| why.to_string())
}

fn render(shape: Shape) -> Box<[u8]> {
    let address = "127.0.0.1:1".parse().expect("fixture address");
    let file = match shape {
        Shape::Standalone => File { listen: Some(address), ..File::default() },
        Shape::EngineAlone => File { workers: Some(address), ..File::default() },
        Shape::WorkerAlone => panic!("wrong-shaped worker fixture"),
    };
    temper_config_shell::render(&file, shape)
}

const LISTEN: &str = "listen: a key of temper-box's or temper-engine's file, not temper-worker's";
const WORKERS: &str = "workers: a key of temper-engine's file, not temper-worker's";

fn shape_refusal(command: &str, key: &str) -> String {
    format!(
        "temper-worker {command}: {FILE}:4: {key}\ntemper-worker {command}: {FILE}: engine: required in temper-worker's file\n"
    )
}

#[test]
fn worker_serve_refuses_other_shapes_at_the_read() {
    let capture = captured(&["serve", "--config", CONFIG]);
    let Reached::Command(command, arguments) = WORKER.reach(&capture.run_as, &capture.arguments) else {
        panic!("serve row")
    };
    assert_eq!(command, &SERVE);
    for (document, key) in [(Shape::EngineAlone, WORKERS), (Shape::Standalone, LISTEN)] {
        let bytes = render(document);
        assert_eq!(
            serve_command(arguments, &capture, |directories, shape| {
                assert_eq!(directories.config.as_ref(), CONFIG.as_bytes());
                assert_eq!(shape, Shape::WorkerAlone);
                read_scripted(&bytes, shape)
                    .map_err(|why| temper_service_shell::serve_error(WORKER.binary, &why))
                    .map(|_| panic!("wrong shape must be refused before service preparation"))
            }),
            Err((1, shape_refusal("serve", key))),
            "{document:?}"
        );
    }
}

#[test]
fn toml_worker_login_prints_each_refusal_on_its_own_line() {
    let unknown = b"engine = \"127.0.0.1:7200\"\nalpha = 1\nbeta = 2\n";
    let standalone = render(Shape::Standalone);
    let capture = captured(&["program-login", "--config", CONFIG, "work"]);
    let Reached::Command(command, arguments) = WORKER.reach(&capture.run_as, &capture.arguments) else {
        panic!("program-login row")
    };
    assert_eq!(command, &PROGRAM_LOGIN);
    for (bytes, expected) in [
        (
            unknown.as_slice(),
            format!(
                "temper-worker program-login: {FILE}:2: alpha: unknown; temper-worker help config lists the keys\ntemper-worker program-login: {FILE}:3: beta: unknown; temper-worker help config lists the keys\n"
            ),
        ),
        (standalone.as_ref(), shape_refusal("program-login", LISTEN)),
    ] {
        assert_eq!(
            login_command(arguments, &capture, |directories, name| {
                assert_eq!(directories.config.as_ref(), CONFIG.as_bytes());
                assert_eq!(name, "work");
                crate::program_login::command_with(
                    directories,
                    name,
                    [],
                    |shape| {
                        assert_eq!(shape, Shape::WorkerAlone);
                        let read = read_scripted(bytes, shape)?;
                        let programs = read.file.programs.clone();
                        Ok((read, programs))
                    },
                    |_, _, _| panic!("refused file must not reach program setup"),
                )
                .map(|_| panic!("refused file must not return a command"))
            }),
            Err((1, expected))
        );
    }
}

#[test]
fn worker_commands_refuse_the_former_file_by_name() {
    let scratch = skein_scratch::Scratch::new("worker-former-file");
    let directories = crate::tests::configuration::directories(scratch.path());
    std::fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
    for present in [false, true] {
        if present {
            std::fs::write(scratch.path().join(FILE), b"invalid TOML").expect("current file");
        }
        for words in [&["serve"][..], &["program-login", "source"][..]] {
            let mut arguments: Vec<_> = words.iter().map(|word| Box::from(word.as_bytes())).collect();
            arguments.extend(directories.flags());
            let capture = super::Capture {
                run_as: b"temper-worker".as_slice().into(),
                arguments: arguments.into(),
                environment: Box::new([]),
                working: Ok(scratch.path().into()),
            };
            assert_eq!(
                super::run(&capture, "test", "test"),
                Err((
                    1,
                    format!(
                        "temper-worker {}: {FORMER}: moved: the configuration is {FILE}; write it from temper-worker help config, then remove {FORMER}\n",
                        words.first().expect("command")
                    )
                ))
            );
        }
    }
}

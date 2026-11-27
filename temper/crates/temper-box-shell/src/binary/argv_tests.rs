use super::{Capture, Command, ExitCode, TABLE, dispatch_routed, dispatch_with, front_pipes};
use skein_io::kernel::Fd;
use std::path::Path;
use temper_config_shell::{FILE, FORMER, File, Shape};

const VERSION: &str = "0.0.0";
const COMMIT: &str = "0123456789ab";

fn captured(words: &[&str]) -> Capture {
    Capture {
        run_as: b"temper-box".as_slice().into(),
        arguments: words.iter().map(|word| word.as_bytes().into()).collect(),
        environment: Box::new(
            [
                ("TEMPER_CONFIG", "/fixture/config"),
                ("TEMPER_DATA", "/fixture/data"),
                ("TEMPER_CACHE", "/fixture/cache"),
            ]
            .map(|(name, value)| (name.as_bytes().into(), value.as_bytes().into())),
        ),
        working: Ok("/fixture".into()),
    }
}

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    text == "temper-box 0.0.0 (0123456789ab)\n"
}

fn help(text: &str) -> bool {
    let (_, commands) = text.split_once("\ncommands:\n").expect("commands");
    let names: Vec<_> = commands
        .split("\n\n")
        .next()
        .expect("command lines")
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    text == TABLE.help()
        && names
            == [
                "init",
                "serve",
                "config",
                "program-login",
                "help",
                "versions",
                "upgrade",
                "upgrade",
                "upgrade",
                "checkpoint",
                "checkpoint",
                "checkpoint",
                "restore",
            ]
        && text.contains("temper-box [--config DIR]")
        && text.contains("upgrade --workers")
        && text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .contains("Judges the workers alone over links and small copies; switches nothing.")
        && ["forge", "fake", "internal", "ctl", "open"].iter().all(|word| !names.contains(word))
}

fn init_help(text: &str) -> bool {
    !text.starts_with("internal:") && text.starts_with("usage:") && text.contains("temper-box init [DIRECTORIES]")
}

fn pages(text: &str) -> bool {
    text == temper_shell::help::answer(TABLE.binary, &temper_shell::help::Request::Index).expect("index page")
}

fn config_page(text: &str) -> bool {
    text.as_bytes() == temper_config_shell::help(temper_config_shell::Shape::Standalone).as_ref()
}

fn commands_page(text: &str) -> bool {
    text == temper_shell::table::commands_page(&TABLE)
}

fn internal_help(text: &str) -> bool {
    text.starts_with(
        "internal: not part of temper-box's public command line; it may change without\nnotice.\n\nusage:\n",
    )
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], u8, Check, &'static str);
const CHECKPOINT_USAGE: &str = "temper-box checkpoint: invalid arguments\nusage: temper-box checkpoint [DIRECTORIES]\nusage: temper-box checkpoint --hold NAME [DIRECTORIES] -- COMMAND [ARGUMENT...]\nusage: temper-box checkpoint --remove NAME [DIRECTORIES]\n";
const ROWS: &[Case] = &[
    (&["checkpoint", "--", "cmd"], 2, empty, CHECKPOINT_USAGE),
    (
        &[
            "checkpoint",
            "--hold",
            "00000000000000000012-00000000000000ff",
            "--remove",
            "00000000000000000012-00000000000000ff",
            "--",
            "cmd",
        ],
        2,
        empty,
        CHECKPOINT_USAGE,
    ),
    (&["--version"], 0, version, ""),
    (&["-h"], 0, help, ""),
    (&["--help"], 0, help, ""),
    (&["help"], 0, help, ""),
    (&["init", "--help"], 0, init_help, ""),
    (&["fake", "llm", "--help"], 0, internal_help, ""),
    (&["forge", "fetch", "--help"], 0, internal_help, ""),
    (&["help", "pages"], 0, pages, ""),
    (&["help", "commands"], 0, commands_page, ""),
    (&["help", "config"], 0, config_page, ""),
    (&["help", "nope"], 1, empty, "temper-box: no page nope; temper-box help lists them\n"),
    (
        &["help", "--search"],
        2,
        empty,
        "temper-box help: invalid arguments\nusage: temper-box help [NAME | pages | --search WORDS...]\n",
    ),
    (
        &["--front", "--echo", "steps"],
        2,
        empty,
        "temper-box: --front cannot be used with --echo steps\nusage: temper-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps] [--record]\n",
    ),
    (
        &["--no-tui"],
        2,
        empty,
        "temper-box: unknown flag --no-tui\nusage: temper-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps] [--record]\n",
    ),
    (&["internal", "tui"], 2, empty, "temper-box: no command internal tui; temper-box --help lists them\n"),
    (&["frobnicate"], 2, empty, "temper-box: no command frobnicate; temper-box --help lists them\n"),
    (&["init", "extra"], 2, empty, "temper-box init: invalid arguments\nusage: temper-box init [DIRECTORIES]\n"),
    (
        &["config", "upgrade", "extra"],
        2,
        empty,
        "temper-box config upgrade: invalid arguments\nusage: temper-box config upgrade [DIRECTORIES]\n",
    ),
    (
        &["upgrade", "--check"],
        2,
        empty,
        "temper-box upgrade --check: invalid arguments\nusage: temper-box upgrade --check [DIRECTORIES] DIRECTORY\n",
    ),
    (
        &["upgrade", "--workers"],
        2,
        empty,
        "temper-box upgrade --workers: invalid arguments\nusage: temper-box upgrade --workers [DIRECTORIES] DIRECTORY\n",
    ),
    (
        &["upgrade", "--back", "extra"],
        2,
        empty,
        "temper-box upgrade --back: invalid arguments\nusage: temper-box upgrade --back [DIRECTORIES]\n",
    ),
];

fn reply(words: &[&str]) -> (ExitCode, String, String) {
    let mut stdout = String::new();
    let result = dispatch_with(
        &captured(words),
        VERSION,
        COMMIT,
        |text| stdout.push_str(text),
        |_, _| panic!("{words:?}: must not run a command"),
    );
    let (exit, stderr) = match result {
        Ok(exit) => (exit, String::new()),
        Err((code, stderr)) => (ExitCode::from(code), stderr),
    };
    (exit, stdout, stderr)
}

fn config_reply(root: &Path, words: &[&str]) -> (ExitCode, String, String) {
    let mut words = words.to_vec();
    words.extend(["--config", "config", "--data", "data", "--cache", "cache"]);
    let capture = Capture { working: Ok(root.into()), ..captured(&words) };
    let mut stdout = String::new();
    let result = dispatch_with(
        &capture,
        VERSION,
        COMMIT,
        |text| panic!("unexpected help: {text}"),
        |command, working| {
            crate::run_with_stdout(command, &capture.environment, working, &capture.run_as, VERSION, COMMIT, |text| {
                stdout.push_str(text);
            })
        },
    );
    match result {
        Ok(exit) => (exit, stdout, String::new()),
        Err((code, stderr)) => (ExitCode::from(code), stdout, stderr),
    }
}

const CONFIG_SOURCE: &[u8] = b"# mine\nlisten = '127.0.0.1:7100'\n";

#[test]
fn config_upgrade_prints_render_without_writing() {
    let scratch = skein_scratch::Scratch::new("box-config-upgrade");
    let config = scratch.path().join("config");
    std::fs::create_dir(&config).expect("config directory");
    let path = config.join(FILE);
    std::fs::write(&path, CONFIG_SOURCE).expect("input file");
    let file = File { listen: Some("127.0.0.1:7100".parse().expect("listen address")), ..File::default() };

    let (exit, stdout, stderr) = config_reply(scratch.path(), &["config", "upgrade"]);
    assert_eq!(exit, ExitCode::SUCCESS, "{stderr}");
    assert_eq!(stdout.as_bytes(), temper_config_shell::render(&file, Shape::Standalone).as_ref());
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(std::fs::read(&path).expect("original file"), CONFIG_SOURCE);
    assert_eq!(std::fs::read_dir(&config).expect("config entries").count(), 1, "only the original file remains");
    assert_eq!(std::fs::read_dir(scratch.path()).expect("root entries").count(), 1, "only config exists");
}

#[test]
fn config_upgrade_refusals_match_the_checkpoint_read() {
    let cases: &[&[(&str, &[u8])]] =
        &[&[(FILE, CONFIG_SOURCE), (FORMER, b"not read")], &[], &[(FILE, b"alpha = 1\nbeta = 2\n")]];
    for files in cases {
        let scratch = skein_scratch::Scratch::new("box-config-upgrade-refusal");
        let config = scratch.path().join("config");
        std::fs::create_dir(&config).expect("config directory");
        for (name, source) in *files {
            std::fs::write(config.join(name), source).expect("input file");
        }
        let (checkpoint, _, refusal) = config_reply(scratch.path(), &["checkpoint"]);
        let (exit, stdout, stderr) = config_reply(scratch.path(), &["config", "upgrade"]);
        assert_eq!(checkpoint, ExitCode::from(1), "{refusal}");
        assert_eq!(exit, ExitCode::from(1), "{stderr}");
        assert!(stdout.is_empty(), "{stdout}");
        assert_eq!(stderr, refusal.replace("temper-box checkpoint:", "temper-box config upgrade:"));
    }
}

#[test]
fn forge_fetch_usage() {
    let words =
        ["forge", "fetch", "--config", "/fixture/config", "--data", "/fixture/data", "--cache", "/fixture/cache"];
    assert_eq!(
        reply(&words),
        (
            ExitCode::from(2),
            String::new(),
            "temper-box forge fetch: invalid arguments\nusage: temper-box forge fetch [DIRECTORIES] FORGE\n".into(),
        )
    );
    let row = TABLE.commands.iter().find(|row| row.reach == skein_shell::Reach::Words(&["forge", "fetch"]));
    assert_eq!(row.expect("fetch row").forms, ["[DIRECTORIES] FORGE"]);
}

#[test]
fn argv_rows_keep_their_replies() {
    for (words, code, expected_stdout, expected_stderr) in ROWS {
        let (exit, stdout, stderr) = reply(words);
        assert_eq!(exit, ExitCode::from(*code), "{words:?}: {stderr}");
        assert!(expected_stdout(&stdout), "{words:?}: unexpected stdout {stdout:?}");
        assert_eq!(stderr, *expected_stderr, "{words:?}");
    }
}

#[test]
fn agent_modes_reach_the_smith_entry() {
    for words in [&["--program-child"][..], &["--program-child", "42", "/fixture/claude", "auth", "status"]] {
        let child_exit = ExitCode::from(1);
        let result = dispatch_routed(
            &captured(words),
            VERSION,
            COMMIT,
            |text| panic!("{words:?}: unexpected box stdout {text:?}"),
            |command, _| {
                assert_eq!(command, Command::ProgramChild, "{words:?}");
                Ok(child_exit)
            },
            || panic!("{words:?}: not a front invocation"),
        );
        assert_eq!(result, Ok(child_exit), "{words:?}: no box stderr");
    }
}

#[test]
fn versions_refuses_extra_argv() {
    assert_eq!(
        reply(&["versions", "x"]),
        (
            ExitCode::from(2),
            String::new(),
            "temper-box versions: invalid arguments\nusage: temper-box versions\n".into()
        )
    );
}

#[test]
fn front_refuses_a_missing_parent_pipe_before_any_start() {
    let pipe_3 = Fd::new(3);
    let pipe_4 = Fd::new(4);
    for (refused, expected_probes) in [(pipe_3, &[pipe_3][..]), (pipe_4, &[pipe_3, pipe_4])] {
        let mut probes = Vec::new();
        let result = dispatch_routed(
            &captured(&["--front"]),
            VERSION,
            COMMIT,
            |text| panic!("unexpected box stdout {text:?}"),
            |_, _| panic!("must refuse before initialization or startup"),
            || {
                front_pipes(|fd| {
                    probes.push(fd);
                    Ok(fd != refused)
                })
            },
        );
        assert_eq!(probes, expected_probes);
        assert_eq!(result, Err((2, "temper-box --front: fd 3 and fd 4 are its parent's pipes\n".into())));
    }
}

use super::{Capture, Command, ExitCode, TABLE, dispatch_routed, dispatch_with, front_pipes};
use skein_io::kernel::Fd;

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

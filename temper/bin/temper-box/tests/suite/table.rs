//! The box binary's table and dispatch (temper/docs/design/protocol/binaries.md, 2).
use std::process::{Command, Stdio};
use temper_box_shell::TABLE;
#[path = "../../../temper-engine/tests/support/upgrade.rs"]
mod upgrade;
#[test]
fn upgrade_refuses_an_empty_deployment_directory() {
    upgrade::empty_directory("temper-box");
}

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    eprintln!("--version stdout:\n{text}");
    let prefix = format!("temper-box {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    // release.md, 3: twelve hexadecimal digits, or unknown outside git.
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
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
        && names == ["init", "serve", "program-login", "help", "versions", "upgrade", "upgrade"]
        && text.contains("temper-box [--config DIR]")
        && ["forge", "fake", "internal", "ctl", "open"].iter().all(|word| !names.contains(word))
}

fn init_help(text: &str) -> bool {
    !text.starts_with("internal:") && text.starts_with("usage:") && text.contains("temper-box init [DIRECTORIES]")
}

fn pages(text: &str) -> bool {
    text == temper_shell::help::answer(TABLE.binary, &temper_shell::help::Request::Index).expect("index page")
}

fn commands_page(text: &str) -> bool {
    text == temper_shell::table::commands_page(&TABLE)
}

fn internal_help(text: &str) -> bool {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .starts_with("internal: not part of temper-box's public command line; it may change without notice.")
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], i32, Check, &'static str);
const ROWS: &[Case] = &[
    (&["--version"], 0, version, ""),
    (&["-h"], 0, help, ""),
    (&["--help"], 0, help, ""),
    (&["help"], 0, help, ""),
    (&["init", "--help"], 0, init_help, ""),
    (&["fake", "llm", "--help"], 0, internal_help, ""),
    (&["forge", "fetch", "--help"], 0, internal_help, ""),
    (&["help", "pages"], 0, pages, ""),
    (&["help", "commands"], 0, commands_page, ""),
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
        &["upgrade", "--back", "extra"],
        2,
        empty,
        "temper-box upgrade --back: invalid arguments\nusage: temper-box upgrade --back [DIRECTORIES]\n",
    ),
];

#[test]
fn the_binary_answers_its_table_and_refuses_arguments() {
    let scratch = skein_scratch::Scratch::new("box-binary");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    std::fs::create_dir(&config).expect("configuration directory");
    let mut failures = Vec::new();
    for (arguments, code, stdout, stderr) in ROWS {
        let arguments: Vec<_> = arguments
            .iter()
            .map(|word| match *word {
                "CONFIG" => config.as_os_str(),
                "DATA" => data.as_os_str(),
                "CACHE" => cache.as_os_str(),
                word => std::ffi::OsStr::new(word),
            })
            .collect();
        let output = Command::new(skein_scratch::paths::binary("temper-box"))
            .args(&arguments)
            .env_clear()
            .env("TEMPER_CONFIG", &config)
            .env("TEMPER_DATA", &data)
            .env("TEMPER_CACHE", &cache)
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-box");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || err != *stderr {
            failures.push(format!(
                "{arguments:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}",
                output.status.code()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn versions_are_the_engines_lines_and_refusals() {
    let output = Command::new(skein_scratch::paths::binary("temper-box"))
        .arg("versions")
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .expect("run versions");
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    assert_eq!(output.stdout, temper_engine_shell::versions().as_bytes());
    assert!(String::from_utf8_lossy(&output.stdout).lines().take(4).eq([
        "level skein 0",
        "level smith 0",
        "level jig 5",
        "level temper 9",
    ]));
    let refused = Command::new(skein_scratch::paths::binary("temper-box"))
        .args(["versions", "x"])
        .env_clear()
        .stdin(Stdio::null())
        .output()
        .expect("refuse versions arguments");
    assert_eq!(refused.status.code(), Some(2), "{refused:?}");
    assert!(refused.stdout.is_empty(), "{refused:?}");
    assert_eq!(
        String::from_utf8(refused.stderr).expect("UTF-8 refusal"),
        "temper-box versions: invalid arguments\nusage: temper-box versions\n"
    );
}

#[test]
fn missing_agent_documents_keep_the_boxs_runtime_words() {
    let scratch = skein_scratch::Scratch::new("box-agent-words");
    let missing = scratch.path().join("AGENT.json");
    let reason = format!("configuration metadata: {}: file absent\n", missing.display());
    for mode in ["agent", "agent-list"] {
        let output = Command::new(skein_scratch::paths::binary("temper-box"))
            .args(["internal", mode])
            .arg(&missing)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("missing agent document");
        assert_eq!(output.status.code(), Some(1), "{mode}: {output:?}");
        assert!(output.stdout.is_empty(), "{mode}: {output:?}");
        let mut expected = if mode == "agent" { format!("smith: {reason}") } else { String::new() };
        for line in reason.lines() {
            use std::fmt::Write;
            writeln!(expected, "temper internal {mode}: {line}").expect("write expected error");
        }
        assert_eq!(String::from_utf8(output.stderr).expect("UTF-8 failure"), expected);
    }
}

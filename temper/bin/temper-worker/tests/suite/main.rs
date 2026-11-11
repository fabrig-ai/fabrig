//! The worker binary's table and dispatch (temper/docs/design/protocol/binaries.md, 5).
use std::process::{Command, Stdio};
use temper_worker_shell::{PROGRAM_LOGIN, WORKER};

const ENGINE: &str = r#"{"engine":{"hosts":{"listener":[{"socket":"127.0.0.1:0"}]}}}"#;
const STANDALONE: &str = r#"{"engine":{"hosts":{"listener":[{"in_process":"workers"}]}}}"#;
const WORKER_DOCUMENT: &str = r#"{"workers":[{"engine":{"socket":"127.0.0.1:1"}}]}"#;
const SERVE: &[&str] = &["serve", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE"];
const LOGIN: &[&str] = &["program-login", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE", "work"];
const LOGIN_WITHOUT_NAME: &[&str] = &["program-login", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE"];

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    let prefix = format!("temper-worker {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    // release.md, 3: twelve hexadecimal digits, or unknown outside git.
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn help(text: &str) -> bool {
    let Some((_, commands)) = text.split_once("\ncommands:\n") else { return false };
    let names: Vec<_> = commands
        .split("\n\n")
        .next()
        .expect("command lines")
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    text == WORKER.help()
        && names == ["serve", "program-login", "help"]
        && ["--config CONFIG", "--data DATA", "--cache CACHE", "TEMPER_CONFIG", "TEMPER_DATA", "TEMPER_CACHE"]
            .iter()
            .all(|word| words.contains(word))
}

fn serve_help(text: &str) -> bool {
    text == WORKER.command_help(WORKER.commands.first().expect("serve command"))
        && !text.starts_with("internal:")
        && text.contains("worker alone")
        && text.contains("temper-engine serve")
        && text.contains("temper-box serve")
}

fn login_help(text: &str) -> bool {
    text == WORKER.command_help(&PROGRAM_LOGIN) && !text.starts_with("internal:")
}

fn pages(text: &str) -> bool {
    text == temper_shell::help::answer(WORKER.binary, &temper_shell::help::Request::Index).expect("index page")
}

fn commands_page(text: &str) -> bool {
    text == temper_shell::table::commands_page(&WORKER)
}

#[derive(Debug)]
enum Stderr {
    Exact(&'static str),
    WorkerChecks,
}
impl Stderr {
    fn accepts(&self, text: &str) -> bool {
        match self {
            Self::Exact(expected) => text == *expected,
            Self::WorkerChecks => text.starts_with("temper-worker serve: ") && !text.contains("temper.json holds"),
        }
    }
}
use Stderr::Exact;
type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], Option<&'static str>, i32, Check, Stderr);

const ROWS: &[Case] = &[
    (&["--version"], None, 0, version, Exact("")),
    (&["-h"], None, 0, help, Exact("")),
    (&["--help"], None, 0, help, Exact("")),
    (&["serve", "--help"], None, 0, serve_help, Exact("")),
    (&["program-login", "--help"], None, 0, login_help, Exact("")),
    (&["help"], None, 0, help, Exact("")),
    (&["help", "pages"], None, 0, pages, Exact("")),
    (&["help", "commands"], None, 0, commands_page, Exact("")),
    (&["frobnicate"], None, 2, empty, Exact("temper-worker: no command frobnicate; temper-worker --help lists them\n")),
    (&[], None, 2, empty, Exact("temper-worker: no command; temper-worker --help lists them\n")),
    (
        &["serve", "extra"],
        None,
        2,
        empty,
        Exact("temper-worker serve: invalid arguments\nusage: temper-worker serve [DIRECTORIES]\n"),
    ),
    (
        SERVE,
        Some(ENGINE),
        1,
        empty,
        Exact("temper-worker serve: temper.json holds an engine alone; temper-engine serve runs it\n"),
    ),
    (
        SERVE,
        Some(STANDALONE),
        1,
        empty,
        Exact(
            "temper-worker serve: temper.json holds an engine and its worker in one process; temper-box serve runs it\n",
        ),
    ),
    (
        LOGIN,
        Some(ENGINE),
        1,
        empty,
        Exact("temper-worker program-login: temper.json holds an engine alone; temper-engine serve runs it\n"),
    ),
    (
        LOGIN,
        Some(STANDALONE),
        1,
        empty,
        Exact(
            "temper-worker program-login: temper.json holds an engine and its worker in one process; temper-box serve runs it\n",
        ),
    ),
    (
        LOGIN_WITHOUT_NAME,
        None,
        2,
        empty,
        Exact(
            "temper-worker program-login: invalid arguments\nusage: temper-worker program-login [DIRECTORIES] NAME\n",
        ),
    ),
    (SERVE, Some(WORKER_DOCUMENT), 1, empty, Stderr::WorkerChecks),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("worker-binary");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    std::fs::create_dir(&config).expect("configuration directory");
    let document = config.join("temper.json");
    let mut failures = Vec::new();
    for (arguments, contents, code, stdout, stderr) in ROWS {
        if let Some(contents) = contents {
            std::fs::write(&document, contents).expect("write document");
        } else if document.exists() {
            std::fs::remove_file(&document).expect("remove document");
        }
        let arguments: Vec<_> = arguments
            .iter()
            .map(|word| match *word {
                "CONFIG" => config.as_os_str(),
                "DATA" => data.as_os_str(),
                "CACHE" => cache.as_os_str(),
                word => std::ffi::OsStr::new(word),
            })
            .collect();
        let output = Command::new(skein_scratch::paths::binary("temper-worker"))
            .args(&arguments)
            .env_clear()
            .env("TEMPER_CONFIG", &config)
            .env("TEMPER_DATA", &data)
            .env("TEMPER_CACHE", &cache)
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-worker");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || !stderr.accepts(&err) {
            failures.push(format!("{arguments:?}, document {contents:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

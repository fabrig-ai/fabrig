//! The engine binary's table and dispatch (temper/docs/design/protocol/binaries.md, 2).
use std::process::{Command, Stdio};
use temper_engine_shell::{ENGINE, commands};
#[path = "../support/upgrade.rs"]
mod upgrade;
#[test]
fn upgrade_refuses_an_empty_deployment_directory() {
    upgrade::empty_directory("temper-engine");
}

#[path = "../../../../crates/temper-engine-shell/src/upgrade/tests/deployment.rs"]
mod deployment;

#[test]
fn upgrade_check_names_the_record_a_start_refuses_without_switching_generation() {
    let (scratch, _directories) = deployment::deployment();
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    let configuration = config.join("temper.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&configuration).expect("configuration")).expect("document");
    // The owner's subject fits; the kept person's full identity does not.
    document["engine"]["limits"]["overrides"]["people.identity_bytes"] = 7.into();
    std::fs::write(configuration, serde_json::to_vec(&document).expect("document bytes")).expect("configuration");
    let active_path = data.join("state/active");
    let active = std::fs::read(&active_path).expect("active generation");
    let refusal = "restart refused during core restore: PeopleBound(IdentityBytes), key People(Person(1))";
    let mut check_refusal = None;
    for arguments in [vec!["upgrade", "--check"], vec!["serve"]] {
        let checking = arguments[0] == "upgrade";
        let mut command = Command::new(skein_scratch::paths::binary("temper-engine"));
        command
            .args(arguments)
            .env_clear()
            .env("TEMPER_CONFIG", &config)
            .env("TEMPER_DATA", &data)
            .env("TEMPER_CACHE", &cache)
            .stdin(Stdio::null());
        if checking {
            command.arg(data.join("state"));
        }
        let output = command.output().expect("run engine");
        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{out}\n{err}");
        let text = format!("{out}{err}");
        let line = text.lines().find(|line| line.contains(refusal)).expect(&text).to_owned();
        if checking {
            // Only wall-time warnings may accompany a refusal; the refusal itself belongs on stdout.
            assert!(err.lines().all(engine_long_pass), "{err}");
            assert!(out.contains(refusal), "{out}");
            assert_eq!(std::fs::read(&active_path).expect("active after refused check"), active);
            check_refusal = Some(line);
        } else {
            assert_eq!(Some(line), check_refusal, "a start refuses the same record in the same words");
        }
    }
}

#[path = "../support/settings.rs"]
mod settings;

#[test]
fn upgrade_check_names_settings_out_of_their_numbers_without_switching_generation() {
    let (scratch, directories) = deployment::deployment();
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    let active_path = data.join("state/active");
    for moved in [false, true] {
        settings::seed_numbered_settings(&directories, moved);
        let active = std::fs::read(&active_path).expect("active generation");
        let before = deployment::tree(&data);
        let output = Command::new(skein_scratch::paths::binary("temper-engine"))
            .args(["upgrade", "--check"])
            .arg(data.join("state"))
            .env_clear()
            .env("TEMPER_CONFIG", &config)
            .env("TEMPER_DATA", &data)
            .env("TEMPER_CACHE", &cache)
            .stdin(Stdio::null())
            .output()
            .expect("run upgrade check");
        let out = String::from_utf8_lossy(&output.stdout);
        let err = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(i32::from(!moved)), "{out}\n{err}");
        assert!(err.lines().all(engine_long_pass), "{err}");
        if moved {
            assert!(out.contains("registry commit, jobs and restores pass"), "{out}");
            assert!(!out.contains("settings out of their numbers"), "{out}");
        } else {
            let refusal = out.lines().find(|line| line.contains("settings out of their numbers")).expect(&out);
            for words in
                ["changed template 9", "changed charter 17", "the stock's 1 to 32", "a deployment's own 33 to 64"]
            {
                assert!(refusal.contains(words), "{refusal}");
            }
        }
        assert_eq!(std::fs::read(&active_path).expect("active after check"), active);
        assert_eq!(deployment::tree(&data), before, "the check leaves the data directory unchanged");
    }
}

fn engine_long_pass(line: &str) -> bool {
    let Some(timings) =
        line.strip_prefix("temper-engine serve: long pass: ").and_then(|line| line.strip_suffix(" ms bound"))
    else {
        return false;
    };
    let Some((total, rest)) = timings.split_once(" ms, engine ") else { return false };
    let Some((engine, bound)) = rest.split_once(" ms of it, past the ") else { return false };
    [total, engine, bound]
        .iter()
        .all(|number| number.bytes().all(|byte| byte.is_ascii_digit()) && number.parse::<u128>().is_ok())
}

#[test]
fn only_engine_long_pass_diagnostics_are_accepted_on_stderr() {
    let warning = "temper-engine serve: long pass: 2378 ms, engine 2378 ms of it, past the 1000 ms bound";
    assert!(engine_long_pass(warning));
    for line in [
        String::new(),
        warning.replace("2378", "+2378"),
        warning.replace("1000", ""),
        warning.replace("engine 2378", "drain 2378"),
        format!("{warning}; extra output"),
        "temper-engine serve: restart refused during core restore: PeopleBound(IdentityBytes), key People(Person(1))"
            .into(),
    ] {
        assert!(!engine_long_pass(&line), "{line}");
    }
}

const WORKER: &str = r#"{"workers":[{"engine":{"socket":"127.0.0.1:1"}}]}"#;
const STANDALONE: &str = r#"{"engine":{"hosts":{"listener":[{"in_process":"workers"}]}}}"#;
const SERVE: &[&str] = &["serve", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE"];
const FETCH: &[&str] = &["forge", "fetch", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE", "1"];

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    eprintln!("--version stdout:\n{text}");
    let prefix = format!("temper-engine {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    // release.md, 3: twelve hexadecimal digits, or unknown outside git.
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn help(text: &str) -> bool {
    let rows: Vec<_> =
        text.lines().filter(|line| line.starts_with("  ")).filter_map(|line| line.split_whitespace().next()).collect();
    text == ENGINE.help()
        && text.contains("upgrade --workers")
        && text
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .contains("Judges the workers alone over a copy; switches nothing.")
        && ["serve", "versions", "upgrade", "checkpoint", "help"].iter().all(|word| rows.contains(word))
        && ["forge", "box", "program-login"].iter().all(|word| !rows.contains(word))
}

fn serve_help(text: &str) -> bool {
    text == ENGINE.command_help(&commands::SERVE)
        && !text.starts_with("internal:")
        && text.contains("engine alone")
        && text.contains("temper-worker serve")
        && text.contains("temper-box serve")
}

fn versions(text: &str) -> bool {
    eprintln!("versions stdout:\n{text}");
    text == temper_engine_shell::versions()
        && text.lines().take(4).eq(["level skein 0", "level smith 0", "level jig 6", "level temper 14"])
        && text.lines().any(|line| line == format!("family temper {}", temper_protocol_store::STORE_VERSION))
        && text.lines().any(|line| line.starts_with("wire temper-worker-wire "))
}

fn pages(text: &str) -> bool {
    text == temper_shell::help::answer(ENGINE.binary, &temper_shell::help::Request::Index).expect("index page")
}

fn commands_page(text: &str) -> bool {
    text == temper_shell::table::commands_page(&ENGINE)
}

fn forge_help(text: &str) -> bool {
    // The notice is wrapped to 80 columns by skein's help renderer.
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .starts_with("internal: not part of temper-engine's public command line; it may change without notice.")
        && text == ENGINE.command_help(&commands::FETCH)
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], Option<&'static str>, i32, Check, &'static str);

const CHECKPOINT_USAGE: &str = "temper-engine checkpoint: invalid arguments\nusage: temper-engine checkpoint [DIRECTORIES]\nusage: temper-engine checkpoint --hold NAME [DIRECTORIES] -- COMMAND [ARGUMENT...]\nusage: temper-engine checkpoint --remove NAME [DIRECTORIES]\n";
const ROWS: &[Case] = &[
    (&["checkpoint", "--remove", "12-ff"], None, 2, empty, CHECKPOINT_USAGE),
    (&["checkpoint", "--hold", "00000000000000000012-00000000000000ff"], None, 2, empty, CHECKPOINT_USAGE),
    (&["--version"], None, 0, version, ""),
    (&["-h"], None, 0, help, ""),
    (&["--help"], None, 0, help, ""),
    (&["serve", "--help"], None, 0, serve_help, ""),
    (&["forge", "fetch", "--help"], None, 0, forge_help, ""),
    (&["help"], None, 0, help, ""),
    (&["help", "pages"], None, 0, pages, ""),
    (&["help", "commands"], None, 0, commands_page, ""),
    (&["help", "nope"], None, 1, empty, "temper-engine: no page nope; temper-engine help lists them\n"),
    (&["box"], None, 2, empty, "temper-engine: no command box; temper-engine --help lists them\n"),
    (
        &["program-login", "a"],
        None,
        2,
        empty,
        "temper-engine: no command program-login; temper-engine --help lists them\n",
    ),
    (&[], None, 2, empty, "temper-engine: no command; temper-engine --help lists them\n"),
    (&["--version", "extra"], None, 2, empty, "temper-engine: no command --version; temper-engine --help lists them\n"),
    (
        SERVE,
        Some(WORKER),
        1,
        empty,
        "temper-engine serve: temper.json holds a worker alone; temper-worker serve runs it\n",
    ),
    (
        SERVE,
        Some(STANDALONE),
        1,
        empty,
        "temper-engine serve: temper.json holds an engine and its worker in one process; temper-box serve runs it\n",
    ),
    (
        FETCH,
        Some(WORKER),
        1,
        empty,
        "temper-engine forge fetch: temper.json holds a worker alone; temper-worker serve runs it\n",
    ),
    (
        &["upgrade", "--check"],
        None,
        2,
        empty,
        "temper-engine upgrade --check: invalid arguments\nusage: temper-engine upgrade --check [DIRECTORIES] DIRECTORY\n",
    ),
    (
        &["upgrade", "--workers"],
        None,
        2,
        empty,
        "temper-engine upgrade --workers: invalid arguments\nusage: temper-engine upgrade --workers [DIRECTORIES] DIRECTORY\n",
    ),
    (
        &["upgrade", "--back", "extra"],
        None,
        2,
        empty,
        "temper-engine upgrade --back: invalid arguments\nusage: temper-engine upgrade --back [DIRECTORIES]\n",
    ),
    (&["versions"], None, 0, versions, ""),
    (
        &["versions", "extra"],
        None,
        2,
        empty,
        "temper-engine versions: invalid arguments\nusage: temper-engine versions\n",
    ),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("engine-binary");
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
        let output = Command::new(skein_scratch::paths::binary("temper-engine"))
            .args(&arguments)
            .env_clear()
            .env("TEMPER_CONFIG", &config)
            .env("TEMPER_DATA", &data)
            .env("TEMPER_CACHE", &cache)
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-engine");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || err != *stderr {
            failures.push(format!("{arguments:?}, document {contents:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

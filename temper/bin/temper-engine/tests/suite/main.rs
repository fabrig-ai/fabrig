//! The engine binary's table and dispatch (temper/docs/design/protocol/binaries.md, 2).
use std::process::{Command, Stdio};
use temper_config_shell::{File, Shape, render};
use temper_engine_shell::ENGINE;
#[path = "../support/upgrade.rs"]
mod upgrade;
#[test]
fn upgrade_refuses_an_empty_deployment_directory() {
    upgrade::empty_directory("temper-engine");
}

use temper_deployment_fixtures as deployment;

#[test]
fn upgrade_check_names_the_record_a_start_refuses_without_switching_generation() {
    let (scratch, _directories) = deployment::deployment();
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    // The owner's subject fits; the kept person's full identity does not.
    let held = data.join("limits-held");
    std::fs::write(&held, b"people.identity_bytes 7\n").expect("held startup figures");
    let held_line = format!(
        "limits held: {}; the limit settings are not taken; remove it and restart to take them",
        held.display()
    );
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
            assert!(err.lines().all(|line| engine_long_pass(line) || line == held_line), "{err}");
            assert_eq!(err.lines().filter(|line| *line == held_line).count(), 1);
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
    use settings::Numbers;
    let (scratch, directories) = deployment::deployment();
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    let active_path = data.join("state/active");
    for numbers in [Numbers::Kept, Numbers::Moved, Numbers::Past, Numbers::Taken] {
        settings::seed_numbered_settings(&directories, numbers);
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
        let refused = matches!(numbers, Numbers::Past | Numbers::Taken);
        assert_eq!(output.status.code(), Some(i32::from(refused)), "{out}\n{err}");
        assert!(err.lines().all(engine_long_pass), "{err}");
        if !refused {
            assert!(out.contains("registry commit, jobs and restores pass; opened in "), "{out}");
            assert!(!out.contains("settings out of their numbers"), "{out}");
        } else if numbers == Numbers::Past {
            let refusal = out.lines().find(|line| line.contains("settings out of their numbers")).expect(&out);
            for words in ["changed charter 65", "the stock's 1 to 32", "a deployment's own 33 to 64"] {
                assert!(refusal.contains(words), "{refusal}");
            }
        } else {
            for words in [
                "the stock's new settings take numbers a deployment's own hold",
                "charter 1",
                "move each to 33 to 64 on the build that runs now, then start this one",
            ] {
                assert!(out.contains(words), "{out}");
            }
        }
        assert_eq!(std::fs::read(&active_path).expect("active after check"), active);
        assert_eq!(deployment::tree(&data), before, "the check leaves the data directory unchanged");
    }
}

#[test]
fn upgrade_check_restores_under_the_limits_a_start_takes_from_the_copy() {
    let (scratch, directories) = deployment::deployment();
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    // The copy's own `limits` setting raises core.settings.versions past the stock's, and the copy holds more.
    let versions = settings::seed_raised_versions(&directories, 64);
    let refusal = format!("restart refused during core restore: Versions, held {versions}, limit {}", versions - 1);
    let hold = data.join("limits-held");
    let held_line = format!(
        "limits held: {}; the limit settings are not taken; remove it and restart to take them",
        hold.display()
    );
    let active_path = data.join("state/active");
    let active = std::fs::read(&active_path).expect("active generation");
    let before = deployment::tree(&data.join("state"));
    // Held at the stock, a start takes neither kind and refuses what it holds past the stock's figure.
    for stock in [false, true] {
        if stock {
            std::fs::write(&hold, b"").expect("limits held at the stock");
        }
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
        assert_eq!(output.status.code(), Some(i32::from(stock)), "{out}\n{err}");
        assert!(err.lines().all(|line| engine_long_pass(line) || (stock && line == held_line)), "{err}");
        if stock {
            assert!(out.contains(&refusal), "{out}");
        } else {
            assert!(out.contains("registry commit, jobs and restores pass; opened in "), "{out}");
            assert!(!out.contains("restart refused"), "{out}");
        }
        assert_eq!(std::fs::read(&active_path).expect("active after check"), active);
        assert_eq!(deployment::tree(&data.join("state")), before, "the check leaves the copy unchanged");
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

const SERVE: &[&str] = &["serve", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE"];
const FETCH: &[&str] = &["forge", "fetch", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE", "1"];

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    text == format!("{}\n", temper_version::line("temper-engine"))
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], Option<Shape>, i32, Check, &'static str);

const CHECKPOINT_USAGE: &str = "temper-engine checkpoint: invalid arguments\nusage: temper-engine checkpoint [DIRECTORIES]\nusage: temper-engine checkpoint --hold NAME [DIRECTORIES] -- COMMAND [ARGUMENT...]\nusage: temper-engine checkpoint --remove NAME [DIRECTORIES]\n";
const ROWS: &[Case] = &[
    (&["--help"], None, 0, |text| text == ENGINE.help(), ""),
    (&["--version"], None, 0, version, ""),
    (&[], None, 2, empty, "temper-engine: no command; temper-engine --help lists them\n"),
    (&["versions"], None, 0, |text| text == temper_engine_shell::versions(), ""),
    (
        SERVE,
        Some(Shape::WorkerAlone),
        1,
        empty,
        "temper-engine serve: {file}:3: engine: a key of temper-worker's file, not temper-engine's\ntemper-engine serve: {file}: workers: required in temper-engine's file\n",
    ),
    (
        FETCH,
        Some(Shape::WorkerAlone),
        1,
        empty,
        "temper-engine forge fetch: forge 1: {file}:3: engine: a key of temper-worker's file, not temper-engine's\ntemper-engine forge fetch: {file}: workers: required in temper-engine's file\n",
    ),
    (&["checkpoint", "--remove", "12-ff"], None, 2, empty, CHECKPOINT_USAGE),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("engine-binary");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    std::fs::create_dir(&config).expect("configuration directory");
    std::fs::create_dir(&data).expect("data directory");
    let directories = temper_agent_fixtures::directories(&config);
    let jig = temper_service_shell::jig_directories(&directories);
    let setup =
        temper_forge_setup::setup_file::chosen(temper_forge_setup::machine::BoxKind::Fake, 3007, b"sam", false, &jig)
            .expect("fake setup");
    temper_forge_setup_shell::setup_file::write(&jig, 1, &setup).expect("setup file");
    std::fs::create_dir_all(data.join("box/groups")).expect("groups directory");
    std::fs::write(data.join("box/groups/1"), b"fake\n").expect("group kind");
    let document = config.join(temper_config_shell::FILE);
    let mut failures = Vec::new();
    for (arguments, contents, code, stdout, stderr) in ROWS {
        if let Some(contents) = contents {
            std::fs::write(&document, shape_file(*contents)).expect("write document");
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
        if output.status.code() != Some(*code)
            || !stdout(&out)
            || err != stderr.replace("{file}", temper_config_shell::FILE)
        {
            failures.push(format!("{arguments:?}, document {contents:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn shape_file(shape: Shape) -> Box<[u8]> {
    let address = "127.0.0.1:1".parse().expect("fixture address");
    let file = match shape {
        Shape::Standalone => File { listen: Some(address), ..File::default() },
        Shape::EngineAlone => File { workers: Some(address), ..File::default() },
        Shape::WorkerAlone => File { engine: Some(address), ..File::default() },
    };
    render(&file, shape)
}

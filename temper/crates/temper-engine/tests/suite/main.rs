//! The engine binary's table and dispatch (temper/docs/design/protocol/README.md, 4.2).
use std::process::{Command, Stdio};

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
    let rows: Vec<_> = text.lines().filter_map(|line| line.trim().strip_prefix("temper-engine ")).collect();
    text.lines().next() == Some("usage: temper-engine COMMAND [ARGUMENT...]")
        && ["serve [DIRECTORIES]", "forge fetch [DIRECTORIES] FORGE", "versions"]
            .iter()
            .all(|words| rows.contains(words))
        && rows.iter().all(|row| ![Some("box"), Some("program-login")].contains(&row.split_whitespace().next()))
}

fn serve_help(text: &str) -> bool {
    text.starts_with("usage: temper-engine serve [DIRECTORIES]\n")
        && text.contains("engine alone")
        && text.contains("temper-worker serve")
        && text.contains("temper serve")
}

fn versions(text: &str) -> bool {
    eprintln!("versions stdout:\n{text}");
    text.lines().next() == Some("level skein none")
        && text.lines().any(|line| line == format!("family temper {}", temper_protocol_store::STORE_VERSION))
        && text.lines().any(|line| line.starts_with("wire temper-payload "))
}

fn pages(text: &str) -> bool {
    text.starts_with("# ") && text.contains("temper")
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], Option<&'static str>, i32, Check, &'static str);

const ROWS: &[Case] = &[
    (&["--version"], None, 0, version, ""),
    (&["--help"], None, 0, help, ""),
    (&["serve", "--help"], None, 0, serve_help, ""),
    (&["help"], None, 0, pages, ""),
    (&["box"], None, 1, empty, "temper-engine: no command box; temper-engine --help lists them\n"),
    (
        &["program-login", "a"],
        None,
        1,
        empty,
        "temper-engine: no command program-login; temper-engine --help lists them\n",
    ),
    (&[], None, 1, empty, "temper-engine: no command; temper-engine --help lists them\n"),
    (&["--version", "extra"], None, 1, empty, "temper-engine: no command --version; temper-engine --help lists them\n"),
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
        "temper-engine serve: temper.json holds an engine and its worker in one process; the box's temper serve runs it\n",
    ),
    (
        FETCH,
        Some(WORKER),
        1,
        empty,
        "temper-engine forge fetch: temper.json holds a worker alone; temper-worker serve runs it\n",
    ),
    (&["versions"], None, 0, versions, ""),
    (
        &["versions", "extra"],
        None,
        1,
        empty,
        "temper-engine versions: invalid arguments\nusage: temper-engine versions \n",
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

//! The worker binary's table and dispatch (temper/docs/design/protocol/binaries.md, 5).
use std::process::{Command, Stdio};
use temper_config_shell::{File, Shape, render};
use temper_worker_shell::WORKER;

const SERVE: &[&str] = &["serve", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE"];
const LOGIN: &[&str] = &["program-login", "--config", "CONFIG", "--data", "DATA", "--cache", "CACHE", "work"];

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    text == format!("{}\n", temper_version::line("temper-worker"))
}

#[derive(Debug)]
enum Stderr {
    Exact(&'static str),
    WorkerChecks,
}
impl Stderr {
    fn accepts(&self, text: &str) -> bool {
        match self {
            Self::Exact(expected) => text == expected.replace("{file}", temper_config_shell::FILE),
            Self::WorkerChecks => {
                text.starts_with("temper-worker serve: ") && !text.contains(&format!("{}:", temper_config_shell::FILE))
            }
        }
    }
}
use Stderr::Exact;
type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], Option<Shape>, i32, Check, Stderr);

const ROWS: &[Case] = &[
    (&["--help"], None, 0, |text| text == WORKER.help(), Exact("")),
    (&["--version"], None, 0, version, Exact("")),
    (&["frobnicate"], None, 2, empty, Exact("temper-worker: no command frobnicate; temper-worker --help lists them\n")),
    (SERVE, Some(Shape::WorkerAlone), 1, empty, Stderr::WorkerChecks),
    (
        LOGIN,
        Some(Shape::EngineAlone),
        1,
        empty,
        Exact(
            "temper-worker program-login: {file}:4: workers: a key of temper-engine's file, not temper-worker's\ntemper-worker program-login: {file}: engine: required in temper-worker's file\n",
        ),
    ),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("worker-binary");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    std::fs::create_dir(&config).expect("configuration directory");
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

fn shape_file(shape: Shape) -> Box<[u8]> {
    let address = "127.0.0.1:1".parse().expect("fixture address");
    let file = match shape {
        Shape::Standalone => File { listen: Some(address), ..File::default() },
        Shape::EngineAlone => File { workers: Some(address), ..File::default() },
        Shape::WorkerAlone => File { engine: Some(address), ..File::default() },
    };
    render(&file, shape)
}

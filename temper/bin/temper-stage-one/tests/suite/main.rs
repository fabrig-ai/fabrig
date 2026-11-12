//! The stage-one binary's table (temper/docs/design/testing/tools.md, 6).
use std::process::{Command, Stdio};

const HELP: &str = "temper-stage-one: configuring's stage 1: its golden, check, judge and evidence.";
const INTERNAL: &str = "internal: not part of temper-stage-one's public command line; it may change without notice.";

fn empty(text: &str) -> bool {
    text.is_empty()
}
fn paragraph(text: &str) -> String {
    text.lines().take_while(|line| !line.is_empty()).collect::<Vec<_>>().join(" ")
}
fn help(text: &str) -> bool {
    paragraph(text) == HELP && !text.contains("\ncommands:\n") && !text.contains("\nusage:\n")
}
fn version(text: &str) -> bool {
    let prefix = format!("temper-stage-one {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
fn internal(text: &str) -> bool {
    paragraph(text) == INTERNAL
}

#[derive(Debug)]
enum Stderr {
    Exact(&'static str),
    CollectRead,
    CollectUsage,
}
impl Stderr {
    fn accepts(&self, text: &str) -> bool {
        match self {
            Self::Exact(expected) => text == *expected,
            Self::CollectUsage => {
                text.starts_with("temper-stage-one collect: missing --temper\nusage:\n  temper-stage-one collect ")
            }
            Self::CollectRead => {
                text.starts_with("temper-stage-one: ./absent ctl ctl revision show 1 --json: ")
                    && !text.contains("usage:")
            }
        }
    }
}
use Stderr::Exact;
type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], i32, Check, Stderr);
const ROWS: &[Case] = &[
    (&["--help"], 0, help, Exact("")),
    (&["-h"], 0, help, Exact("")),
    (&["help"], 0, help, Exact("")),
    (&["--version"], 0, version, Exact("")),
    (&["obligations", "--help"], 0, internal, Exact("")),
    (&["collect", "--help"], 0, internal, Exact("")),
    (&["check", "--help"], 0, internal, Exact("")),
    (&["judge", "--help"], 0, internal, Exact("")),
    (&["verdict", "--help"], 0, internal, Exact("")),
    (&["collect"], 2, empty, Stderr::CollectUsage),
    (&["nope"], 2, empty, Exact("temper-stage-one: no command nope; temper-stage-one --help says more\n")),
    (
        &["check"],
        2,
        empty,
        Exact("temper-stage-one check: EVIDENCE is not given\nusage:\n  temper-stage-one check EVIDENCE [--json]\n"),
    ),
    (
        &["obligations", "--nope"],
        2,
        empty,
        Exact("temper-stage-one obligations: invalid arguments\nusage:\n  temper-stage-one obligations [--json]\n"),
    ),
    (
        &["judge"],
        2,
        empty,
        Exact("temper-stage-one judge: EVIDENCE is not given\nusage:\n  temper-stage-one judge EVIDENCE\n"),
    ),
    (
        &["verdict"],
        2,
        empty,
        Exact(
            "temper-stage-one verdict: EVIDENCE is not given\nusage:\n  temper-stage-one verdict EVIDENCE [--json]\n",
        ),
    ),
    (
        &[
            "collect",
            "--temper",
            "./absent",
            "--ctl",
            "ctl",
            "--project",
            "1",
            "--revision",
            "1",
            "--proposal",
            "t1/1",
            "--chat",
            "1",
            "--forge",
            "code.example",
            "--repository",
            "team/rules",
            "evidence",
        ],
        2,
        empty,
        Stderr::CollectRead,
    ),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("stage-one-binary");
    let mut failures = Vec::new();
    let mut help_page = None;
    for (arguments, code, stdout, stderr) in ROWS {
        let output = Command::new(env!("CARGO_BIN_EXE_temper-stage-one"))
            .args(*arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-stage-one");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        let same_help = match *arguments {
            ["--help"] => {
                help_page = Some(out.clone());
                true
            }
            ["-h" | "help"] => help_page.as_deref() == Some(out.as_str()),
            _ => true,
        };
        if output.status.code() != Some(*code) || !stdout(&out) || !stderr.accepts(&err) || !same_help {
            failures.push(format!(
                "{arguments:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}",
                output.status.code()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

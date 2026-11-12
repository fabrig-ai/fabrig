//! The static host's table and refusals (temper/docs/design/testing/tools.md, 6).
use std::process::{Command, Stdio};

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn help(text: &str) -> bool {
    let opening = text.lines().take_while(|line| !line.is_empty()).collect::<Vec<_>>().join(" ");
    opening == "temper-playground-serve: the browser tier's static host for the playground's bundle."
        && !text.contains("OUT")
        && text.contains("-h, --help")
        && text.contains("--version")
        && text.ends_with("Read temper/docs/design/playground.md, section 7.\n")
        && text.lines().all(|line| line.chars().count() <= 80)
}

fn version(text: &str) -> bool {
    let prefix = format!("temper-playground-serve {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

#[derive(Debug)]
enum Stderr {
    Exact(&'static str),
    MissingOutput,
}
impl Stderr {
    fn accepts(&self, text: &str, missing: &str) -> bool {
        match self {
            Self::Exact(expected) => text == *expected,
            Self::MissingOutput => text == missing,
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
    (&[], 2, empty, Exact("temper-playground-serve: OUT is not given\nusage:\n  temper-playground-serve OUT\n")),
    (&["--nope"], 2, empty, Exact("temper-playground-serve: no flag --nope\nusage:\n  temper-playground-serve OUT\n")),
    (
        &["one", "two"],
        2,
        empty,
        Exact("temper-playground-serve: expected one OUT\nusage:\n  temper-playground-serve OUT\n"),
    ),
    (&["missing"], 1, empty, Stderr::MissingOutput),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("playground-serve-binary");
    let missing = scratch.path().join("missing").canonicalize().expect_err("output does not exist");
    let missing = format!("temper-playground-serve: {missing}\n");
    let mut help_page = None;
    let mut failures = Vec::new();
    for (arguments, code, stdout, stderr) in ROWS {
        let output = Command::new(env!("CARGO_BIN_EXE_temper-playground-serve"))
            .args(*arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-playground-serve");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if [Some("--help"), Some("-h"), Some("help")].contains(&arguments.first().copied()) {
            let page = help_page.get_or_insert_with(|| out.clone());
            assert_eq!(&out, page, "all help aliases print the same page");
        }
        if output.status.code() != Some(*code) || !stdout(&out) || !stderr.accepts(&err, &missing) {
            failures.push(format!(
                "{arguments:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}",
                output.status.code()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

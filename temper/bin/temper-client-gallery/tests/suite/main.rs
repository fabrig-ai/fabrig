//! The gallery binary's table (temper/docs/design/testing/tools.md, 6).
use std::process::{Command, Stdio};

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    let prefix = format!("temper-client-gallery {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn help(text: &str) -> bool {
    let opening = text.lines().take_while(|line| !line.is_empty()).collect::<Vec<_>>().join(" ");
    opening == "temper-client-gallery: the browser's pages in named states, printed for screenshots."
        && !text.contains("OUT")
        && text.contains("-h, --help")
        && text.contains("--version")
        && text.ends_with("Read temper/docs/design/client/web/gallery.md, section 1.\n")
}

#[derive(Debug)]
enum Stderr {
    Exact(&'static str),
}
impl Stderr {
    fn accepts(&self, text: &str) -> bool {
        match self {
            Self::Exact(expected) => text == *expected,
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
    (&[], 2, empty, Exact("temper-client-gallery: OUT is not given\nusage:\n  temper-client-gallery OUT\n")),
    (&["--nope"], 2, empty, Exact("temper-client-gallery: no flag --nope\nusage:\n  temper-client-gallery OUT\n")),
    (
        &["one", "two"],
        2,
        empty,
        Exact("temper-client-gallery: expected one OUT\nusage:\n  temper-client-gallery OUT\n"),
    ),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("gallery-binary");
    let mut failures = Vec::new();
    let mut page = None;
    for (arguments, code, stdout, stderr) in ROWS {
        let output = Command::new(env!("CARGO_BIN_EXE_temper-client-gallery"))
            .args(*arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-client-gallery");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || !stderr.accepts(&err) {
            failures.push(format!(
                "{arguments:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}",
                output.status.code()
            ));
        }
        if help(&out) {
            if let Some(expected) = &page {
                assert_eq!(&out, expected, "help's three spellings print the same page");
            } else {
                page = Some(out);
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(std::fs::read_dir(scratch.path()).expect("scratch directory").count(), 0, "no row prints a gallery");
}

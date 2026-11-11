//! The recorder binary's table (temper/docs/design/testing/tools.md, 6).
use std::process::{Command, Stdio};

fn empty(text: &str) -> bool {
    text.is_empty()
}

fn version(text: &str) -> bool {
    let prefix = format!("temper-git-transcripts {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn help(text: &str) -> bool {
    let opening = text.lines().take_while(|line| !line.is_empty()).collect::<Vec<_>>().join(" ");
    opening == "temper-git-transcripts: git's transcripts, recorded from real git against a throwaway Forgejo."
        && !text.contains("--git")
        && text.contains("-h, --help")
        && text.contains("--version")
        && text.ends_with("Read temper/docs/design/protocol/git.md, section 9.\n")
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], i32, Check, &'static str);
const ROWS: &[Case] = &[
    (&["--help"], 0, help, ""),
    (&["-h"], 0, help, ""),
    (&["help"], 0, help, ""),
    (&["--version"], 0, version, ""),
    (
        &["--nope"],
        2,
        empty,
        "temper-git-transcripts: unknown argument --nope\nusage:\n  temper-git-transcripts --git PATH --scratch DIR --out DIR\n",
    ),
    (
        &[],
        2,
        empty,
        "temper-git-transcripts: --git is not given\nusage:\n  temper-git-transcripts --git PATH --scratch DIR --out DIR\n",
    ),
    (
        &["--git", "/usr/bin/git", "--scratch", "scratch", "--out", "out"],
        1,
        empty,
        "temper-git-transcripts: TEMPER_FORGEJO must name the pinned Forgejo\n",
    ),
];

#[test]
fn the_binary_answers_its_table_and_refuses_other_shapes() {
    let scratch = skein_scratch::Scratch::new("git-transcripts-binary");
    let mut failures = Vec::new();
    let mut page = None;
    for (arguments, code, stdout, stderr) in ROWS {
        let output = Command::new(env!("CARGO_BIN_EXE_temper-git-transcripts"))
            .args(*arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run temper-git-transcripts");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || err != *stderr {
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
    assert_eq!(std::fs::read_dir(scratch.path()).expect("scratch directory").count(), 0, "no row starts the recorder");
}

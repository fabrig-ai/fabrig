//! The binary's table (temper/docs/design/testing/tools.md, 6).
use std::process::{Command, Stdio};
fn opening(text: &str) -> String {
    text.lines().take_while(|line| !line.is_empty()).collect::<Vec<_>>().join(" ")
}
fn help(text: &str) -> bool {
    opening(text) == "temper-client-walk: journeys walked in a world, their marks printed as pages."
        && !text.contains("print [") && !text.contains("  pages")
        && text.contains("-h, --help") && text.contains("--version")
        && text.split_whitespace().collect::<Vec<_>>().join(" ").ends_with("temper-client-walk COMMAND --help prints a command; read temper/docs/design/client/journey-walks-world.md, section 1, and temper/docs/design/client/journey-walks.md, section 5.")
}
fn version(text: &str) -> bool {
    let prefix = format!("temper-client-walk {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else { return false };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
fn command_help(text: &str) -> bool {
    opening(text) == "internal: not part of temper-client-walk's public command line; it may change without notice."
}
fn print_help(text: &str) -> bool {
    command_help(text) && text.contains("print [--out DIR] [NAME]...") && text.contains("--out DIR")
}
fn pages_help(text: &str) -> bool {
    command_help(text) && text.contains("temper-client-walk pages")
}
fn empty(text: &str) -> bool {
    text.is_empty()
}
fn unknown(text: &str) -> bool {
    text.starts_with("temper-client-walk:") && text.contains("no-such-command")
}
type Check = fn(&str) -> bool;
type Row = (&'static [&'static str], i32, Check, Check);
const ROWS: &[Row] = &[
    (&["--help"], 0, help, empty),
    (&["-h"], 0, help, empty),
    (&["help"], 0, help, empty),
    (&["--version"], 0, version, empty),
    (&["print", "--help"], 0, print_help, empty),
    (&["pages", "--help"], 0, pages_help, empty),
    (&["no-such-command"], 2, empty, unknown),
];
#[test]
fn the_binary_answers_its_table_and_refuses_unknown_commands() {
    let scratch = skein_scratch::Scratch::new("walk-table");
    let mut page = None;
    for (arguments, code, stdout, stderr) in ROWS {
        let output = Command::new(skein_scratch::paths::binary("temper-client-walk"))
            .args(*arguments)
            .env_clear()
            .current_dir(scratch.path())
            .stdin(Stdio::null())
            .output()
            .expect("run binary");
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        assert_eq!(output.status.code(), Some(*code), "{arguments:?}: {err}");
        assert!(stdout(&out), "{arguments:?}: stdout {out:?}");
        assert!(stderr(&err), "{arguments:?}: stderr {err:?}");
        if help(&out) {
            if let Some(previous) = &page {
                assert_eq!(&out, previous);
            } else {
                page = Some(out);
            }
        }
    }
    assert_eq!(std::fs::read_dir(scratch.path()).expect("scratch").count(), 0);
}

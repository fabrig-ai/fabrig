use std::path::{Path, PathBuf};

use crate::host::{Answer, Ask, Gathered, Host, Program, Ran, Step, Stream};

use super::run;

const NAME: &str = "look-request";
const TIP: &str = "abcdef0123456789abcdef0123456789abcdef01";
const PARENT: &str = "0123456789abcdef0123456789abcdef01234567";
const SHORT: &str = "abcdef0";
const ROOT: &str = "/landing tree";
const SCRATCH: &str = "/landing tree/.git/look-compare";
const PRINT: &str = "/landing tree/.git/look-compare/print";
const WEB: &str = "temper/docs/design/client/web/journeys";
const TUI: &str = "temper/docs/design/client/tui/journeys";
const SHELL_WEB: &str = "temper/crates/temper-client-shell-web";
const ON_DISK_WALK: &str = "walk 2\nweb mark\n";
const SIGN_IN: &str = "sign-in";
const SIGN_IN_PAGE: &str = "temper/docs/design/client/web/journeys/sign-in/02.html";
const SIGN_IN_SCRATCH: &str = "/landing tree/.git/look-compare/web--sign-in--02.html";
const SIGN_IN_P: &str = "a\nb\nc\n";
const SIGN_IN_C: &str = "a\nB\nc\n";
const HEADER: &str = "| Item | Result |\n|---|---|\n";
const FILL: &str = "| each state the message names in words | to fill, a row each |\n";

#[derive(Default)]
struct Steps(Vec<Step>);

impl Steps {
    fn step(&mut self, ask: Ask, answer: Answer) {
        self.0.push(Step { ask, answer });
    }

    fn git(&mut self, args: &[&str], directory: Option<&str>, stdout: &str) {
        self.git_reply(args, directory, 0, stdout, "");
    }

    fn git_reply(&mut self, args: &[&str], directory: Option<&str>, code: i32, stdout: &str, stderr: &str) {
        let mut program = Program::new(b"git", &args.iter().map(|arg| arg.as_bytes()).collect::<Vec<_>>());
        program.directory = directory.map(PathBuf::from);
        program.stderr = Stream::Keep;
        self.step(
            Ask::Run(program),
            Answer::Ran(Ran { code: Some(code), stdout: stdout.as_bytes().into(), stderr: stderr.as_bytes().into() }),
        );
    }

    fn fetch(&mut self, tip: &str) {
        self.git(
            &[
                "fetch",
                "--no-tags",
                "--no-recurse-submodules",
                "--refmap=",
                "origin",
                "refs/heads/docs/look/look-request",
            ],
            None,
            "",
        );
        self.git(&["rev-parse", "--verify", "FETCH_HEAD^{commit}"], None, &format!("{tip}\n"));
    }

    fn request(&mut self, paths: &[&str]) {
        self.fetch(TIP);
        self.git(&["show", "-s", "--format=%P", TIP], None, &format!("{PARENT}\n"));
        self.git(
            &["show", "-s", "--format=%(trailers:key=Look-request,unfold)", TIP],
            None,
            "Look-request: look-request\n\n",
        );
        self.git(&["rev-parse", "--show-toplevel"], None, &format!("{ROOT}\n"));
        self.scratch();
        let mut listing = String::new();
        for path in paths {
            listing.push_str(path);
            listing.push('\0');
        }
        self.git(
            &["diff-tree", "--no-commit-id", "--name-only", "--no-renames", "-r", "-z", PARENT, TIP, "--", WEB, TUI],
            Some(ROOT),
            &listing,
        );
    }

    fn scratch(&mut self) {
        self.git(&["rev-parse", "--path-format=absolute", "--git-path", "look-compare"], None, &format!("{SCRATCH}\n"));
        self.step(Ask::Remove(SCRATCH.into()), Answer::Done);
        self.step(Ask::Directory(SCRATCH.into()), Answer::Done);
    }

    fn blob(&mut self, commit: &str, path: &str, bytes: Option<&str>) {
        const OBJECT: &str = "aaaaabbbbbcccccdddddeeeeefffff0000011111";
        let tree = if bytes.is_some() { format!("100644 blob {OBJECT}\t{path}\0") } else { String::new() };
        self.git(&["--literal-pathspecs", "ls-tree", "--full-tree", "-z", commit, "--", path], None, &tree);
        if let Some(bytes) = bytes {
            self.git(&["cat-file", "blob", OBJECT], None, bytes);
        }
    }

    fn read(&mut self, path: &str, bytes: Option<&str>) {
        let answer = bytes.map_or(Answer::Missing, |bytes| Answer::Bytes(bytes.as_bytes().into()));
        self.step(Ask::Read(path.into()), answer);
    }

    fn write(&mut self, directory: &str, leaf: &str, content: &str) {
        self.step(Ask::Write(Path::new(directory).join(leaf), content.as_bytes().into()), Answer::Done);
    }

    fn print(&mut self, names: &[&str], code: i32, stdout: &str, stderr: &str) {
        let mut program = Program::new(
            b"cargo",
            &[
                b"run",
                b"--locked",
                b"--quiet",
                b"-p",
                b"temper-client-walk",
                b"--",
                b"print",
                b"--out",
                PRINT.as_bytes(),
            ],
        );
        program.directory = Some(ROOT.into());
        program.environment.push((b"TEMPER_INTERNAL_BUNDLE".as_slice().into(), b"none".as_slice().into()));
        program.arguments.extend(names.iter().map(|name| name.as_bytes().into()));
        self.step(
            Ask::Run(program),
            Answer::Ran(Ran { code: Some(code), stdout: stdout.as_bytes().into(), stderr: stderr.as_bytes().into() }),
        );
    }

    fn sign_in_request(&mut self) {
        self.request(&[SIGN_IN_PAGE]);
        self.blob(TIP, &format!("{WEB}/{SIGN_IN}/walk"), Some(ON_DISK_WALK));
        self.read(&format!("{ROOT}/{WEB}/{SIGN_IN}/walk"), Some(ON_DISK_WALK));
    }

    fn printed_sign_in(&mut self) {
        self.sign_in_request();
        self.print(&[SIGN_IN], 0, &format!("printed {SIGN_IN}\nprinted into {PRINT}\n"), "");
    }

    fn pages(&mut self, path: &str, directory: &str, raw: [Option<&str>; 3], comparable: [&str; 3]) {
        self.blob(PARENT, path, raw[0]);
        self.blob(TIP, path, raw[1]);
        self.read(&format!("{PRINT}/{path}"), raw[2]);
        self.step(Ask::Directory(directory.into()), Answer::Done);
        for (leaf, content) in ["parent", "requested", "landed"].into_iter().zip(comparable) {
            self.write(directory, leaf, content);
        }
    }

    fn merge(&mut self, directory: &str, code: i32, merged: &str) {
        self.git_reply(
            &[
                "-c",
                "merge.conflictStyle=merge",
                "merge-file",
                "-p",
                "--marker-size=7",
                "-L",
                "landed",
                "-L",
                "parent",
                "-L",
                "requested",
                "landed",
                "parent",
                "requested",
            ],
            Some(directory),
            code,
            merged,
            "",
        );
        self.write(directory, "merged", merged);
    }

    fn diff(&mut self, directory: &str, expected: &str, patch: &str) {
        self.git_reply(
            &[
                "diff",
                "--no-index",
                "--unified=0",
                "--inter-hunk-context=0",
                "--no-color",
                "--no-ext-diff",
                "--no-textconv",
                "--text",
                "--diff-algorithm=myers",
                "--no-indent-heuristic",
                "--",
                "landed",
                expected,
            ],
            Some(directory),
            1,
            patch,
            "",
        );
    }

    fn web_rules(&mut self, parent: &str, requested: &str, temper: &str, task_pages: &str) {
        let path = format!("{WEB}/overlay.css");
        self.request(&[&path]);
        self.blob(PARENT, &path, Some(parent));
        self.blob(TIP, &path, Some(requested));
        self.read(&format!("{ROOT}/{SHELL_WEB}/temper.css"), Some(temper));
        self.read(&format!("{ROOT}/{SHELL_WEB}/task-pages.css"), Some(task_pages));
    }

    fn check(self, code: u8, stdout: &str, stderr: &str) {
        let args = [NAME.as_bytes().into(), SHORT.as_bytes().into()];
        check(self, &args, code, stdout, stderr);
    }
}

fn check(steps: Steps, args: &[Box<[u8]>], code: u8, stdout: &str, stderr: &str) {
    let mut host = Host::canned(steps.0);
    assert_eq!(run(args, &mut host), code, "exit with {args:?}");
    let Gathered { stdout: out, stderr: err } = host.finish();
    assert_eq!(String::from_utf8(out).expect("table is text"), stdout);
    assert_eq!(String::from_utf8(err).expect("details are text"), stderr);
}

#[test]
fn every_edit_landed() {
    let mut steps = Steps::default();
    steps.printed_sign_in();
    steps.pages(
        SIGN_IN_PAGE,
        SIGN_IN_SCRATCH,
        [Some(SIGN_IN_P), Some(SIGN_IN_C), Some(SIGN_IN_C)],
        [SIGN_IN_P, SIGN_IN_C, SIGN_IN_C],
    );
    steps.merge(SIGN_IN_SCRATCH, 0, SIGN_IN_C);
    steps.check(0, &format!("{HEADER}| `web/journeys/sign-in/02.html` | landed |\n{FILL}"), "");
}

#[test]
fn missing_edit_gets_its_hunk() {
    let mut steps = Steps::default();
    steps.printed_sign_in();
    steps.pages(
        SIGN_IN_PAGE,
        SIGN_IN_SCRATCH,
        [Some(SIGN_IN_P), Some(SIGN_IN_C), Some(SIGN_IN_P)],
        [SIGN_IN_P, SIGN_IN_C, SIGN_IN_P],
    );
    steps.merge(SIGN_IN_SCRATCH, 0, SIGN_IN_C);
    let hunk = "@@ -2 +2 @@\n-b\n+B\n";
    steps.diff(SIGN_IN_SCRATCH, "merged", &format!("diff --git a/landed b/merged\n--- a/landed\n+++ b/merged\n{hunk}"));
    let item = "`web/journeys/sign-in/02.html`, line 2";
    steps.check(1, &format!("{HEADER}| {item} | differs |\n{FILL}"), &format!("look-compare: {item}\n{hunk}"));
}

#[test]
fn edits_to_one_tag_conflict() {
    let mut steps = Steps::default();
    steps.printed_sign_in();
    let landed = "a\n<b class=landing\nc\nd\ne\n";
    let parent = "a\n<b class=before\nc\nd\ne\n";
    let requested = "a\n<b class=request\nc\nd\nE\n";
    steps.pages(
        SIGN_IN_PAGE,
        SIGN_IN_SCRATCH,
        [Some(parent), Some(requested), Some(landed)],
        [parent, requested, landed],
    );
    let conflict = "<<<<<<< landed\n<b class=landing\n=======\n<b class=request\n>>>>>>> requested\n";
    steps.merge(SIGN_IN_SCRATCH, 1, &format!("a\n{conflict}c\nd\nE\n"));
    steps.write(SIGN_IN_SCRATCH, "merged-with-landing-conflicts", "a\n<b class=landing\nc\nd\nE\n");
    let hunk = "@@ -5 +5 @@\n-e\n+E\n";
    steps.diff(SIGN_IN_SCRATCH, "merged-with-landing-conflicts", hunk);
    let page = "`web/journeys/sign-in/02.html`";
    steps.check(
        1,
        &format!("{HEADER}| {page}, line 2 | conflict |\n| {page}, line 5 | differs |\n{FILL}"),
        &format!("look-compare: {page}, line 2\n{conflict}look-compare: {page}, line 5\n{hunk}"),
    );
}

#[test]
fn moved_elements_do_not_report_generated_id_edits() {
    let mut steps = Steps::default();
    steps.printed_sign_in();
    let label = "<label id=\"skein-ui-label-21\">\n<input aria-labelledby=\"skein-ui-label-21\">\n";
    let parent = format!("head\n{label}mark\n");
    let moved = format!("head\nmark\n{label}");
    let landed = moved.replace("-21\"", "-98\"");
    let parent_numbered = parent.replace("-21\"", "-1\"");
    let moved_numbered = moved.replace("-21\"", "-1\"");
    steps.pages(
        SIGN_IN_PAGE,
        SIGN_IN_SCRATCH,
        [Some(&parent), Some(&moved), Some(&landed)],
        [&parent_numbered, &moved_numbered, &moved_numbered],
    );
    steps.merge(SIGN_IN_SCRATCH, 0, &moved_numbered);
    steps.check(0, &format!("{HEADER}| `web/journeys/sign-in/02.html` | landed |\n{FILL}"), "");
}

#[test]
fn rules_compare_values_conditions_and_removals() {
    let mut steps = Steps::default();
    steps.web_rules(
        ".temper-d { margin: 0 }",
        ".temper-a { color: red; }\n.temper-b { color: red; padding: 0 }\n@media (max-width: 760px) { .temper-c { display: none } }",
        ".temper-a{color:red}\n.temper-b { color: blue; padding: 0; }", "",
    );
    steps.check(1, &format!(
        "{HEADER}| rule `.temper-a` | landed |\n| rule `.temper-b` | differs: color |\n| rule `.temper-c` in `@media (max-width: 760px)` | missing |\n| removed rule `.temper-d` | landed |\n| the order the rules cascade in | to read |\n{FILL}"
    ), "");
}

#[test]
fn rules_alone_do_not_print_and_keep_nested_conditions_and_quotes() {
    let mut steps = Steps::default();
    steps.web_rules(
        "@font-face {font-family: before;}",
        concat!(
            "@supports (display: grid) { @media (width > 12px) {\n",
            "button|action[title=';{😀}'] { PADDING: 1px 4px; --Mixed: url(\"a;/* }\"); }\n",
            "button|action[title=';{😀}'] { padding: 0; content: 'keep  /*  */  these'; }\n",
            "}}\n@font-face {font-family: after;}\n@import url(extra.css);"
        ),
        "@supports (display: grid) {@media (width > 12px) {button|action[title=';{😀}'] {padding: 7px; --Mixed: url(\"a;/* }\");}}}",
        "@supports (display: grid) {@media (width > 12px) {button/*comment*/|action[title=';{😀}'] {padding:  0; content: 'keep  /*  */  these';}}}",
    );
    steps.check(0, &format!(
        "{HEADER}| rule `button\\|action[title=';{{😀}}']` in `@supports (display: grid) > @media (width > 12px)` | landed |\n| at-rule `@font-face` | to read |\n| at-rule `@import url(extra.css)` | to read |\n| the order the rules cascade in | to read |\n{FILL}"
    ), "");
}

#[test]
fn a_new_walks_page_is_compared_whole() {
    let mut steps = Steps::default();
    let walk = format!("{WEB}/new-walk/walk");
    let page = format!("{WEB}/new-walk/01.html");
    let directory = format!("{SCRATCH}/web--new-walk--01.html");
    steps.request(&[&page, &walk]);
    steps.blob(TIP, &walk, Some(ON_DISK_WALK));
    steps.read(&format!("{ROOT}/{walk}"), Some(ON_DISK_WALK));
    steps.print(&["new-walk"], 0, &format!("printed new-walk\nprinted into {PRINT}\n"), "");
    steps.pages(&page, &directory, [None, Some("a\nadded\nb\n"), Some("a\nb\n")], ["", "a\nadded\nb\n", "a\nb\n"]);
    let hunk = "@@ -1,0 +2 @@ a\n+added\n";
    steps.diff(&directory, "requested", hunk);
    let item = "`web/journeys/new-walk/01.html`, after line 1";
    steps.check(1, &format!("{HEADER}| {item} | differs |\n{FILL}"), &format!("look-compare: {item}\n{hunk}"));
}

#[test]
fn a_stopped_walk_gets_only_its_directory_row() {
    let mut steps = Steps::default();
    let tui_walk = format!("{TUI}/terminal/walk");
    let tui_page = format!("{TUI}/terminal/01.txt");
    let tui_scratch = format!("{SCRATCH}/tui--terminal--01.txt");
    steps.request(&[&tui_page, SIGN_IN_PAGE]);
    let sign_in_walk = format!("{WEB}/{SIGN_IN}/walk");
    steps.blob(TIP, &sign_in_walk, Some(ON_DISK_WALK));
    steps.blob(TIP, &tui_walk, Some("walk 2\ntui mark\n"));
    steps.read(&format!("{ROOT}/{sign_in_walk}"), Some(ON_DISK_WALK));
    steps.read(&format!("{ROOT}/{tui_walk}"), Some("walk 2\ntui mark\n"));
    steps.step(Ask::Exists(format!("{ROOT}/{WEB}/terminal/walk").into()), Answer::Is(false));
    let why = "sign-in:2: expected form | found nothing";
    steps.print(
        &[SIGN_IN, "terminal"],
        1,
        &format!("stopped {SIGN_IN}: {why}\nprinted terminal\nprinted into {PRINT}\n"),
        "",
    );
    steps.pages(&tui_page, &tui_scratch, [Some("old\n"), Some("new\n"), Some("new\n")], ["old\n", "new\n", "new\n"]);
    steps.merge(&tui_scratch, 0, "new\n");
    steps.check(1, &format!("{HEADER}| `web/journeys/sign-in` | stopped: sign-in:2: expected form \\| found nothing |\n| `tui/journeys/terminal/01.txt` | landed |\n{FILL}"), "");
}

#[test]
fn tui_rules_are_listed_to_read() {
    let mut steps = Steps::default();
    let overlay = format!("{TUI}/overlay.css");
    steps.request(&[&overlay]);
    steps.blob(PARENT, &overlay, Some(".old { color: blue; }"));
    steps.blob(TIP, &overlay, Some(".title { color: red; }"));
    steps.check(0, &format!("{HEADER}| TUI rule `.title` | to read |\n| TUI rule `.old` | to read |\n{FILL}"), "");
}

#[test]
fn a_hand_made_directory_gets_one_row_and_walk_sources_get_none() {
    let mut steps = Steps::default();
    let mockup = format!("{WEB}/mockup");
    steps.request(&[
        &format!("{mockup}/parts/image.svg"),
        &format!("{WEB}/index.html"),
        &format!("{mockup}/01.html"),
        &format!("{WEB}/sources/README.md"),
    ]);
    steps.blob(TIP, &format!("{mockup}/walk"), Some("walk 20\n"));
    steps.blob(TIP, &format!("{WEB}/sources/walk"), Some(ON_DISK_WALK));
    steps.check(0, &format!("{HEADER}| `web/journeys/mockup` | not compared: hand-made |\n{FILL}"), "");
}

#[test]
fn a_tip_that_isnt_c_is_refused() {
    let mut steps = Steps::default();
    steps.fetch(PARENT);
    steps.check(2, "", &format!("look-compare: docs/look/{NAME} fetched {PARENT}, not C ({SHORT})\n"));
}

#[test]
fn refused_arguments_ask_nothing() {
    for (words, why) in [
        (&[][..], "expected NAME and C"),
        (&[b"name".as_slice()][..], "expected NAME and C"),
        (&[b"name".as_slice(), b"0123456", b"extra"][..], "expected NAME and C"),
        (&[b"".as_slice(), b"0123456"][..], "NAME must be [a-z0-9][a-z0-9-]*"),
        (&[b"-name".as_slice(), b"0123456"], "NAME must be [a-z0-9][a-z0-9-]*"),
        (&[b"Name".as_slice(), b"0123456"], "NAME must be [a-z0-9][a-z0-9-]*"),
        (&[b"a/b".as_slice(), b"0123456"], "NAME must be [a-z0-9][a-z0-9-]*"),
        (&[b"\xff".as_slice(), b"0123456"], "NAME must be [a-z0-9][a-z0-9-]*"),
        (&[b"2-name".as_slice(), b"012345"], "C must be 7 to 64 lowercase hexadecimal digits"),
        (&[b"name".as_slice(), b"01234abC"], "C must be 7 to 64 lowercase hexadecimal digits"),
        (&[b"name".as_slice(), b"01234abg"], "C must be 7 to 64 lowercase hexadecimal digits"),
        (&[b"name".as_slice(), b"01234ab\xff"], "C must be 7 to 64 lowercase hexadecimal digits"),
        (
            &[b"name".as_slice(), b"00000000000000000000000000000000000000000000000000000000000000000"],
            "C must be 7 to 64 lowercase hexadecimal digits",
        ),
    ] {
        let args = words.iter().map(|word| Box::from(*word)).collect::<Vec<_>>();
        check(Steps::default(), &args, 2, "", &format!("look-compare: {why}\n"));
    }
}

#[test]
fn a_failed_print_has_no_table() {
    let mut steps = Steps::default();
    steps.sign_in_request();
    steps.print(&[SIGN_IN], 3, "printed sign-in\n", "temper-client-walk: directory unwritable\n");
    steps.check(3, "", "temper-client-walk: directory unwritable\nlook-compare: the print couldn't finish (exit 3)\n");
}

#[test]
fn a_web_walk_shadowing_the_tui_refuses_before_print() {
    let mut steps = Steps::default();
    let walk = format!("{TUI}/sign-in/walk");
    steps.request(&[&format!("{TUI}/sign-in/02.txt")]);
    steps.blob(TIP, &walk, Some("walk 2\ntui mark\n"));
    steps.read(&format!("{ROOT}/{walk}"), Some("walk 2\ntui mark\n"));
    steps.step(Ask::Exists(format!("{ROOT}/{WEB}/sign-in/walk").into()), Answer::Is(true));
    steps.check(
        3,
        "",
        "look-compare: tui/journeys/sign-in is shadowed by a walk in web/journeys/sign-in; rename one directory\n",
    );
}

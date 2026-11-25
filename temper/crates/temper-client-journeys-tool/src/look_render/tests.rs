use std::path::{Path, PathBuf};

use skein_browser::boundary::{Element, Node, Property, Rect, Refusal, Trouble};
use skein_lib::Token;

use crate::{
    host::{
        self, Answer, Ask, Gathered, Host,
        browser::{Command, Observation, Reply},
    },
    look_compare::tests::{NAME, ON_DISK_WALK, PARENT, ROOT, SHELL_WEB, SHORT, SIGN_IN_PAGE, Steps, TIP, TUI, WEB},
};

use super::run;

const SCRATCH: &str = "/landing tree/.git/look-render";
const PROFILE: &str = "/landing tree/.git/look-render/browser";
const DIFF: &str = "/landing tree/.git/look-render/diff";
const WEB_WALK: &str = "temper/docs/design/client/web/journeys/sign-in/walk";
const OVERLAY: &str = "temper/docs/design/client/web/journeys/overlay.css";
const TUI_OVERLAY: &str = "temper/docs/design/client/tui/journeys/overlay.css";
const HAND_MADE_DIR: &str = "temper/docs/design/client/web/journeys/not-a-walk";
const HAND_MADE_PAGE: &str = "temper/docs/design/client/web/journeys/not-a-walk/index.html";
const ITEM: &str = "web/journeys/sign-in/02.html";
const PERSON: Token = Token::new(1);
const REQUEST: Token = Token::new(2);
const PRINT: Token = Token::new(3);
const BUTTON: Node = 15;
const LANDED_BUTTON: Node = 115;
const LINE: usize = 41; // The button follows the five header lines and this fixture's blank lines.
const HEADER: &str =
    "<!doctype html>\n<html>\n<head><title>Sign in</title></head>\n<!-- press to continue -->\n<body>\n";
const WIDTHS: [u32; 3] = [1440, 1000, 420]; // journey-writer.md, 6.2, step 7.
const HEIGHT: u32 = 900;
const BOX: Rect = Rect { x: -5, y: 20, width: 80, height: 30 };
const LEFT_COLOR: &str = "rgb(1, 2, 3)";
const RIGHT_COLOR: &str = "rgb(4, 5, 6)";
const IDENTICAL: &str = "identical\tweb/journeys/sign-in/02.html\nresult\tidentical\n";
const DIFFERS: &str = "result\tdiffers\n";
const NOTHING: &str = "result\tnothing to render\n";

struct Files<'a> {
    path: &'a str,
    source: &'a str,
    copy: &'a str,
}

impl<'a> Files<'a> {
    fn same(path: &'a str, bytes: &'a str) -> Self {
        Self { path, source: bytes, copy: bytes }
    }
}

#[derive(Default)]
struct Effects(Steps);

impl Effects {
    fn begin(&mut self, changed: &[&str]) {
        let s = &mut self.0;
        s.fetch(TIP);
        s.git(&["show", "-s", "--format=%P", TIP], None, &format!("{PARENT}\n"));
        s.git(
            &["show", "-s", "--format=%(trailers:key=Look-request,unfold)", TIP],
            None,
            "Look-request: look-request\n",
        );
        s.git(&["rev-parse", "--show-toplevel"], None, &format!("{ROOT}\n"));
        s.git(&["rev-parse", "--path-format=absolute", "--git-path", "look-render"], None, &format!("{SCRATCH}\n"));
        s.step(Ask::Remove(SCRATCH.into()), Answer::Done);
        s.step(Ask::Directory(SCRATCH.into()), Answer::Done);
        let mut paths = String::new();
        for path in changed {
            paths.push_str(path);
            paths.push('\0');
        }
        s.git(
            &["diff-tree", "--no-commit-id", "--name-only", "--no-renames", "-r", "-z", PARENT, TIP, "--", WEB, TUI],
            Some(ROOT),
            &paths,
        );
    }

    fn copy_request(&mut self, files: &[Files<'_>]) {
        const OBJECT: &str = "aaaaabbbbbcccccdddddeeeeefffff0000011111";
        let mut paths = String::new();
        for file in files {
            use std::fmt::Write as _;
            write!(paths, "100644 blob {OBJECT}\t{}\0", file.path).expect("write string");
        }
        let request = format!("{SCRATCH}/request");
        self.0.step(Ask::Directory(request.clone().into()), Answer::Done);
        self.0.git(&["ls-tree", "--full-tree", "-rz", TIP, "--", WEB, TUI, SHELL_WEB], Some(ROOT), &paths);
        let mut directories = std::collections::BTreeSet::new();
        for file in files {
            self.0.git(&["cat-file", "blob", OBJECT], Some(ROOT), file.source);
            self.copied(&request, file, &mut directories);
        }
    }

    fn copy_print(&mut self, files: &[Files<'_>]) {
        let print = format!("{SCRATCH}/print");
        self.0.step(Ask::Directory(print.clone().into()), Answer::Done);
        let mut directories = std::collections::BTreeSet::new();
        for tree in [WEB, TUI, SHELL_WEB] {
            let mut paths = files
                .iter()
                .filter(|file| file.path.starts_with(tree))
                .map(|file| file.path.into())
                .collect::<Vec<_>>();
            if tree == WEB {
                paths.push(OVERLAY.into()); // On disk, but not read into the print copy.
            }
            self.0.step(Ask::Files { root: ROOT.into(), tree }, Answer::Files(paths));
            for file in files.iter().filter(|file| file.path.starts_with(tree)) {
                self.0.read(&format!("{ROOT}/{}", file.path), Some(file.source));
                self.copied(&print, file, &mut directories);
            }
        }
        for path in [OVERLAY, TUI_OVERLAY] {
            self.copied(&print, &Files::same(path, ""), &mut directories);
        }
    }

    fn copied(&mut self, copy: &str, file: &Files<'_>, directories: &mut std::collections::BTreeSet<PathBuf>) {
        let leaf = Path::new(file.path);
        let parent = Path::new(copy).join(leaf.parent().expect("repo-relative file"));
        if directories.insert(parent.clone()) {
            self.0.step(Ask::Directories(parent), Answer::Done);
        }
        self.0.write(copy, file.path, file.copy);
    }

    fn chrome(&mut self, command: Command, reply: Reply) {
        self.chrome_with(command, reply, Vec::new());
    }

    fn chrome_with(&mut self, command: Command, reply: Reply, trouble: Vec<Observation>) {
        self.0.step(Ask::Browser(command), Answer::Browser(host::browser::Answer { reply, trouble }));
    }

    fn start_as(&mut self, answer: Reply) {
        for path in [DIFF, PROFILE] {
            self.0.step(Ask::Directory(path.into()), Answer::Done);
        }
        self.chrome(Command::Start { profile: PROFILE.into() }, answer);
    }

    fn start(&mut self) {
        self.start_as(Reply::Done);
        self.chrome(Command::Person { person: PERSON }, Reply::Done);
    }

    fn done(&mut self, command: Command) {
        self.chrome(command, Reply::Done);
    }

    fn open(&mut self, page: Token, copy: &str) {
        let url = format!("file:///landing%20tree/.git/look-render/{copy}/{SIGN_IN_PAGE}");
        self.done(Command::Open { person: PERSON, page, url: url.into_bytes().into_boxed_slice(), files: true });
    }

    fn opened_pair(&mut self) {
        self.open(REQUEST, "request");
        self.open(PRINT, "print");
    }

    fn close_pair(&mut self) {
        self.done(Command::Close { page: PRINT });
        self.done(Command::Close { page: REQUEST });
    }

    fn elements(&mut self, page: Token, tag: &str) {
        let offset = if page == PRINT { LANDED_BUTTON - BUTTON } else { 0 };
        let elements = [("html", 0), ("head", 1), ("title", 2), ("body", 1), (tag, 2)]
            .into_iter()
            .enumerate()
            .map(|(at, (name, depth))| Element {
                node: BUTTON - 4 + offset + u64::try_from(at).expect("fixture index"),
                name: name.as_bytes().into(),
                depth,
            })
            .collect();
        self.chrome(Command::Elements { page, selector: None }, Reply::Elements { elements, more: 0 });
    }

    fn edit(&mut self, parent: &str, request: &str, patch: &str) {
        self.0.blob(PARENT, SIGN_IN_PAGE, Some(parent));
        self.diff(parent, request, patch);
    }

    fn diff(&mut self, from: &str, to: &str, patch: &str) {
        self.0.write(DIFF, "landed", from);
        self.0.write(DIFF, "requested", to);
        self.0.diff(DIFF, "requested", patch);
    }

    fn viewport(&mut self, width: u32) {
        for page in [REQUEST, PRINT] {
            self.done(Command::Viewport { page, width, height: HEIGHT });
        }
    }

    fn measure(&mut self, width: u32, styles: [&[(&str, &str)]; 2], boxes: [Option<Rect>; 2]) {
        self.viewport(width);
        for ((page, node), properties) in [(REQUEST, BUTTON), (PRINT, LANDED_BUTTON)].into_iter().zip(styles) {
            let style = properties
                .iter()
                .map(|(name, value)| Property { name: name.as_bytes().into(), value: value.as_bytes().into() })
                .collect();
            self.chrome(Command::Style { page, node }, Reply::Style(style));
        }
        for ((page, node), rect) in [(REQUEST, BUTTON), (PRINT, LANDED_BUTTON)].into_iter().zip(boxes) {
            self.chrome(Command::Rect { page, node }, Reply::Rect(rect));
        }
    }

    fn verdict(mut self, out: &str) {
        self.close_pair();
        self.stop_and_check(0, out, "");
    }

    fn stop_and_check(mut self, code: u8, out: &str, err: &str) {
        self.done(Command::Stop);
        self.check(code, out, err);
    }

    fn check(self, code: u8, out: &str, err: &str) {
        let Steps(steps) = self.0;
        let mut host = Host::canned(steps);
        let args = [NAME.as_bytes().into(), SHORT.as_bytes().into()];
        assert_eq!(run(&args, &mut host), code);
        let Gathered { stdout, stderr } = host.finish();
        assert_eq!(String::from_utf8(stdout).expect("fixture output is text"), out);
        assert_eq!(String::from_utf8(stderr).expect("fixture diagnostics are text"), err);
    }
}

fn page(tag: &str, class: &str) -> String {
    format!("{HEADER}{}<{tag} class=\"{class}\">Continue</{tag}>\n</body>\n</html>\n", "\n".repeat(LINE - 6))
}

fn fixture(landed: Option<&str>) -> (Effects, String) {
    let edited = page("button", "temper-chip");
    let mut steps = Effects::default();
    steps.begin(&[SIGN_IN_PAGE]);
    steps.0.blob(TIP, WEB_WALK, Some(ON_DISK_WALK));
    steps.copy_request(&[Files::same(SIGN_IN_PAGE, &edited), Files::same(WEB_WALK, ON_DISK_WALK)]);
    let files = landed.map_or_else(Vec::new, |landed| vec![Files::same(SIGN_IN_PAGE, landed)]);
    steps.copy_print(&files);
    (steps, edited)
}

fn button_patch() -> String {
    format!(
        "@@ -{LINE} +{LINE} @@\n-<button class=\"old\">Continue</button>\n+<button class=\"temper-chip\">Continue</button>\n"
    )
}

fn prepared(landed: &str, tag: &str) -> (Effects, String) {
    let (mut steps, edited) = fixture(Some(landed));
    steps.start();
    steps.opened_pair();
    steps.elements(REQUEST, "button");
    steps.elements(PRINT, tag);
    (steps, edited)
}

fn edited_button() -> Effects {
    let document = page("button", "temper-chip");
    let (mut steps, edited) = prepared(&document, "button");
    steps.edit(&page("button", "old"), &edited, &button_patch());
    steps
}

fn difference(width: u32, property: &str, left: &str, right: &str) -> String {
    format!("differs\t{ITEM}\t{width}\t41 button.temper-chip\t{property}\t{left}\t{right}\n")
}

fn uncompared(item: &str, reason: &str) -> String {
    format!("uncompared\t{item}\t{reason}\nresult\tnot compared\n")
}

#[test]
fn equal_views_have_one_identical_line_for_the_page() {
    let mut steps = edited_button();
    for width in WIDTHS {
        steps.measure(width, [[("color", LEFT_COLOR)].as_slice(); 2], [Some(BOX); 2]);
    }
    steps.verdict(IDENTICAL);
}

#[test]
fn computed_color_differences_are_listed_widest_first() {
    let mut steps = edited_button();
    for width in WIDTHS {
        steps.measure(width, [&[("color", LEFT_COLOR)], &[("color", RIGHT_COLOR)]], [Some(BOX); 2]);
    }
    steps.verdict(&format!("differs\tweb/journeys/sign-in/02.html\t1440\t41 button.temper-chip\tcolor\trgb(1, 2, 3)\trgb(4, 5, 6)\n{}{DIFFERS}",
        [1000, 420].into_iter().map(|width| difference(width, "color", LEFT_COLOR, RIGHT_COLOR)).collect::<String>()));
}

#[test]
fn box_coordinates_one_pixel_apart_are_equal() {
    let mut steps = edited_button();
    for width in WIDTHS {
        steps.measure(width, [&[]; 2], [Some(BOX), Some(Rect { x: -4, y: 21, width: 81, height: 29 })]);
    }
    steps.verdict(IDENTICAL);
}

#[test]
fn box_coordinates_two_pixels_apart_or_absent_differ() {
    for (landed, expected) in [
        (Some(Rect { x: -7, ..BOX }), "-7,20 80x30"),
        (Some(Rect { y: 22, ..BOX }), "-5,22 80x30"),
        (Some(Rect { width: 78, ..BOX }), "-5,20 78x30"),
        (Some(Rect { height: 32, ..BOX }), "-5,20 80x32"),
        (None, "none"),
    ] {
        let mut steps = edited_button();
        let [wide, medium, narrow] = WIDTHS;
        steps.measure(wide, [&[]; 2], [Some(BOX), landed]);
        steps.measure(medium, [&[]; 2], [Some(BOX); 2]);
        steps.measure(narrow, [&[]; 2], [None; 2]);
        steps.verdict(&format!("{}{DIFFERS}", difference(wide, "box", "-5,20 80x30", expected)));
    }
}

#[test]
fn a_touched_tag_with_no_pair_reports_one_difference() {
    let landed = page("p", "temper-chip");
    let (mut steps, edited) = prepared(&landed, "p");
    steps.edit(&page("button", "old"), &edited, &button_patch());
    steps.diff(&edited, &landed, &format!("@@ -{LINE} +{LINE} @@\n-<button class=\"temper-chip\">Continue</button>\n+<p class=\"temper-chip\">Continue</p>\n"));
    steps.verdict(&format!("differs\t{ITEM}\t-\t41 button.temper-chip\tpair\tpresent\tnone\n{DIFFERS}"));
}

fn overlay(requested: &str) -> Effects {
    let source = page("button", "temper-chip");
    let mut steps = Effects::default();
    steps.begin(&[OVERLAY]);
    steps.copy_request(&[
        Files::same(OVERLAY, requested),
        Files::same(SIGN_IN_PAGE, &source),
        Files::same(WEB_WALK, ON_DISK_WALK),
    ]);
    steps.copy_print(&[Files::same(SIGN_IN_PAGE, &source)]);
    steps.0.blob(PARENT, OVERLAY, Some(""));
    steps.0.blob(TIP, OVERLAY, Some(requested));
    steps.start();
    steps.open(REQUEST, "request");
    steps
}

#[test]
fn overlay_selectors_drop_user_states_and_pseudo_elements() {
    for state in
        [":hover", ":active", ":focus", ":focus-visible", ":focus-within", "::before", ":first-line", "::part(label)"]
    {
        let mut steps = overlay(&format!("@media (min-width: 420px) {{ .temper-chip{state} {{ color: red }} }}"));
        let element = Element { node: BUTTON, name: b"button".as_slice().into(), depth: 2 };
        steps.chrome(
            Command::Elements { page: REQUEST, selector: Some(b".temper-chip".as_slice().into()) },
            Reply::Elements { elements: vec![element], more: 0 },
        );
        steps.open(PRINT, "print");
        steps.elements(REQUEST, "button");
        steps.elements(PRINT, "button");
        for width in WIDTHS {
            steps.measure(width, [&[]; 2], [Some(BOX); 2]);
        }
        steps.verdict(IDENTICAL);
    }
}

#[test]
fn a_selector_refused_by_chromium_selects_no_element() {
    let mut steps = overlay(".temper-chip:some-state { color: red }");
    steps.chrome(
        Command::Elements { page: REQUEST, selector: Some(b".temper-chip:some-state".as_slice().into()) },
        Reply::Refused {
            why: Refusal::Protocol,
            detail: b"DOM.querySelectorAll: DOM Error while querying".as_slice().into(),
        },
    );
    steps.done(Command::Close { page: REQUEST });
    steps.stop_and_check(0, NOTHING, "");
}

#[test]
fn differing_custom_properties_do_not_count_as_differences() {
    let mut steps = edited_button();
    for width in WIDTHS {
        steps.measure(width, [&[("--x", "red")], &[("--x", "blue")]], [None; 2]);
    }
    steps.verdict(IDENTICAL);
}

#[test]
fn a_dom_not_following_start_tags_is_not_compared() {
    let document = page("button", "temper-chip");
    let (mut steps, _) = prepared(&document, "a");
    steps.close_pair();
    steps.stop_and_check(0, &uncompared(ITEM, "its markup doesn't parse as written"), "");
}

#[test]
fn hand_made_pages_receive_a_row_without_rendering() {
    let mut steps = Effects::default();
    steps.begin(&[HAND_MADE_PAGE]);
    steps.0.blob(TIP, &format!("{HAND_MADE_DIR}/walk"), Some("walk 1\n"));
    steps.copy_request(&[Files::same(HAND_MADE_PAGE, "<p>Hello</p>")]);
    steps.copy_print(&[]);
    steps.0.git(
        &[
            "--literal-pathspecs",
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "--no-renames",
            "-rz",
            PARENT,
            TIP,
            "--",
            HAND_MADE_DIR,
        ],
        Some(ROOT),
        &format!("{HAND_MADE_PAGE}\0"),
    );
    steps.start();
    steps.stop_and_check(0, &uncompared("web/journeys/not-a-walk/index.html", "hand-made"), "");
}

#[test]
fn a_page_missing_from_the_print_is_not_compared() {
    let (mut steps, _) = fixture(None);
    steps.start();
    steps.stop_and_check(0, &uncompared(ITEM, "nothing printed beside it"), "");
}

#[test]
fn edits_to_lines_with_no_start_tag_touch_nothing() {
    let document = page("button", "temper-chip");
    let (mut steps, edited) = prepared(&document, "button");
    let parent = edited.replace("press to continue", "press to sign in");
    steps.edit(&parent, &edited, "@@ -4 +4 @@\n-<!-- press to sign in -->\n+<!-- press to continue -->\n");
    steps.verdict(NOTHING);
}

#[test]
fn field_values_escape_backslash_tab_and_line_endings() {
    let mut steps = edited_button();
    let [wide, medium, narrow] = WIDTHS;
    steps.measure(wide, [&[("content", "x\ty\\z\r\n")], &[("content", "print")]], [None; 2]);
    for width in [medium, narrow] {
        steps.measure(width, [&[]; 2], [None; 2]);
    }
    steps.verdict(&format!("{}{DIFFERS}", difference(wide, "content", r"x\ty\\z\r\n", "print")));
}

#[test]
fn reports_list_fifty_differences_then_count_the_rest() {
    const ALL: u32 = 52; // Two past the page's fifty listed differences (journey-writer.md, 6.2).
    const LISTED: u32 = 50;
    let mut steps = edited_button();
    let names = (0..ALL).rev().map(|at| format!("property-{at:02}")).collect::<Vec<_>>();
    let left = names.iter().map(|name| (name.as_str(), "left")).collect::<Vec<_>>();
    let right = names.iter().map(|name| (name.as_str(), "right")).collect::<Vec<_>>();
    let [wide, medium, narrow] = WIDTHS;
    steps.measure(wide, [&left, &right], [None; 2]);
    for width in [medium, narrow] {
        steps.measure(width, [&[]; 2], [None; 2]);
    }
    let expected =
        (0..LISTED).map(|at| difference(wide, &format!("property-{at:02}"), "left", "right")).collect::<String>();
    steps.verdict(&format!("{expected}more\t{ITEM}\t2\n{DIFFERS}"));
}

#[test]
fn copies_open_real_disclosures_and_leave_every_other_byte() {
    const WRITTEN: &str = "<!doctype html>\r\n<html><head><title><dialog></title><script>\"<details>\"</script></head>\n<body><!-- <details> --><details\n  data-note='open > <dialog>'>x</details><DIALOG>y</DIALOG><details OPEN>z</details></body></html>\n";
    const OPENED: &str = "<!doctype html>\r\n<html><head><title><dialog></title><script>\"<details>\"</script></head>\n<body><!-- <details> --><details open\n  data-note='open > <dialog>'>x</details><DIALOG open>y</DIALOG><details OPEN>z</details></body></html>\n";
    const SHEET: &str = "temper/crates/temper-client-shell-web/temper.css";
    let mut steps = Effects::default();
    steps.begin(&[SIGN_IN_PAGE]);
    steps.0.blob(TIP, WEB_WALK, Some(ON_DISK_WALK));
    let disclosure = || Files { path: SIGN_IN_PAGE, source: WRITTEN, copy: OPENED };
    steps.copy_request(&[disclosure(), Files::same(WEB_WALK, ON_DISK_WALK), Files::same(SHEET, "body { color: red }")]);
    steps.copy_print(&[disclosure(), Files::same(SHEET, "body { color: blue }")]);
    steps.start();
    steps.opened_pair();
    for page in [REQUEST, PRINT] {
        let elements = ["html", "head", "title", "script", "body", "details", "dialog", "details"]
            .into_iter()
            .enumerate()
            .map(|(at, tag)| Element {
                node: u64::try_from(at).expect("fixture index"),
                depth: 1,
                name: tag.as_bytes().into(),
            })
            .collect();
        steps.chrome(Command::Elements { page, selector: None }, Reply::Elements { elements, more: 0 });
    }
    steps.0.blob(PARENT, SIGN_IN_PAGE, Some(WRITTEN)); // No comparable edit to a tag in this fixture.
    steps.verdict(NOTHING);
}

#[test]
fn a_page_trouble_is_reported_without_judging_the_page() {
    let document = page("button", "temper-chip");
    let (mut steps, _) = prepared(&document, "button");
    steps.0.blob(PARENT, SIGN_IN_PAGE, Some(&document));
    steps.chrome_with(
        Command::Close { page: PRINT },
        Reply::Done,
        vec![Observation {
            page: PRINT,
            kind: Trouble::Log,
            text: b"request blocked".as_slice().into(),
            url: b"https://example.invalid/image".as_slice().into(),
        }],
    );
    steps.done(Command::Close { page: REQUEST });
    steps.stop_and_check(
        0,
        NOTHING,
        &format!("look-render: {ITEM}: Log: request blocked (https://example.invalid/image)\n"),
    );
}

#[test]
fn a_refused_read_keeps_stdout_empty_and_closes_chromium() {
    let mut steps = edited_button();
    steps.viewport(1440);
    steps.chrome(
        Command::Style { page: REQUEST, node: BUTTON },
        Reply::Refused { why: Refusal::Gone, detail: Box::default() },
    );
    steps.stop_and_check(3, "", "look-render: Chromium refused a read (Gone)\n");
}

#[test]
fn missing_chromium_configuration_has_no_standard_output() {
    let (mut steps, _) = fixture(None);
    let reason = "set SKEIN_TEST_CHROMIUM to the Chromium executable";
    steps.start_as(Reply::Failed(reason.into()));
    steps.stop_and_check(3, "", &format!("look-render: {reason}\n"));
}

#[test]
fn an_origin_tip_not_matching_c_is_refused_before_copies() {
    let mut steps = Effects::default();
    steps.0.fetch(PARENT);
    steps.check(2, "", &format!("look-render: docs/look/{NAME} fetched {PARENT}, not C ({SHORT})\n"));
}

#[test]
fn a_kept_line_pairs_each_start_tag_at_its_column() {
    // The same line starts in raw text on one side and markup on the other.
    let requested = super::html::tags("<style>\n<button></button></style><button>");
    let printed = super::html::tags("<main>\n<button></button></style><button>");
    let changes = crate::look_compare::pages::hunks("@@ -1 +1 @@\n-<style>\n+<main>\n").expect("one changed line");
    assert_eq!(super::diff::pairs(&requested, &printed, &changes).expect("valid alignment"), vec![None, Some(2)]);
}

#[test]
fn a_hunk_never_pairs_tags_past_the_first_different_name() {
    let requested = super::html::tags("<main>\n<button>\n<aside>\n");
    let printed = super::html::tags("<main>\n<dialog>\n<aside>\n");
    let changes = crate::look_compare::pages::hunks(
        "@@ -1,3 +1,3 @@\n-<main>\n-<button>\n-<aside>\n+<main>\n+<dialog>\n+<aside>\n",
    )
    .expect("one hunk");
    assert_eq!(super::diff::pairs(&requested, &printed, &changes).expect("valid alignment"), vec![Some(0), None, None]);
}

#[test]
fn touching_a_parent_touches_descendants_but_not_its_siblings() {
    let descendants = [("main", 0), ("details", 1), ("div", 2), ("span", 3), ("footer", 1), ("a", 2)];
    let elements = descendants
        .into_iter()
        .enumerate()
        .map(|(at, (name, depth))| Element {
            node: u64::try_from(at).expect("fixture index"),
            depth,
            name: name.as_bytes().into(),
        })
        .collect::<Vec<_>>();
    let mut selected = [false, true, false, false, false, false];
    super::descendants(&elements, &mut selected);
    assert_eq!(selected, [false, true, true, true, false, false]);
}

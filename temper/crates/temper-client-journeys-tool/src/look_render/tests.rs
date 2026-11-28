use std::path::{Path, PathBuf};

use skein_browser::{
    Limits,
    boundary::{Element, Node, Property, Rect, Refusal, Trouble},
};
use skein_lib::Token;

use crate::{
    host::{
        self, Answer, Ask, Gathered, Host, Program, Ran, Stream,
        browser::{Command, Observation, Reply},
    },
    look_compare::tests::{NAME, ON_DISK_WALK, PARENT, ROOT, SHELL_WEB, SHORT, SIGN_IN_PAGE, Steps, TIP, TUI, WEB},
};

use crate::look_git::{check_lists, look_step_tests, record};

use super::run;

pub(crate) mod lines;

const SCRATCH: &str = "/landing tree/.git/look-render";
const PROFILE: &str = "/landing tree/.git/look-render/browser";
const DIFF: &str = "/landing tree/.git/look-render/diff";
const WEB_WALK: &str = "temper/docs/design/client/web/journeys/sign-in/walk";
const OVERLAY: &str = "temper/docs/design/client/web/journeys/overlay.css";
const TUI_OVERLAY: &str = "temper/docs/design/client/tui/journeys/overlay.css";
const HAND_MADE_DIR: &str = "temper/docs/design/client/web/journeys/not-a-walk";
const HAND_MADE_PAGE: &str = "temper/docs/design/client/web/journeys/not-a-walk/index.html";
const GALLERY: &str = "/landing tree/.git/look-render/gallery";
const STATE_FILE: &str = "accounts-usage-filling.html";
const STATE_README: &str = "Hand-made from the gallery's state `accounts-usage-filling`: an account's folded row with its 5 h and week windows.";
const DRAWN_NAME: &str = "`accounts-after.html` the request";
const BEFORE_NAME: &str = "`accounts-before.html`";
const DRAWN_DIR: &str = "temper/docs/design/client/web/journeys/usage-model-week";
const DRAWN_README: &str = "temper/docs/design/client/web/journeys/usage-model-week/README.md";
const DRAWN_PAGE: &str = "temper/docs/design/client/web/journeys/usage-model-week/accounts-after.html";
const DRAWN_ITEM: &str = "web/journeys/usage-model-week/accounts-after.html";
const BEFORE: &str = "temper/docs/design/client/web/journeys/usage-model-week/accounts-before.html";
const BEFORE_ITEM: &str = "web/journeys/usage-model-week/accounts-before.html";
const STATE_SOURCE: &str = concat!(
    "<!doctype html>\n<html><head></head><body><details><dialog></dialog></details>",
    "<span class=\"temper-usage-name\"><i>Account</i></span></body></html>\n",
);
const STATE_OPENED: &str = concat!(
    "<!doctype html>\n<html><head></head><body><details open><dialog open></dialog></details>",
    "<span class=\"temper-usage-name\"><i>Account</i></span></body></html>\n",
);
const STATE_TAGS: &[(&str, u32)] =
    &[("html", 0), ("head", 1), ("body", 1), ("details", 2), ("dialog", 3), ("span", 2), ("i", 3)];
const FIRST_REQUEST_NODE: Node = 11;
const FIRST_PRINT_NODE: Node = 111;
const SPAN: Node = FIRST_REQUEST_NODE + 5; // Sixth start tag in STATE_SOURCE.
const LANDED_SPAN: Node = FIRST_PRINT_NODE + 5;
const USAGE_NAME: &str = ".temper-usage-name";
const REQUEST_OVERLAY: &str = ".temper-usage-name { width: 48px; padding: 0 2px }";
const UNSPECIFIED: &str = "not the requested look";
const ITEM: &str = "web/journeys/sign-in/02.html";
const PERSON: Token = Token::new(1);
const REQUEST: Token = Token::new(2);
const PRINT: Token = Token::new(3);
const BUTTON: Node = 15;
const LANDED_BUTTON: Node = 115;
const LINE: usize = 41; // The button follows the five header lines and this fixture's blank lines.
const HEADER: &str =
    "<!doctype html>\n<html>\n<head><title>Sign in</title></head>\n<!-- press to continue -->\n<body>\n";
const COMPARABLE_HEADER: &str =
    "<!doctype html>\n<html\n>\n<head\n><title\n>Sign in</title\n></head\n>\n<!-- press to continue -->\n<body\n>\n";
const DIFF_LINE: usize = LINE + 6; // Six tag boundaries in HEADER put the button on a later line in the diff.
const WIDTHS: [u32; 3] = [1440, 1000, 420]; // client/journey-writer-requests.md, 1.2, step 7.
const HEIGHT: u32 = 900;
const BOX: Rect = Rect { x: -5, y: 20, width: 80, height: 30 };
const BOX_STYLE_CHANGE: [&[(&str, &str)]; 2] = [
    &[
        ("width", "80px"),
        ("height", "30px"),
        ("inline-size", "80px"),
        ("block-size", "30px"),
        ("transform-origin", "40px 15px"),
        ("perspective-origin", "40px 15px"),
    ],
    &[
        ("width", "81px"),
        ("height", "50px"),
        ("inline-size", "81px"),
        ("block-size", "50px"),
        ("transform-origin", "40.5px 25px"),
        ("perspective-origin", "40.5px 25px"),
    ],
];
const LEFT_COLOR: &str = "rgb(1, 2, 3)";
const RIGHT_COLOR: &str = "rgb(4, 5, 6)";
const IDENTICAL: &str = "identical\tweb/journeys/sign-in/02.html\nresult\tidentical\n";
const DIFFERS: &str = "result\tdiffers\n";
const NOTHING: &str = "result\tnothing to render\n";

#[derive(Clone, Copy)]
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
        self.0.fetch(TIP);
        self.request_changes(changed);
    }

    fn request_changes(&mut self, changed: &[&str]) {
        let s = &mut self.0;
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

    fn hand_made_paths(&mut self, directory: &str, changed: &[&str]) {
        let mut paths = String::new();
        for path in changed {
            paths.push_str(path);
            paths.push('\0');
        }
        self.0.git(
            &[
                "--literal-pathspecs",
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "--no-renames",
                "-r",
                "-z",
                PARENT,
                TIP,
                "--",
                directory,
            ],
            Some(ROOT),
            &paths,
        );
    }

    fn gallery_print(&mut self, code: i32, stderr: &str) {
        let mut program = Program::new(
            b"cargo",
            &[b"run", b"--locked", b"--quiet", b"-p", b"temper-client-gallery", b"--", GALLERY.as_bytes()],
        );
        program.directory = Some(ROOT.into());
        program.environment.push((b"TEMPER_INTERNAL_BUNDLE".as_slice().into(), b"none".as_slice().into()));
        program.stderr = Stream::Keep;
        let mut ran = Ran::new(Some(code), b"gallery output\n");
        ran.stderr = stderr.as_bytes().into();
        self.0.step(Ask::Run(program), Answer::Ran(ran));
    }

    fn gallery_page(&mut self, source: &str, opened: &str) {
        self.0.read(&format!("{GALLERY}/{STATE_FILE}"), Some(source));
        self.0.write(GALLERY, STATE_FILE, opened);
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
        self.open_file(page, copy, SIGN_IN_PAGE);
    }

    fn open_file(&mut self, page: Token, copy: &str, leaf: &str) {
        let url = format!("file:///landing%20tree/.git/look-render/{copy}/{leaf}");
        self.done(Command::Open { person: PERSON, page, url: url.into_bytes().into_boxed_slice(), files: true });
    }

    fn opened_pair(&mut self) {
        self.open(REQUEST, "request");
        self.open(PRINT, "print");
    }

    fn close_pair(&mut self) {
        self.close_pages([REQUEST, PRINT]);
    }

    fn close_pages(&mut self, [request, print]: [Token; 2]) {
        self.done(Command::Close { page: print });
        self.done(Command::Close { page: request });
    }

    fn gallery_pair(&mut self, [request, print]: [Token; 2], requested: &[(&str, u32)]) {
        self.open_file(print, "gallery", STATE_FILE);
        self.elements_of(request, FIRST_REQUEST_NODE, requested);
        self.elements_of(print, FIRST_PRINT_NODE, STATE_TAGS);
    }

    fn gallery_span(&mut self, [request, print]: [Token; 2]) {
        self.selects(request, USAGE_NAME, Some((SPAN, "span")));
        self.gallery_pair([request, print], STATE_TAGS);
    }

    fn selects(&mut self, page: Token, selector: &str, hit: Option<(Node, &str)>) {
        let elements =
            hit.into_iter().map(|(node, name)| Element { node, name: name.as_bytes().into(), depth: 2 }).collect();
        self.chrome(
            Command::Elements { page, selector: Some(selector.as_bytes().into()) },
            Reply::Elements { elements, more: 0 },
        );
    }

    fn measure_gallery(&mut self, [request, print]: [Token; 2], from: Node, styles: [&[(&str, &str)]; 2]) {
        for width in WIDTHS {
            self.viewport_pair([request, print], width);
            self.styles([(request, from), (print, LANDED_SPAN)], styles);
        }
    }

    fn elements(&mut self, page: Token, tag: &str) {
        let last = if page == PRINT { LANDED_BUTTON } else { BUTTON };
        self.elements_of(page, last - 4, &[("html", 0), ("head", 1), ("title", 2), ("body", 1), (tag, 2)]);
    }

    fn elements_of(&mut self, page: Token, first: Node, tags: &[(&str, u32)]) {
        let elements = tags
            .iter()
            .enumerate()
            .map(|(at, &(name, depth))| Element {
                node: first + u64::try_from(at).expect("fixture index"),
                name: name.as_bytes().into(),
                depth,
            })
            .collect();
        self.chrome(Command::Elements { page, selector: None }, Reply::Elements { elements, more: 0 });
    }

    fn edit(&mut self, parent: &str, [from, to]: [&str; 2], patch: &str) {
        self.0.blob(PARENT, SIGN_IN_PAGE, Some(parent));
        self.diff_files([("parent", from), ("requested", to)], patch);
    }

    fn diff(&mut self, from: &str, to: &str, patch: &str) {
        self.diff_files([("requested", from), ("landed", to)], patch);
    }

    fn diff_files(&mut self, [(from_file, from), (to_file, to)]: [(&str, &str); 2], patch: &str) {
        self.0.write(DIFF, from_file, from);
        self.0.write(DIFF, to_file, to);
        self.0.diff(DIFF, [from_file, to_file], patch);
    }

    fn viewport(&mut self, width: u32) {
        self.viewport_pair([REQUEST, PRINT], width);
    }

    fn viewport_pair(&mut self, pages: [Token; 2], width: u32) {
        for page in pages {
            self.done(Command::Viewport { page, width, height: HEIGHT });
        }
    }

    fn measure(&mut self, width: u32, styles: [&[(&str, &str)]; 2], boxes: [Option<Rect>; 2]) {
        self.viewport(width);
        self.measure_pair([(REQUEST, BUTTON), (PRINT, LANDED_BUTTON)], styles, boxes);
    }

    fn styles(&mut self, pair: [(Token, Node); 2], styles: [&[(&str, &str)]; 2]) {
        for ((page, node), properties) in pair.into_iter().zip(styles) {
            let style = properties
                .iter()
                .map(|(name, value)| Property { name: name.as_bytes().into(), value: value.as_bytes().into() })
                .collect();
            self.chrome(Command::Style { page, node }, Reply::Style(style));
        }
    }

    fn measure_pair(&mut self, pair: [(Token, Node); 2], styles: [&[(&str, &str)]; 2], boxes: [Option<Rect>; 2]) {
        self.styles(pair, styles);
        for ((page, node), rect) in pair.into_iter().zip(boxes) {
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
        record(&steps);
        let mut host = Host::canned(steps);
        let args = [NAME.as_bytes().into(), SHORT.as_bytes().into()];
        assert_eq!(run(&args, &mut host), code);
        let Gathered { stdout, stderr } = host.finish();
        let stdout = std::str::from_utf8(&stdout).expect("fixture output is text");
        assert_eq!(stdout, out);
        assert_eq!(std::str::from_utf8(&stderr).expect("fixture diagnostics are text"), err);
        lines::record(stdout);
    }
}

fn drawn_request(readme: &str, pages: &[Files<'_>], [parent, overlay]: [&str; 2]) -> Effects {
    let mut steps = Effects::default();
    let mut changed = vec![DRAWN_README];
    changed.extend(pages.iter().map(|file| file.path));
    let mut files = vec![Files::same(DRAWN_README, readme)];
    files.extend_from_slice(pages);
    if parent == overlay {
        steps.begin(&changed);
    } else {
        let mut all = changed.clone();
        all.push(OVERLAY);
        steps.begin(&all);
        files.push(Files::same(OVERLAY, overlay));
    }
    steps.0.blob(TIP, &format!("{DRAWN_DIR}/walk"), None);
    steps.copy_request(&files);
    steps.copy_print(&[]);
    if parent != overlay {
        steps.0.blob(PARENT, OVERLAY, Some(parent));
        steps.0.blob(TIP, OVERLAY, Some(overlay));
    }
    steps.hand_made_paths(DRAWN_DIR, &changed);
    steps.0.blob(TIP, DRAWN_README, Some(readme));
    steps
}

fn named_drawing(overlays: [&str; 2]) -> Effects {
    drawn_request(&format!("{STATE_README}\n{DRAWN_NAME}, its week filling"), &[drawn_page(DRAWN_PAGE)], overlays)
}

fn gallery_drawing(overlays: [&str; 2]) -> Effects {
    let mut steps = named_drawing(overlays);
    start_gallery(&mut steps);
    steps.open_file(REQUEST, "request", DRAWN_PAGE);
    steps
}

fn start_gallery(steps: &mut Effects) {
    steps.gallery_print(0, "");
    steps.gallery_page(STATE_SOURCE, STATE_OPENED);
    steps.start();
}

fn gallery_differences(rows: &[(&str, &str, &str)]) -> String {
    use std::fmt::Write as _;

    let mut expected = String::new();
    for width in WIDTHS {
        for (name, from, to) in rows {
            writeln!(expected, "differs\t{DRAWN_ITEM}\t{width}\t2 span.temper-usage-name\t{name}\t{from}\t{to}")
                .expect("write string");
        }
    }
    expected.push_str(DIFFERS);
    expected
}

fn drawn_page(path: &str) -> Files<'_> {
    Files { path, source: STATE_SOURCE, copy: STATE_OPENED }
}

fn page(tag: &str, class: &str) -> String {
    format!("{HEADER}{}<{tag} class=\"{class}\">Continue</{tag}>\n</body>\n</html>\n", "\n".repeat(LINE - 6))
}

fn comparable_page(tag: &str, class: &str) -> String {
    format!(
        "{COMPARABLE_HEADER}{}<{tag} class=\"{class}\"\n>Continue</{tag}\n>\n</body\n>\n</html\n>\n",
        "\n".repeat(LINE - 6)
    )
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
    format!("@@ -{DIFF_LINE} +{DIFF_LINE} @@\n-<button class=\"old\"\n+<button class=\"temper-chip\"\n")
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
    let (mut steps, _) = prepared(&document, "button");
    button_edit(&mut steps);
    steps
}

fn button_edit(steps: &mut Effects) {
    steps.edit(
        &page("button", "old"),
        [&comparable_page("button", "old"), &comparable_page("button", "temper-chip")],
        &button_patch(),
    );
}

fn difference(width: u32, property: &str, left: &str, right: &str) -> String {
    format!("differs\t{ITEM}\t{width}\t41 button.temper-chip\t{property}\t{left}\t{right}\n")
}

fn uncompared(item: &str, reason: &str) -> String {
    format!("uncompared\t{item}\t{reason}\nresult\tnot compared\n")
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

const INSERTION: usize = 3; // Between <body> and <details> in STATE_TAGS.
const NEW_RULE: &str = ".new { color: red }";

fn inline_drawing(overlay: &str) -> Effects {
    const HEADER: &str = "<!doctype html>\n<html><head></head><body>";
    const TAIL: &str = "</dialog></details><span class=\"temper-usage-name\"><i>Account</i></span></body></html>\n";
    const NEW_TAG: &str = "<hr class=\"new\" data-node=7>";
    let drawing = format!("{HEADER}{NEW_TAG}<details DATA-NODE='8'><dialog data-node=\"9\">{TAIL}");
    let drawing_open = format!("{HEADER}{NEW_TAG}<details open DATA-NODE='8'><dialog open data-node=\"9\">{TAIL}");
    let mut steps = drawn_request(
        &format!("{STATE_README}\n{DRAWN_NAME}"),
        &[Files { path: DRAWN_PAGE, source: &drawing, copy: &drawing_open }],
        ["", overlay],
    );
    steps.gallery_print(0, "");
    steps.gallery_page(
        &format!("{HEADER}<details data-node=\"1\"><dialog data-node=\"2\">{TAIL}"),
        &format!("{HEADER}<details open data-node=\"1\"><dialog open data-node=\"2\">{TAIL}"),
    );
    steps.start();
    steps.open_file(REQUEST, "request", DRAWN_PAGE);
    steps.selects(REQUEST, ".new", Some((FIRST_REQUEST_NODE + u64::try_from(INSERTION).expect("fixture index"), "hr")));
    steps
}

fn inline_alignment(steps: &mut Effects) {
    const LINE_HEADER: &str = "<!doctype html>\n<html\n><head\n></head\n><body\n";
    const LINE_TAIL: &str = concat!(
        "><details\n><dialog\n></dialog\n></details\n><span class=\"temper-usage-name\"\n><i\n>",
        "Account</i\n></span\n></body\n></html\n>\n",
    );
    let mut tags = STATE_TAGS.to_vec();
    tags.insert(INSERTION, ("hr", 2));
    steps.gallery_pair([REQUEST, PRINT], &tags);
    steps.diff(
        &format!("{LINE_HEADER}><hr class=\"new\"\n{LINE_TAIL}"),
        &format!("{LINE_HEADER}{LINE_TAIL}"),
        "@@ -6 +5,0 @@\n-><hr class=\"new\"\n",
    );
}

fn check_step(name: &str, run: impl FnOnce()) {
    check_lists(name, || lines::check(name, run));
}

fn page_with_truncated_elements(truncated: Token) {
    let document = page("button", "temper-chip");
    let (mut steps, _) = fixture(Some(&document));
    steps.start();
    steps.opened_pair();
    for page in [REQUEST, PRINT] {
        if page == truncated {
            // The names from an incomplete reply must not be judged.
            const ONE_PAST_LIMIT: u32 = 1;
            steps.chrome(
                Command::Elements { page, selector: None },
                Reply::Elements { elements: Vec::new(), more: ONE_PAST_LIMIT },
            );
        } else {
            steps.elements(page, "button");
        }
    }
    let limit = Limits::default().elements;
    steps.verdict(&uncompared(ITEM, &format!("too large to compare, over {limit} elements")));
}

look_step_tests! { @check check_step;
#[test]
fn a_locally_held_request_is_render_checked_without_fetching() {
    let mut steps = Effects::default();
    steps.0.local_request();
    steps.request_changes(&[]);
    steps.copy_request(&[]);
    steps.copy_print(&[]);
    steps.start();
    steps.stop_and_check(0, NOTHING, "");
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
    let [left, _] = BOX_STYLE_CHANGE;
    let near = &[
        ("width", "81px"), ("height", "29px"), ("inline-size", "81px"), ("block-size", "29px"),
        ("transform-origin", "40.5px 14.5px"), ("perspective-origin", "40.5px 14.5px"),
    ];
    for width in WIDTHS {
        steps.measure(width, [left, near], [Some(BOX), Some(Rect { x: -4, y: 21, width: 81, height: 29 })]);
    }
    steps.verdict(IDENTICAL);
}

#[test]
fn box_difference_replaces_all_six_dependent_style_differences() {
    let mut steps = edited_button();
    let mut expected = String::new();
    for width in WIDTHS {
        steps.measure(width, BOX_STYLE_CHANGE, [Some(BOX), Some(Rect { width: 81, height: 50, ..BOX })]);
        expected.push_str(&difference(width, "box", "-5,20 80x30", "-5,20 81x50"));
    }
    expected.push_str(DIFFERS);
    steps.verdict(&expected);
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
    let landed = comparable_page("p", "temper-chip");
    let (mut steps, _) = prepared(&landed, "p");
    button_edit(&mut steps);
    steps.diff(
        &comparable_page("button", "temper-chip"),
        &comparable_page("p", "temper-chip"),
        &format!("@@ -{DIFF_LINE},2 +{DIFF_LINE},2 @@\n-<button class=\"temper-chip\"\n->Continue</button\n+<p class=\"temper-chip\"\n+>Continue</p\n"),
    );
    steps.verdict(&format!("differs\t{ITEM}\t-\t41 button.temper-chip\tpair\tpresent\tnone\n{DIFFERS}"));
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
fn a_truncated_request_dom_is_too_large_to_compare() {
    page_with_truncated_elements(REQUEST);
}

#[test]
fn a_truncated_print_dom_is_too_large_to_compare() {
    page_with_truncated_elements(PRINT);
}

#[test]
fn hand_made_pages_receive_a_row_without_rendering() {
    let mut steps = Effects::default();
    steps.begin(&[HAND_MADE_PAGE]);
    steps.0.blob(TIP, &format!("{HAND_MADE_DIR}/walk"), Some("walk 1\n"));
    steps.copy_request(&[Files::same(HAND_MADE_PAGE, "<p>Hello</p>")]);
    steps.copy_print(&[]);
    steps.hand_made_paths(HAND_MADE_DIR, &[HAND_MADE_PAGE]);
    steps.0.blob(TIP, &format!("{HAND_MADE_DIR}/README.md"), None);
    steps.start();
    steps.stop_and_check(0, &uncompared("web/journeys/not-a-walk/index.html", "hand-made"), "");
}

#[test]
fn only_the_named_request_is_compared_with_the_opened_state() {
    let readme = format!("{STATE_README}\n{DRAWN_NAME}, its week filling\n{BEFORE_NAME} the look before");
    let mut steps = drawn_request(&readme, &[drawn_page(DRAWN_PAGE), drawn_page(BEFORE)], ["", REQUEST_OVERLAY]);
    start_gallery(&mut steps);
    let pair = [REQUEST, PRINT];
    steps.open_file(REQUEST, "request", DRAWN_PAGE);
    steps.gallery_span(pair);
    steps.measure_gallery(pair, SPAN, [&[]; 2]);
    steps.verdict(&format!("identical\t{DRAWN_ITEM}\nuncompared\t{BEFORE_ITEM}\t{UNSPECIFIED}\nresult\tidentical\n"));
}

#[test]
fn two_drawings_share_one_gallery_print_and_state_page() {
    let readme = format!(
        "{STATE_README}\nAlso drawn on the gallery's state `accounts-usage-filling`.\n\
        {DRAWN_NAME}, its week filling\n{BEFORE_NAME} the request"
    );
    let mut steps = drawn_request(&readme, &[drawn_page(DRAWN_PAGE), drawn_page(BEFORE)], ["", REQUEST_OVERLAY]);
    start_gallery(&mut steps);
    for (path, [request, print]) in [(DRAWN_PAGE, [REQUEST, PRINT]), (BEFORE, [Token::new(4), Token::new(5)])] {
        steps.open_file(request, "request", path);
        steps.gallery_span([request, print]);
        steps.measure_gallery([request, print], SPAN, [&[]; 2]);
        steps.close_pages([request, print]);
    }
    steps.stop_and_check(0, &format!("identical\t{DRAWN_ITEM}\nidentical\t{BEFORE_ITEM}\nresult\tidentical\n"), "");
}

#[test]
fn naming_a_state_without_request_pages_never_prints_it() {
    for suffix in [None, Some("ed look"), Some("2"), Some("-like"), Some("_like"), Some("'s"), Some("’s")] {
        let readme = suffix.map_or_else(|| STATE_README.to_owned(), |suffix| format!("{STATE_README}\n{DRAWN_NAME}{suffix}"));
        let mut steps = drawn_request(&readme, &[drawn_page(DRAWN_PAGE), drawn_page(BEFORE)], ["", REQUEST_OVERLAY]);
        steps.start();
        let expected = format!(
            "uncompared\t{DRAWN_ITEM}\t{UNSPECIFIED}\nuncompared\t{BEFORE_ITEM}\t{UNSPECIFIED}\nresult\tnot compared\n"
        );
        steps.stop_and_check(0, &expected, "");
    }
}

#[test]
fn gallery_compares_only_selected_elements_and_their_declared_style() {
    let mut steps = gallery_drawing(["", REQUEST_OVERLAY]);
    let pair = [REQUEST, PRINT];
    steps.gallery_span(pair);
    steps.measure_gallery(pair, SPAN, [
        &[("width", "48px"), ("padding-left", "2px"), ("color", LEFT_COLOR), ("height", "12px"), ("--x", "red")],
        &[("width", "40px"), ("padding-left", "1px"), ("color", RIGHT_COLOR), ("height", "10px"), ("--x", "blue")],
    ]); // No Style for <i> inside the span, or for other tags; no Rect for even the span.
    steps.verdict(&gallery_differences(&[("padding-left", "2px", "1px"), ("width", "48px", "40px")]));
}

#[test]
fn a_shorthand_can_set_longhands_with_different_prefixes() {
    let mut steps = gallery_drawing([
        "", ".temper-usage-name { inset: 2px; gap: 2px; transform: none; scroll-margin: 2px }",
    ]);
    let pair = [REQUEST, PRINT];
    steps.gallery_span(pair);
    steps.measure_gallery(pair, SPAN, [
        &[("top", "2px"), ("row-gap", "2px"), ("transform", "none"), ("transform-origin", "0px 0px"), ("scroll-margin-top", "2px")],
        &[("top", "1px"), ("row-gap", "1px"), ("transform", "none"), ("transform-origin", "24px 0px"), ("scroll-margin-top", "1px")],
    ]);
    steps.verdict(&gallery_differences(&[("row-gap", "2px", "1px"), ("scroll-margin-top", "2px", "1px"), ("top", "2px", "1px")]));
}

#[test]
fn changed_and_removed_rules_carry_their_parents_declared_names() {
    let mut steps = gallery_drawing([
        ".temper-usage-name:hover { --temper-name-width: 48px } span { min-width: 2px } .unchanged { opacity: 1 }",
        ".temper-usage-name:hover { color: red } .unchanged { opacity: 1 }",
    ]);
    for selector in [USAGE_NAME, "span"] {
        steps.selects(REQUEST, selector, Some((SPAN, "span")));
    }
    let pair = [REQUEST, PRINT];
    steps.gallery_pair(pair, STATE_TAGS);
    steps.measure_gallery(pair, SPAN, [
        &[("--temper-name-width", "48px"), ("min-width", "2px"), ("color", LEFT_COLOR)],
        &[("--temper-name-width", "40px"), ("min-width", "0px"), ("color", RIGHT_COLOR)],
    ]);
    steps.verdict(&gallery_differences(&[
        ("--temper-name-width", "48px", "40px"), ("color", LEFT_COLOR, RIGHT_COLOR), ("min-width", "2px", "0px"),
    ]));
}

#[test]
fn an_untouched_gallery_drawing_has_no_page_verdict() {
    let mut steps = gallery_drawing(["", REQUEST_OVERLAY]);
    steps.selects(REQUEST, USAGE_NAME, None);
    steps.done(Command::Close { page: REQUEST });
    steps.stop_and_check(0, NOTHING, "");
}

#[test]
fn a_readme_must_name_one_distinct_valid_gallery_state() {
    for readme in [
        "Hand-made from an account's folded row.",
        "Two examples: the gallery's state `accounts-usage-filling` and the gallery's state `account-2`.",
        "Not a state name: the gallery's state `../accounts-usage-filling`.",
    ] {
        let mut steps = drawn_request(&format!("{readme}\n{DRAWN_NAME}"), &[drawn_page(DRAWN_PAGE)], [""; 2]);
        steps.start();
        steps.stop_and_check(0, &uncompared(DRAWN_ITEM, "hand-made"), "");
    }
}

#[test]
fn a_tui_hand_made_page_cannot_name_a_gallery_state() {
    const DIR: &str = "temper/docs/design/client/tui/journeys/usage-model-week";
    const README: &str = "temper/docs/design/client/tui/journeys/usage-model-week/README.md";
    const PAGE: &str = "temper/docs/design/client/tui/journeys/usage-model-week/accounts-after.html";
    let mut steps = Effects::default();
    steps.begin(&[README, PAGE]);
    steps.0.blob(TIP, &format!("{DIR}/walk"), None);
    steps.copy_request(&[Files::same(README, &format!("{STATE_README}\n{DRAWN_NAME}")), drawn_page(PAGE)]);
    steps.copy_print(&[]);
    steps.hand_made_paths(DIR, &[README, PAGE]);
    steps.start();
    steps.stop_and_check(0, &uncompared("tui/journeys/usage-model-week/accounts-after.html", "hand-made"), "");
}

#[test]
fn a_state_missing_from_the_gallery_is_not_compared() {
    let mut steps = named_drawing(["", REQUEST_OVERLAY]);
    steps.gallery_print(0, "");
    steps.0.read(&format!("{GALLERY}/{STATE_FILE}"), None);
    steps.start();
    steps.stop_and_check(0, &uncompared(DRAWN_ITEM, "nothing printed beside it"), "");
}

#[test]
fn gallery_print_failure_aborts_before_chromium_without_stdout() {
    let mut steps = named_drawing(["", REQUEST_OVERLAY]);
    steps.gallery_print(1, "error: gallery could not print\nsecond cargo diagnostic line\n");
    steps.check(3, "", "look-render: gallery print: exit 1: error: gallery could not print\n");
}

#[test]
fn an_inserted_inline_tag_is_skipped_without_disturbing_pairs() {
    const SHIFTED_SPAN: Node = SPAN + 1; // One inserted tag before the span.
    let mut steps = inline_drawing(&format!("{NEW_RULE} {REQUEST_OVERLAY}"));
    steps.selects(REQUEST, USAGE_NAME, Some((SHIFTED_SPAN, "span")));
    inline_alignment(&mut steps);
    steps.measure_gallery([REQUEST, PRINT], SHIFTED_SPAN, [[("width", "48px")].as_slice(); 2]);
    steps.verdict(&format!("identical\t{DRAWN_ITEM}\nresult\tidentical\n"));
}

#[test]
fn a_gallery_drawing_with_no_paired_selection_is_uncompared() {
    let mut steps = inline_drawing(NEW_RULE);
    inline_alignment(&mut steps);
    steps.verdict(&uncompared(DRAWN_ITEM, "nothing printed beside it"));
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
    let comparable = comparable_page("button", "temper-chip");
    let before = comparable.replace("press to continue", "press to sign in");
    steps.edit(&parent, [&before, &comparable], "@@ -9 +9 @@\n-<!-- press to sign in -->\n+<!-- press to continue -->\n");
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
    const ALL: u32 = 52; // Two past the page's fifty listed differences (client/journey-writer-requests.md, 1.2).
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
}

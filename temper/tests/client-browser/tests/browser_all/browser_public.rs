//! The public walk in Chromium (temper/docs/design/client/web/public.md, 4): the
//! signed-out page with no stream, its public reads, and a 404 under the page's policy.
use skein_fake_person::{Face, Find, Found, Next, Step as PersonStep, Target};
use skein_fake_person_browser::harness::Output;
use skein_fake_person_browser::{BrowserFace, Command, Opens, Outcome, Play, Script, Sent, Stdin, Step};
use skein_lib::Time;
use skein_ui::Role;
use std::{fs, path::Path};
use temper_client_browser_world::{Tier, WITHIN, temper};

const TITLE: &[u8] = b"Export widget's table as CSV";
const CHAT: &[u8] = b"Add CSV export to widget";
const UNKNOWN: &[u8] = b"Not public, or not here.";
const PROJECT: &[u8] = b"Demo";
// Public identifiers have sixteen hexadecimal digits (jig/docs/design/domain/people/public-reading.md, 8).
const ID_DIGITS: usize = 16;

fn see(find: Find) -> Step {
    Step::Person(PersonStep::See { find, within: WITHIN })
}
fn press(find: Find) -> Step {
    Step::Person(PersonStep::Press { find })
}
fn go(path: &[u8]) -> Step {
    Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(path) })
}
fn heading(name: &[u8]) -> Find {
    Find::exactly_named(Role::Heading { level: 1 }, name)
}
fn exited() -> Step {
    Step::Exited { code: 0, within: WITHIN }
}
fn sign_in(directory: &Path) -> Step {
    Step::Run(Command {
        program: temper(),
        args: vec!["ctl".into(), directory.into(), "sign-in".into(), "-".into()],
        env: vec![],
        stdin: Stdin::Printed,
        opens: None,
        within: WITHIN,
    })
}
fn open_printed() -> Step {
    Step::Run(Command {
        program: "/bin/cat".into(),
        args: vec![],
        env: vec![],
        stdin: Stdin::Printed,
        opens: Some(Opens {
            prefix: b"http://127.0.0.1:".as_slice().into(),
            on: Output::Stdout,
            keep: b"owner-link".as_slice().into(),
        }),
        within: WITHIN,
    })
}
fn visibility(directory: &Path, public: bool) -> Step {
    let visibility = if public { "public" } else { "private" };
    // The harness exposes a child's exit but not its stdout to Play. Keep only ctl's
    // visibility answer in this story's scratch; arguments are separate shell parameters.
    Step::Run(Command {
        program: "/bin/sh".into(),
        args: vec![
            "-c".into(),
            "umask 077; exec \"$1\" ctl \"$2\" project amend Demo --visibility \"$3\" --key \"$3\" > \"$4\"".into(),
            "public-walk-ctl".into(),
            temper().into_os_string(),
            directory.into(),
            visibility.into(),
            directory.join(format!("{visibility}.txt")).into_os_string(),
        ],
        env: vec![],
        stdin: Stdin::Closed,
        opens: None,
        within: WITHIN,
    })
}

fn owner_steps(tier: &Tier, client: &Path) -> Vec<Step> {
    vec![
        // The first profile signs ctl in with the initial printed link. This
        // profile opens the next link, so neither reuses a spent sign-in link.
        open_printed(),
        exited(),
        see(Find::exactly_named(Role::Button, b"Try the playground")),
        press(Find::exactly_named(Role::Button, b"Try the playground")),
        Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(b"/chats/new".as_slice()) }),
        see(Find::exactly_named(Role::Status, b"Ready")),
        Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Status, b"Starting"), within: WITHIN }),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        see(heading(b"New chat")),
        visibility(client, true),
        exited(),
        see(Find::exactly_named(Role::Button, CHAT)),
        press(Find::exactly_named(Role::Button, CHAT)),
        see(Find::text("I’ll delegate this change".as_bytes())
            .within(Box::new([Target::ExactRole { role: Role::Log, name: b"Conversation".as_slice().into() }]))),
        see(Find::exactly_named(Role::Region, b"Task 2")),
        see(Find::exactly_named(Role::Link, b"Task 2")
            .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task 2".as_slice().into() }]))),
        press(Find::exactly_named(Role::Button, b"Owner")),
        press(Find::exactly_named(Role::MenuItem, b"Sign out")),
        see(tier.states().signed_out),
        Step::Reopen,
    ]
}

#[derive(Clone, Copy, Debug)]
enum Page {
    Owner,
    Projects,
    Board,
    Task,
    Landings,
    Private,
    Empty,
    Malformed,
    Done,
}

struct Walk {
    client: skein_scratch::Scratch,
    script: Script,
    page: Page,
    reading: bool,
    address_pending: bool,
    text_at: u32,
    text: Vec<String>,
    board: String,
    task: String,
    started: Option<Time>,
}
impl Walk {
    fn new(tier: &Tier) -> Self {
        let client = skein_scratch::Scratch::new("public-walk-ctl");
        let script = Script::new(owner_steps(tier, client.path()).into_boxed_slice());
        Self {
            client,
            script,
            page: Page::Owner,
            reading: false,
            address_pending: false,
            text_at: 0,
            text: vec![],
            board: String::new(),
            task: String::new(),
            started: None,
        }
    }
    fn open(&mut self, page: Page, steps: Vec<Step>) {
        self.page = page;
        self.script = Script::new(steps.into_boxed_slice());
        self.reading = false;
        self.address_pending = false;
        self.text_at = 0;
        self.text.clear();
    }
    fn published(&mut self) {
        let answer = fs::read_to_string(self.client.path().join("public.txt")).expect("ctl public answer");
        let mut lines = answer.lines();
        assert_eq!(lines.next(), Some("project 1: public"), "ctl's visibility result");
        self.board = lines.next().and_then(|line| line.strip_prefix("page: ")).expect("ctl public path").into();
        assert!(lines.next().is_none(), "only the visibility result and its page");
        identifier(self.board.strip_prefix("/public/projects/").expect("public project path"));
        eprintln!("ctl made public: {answer}");
        self.open(
            Page::Projects,
            vec![
                go(b"/public"),
                see(heading(b"Public projects")),
                see(Find::exactly_named(Role::Link, PROJECT)),
                Step::Address(b"/public".as_slice().into()),
            ],
        );
    }
    fn check_page(&self, path: &str) {
        let text = self.text.join("\n");
        eprintln!("public page {path}:\n{text}");
        for forbidden in [
            "Add CSV export to widget",
            "I’ll delegate this change",
            "CSV export is available from widget",
            "This was a scripted change",
            "Task 2",
            "Task 3",
            "T1",
            "T2",
            "T3",
            "Project 1",
            "Signed in",
            "Sign out",
            "Live",
            "Offline",
            "Conversation",
            "Playground forge",
            "Owner",
            "owner",
            "temper/",
            "main",
            "Review the change",
            "Checked widget CSV export.",
            "https://",
            "spent \u{b7} of",
        ] {
            assert!(!text.contains(forbidden), "{path} exposed {forbidden:?}: {text}");
        }
        self.check_page_fields(&text);
    }
    fn check_page_fields(&self, text: &str) {
        if matches!(self.page, Page::Board) {
            assert!(text.contains("Demo"), "public project name is the board heading");
            assert!(text.contains("Goal "), "board names the goal by public identifier");
            assert!(text.contains(std::str::from_utf8(TITLE).expect("title")), "board shows the goal's title: {text}");
        }
        if matches!(self.page, Page::Task) {
            assert!(text.contains("Goal "), "card names its public goal");
            assert!(text.contains(std::str::from_utf8(TITLE).expect("title")), "card shows the goal's title: {text}");
        }
        if matches!(self.page, Page::Landings) {
            // The demo lands by the stock queue (temper/docs/design/client/web/public.md, 4).
            let shown = self.text.join("");
            assert!(Tier::public_landings_reads(&shown), "public landings page: {shown}");
        }
        if matches!(self.page, Page::Private) {
            assert_eq!(
                self.text,
                ["temper", "Sign in", "Not public, or not here.", "Read public projects"],
                "unknown drops the goal and its change"
            );
        }
        if matches!(self.page, Page::Empty | Page::Malformed) {
            assert_eq!(
                self.text,
                ["temper", "Sign in", "Public projects", "No public projects yet."],
                "no public project remains"
            );
        }
    }
    fn advance(&mut self) {
        match self.page {
            Page::Owner => self.published(),
            Page::Projects => self.open(
                Page::Board,
                vec![
                    press(Find::exactly_named(Role::Link, PROJECT)),
                    see(heading(PROJECT)),
                    see(Find::named(Role::Link, b"Goal ")),
                    see(Find::text(TITLE)),
                    Step::Address(self.board.as_bytes().into()),
                ],
            ),
            Page::Board => {
                self.open(Page::Task, vec![press(Find::named(Role::Link, b"Goal ")), see(Find::text(TITLE))]);
            }
            Page::Task => self.open(
                Page::Landings,
                vec![
                    go(format!("{}/landings", self.board).as_bytes()),
                    see(heading(b"Landings")),
                    Step::Address(format!("{}/landings", self.board).into_bytes().into()),
                ],
            ),
            Page::Landings => self.open(
                Page::Private,
                vec![
                    go(self.task.as_bytes()),
                    see(Find::text(TITLE)),
                    visibility(self.client.path(), false),
                    exited(),
                    see(Find::text(UNKNOWN)),
                    see(Find::exactly_named(Role::Link, b"Read public projects")),
                    Step::Address(self.task.as_bytes().into()),
                ],
            ),
            Page::Private => self.unpublished(),
            Page::Empty => self.open(
                Page::Malformed,
                vec![
                    go(b"/public/projects/ABCDEF0123456789"),
                    see(heading(b"Public projects")),
                    Step::Address(b"/public".as_slice().into()),
                ],
            ),
            Page::Malformed => self.page = Page::Done,
            Page::Done => {}
        }
    }
    fn unpublished(&mut self) {
        let answer = fs::read_to_string(self.client.path().join("private.txt")).expect("ctl private answer");
        assert_eq!(answer, "project 1: private\n");
        self.open(
            Page::Empty,
            vec![
                press(Find::exactly_named(Role::Link, b"Read public projects")),
                see(heading(b"Public projects")),
                Step::Person(PersonStep::Gone { find: Find::exactly_named(Role::Link, PROJECT), within: WITHIN }),
                Step::Address(b"/public".as_slice().into()),
            ],
        );
    }
}
fn identifier(id: &str) {
    assert_eq!(id.len(), ID_DIGITS, "public identifier width");
    assert!(
        id.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "lowercase public identifier"
    );
}

impl Play for Walk {
    /// Revoked public reads answer 404 (jig/docs/design/protocol/client/default-http.md, 5).
    fn expected(&self) -> Vec<Box<[u8]>> {
        vec![
            Box::from(b"Failed to load resource: the server responded with a status of 404 (Not Found)".as_slice()),
            Box::from(b"Failed to load resource: the server responded with a status of 401 (Unauthorized)".as_slice()),
        ]
    }
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        let started = *self.started.get_or_insert(now);
        if matches!(self.page, Page::Done) {
            return Next::Done;
        }
        if !self.reading {
            match self.script.next(face, now) {
                Next::Done => {
                    eprintln!(
                        "{:?} complete at {} ms",
                        self.page,
                        now.saturating_since(started).as_nanos() / 1_000_000
                    );
                    if matches!(self.page, Page::Owner) {
                        self.advance();
                        return Next::Wait { until: None };
                    }
                    self.reading = true;
                    face.address();
                    self.address_pending = true;
                }
                next @ (Next::Do(_) | Next::Wait { .. } | Next::Failed { .. }) => return next,
            }
        }
        if self.address_pending {
            let Some(address) = face.take_address() else {
                return Next::Wait { until: None };
            };
            let address = std::str::from_utf8(&address).expect("page address");
            let path = address.split_once("://").expect("absolute address").1;
            let path = path.find('/').and_then(|at| path.get(at..)).expect("page path");
            if matches!(self.page, Page::Task) {
                let prefix = format!("{}/tasks/", self.board);
                identifier(path.strip_prefix(&prefix).expect("goal's public task address"));
                self.task = path.into();
            }
            self.address_pending = false;
        }
        match face.find(&Find::text(b"").nth(self.text_at)) {
            Found::Pending => {}
            Found::Node(node) => {
                let seen = face.seen(node).expect("text node");
                self.text.push(String::from_utf8_lossy(&seen.name).into_owned());
                self.text_at = self.text_at.checked_add(1).expect("bounded page text");
            }
            Found::Absent => {
                assert!(!self.text.is_empty(), "read each public page's text");
                self.check_page(&format!("{:?}", self.page));
                self.advance();
            }
        }
        Next::Wait { until: None }
    }
    fn at(&self) -> u32 {
        self.script.at()
    }
    fn check_deadline(&mut self, face: &mut BrowserFace<'_>, now: Time) {
        self.script.check_deadline(face, now);
    }
}

const LONG_TITLE: &str =
    "Export widget's table as CSV with visible columns, localized headings and a readable filename for monthly reports";

struct TextBoxes {
    boxes: Vec<(String, skein_browser::boundary::Rect)>,
}
impl TextBoxes {
    fn new() -> Self {
        Self { boxes: vec![] }
    }
    fn next(&mut self, face: &mut BrowserFace<'_>) -> bool {
        let find = Find::text(b"").within(Box::new([Target::Role { role: Role::Main, name: Box::new([]) }]));
        match face.find_boxed(&find) {
            Found::Pending => false,
            Found::Absent => true,
            Found::Node(_) => {
                let (matches, more) = face.boxed_matches();
                assert_eq!(more, 0, "public text fits the browser's match limit");
                for seen in matches {
                    let name = String::from_utf8_lossy(&seen.name).into_owned();
                    let rect = seen.rect.unwrap_or_else(|| panic!("visible text {name:?} has no box"));
                    self.boxes.push((name, rect));
                }
                true
            }
        }
    }
    fn check(&self, width: u32, task: bool) {
        let text = self.boxes.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join("\n");
        assert!(text.contains(LONG_TITLE), "long title stays readable: {text}");
        for field in ["Goal ", "task ", "for ", " spent"] {
            assert!(text.contains(field), "populated public field {field}: {text}");
        }
        assert!(!text.contains("priority "), "priority is absent: {text}");
        let executors = self
            .boxes
            .iter()
            .filter(|(name, _)| ["an agent", "a procedure", "a person", "a job"].contains(&name.as_str()))
            .count();
        assert_eq!(executors, usize::from(task), "executor appears on the task page own row alone: {text}");
        assert!(
            ["waiting", "running", "active", "preparing", "done", "held", "ended"]
                .iter()
                .any(|phase| text.contains(phase)),
            "phase is populated: {text}"
        );
        for (at, (name, rect)) in self.boxes.iter().enumerate() {
            assert!(rect.width > 0 && rect.height > 0, "visible {name}: {rect:?}");
            assert!(rect.x >= 16, "text and navigation keep a gutter: {name}: {rect:?}");
            let right = rect.x + i64::try_from(rect.width).expect("text width");
            assert!(right <= i64::from(width) - 16, "no horizontal overflow: {name}: {rect:?}");
            if name.contains("Export widget") {
                assert!(rect.width >= 200, "title keeps a readable column: {rect:?}");
            }
            for (other, second) in self.boxes.iter().skip(at + 1) {
                let overlaps = rect.x < second.x + i64::try_from(second.width).expect("width")
                    && second.x < right
                    && rect.y < second.y + i64::try_from(second.height).expect("height")
                    && second.y < rect.y + i64::try_from(rect.height).expect("height");
                assert!(!overlaps, "text overlaps at {width}px: {name} {rect:?}, {other} {second:?}");
            }
        }
    }
}
struct Readable {
    walk: Walk,
    stage: u32,
    boxes: TextBoxes,
    measuring: bool,
}
impl Readable {
    fn width(&self) -> u32 {
        if self.stage == 2 || self.stage == 4 { 1440 } else { 420 }
    }
    fn advance(&mut self) {
        self.boxes = TextBoxes::new();
        self.measuring = false;
        self.stage += 1;
        if self.stage == 6 {
            self.walk.open(Page::Projects, vec![go(b"/public"), see(heading(b"Public projects"))]);
            return;
        }
        let mut steps = vec![Step::Viewport { width: self.width(), height: 900 }];
        match self.stage {
            2 => steps.extend([
                go(self.walk.board.as_bytes()),
                see(heading(PROJECT)),
                see(Find::text(LONG_TITLE.as_bytes())),
                see(Find::named(Role::Link, b"task ")),
            ]),
            4 => steps.extend([
                press(Find::named(Role::Link, b"Goal ")),
                see(Find::text(LONG_TITLE.as_bytes())),
                see(Find::named(Role::Link, b"task ")),
            ]),
            3 | 5 => {}
            _ => panic!("bounded public layout walk"),
        }
        self.walk.script = Script::new(steps.into_boxed_slice());
    }
}
impl Play for Readable {
    fn expected(&self) -> Vec<Box<[u8]>> {
        self.walk.expected()
    }
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        if self.stage == 6 {
            return self.walk.next(face, now);
        }
        if self.measuring {
            if self.boxes.next(face) {
                self.boxes.check(self.width(), self.stage >= 4);
                self.advance();
            }
            return Next::Wait { until: None };
        }
        match self.walk.script.next(face, now) {
            Next::Done if self.stage == 0 => {
                self.walk.published();
                self.stage = 1;
            }
            Next::Done if self.stage == 1 => self.advance(),
            Next::Done => {
                eprintln!("measure public text: stage={} width={} task={}", self.stage, self.width(), self.stage >= 4);
                self.measuring = true;
            }
            next => return next,
        }
        Next::Wait { until: None }
    }
    fn at(&self) -> u32 {
        self.walk.script.at()
    }
    fn check_deadline(&mut self, face: &mut BrowserFace<'_>, now: Time) {
        self.walk.script.check_deadline(face, now);
    }
}

#[test]
fn public_board_and_task_lines_are_readable_at_desktop_and_phone_widths() {
    let tier = Tier::public_demo_title("public-readable-lines", LONG_TITLE);
    let mut reader = Readable { walk: Walk::new(&tier), stage: 0, boxes: TextBoxes::new(), measuring: false };
    let mut ctl = Script::new(Box::new([sign_in(reader.walk.client.path()), exited(), open_printed(), exited()]));
    let mut setup = tier.setup();
    setup.printed.sent = Sent::Command;
    assert_eq!(temper_client_browser_world::harness(setup).run(&mut [&mut ctl, &mut reader]), Outcome::Done);
}

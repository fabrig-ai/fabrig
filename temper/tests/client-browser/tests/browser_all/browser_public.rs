//! Geometry of a public board and its goal in Chromium (temper/docs/design/client/web/public.md, 4).
use skein_fake_person::{Find, Found, Next, Step as PersonStep, Target};
use skein_fake_person_browser::{BrowserFace, Outcome, Play, Script, Step};
use skein_lib::Time;
use skein_ui::Role;
use std::{fs, path::Path};
use temper_client_browser_world::{Tier, WITHIN, harness, page_address};

const CUT_TITLE: &str = "Export widget's table as CSV with visible columns, localized headings and a…";
const DESKTOP: u32 = 1440;
const PHONE: u32 = 420;
const HEIGHT: u32 = 900;

fn see(find: Find) -> Step {
    Step::Person(PersonStep::See { find, within: WITHIN })
}

fn public_id(id: &str) {
    assert_eq!(id.len(), 16, "public identifier width: {id:?}");
    assert!(id.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)), "lowercase hex: {id:?}");
}

fn printed_board(answer: &Path) -> String {
    let answer = fs::read_to_string(answer).expect("ctl's project amend answer");
    let board = answer.lines().find_map(|line| line.strip_prefix("page: ")).expect("ctl printed the board path");
    public_id(board.strip_prefix("/public/projects/").expect("public board path"));
    board.into()
}

#[derive(Clone, Copy, Debug)]
enum Stage {
    Seed,
    BoardDesktop,
    BoardPhone,
    GoalDesktop,
    GoalPhone,
}

struct Readable<'a> {
    answer: &'a Path,
    board: String,
    stage: Stage,
    script: Script,
    address_pending: bool,
}

impl Readable<'_> {
    fn advance(&mut self, stage: Stage, width: u32, opening: Option<Step>) {
        self.stage = stage;
        let mut steps = vec![Step::Viewport { width, height: HEIGHT }];
        if let Some(opening) = opening {
            steps.push(opening);
        }
        let heading = Role::Heading { level: 1 };
        let ready = match stage {
            Stage::BoardDesktop | Stage::BoardPhone => Find::exactly_named(heading, b"Demo"),
            Stage::GoalDesktop | Stage::GoalPhone => Find::named(heading, b"Goal "),
            Stage::Seed => unreachable!("ctl's script comes before the first page"),
        };
        steps.extend([see(ready), see(Find::text(CUT_TITLE.as_bytes())), see(Find::named(Role::Link, b"task "))]);
        if matches!(stage, Stage::GoalDesktop | Stage::GoalPhone) {
            // The change record arrives separately from the card; measure only after both draw.
            steps.push(see(Find::text(b"CI at its head")));
        }
        self.script = Script::new(steps.into_boxed_slice());
    }

    fn check_address(&mut self, face: &mut BrowserFace<'_>) -> bool {
        let Some(address) = face.take_address() else { return false };
        let address = std::str::from_utf8(&address).expect("page URL");
        let (_, origin_and_path) = address.split_once("://").expect("absolute page URL");
        let path = &origin_and_path[origin_and_path.find('/').expect("goal page path")..];
        let prefix = format!("{}/tasks/", self.board);
        public_id(path.strip_prefix(&prefix).expect("goal's task path under the board"));
        self.address_pending = false;
        true
    }
}

impl Play for Readable<'_> {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        if self.address_pending && !self.check_address(face) {
            return Next::Wait { until: None };
        }
        match self.script.next(face, now) {
            Next::Done => {}
            next => return next,
        }
        match self.stage {
            Stage::Seed => {
                self.board = printed_board(self.answer);
                let go = Step::Person(PersonStep::Go { address: page_address(self.board.as_bytes()) });
                self.advance(Stage::BoardDesktop, DESKTOP, Some(go));
            }
            Stage::BoardDesktop if measured(face, self.stage, DESKTOP) => self.advance(Stage::BoardPhone, PHONE, None),
            Stage::BoardPhone if measured(face, self.stage, PHONE) => {
                let goal = Step::Person(PersonStep::Press { find: Find::named(Role::Link, b"Goal ") });
                self.advance(Stage::GoalDesktop, DESKTOP, Some(goal));
            }
            Stage::GoalDesktop if measured(face, self.stage, DESKTOP) => {
                face.address();
                self.address_pending = true;
                self.advance(Stage::GoalPhone, PHONE, None);
            }
            Stage::GoalPhone if measured(face, self.stage, PHONE) => return Next::Done,
            Stage::BoardDesktop | Stage::BoardPhone | Stage::GoalDesktop | Stage::GoalPhone => {}
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

fn measured(face: &mut BrowserFace<'_>, stage: Stage, width: u32) -> bool {
    let find = Find::text(b"").within(Box::new([Target::Role { role: Role::Main, name: Box::new([]) }]));
    match face.find_boxed(&find) {
        Found::Pending => return false,
        Found::Absent => panic!("no text to measure on {stage:?} at {width}px"),
        Found::Node(_) => {}
    }
    let (matches, more) = face.boxed_matches();
    assert_eq!(more, 0, "{stage:?} at {width}px fits the browser's text match limit");
    let boxes: Vec<_> = matches
        .iter()
        .map(|seen| {
            let name = String::from_utf8_lossy(&seen.name);
            let rect = seen.rect.unwrap_or_else(|| panic!("{stage:?} at {width}px: visible text {name:?} has no box"));
            (name, rect)
        })
        .collect();
    let mut title = false;
    for (at, (name, rect)) in boxes.iter().enumerate() {
        assert!(rect.width > 0 && rect.height > 0, "{stage:?}: visible {name}: {rect:?}");
        assert!(rect.x >= 16, "{stage:?}: text and navigation keep a gutter: {name}: {rect:?}");
        let right = rect.x + i64::try_from(rect.width).expect("text width");
        assert!(right <= i64::from(width) - 16, "{stage:?}: no horizontal overflow: {name}: {rect:?}");
        if name.contains("Export widget") {
            title = true;
            assert!(rect.width >= 200, "{stage:?}: title keeps a readable column: {name}: {rect:?}");
        }
        for (other, second) in boxes.iter().skip(at + 1) {
            let overlaps = rect.x < second.x + i64::try_from(second.width).expect("width")
                && second.x < right
                && rect.y < second.y + i64::try_from(second.height).expect("height")
                && second.y < rect.y + i64::try_from(rect.height).expect("height");
            assert!(!overlaps, "{stage:?}: text overlaps at {width}px: {name} {rect:?}, {other} {second:?}");
        }
    }
    assert!(title, "{stage:?} at {width}px has a measured title box");
    true
}

#[test]
fn public_board_and_task_lines_are_readable_at_desktop_and_phone_widths() {
    let tier = Tier::public_demo("public-readable-lines");
    let _model = tier.scripted_model();
    let capture = skein_scratch::Scratch::new("public-board-answer");
    let answer = capture.path().join("publish.txt");
    let script = tier.ctl_public_board(&answer);
    let mut reader =
        Readable { answer: &answer, board: String::new(), stage: Stage::Seed, script, address_pending: false };
    assert_eq!(harness(tier.held_setup()).run(&mut [&mut reader]), Outcome::Done);
}

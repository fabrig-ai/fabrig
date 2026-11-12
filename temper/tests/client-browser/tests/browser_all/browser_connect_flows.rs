//! Scrolling the repository sheet and measuring its phone geometry
//! (temper/docs/design/client/forge/family.md, 4; temper/docs/design/client/forge/repositories.md, 1).
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Command, Harness, Lent, Opens, Script, Sent, Stdin, Step, harness::Output};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, Stories, Tier, WITHIN};

// Thirty rows put the foot beyond the browser's near-end zone before a person scrolls.
const REPOSITORIES: u32 = 45;
const PAGE: u32 = 30;
const PHONE_HEIGHT: u32 = 800;
const REPOSITORIES_ADDRESS: &[u8] = b"/projects/1/settings/repositories";

#[test]
fn scrolling_the_sheet_reads_more_then_search_picks_and_adopts() {
    let (tier, mut github) = Tier::github_repositories("sheet-scroll", REPOSITORIES, PAGE);
    let mut setup = tier.setup();
    setup.printed.sent = Sent::Command;
    let mut stories = Stories::of_person(1);
    let mut steps = vec![Step::Viewport { width: 1280, height: 900 }];
    stories.begin("scroll, search and adopt", &steps);
    steps.extend(open_sheet());
    steps.extend([
        see(Find::text("30 so far · more as you scroll".as_bytes())),
        Step::KeepBox {
            find: Find::exactly_named(Role::List, b"Repositories"),
            name: b"list-before".as_slice().into(),
        },
        // Pressing the last loaded row scrolls it into view; clear its pick before reading the count.
        press(Role::Check, b"acme/repo-30"),
        press(Role::Check, b"acme/repo-30"),
        see(Find::text(b"45 repositories")),
        Step::KeepBox { find: Find::exactly_named(Role::List, b"Repositories"), name: b"list-after".as_slice().into() },
        input(b"Find by name", b"repo-45"),
        see(Find::text(b"1 of 45 repositories")),
        press(Role::Check, b"acme/repo-45"),
        see(Find::text("1 of 45 repositories · 1 picked".as_bytes())),
        Step::Person(PersonStep::Press { find: Find::named(Role::Disclosure, b"More") }),
        input(b"Landing branch", b"main"),
        press(Role::Button, b"Adopt acme/repo-45"),
        gone(Find::exactly_named(Role::Dialog, b"Add repositories to Project 1")),
        Step::Address(REPOSITORIES_ADDRESS.into()),
        see(Find::named(Role::Link, b"acme/repo-45")),
    ]);
    stories.begin("phone geometry", &steps);
    steps.extend(phone_steps());
    let mut script = Script::new(steps.into_boxed_slice());
    let mut connect = connect_github(&tier);
    let mut harness = temper_client_browser_world::harness(setup);
    let outcome =
        harness.run_lending(&mut [&mut connect, &mut script], &mut [Lent { name: b"github", peer: &mut github }]);
    stories.assert_done(outcome);
    assert_eq!(
        box_of(&harness, b"list-before"),
        box_of(&harness, b"list-after"),
        "scroll: reading on keeps the list's bounds"
    );
    phone_measures(&harness);
}

fn connect_github(tier: &Tier) -> Script {
    let mut steps = tier.github_connect();
    steps.extend([
        see(Find::exactly_named(Role::Link, b"Create GitHub App")),
        press(Role::Link, b"Create GitHub App"),
        see(Find::exactly_named(Role::Heading { level: 2 }, b"GitHub is connected")),
        // Opening ctl's spent link asks the server to print the browser's fresh one.
        open_printed(),
        Step::Exited { code: 0, within: WITHIN },
    ]);
    Script::new(steps.into_boxed_slice())
}
fn open_sheet() -> Vec<Step> {
    let mut steps = vec![
        open_printed(),
        Step::Exited { code: 0, within: WITHIN },
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model")),
    ];
    steps.extend(show_sheet());
    steps
}
fn see(find: Find) -> Step {
    Step::Person(PersonStep::See { find, within: WITHIN })
}
fn gone(find: Find) -> Step {
    Step::Person(PersonStep::Gone { find, within: ABSENT })
}
fn press(role: Role, name: &[u8]) -> Step {
    Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) })
}
fn input(name: &[u8], words: &[u8]) -> Step {
    Step::Person(PersonStep::Type { find: Find::exactly_named(Role::TextField, name), words: words.into() })
}
fn go(address: &[u8]) -> Step {
    Step::Person(PersonStep::Go { address: temper_client_browser_world::page_address(address) })
}
fn box_of(harness: &Harness, name: &[u8]) -> (i64, i64) {
    let kept = harness.kept().iter().find(|kept| kept.name.as_ref() == name).expect("kept box");
    let value = std::str::from_utf8(&kept.value).expect("box coordinates");
    let (top, height) = value.split_once(' ').expect("top and height");
    (top.parse().expect("signed top"), height.parse().expect("height"))
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
            keep: b"sheet-reader".as_slice().into(),
        }),
        within: WITHIN,
    })
}

fn show_sheet() -> Vec<Step> {
    vec![
        go(REPOSITORIES_ADDRESS),
        see(Find::exactly_named(Role::Link, b"Add repositories")),
        press(Role::Link, b"Add repositories"),
        see(Find::exactly_named(Role::Dialog, b"Add repositories to Project 1")),
        see(Find::exactly_named(Role::Check, b"acme/app")),
    ]
}

fn phone_steps() -> Vec<Step> {
    let mut steps = vec![Step::Viewport { width: 420, height: PHONE_HEIGHT }];
    steps.extend(show_sheet());
    steps.extend([
        see(Find::exactly_named(Role::TextField, b"Find by name")),
        press(Role::Check, b"acme/app"),
        Step::KeepBox {
            find: Find::exactly_named(Role::Dialog, b"Add repositories to Project 1"),
            name: b"sheet".as_slice().into(),
        },
        Step::KeepBox { find: Find::exactly_named(Role::Button, b"Adopt acme/app"), name: b"adopt".as_slice().into() },
    ]);
    steps
}

fn phone_measures(harness: &Harness) {
    let (top, height) = box_of(harness, b"sheet");
    let bottom = i64::from(PHONE_HEIGHT);
    assert!(
        top > 0 && (top + height - bottom).abs() <= 1,
        "phone geometry: the sheet rises from the bottom: {top} + {height}"
    );
    let (button_top, button_height) = box_of(harness, b"adopt");
    assert!(
        button_height > 0 && button_top >= top && button_top + button_height <= bottom,
        "phone geometry: the primary button is inside the viewport"
    );
}

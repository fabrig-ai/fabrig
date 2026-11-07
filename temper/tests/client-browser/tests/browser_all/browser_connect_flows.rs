//! Scrolling the repository sheet, its phone geometry, and a model's return to its row
//! (temper/docs/design/client/README.md, 4.4; temper/docs/design/client/forge.md, 5.1).
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Command, Harness, Lent, Opens, Outcome, Script, Sent, Stdin, Step, harness::Output};
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, Tier, WITHIN};

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
    let mut steps = vec![Step::Viewport { width: 1280, height: 900 }];
    steps.extend(open_sheet(&tier));
    steps.extend([
        see(Find::text("30 so far · more as you scroll".as_bytes())),
        // Focus a row without leaving it picked. PageDown scrolls its nearest scrollable list.
        press(Role::Check, b"acme/app"),
        press(Role::Check, b"acme/app"),
        Step::KeepBox {
            find: Find::exactly_named(Role::List, b"Repositories"),
            name: b"list-before".as_slice().into(),
        },
        Step::Key(Key::PageDown),
        Step::Key(Key::PageDown),
        Step::Key(Key::PageDown),
        see(Find::text(b"45 repositories")),
        Step::KeepBox { find: Find::exactly_named(Role::List, b"Repositories"), name: b"list-after".as_slice().into() },
        input(b"Find by name", b"repo-45"),
        see(Find::text(b"1 of 45 repositories")),
        press(Role::Check, b"acme/repo-45"),
        see(Find::text("1 of 45 repositories · 1 picked".as_bytes())),
        press(Role::Button, b"Adopt acme/repo-45"),
        gone(Find::exactly_named(Role::Dialog, b"Add repositories to Project 1")),
        Step::Address(REPOSITORIES_ADDRESS.into()),
        see(Find::exactly_named(Role::Link, b"acme/repo-45")),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut harness = Harness::new(setup);
    assert_eq!(
        harness.run_lending(&mut [&mut script], &mut [Lent { name: b"github", peer: &mut github }]),
        Outcome::Done
    );
    assert_eq!(box_of(&harness, b"list-before"), box_of(&harness, b"list-after"), "reading on keeps the list's bounds");
}

#[test]
fn at_phone_width_the_sheet_meets_the_bottom_and_its_button_is_visible() {
    let (tier, mut github) = Tier::github_repositories("sheet-phone", REPOSITORIES, PAGE);
    let mut setup = tier.setup();
    setup.printed.sent = Sent::Command;
    let mut steps = vec![Step::Viewport { width: 420, height: PHONE_HEIGHT }];
    steps.extend(open_sheet(&tier));
    steps.extend([
        see(Find::exactly_named(Role::TextField, b"Find by name")),
        press(Role::Check, b"acme/app"),
        Step::KeepBox {
            find: Find::exactly_named(Role::Dialog, b"Add repositories to Project 1"),
            name: b"sheet".as_slice().into(),
        },
        Step::KeepBox { find: Find::exactly_named(Role::Button, b"Adopt acme/app"), name: b"adopt".as_slice().into() },
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    let mut harness = Harness::new(setup);
    assert_eq!(
        harness.run_lending(&mut [&mut script], &mut [Lent { name: b"github", peer: &mut github }]),
        Outcome::Done
    );
    let (top, height) = box_of(&harness, b"sheet");
    let bottom = i64::from(PHONE_HEIGHT);
    assert!(top > 0 && (top + height - bottom).abs() <= 1, "the sheet rises from the bottom: {top} + {height}");
    let (button_top, button_height) = box_of(&harness, b"adopt");
    assert!(
        button_height > 0 && button_top >= top && button_top + button_height <= bottom,
        "the primary button is inside the viewport"
    );
}

#[test]
fn connecting_a_model_returns_to_accounts_with_its_line() {
    let tier = Tier::accounts("connect-model-row");
    let mut script = Script::new(Box::new([
        see(Find::exactly_named(Role::Link, b"Set up temper")),
        go(b"/accounts"),
        see(Find::text(b"Your accounts")),
        press(Role::Link, b"+ Connect a model"),
        see(Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model")),
        press(Role::Link, b"Add key"),
        input(b"Key (pasted once)", b"private-browser-key"),
        press(Role::Button, b"Add key"),
        see(Find::text(b"Added your Test provider API key")),
        Step::Address(b"/accounts".as_slice().into()),
        see(Find::named(Role::Disclosure, b"Test provider API key")),
        gone(Find::exactly_named(Role::Heading { level: 1 }, b"A Test provider API key")),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

fn open_sheet(tier: &Tier) -> Vec<Step> {
    let mut steps = tier.github_connect();
    steps.extend([
        see(Find::exactly_named(Role::Link, b"Create GitHub App")),
        press(Role::Link, b"Create GitHub App"),
        see(Find::exactly_named(Role::Heading { level: 2 }, b"GitHub is connected")),
        // ctl consumed the first link; sign the browser in with the server's fresh one.
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
        }),
        Step::Exited { code: 0, within: WITHIN },
        see(Find::exactly_named(Role::Link, b"Set up temper")),
        go(REPOSITORIES_ADDRESS),
        see(Find::exactly_named(Role::Link, b"Add repositories")),
        press(Role::Link, b"Add repositories"),
        see(Find::exactly_named(Role::Dialog, b"Add repositories to Project 1")),
        see(Find::exactly_named(Role::Check, b"acme/app")),
    ]);
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
    Step::Person(PersonStep::Go { address: address.into() })
}
fn box_of(harness: &Harness, name: &[u8]) -> (i64, i64) {
    let kept = harness.kept().iter().find(|kept| kept.name.as_ref() == name).expect("kept box");
    let value = std::str::from_utf8(&kept.value).expect("box coordinates");
    let (top, height) = value.split_once(' ').expect("top and height");
    (top.parse().expect("signed top"), height.parse().expect("height"))
}

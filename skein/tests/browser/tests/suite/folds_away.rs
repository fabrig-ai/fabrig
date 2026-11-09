//! Panel folding through the page part in real Chromium (skein/docs/design/ui-web.md, 7).
use skein_browser::boundary::Key;
use skein_fake_person_browser::{
    Outcome, Script, Setup, harness,
    person::{Find, Step},
};
use skein_lib::Duration;
use skein_ui::Role;

fn press(name: &[u8]) -> harness::Step {
    harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, name) })
}
fn disclosure(name: &[u8]) -> harness::Step {
    harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Disclosure, name) })
}
fn see(name: &[u8]) -> harness::Step {
    harness::Step::Person(Step::See { find: Find::text(name), within: Duration::from_secs(2) })
}
fn check(selector: &[u8]) -> harness::Step {
    harness::Step::Keep {
        selector: selector.into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: selector.into(),
    }
}
/// A go ends at the page's load, and the loader starts the module after it
/// (skein/docs/design/ui-web.md, 4.3 and 14.2), so each story sees the
/// fixture's first apply before it acts or reads.
fn arrive(steps: impl IntoIterator<Item = harness::Step>) -> Script {
    let arrived = [
        harness::Step::Person(Step::Go { address: b"/test/folds-away".as_slice().into() }),
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Disclosure, b"Editor"),
            within: Duration::from_secs(2),
        }),
    ];
    Script::new(arrived.into_iter().chain(steps).collect())
}
#[expect(
    clippy::disallowed_methods,
    reason = "skein's round: reads SKEIN_UI_EXAMPLE_BUNDLE to locate the browser test bundle"
)]
pub(super) fn setup() -> Setup<'static> {
    let exe = std::env::current_exe().expect("test executable");
    let server = exe.parent().and_then(std::path::Path::parent).expect("target debug").join("skein-ui-example");
    assert!(server.is_file(), "prebuild skein-ui-example-server");
    let bundle = std::env::var_os("SKEIN_UI_EXAMPLE_BUNDLE").expect("build and supply the example bundle");
    Setup {
        server,
        args: vec!["127.0.0.1:0".into(), "--bundle".into(), bundle],
        printed: harness::Printed {
            sent: harness::Sent::Page,
            prefix: b"skein-ui-example: listening at ",
            on: harness::Output::Stderr,
            names: harness::Names::Address,
        },
        restart: vec![],
        downloads: None,
        chromium: super::real::chromium(),
        origins: vec![],
        expected: vec![],
    }
}

#[test]
fn removing_a_focused_form_returns_to_its_disclosures_summary() {
    let mut person = arrive([
        harness::Step::Focused(Find::exactly_named(Role::Disclosure, b"Editor")),
        check(b"details:not([tabindex]):has(> summary:focus:not([tabindex]))"),
        disclosure(b"Editor"),
        harness::Step::Type {
            find: Find::exactly_named(Role::TextField, b"Name"),
            words: b"Changed".as_slice().into(),
        },
        harness::Step::Key(Key::Escape),
        harness::Step::Focused(Find::exactly_named(Role::Disclosure, b"Editor")),
        check(b"main:not(:focus)"),
        check(b"details:not([tabindex]):has(> summary:focus:not([tabindex])):not(:has(form))"),
    ]);
    assert_eq!(harness::Harness::new(setup()).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn a_panel_folds_away_without_taking_the_outside_press_or_a_forms_escape() {
    let mut person = arrive([
        disclosure(b"Panel"),
        check(b"details[data-folds-away][open]"),
        press(b"Outside"),
        see(b"Pressed 4"),
        check(b"details[data-folds-away]:not([open])"),
        disclosure(b"Panel"),
        press(b"Inside panel"),
        see(b"Pressed 2"),
        check(b"details[data-folds-away][open]"),
        press(b"Actions"),
        harness::Step::Key(Key::Escape),
        check(b"[role=menu][hidden]"),
        check(b"details[data-folds-away][open]"),
        harness::Step::Key(Key::Escape),
        check(b"details[data-folds-away]:not([open])"),
        harness::Step::Focused(Find::exactly_named(Role::Disclosure, b"Panel")),
        see(b"Pressed 2"),
        harness::Step::Key(Key::Escape),
        see(b"Pressed 1"),
        disclosure(b"Panel"),
        press(b"Component outside"),
        check(b"details[data-folds-away]:not([open])"),
        harness::Step::Keep {
            selector: b"[data-component][data-pressed=yes]".as_slice().into(),
            attribute: Some(b"data-pressed".as_slice().into()),
            name: b"component".as_slice().into(),
        },
        disclosure(b"Plain"),
        disclosure(b"Call"),
        press(b"Outside"),
        check(b"details:not([data-folds-away])[open]:has(> summary:first-child)"),
        press(b"Inside plain"),
        harness::Step::Key(Key::Escape),
        check(b"details:not([data-folds-away])[open]:nth-of-type(2)"),
        check(b"details:not([data-folds-away])[open]:nth-of-type(3)"),
        press(b"Inside call"),
        see(b"Pressed 2"),
        disclosure(b"Panel"),
        harness::Step::KeyWithShift(Key::Tab),
        harness::Step::Focused(Find::exactly_named(Role::Button, b"Form escape")),
        harness::Step::Key(Key::Escape),
        check(b"details[data-folds-away]:not([open])"),
        harness::Step::Focused(Find::exactly_named(Role::Button, b"Form escape")),
        see(b"Pressed 2"),
    ]);
    assert_eq!(harness::Harness::new(setup()).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn a_press_outside_closes_a_menu_and_goes_on() {
    let mut person = arrive([
        disclosure(b"Panel"),
        press(b"Actions"),
        check(b"[role=menu]:not([hidden])"),
        press(b"Inside panel"),
        see(b"Pressed 2"),
        check(b"[role=menu][hidden]"),
        check(b"details[data-folds-away][open]"),
        harness::Step::Focused(Find::exactly_named(Role::Button, b"Inside panel")),
        press(b"Actions"),
        press(b"Outside"),
        see(b"Pressed 4"),
        check(b"[role=menu][hidden]"),
        check(b"details[data-folds-away]:not([open])"),
        harness::Step::Focused(Find::exactly_named(Role::Button, b"Outside")),
        disclosure(b"Panel"),
        press(b"Actions"),
        press(b"Component outside"),
        check(b"[role=menu][hidden]"),
        harness::Step::Keep {
            selector: b"[data-component][data-pressed=yes]".as_slice().into(),
            attribute: Some(b"data-pressed".as_slice().into()),
            name: b"component".as_slice().into(),
        },
        harness::Step::Focused(Find::exactly_named(Role::Button, b"Component outside")),
        disclosure(b"Panel"),
        press(b"Actions"),
        disclosure(b"Plain"),
        check(b"[role=menu][hidden]"),
        check(b"details:not([data-folds-away])[open]:nth-of-type(2)"),
        harness::Step::Focused(Find::exactly_named(Role::Disclosure, b"Plain")),
    ]);
    assert_eq!(harness::Harness::new(setup()).run(&mut [&mut person]), Outcome::Done);
}

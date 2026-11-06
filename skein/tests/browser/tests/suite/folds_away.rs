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
fn a_panel_folds_away_without_taking_the_outside_press_or_a_forms_escape() {
    let mut person = Script::new(Box::new([
        harness::Step::Person(Step::Go { address: b"/test/folds-away".as_slice().into() }),
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
    ]));
    assert_eq!(harness::Harness::new(setup()).run(&mut [&mut person]), Outcome::Done);
}

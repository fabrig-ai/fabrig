//! A log resumes following through the page part in Chromium (skein/docs/design/ui/web/README.md, 5).
use skein_fake_person_browser::{
    Outcome, Script, harness,
    person::{Find, Step},
};
use skein_lib::Duration;
use skein_ui::Role;

fn press(name: &[u8]) -> harness::Step {
    harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, name) })
}
fn see(words: &[u8]) -> harness::Step {
    harness::Step::Person(Step::See { find: Find::text(words), within: Duration::from_secs(2) })
}
fn check(selector: &[u8]) -> harness::Step {
    harness::Step::Keep {
        selector: selector.into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: selector.into(),
    }
}
#[test]
fn a_log_scrolled_away_returns_to_its_end_and_keeps_following() {
    let mut person = Script::new(Box::new([
        harness::Step::Person(Step::Go { address: b"/test/log-end".as_slice().into() }),
        see(b"Scroll away"),
        press(b"Scroll away"),
        see(b"Away"),
        check(b"[role=log][data-log-away]"),
        press(b"Latest"),
        see(b"At end"),
        check(b"[role=log]:not([data-log-away])"),
        press(b"Add line"),
        see(b"New line followed"),
        check(b"[role=log]:not([data-log-away])"),
    ]));
    assert_eq!(harness::Harness::new(super::folds_away::setup()).run(&mut [&mut person]), Outcome::Done);
}

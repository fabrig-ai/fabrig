//! Measured reading through the page part in Chromium (skein/docs/design/ui-web.md, 7).
use skein_browser::boundary::Key;
use skein_fake_person_browser::{
    Outcome, Script, Setup, harness,
    person::{Find, Step},
};
use skein_ui::Role;

fn setup() -> Setup<'static> {
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

fn press(name: &[u8]) -> harness::Step {
    harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, name) })
}
fn go(path: &[u8]) -> harness::Step {
    harness::Step::Person(Step::Go { address: path.into() })
}
fn keep(name: &str, metric: &str) -> harness::Step {
    harness::Step::Keep {
        selector: b"[role=log]".as_slice().into(),
        attribute: Some(format!("data-{metric}").into_bytes().into()),
        name: name.as_bytes().into(),
    }
}
fn snapshot(steps: &mut Vec<harness::Step>, name: &str, metric: &str) {
    steps.push(press(b"Inspect"));
    steps.push(keep(name, metric));
}
fn run(mut steps: Vec<harness::Step>) -> harness::Harness {
    steps.insert(
        1,
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Button, b"Inspect"),
            within: skein_lib::Duration::from_secs(2),
        }),
    );
    let mut person = Script::new(steps.into_boxed_slice());
    let mut harness = harness::Harness::new(setup());
    assert_eq!(harness.run(&mut [&mut person]), Outcome::Done);
    harness
}
fn value<'a>(harness: &'a harness::Harness, name: &str) -> &'a str {
    let kept = harness.kept().iter().find(|kept| kept.name.as_ref() == name.as_bytes()).expect("kept metric");
    std::str::from_utf8(&kept.value).expect("fixture metric")
}
fn number(harness: &harness::Harness, name: &str) -> i32 {
    value(harness, name).parse().expect("numeric fixture metric")
}
fn held(harness: &harness::Harness, before: &str, after: &str) {
    let before: f64 = value(harness, before).parse().expect("pixel position");
    let after: f64 = value(harness, after).parse().expect("pixel position");
    let distance = (before - after).abs();
    assert!(distance <= 1.0, "{before} -> {after}: moved {distance}px");
}

#[test]
fn a_short_log_reads_each_page_once_until_there_is_no_more() {
    let harness = run(vec![
        go(b"/test/reading/short"),
        press(b"Inspect"),
        keep("reads", "reads"),
        keep("empty-before", "empty-before"),
        keep("empty-after", "empty-after"),
        keep("empty-was-busy", "empty-was-busy"),
        keep("height", "height"),
        keep("viewport", "viewport"),
        keep("top", "top"),
        press(b"Inspect"),
        keep("settled", "reads"),
    ]);
    assert_eq!(value(&harness, "reads"), "4", "empty page is followed by pages that draw");
    assert_eq!(value(&harness, "settled"), "4");
    assert_eq!(value(&harness, "empty-was-busy"), "true");
    assert_eq!(number(&harness, "empty-before"), 40, "two opening lines");
    assert_eq!(number(&harness, "empty-after"), number(&harness, "empty-before"), "empty page moves no height");
    assert_eq!(number(&harness, "top"), 0);
    assert_eq!(number(&harness, "height"), 120, "six whole lines");
    assert_eq!(number(&harness, "height"), number(&harness, "viewport"));
}

fn positions(path: &[u8]) -> Vec<harness::Step> {
    vec![
        go(path),
        press(b"Inspect"),
        keep("opening-top", "top"),
        keep("opening-height", "height"),
        keep("viewport", "viewport"),
        keep("reads", "reads"),
        press(b"Read back"),
        press(b"Remember line"),
        keep("before", "position"),
        press(b"Prepend"),
        keep("inserted", "position"),
        press(b"Remove head"),
        keep("removed", "position"),
        press(b"Append"),
        keep("appended", "position"),
    ]
}
fn check_positions(harness: &harness::Harness) {
    let top = number(harness, "opening-top");
    let viewport = number(harness, "viewport");
    assert!(top * 2 >= viewport * 3, "opening head is {top}px offscreen");
    assert_eq!(top + viewport, number(harness, "opening-height"), "opening foot is visible");
    assert_eq!(value(harness, "reads"), "3", "reading stops beyond the near zone");
    for after in ["inserted", "removed", "appended"] {
        held(harness, "before", after);
    }
}

#[test]
fn the_view_stays_with_the_browsers_own_anchoring() {
    check_positions(&run(positions(b"/test/reading/anchored")));
}

#[test]
fn reading_without_browser_anchoring_follows_requests_and_page_keys() {
    let mut steps = positions(b"/test/reading/none");
    steps.extend([
        press(b"Continues"),
        keep("continues-away", "away"),
        keep("foot-near", "feet"),
        keep("continues-top", "top"),
        press(b"Append"),
        keep("continues-append", "top"),
        press(b"Read back"),
        press(b"End"),
        keep("end-top", "top"),
        keep("end-height", "height"),
        keep("end-away", "away"),
        press(b"Read back"),
        harness::Step::Type {
            find: Find::exactly_named(Role::TextArea, b"Message"),
            words: b"Reading".as_slice().into(),
        },
        keep("field-focus", "focus"),
        harness::Step::Key(Key::PageUp),
    ]);
    snapshot(&mut steps, "field-up", "top");
    steps.extend([
        harness::Step::Type {
            find: Find::exactly_named(Role::TextArea, b"Message"),
            words: b"Reading".as_slice().into(),
        },
        harness::Step::Key(Key::PageDown),
    ]);
    snapshot(&mut steps, "field-down", "top");
    steps.extend([press(b"Body focus"), keep("body-focus", "focus"), harness::Step::Key(Key::PageDown)]);
    snapshot(&mut steps, "body-down", "top");
    snapshot(&mut steps, "heads-before", "heads");
    steps.extend([
        press(b"Body focus"),
        harness::Step::Key(Key::PageUp),
        harness::Step::Key(Key::PageUp),
        harness::Step::Key(Key::PageUp),
    ]);
    snapshot(&mut steps, "body-up", "top");
    snapshot(&mut steps, "heads-after", "heads");
    steps.extend([
        press(b"Read back"),
        press(b"Open dialog"),
        harness::Step::Key(Key::PageUp),
        press(b"Inspect dialog"),
        keep("dialog-up", "top"),
        harness::Step::Key(Key::PageDown),
        press(b"Inspect dialog"),
        keep("dialog-down", "top"),
        press(b"Close dialog"),
    ]);
    let harness = run(steps);
    check_positions(&harness);
    assert_eq!(value(&harness, "continues-away"), "true");
    assert!(number(&harness, "foot-near") >= 1);
    held(&harness, "continues-top", "continues-append");
    assert_eq!(number(&harness, "end-top") + number(&harness, "viewport"), number(&harness, "end-height"));
    assert_eq!(value(&harness, "end-away"), "false");
    assert_eq!(value(&harness, "field-focus"), "TEXTAREA");
    assert_eq!(number(&harness, "field-up"), 60);
    assert_eq!(number(&harness, "field-down"), 240);
    assert_eq!(value(&harness, "body-focus"), "BODY");
    assert_eq!(number(&harness, "body-down"), 420);
    assert_eq!(number(&harness, "body-up"), 0);
    assert!(number(&harness, "heads-after") > number(&harness, "heads-before"));
    assert_eq!(number(&harness, "dialog-up"), 240);
    assert_eq!(number(&harness, "dialog-down"), 240);
}

#[test]
fn scrolling_into_the_head_can_render_busy_during_the_scroll_callback() {
    let mut steps = vec![go(b"/test/reading/none")];
    snapshot(&mut steps, "before-scroll", "reads");
    steps.extend([
        harness::Step::Type {
            find: Find::exactly_named(Role::TextArea, b"Message"),
            words: b"Reading".as_slice().into(),
        },
        harness::Step::Key(Key::PageUp),
    ]);
    steps.push(harness::Step::Focused(Find::exactly_named(Role::TextArea, b"Message")));
    snapshot(&mut steps, "after-scroll", "reads");
    let harness = run(steps);
    assert!(number(&harness, "after-scroll") > number(&harness, "before-scroll"));
}

#[test]
fn a_status_at_the_head_does_not_anchor_instead_of_the_words() {
    let harness = run(vec![
        go(b"/test/reading/status"),
        press(b"Head"),
        press(b"Remember line"),
        keep("before-status-prepend", "position"),
        press(b"Prepend"),
        keep("after-status-prepend", "position"),
    ]);
    held(&harness, "before-status-prepend", "after-status-prepend");
}

#[test]
fn a_contents_wrapper_keeps_its_visible_words_in_place() {
    let harness = run(vec![
        go(b"/test/reading/contents"),
        press(b"Head"),
        press(b"Remember line"),
        keep("before-contents-prepend", "position"),
        press(b"Prepend"),
        keep("after-contents-prepend", "position"),
    ]);
    held(&harness, "before-contents-prepend", "after-contents-prepend");
}

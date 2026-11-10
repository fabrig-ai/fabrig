//! The example supplies its names to the kit's browser-shell stories.
use skein_fake_person::{Find, Step};
use skein_fake_person_browser::{Outcome, Script, Setup, harness, stories};
use skein_lib::Duration;
use skein_ui::Role;
use skein_ui_web_tests::{chromium, example_bundle, server_binary};

fn setup() -> Setup<'static> {
    setup_bundle(example_bundle())
}

fn setup_bundle(bundle: std::path::PathBuf) -> Setup<'static> {
    Setup {
        server: server_binary(),
        args: vec!["127.0.0.1:0".into(), "--bundle".into(), bundle.as_os_str().into()],
        printed: skein_fake_person_browser::harness::Printed {
            sent: skein_fake_person_browser::harness::Sent::Page,
            prefix: b"skein-ui-example: listening at ",
            on: skein_fake_person_browser::harness::Output::Stderr,
            names: skein_fake_person_browser::harness::Names::Address,
        },
        restart: vec!["{address}".into(), "--bundle".into(), bundle.into_os_string()],
        downloads: None,
        chromium: chromium(),
        origins: vec![],
        expected: vec![],
    }
}

/// Each write reports once, with the gesture kept (skein/docs/design/ui/web/testing.md, 3).
#[test]
fn a_clipboard_write_keeps_the_press_and_reports_once() {
    let see =
        |words: &[u8]| harness::Step::Person(Step::See { find: Find::text(words), within: Duration::from_secs(10) });
    let go = |address: &[u8]| harness::Step::Person(Step::Go { address: address.into() });
    let press = || harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, b"Copy") });
    let settle = || harness::Step::Person(Step::Pause { span: Duration::from_millis(100) });
    let mut pressed = Script::new(Box::new([
        go(b"/test/copy"),
        see(b"Ready"),
        press(),
        see(b"Done active, 1"),
        settle(),
        see(b"Done active, 1"),
    ]));
    let mut at_start =
        Script::new(Box::new([go(b"/test/copy?at-start"), see(b"Refused idle, 1"), settle(), see(b"Refused idle, 1")]));
    let mut missing = Script::new(Box::new([
        go(b"/test/copy?no-clipboard"),
        see(b"Ready"),
        press(),
        see(b"Refused active, 1"),
        settle(),
        see(b"Refused active, 1"),
    ]));
    assert_eq!(harness::Harness::new(setup()).run(&mut [&mut pressed, &mut at_start, &mut missing]), Outcome::Done);
}

#[test]
fn a_panic_reloads_then_says_it_failed() {
    assert_eq!(stories::panic(&setup(), b"/test/panic", Find::text(b"The application failed.")), Outcome::Done);
}

#[test]
fn a_refused_write_makes_no_note() {
    let mut writer = Script::new(Box::new([
        harness::Step::Person(Step::See { find: Find::text(b"Live"), within: Duration::from_secs(5) }),
        harness::Step::Person(Step::Type {
            find: Find::exactly_named(Role::TextField, b"Title"),
            words: b"Groceries".as_slice().into(),
        }),
        harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, b"Add") }),
    ]));
    assert_eq!(
        stories::write_refused(
            &setup(),
            b"/test/small-store",
            &mut writer,
            Find::exactly_named(Role::Link, b"Groceries"),
            Find::text(b"Live")
        ),
        Outcome::Done
    );
}

#[test]
fn typing_in_the_title_survives_a_note_arriving() {
    let mut writer = Script::new(Box::new([
        harness::Step::Person(Step::See { find: Find::text(b"Live"), within: Duration::from_secs(5) }),
        harness::Step::Person(Step::Type {
            find: Find::exactly_named(Role::TextField, b"Title"),
            words: b"Groceries".as_slice().into(),
        }),
        harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, b"Add") }),
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Link, b"Groceries"),
            within: Duration::from_secs(5),
        }),
    ]));
    assert_eq!(
        stories::typing_while_patched(
            &setup(),
            Find::exactly_named(Role::TextField, b"Title"),
            &mut writer,
            Find::exactly_named(Role::Link, b"Groceries"),
            Find::text(b"Live")
        ),
        Outcome::Done
    );
}
#[test]
fn back_and_forward_move_between_the_list_and_a_note() {
    assert_eq!(
        stories::back_and_forward(
            &setup(),
            stories::Shown { address: b"/", sees: Find::exactly_named(Role::Heading { level: 1 }, b"Notes") },
            stories::Shown {
                address: b"/notes/7",
                sees: Find::exactly_named(Role::Heading { level: 1 }, b"No such note.")
            },
            Find::text(b"Live")
        ),
        Outcome::Done
    );
}

#[path = "support/live_tree.rs"]
mod live_tree;

#[test]
fn a_placeholder_gives_way_to_the_client() {
    let mut person = Script::new(Box::new([
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Notes"),
            within: Duration::from_secs(2),
        }),
        harness::Step::Person(Step::Gone { find: Find::text(b"Starting notes"), within: Duration::from_secs(2) }),
    ]));
    assert_eq!(harness::Harness::new(setup()).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn a_module_not_found_says_so_and_offers_to_try_again() {
    let scratch = skein_scratch::Scratch::new("bundle-module-missing");
    let mut setup = setup_bundle(bundle_without_module(&scratch));
    setup.expected =
        vec![b"Failed to load resource: the server responded with a status of 404 (Not Found)".as_slice().into()];
    let mut person = Script::new(Box::new([
        harness::Step::Person(Step::See {
            find: Find::text(b"The server answered 404."),
            within: Duration::from_secs(2),
        }),
        harness::Step::Person(Step::Press { find: Find::exactly_named(Role::Button, b"Try again") }),
        harness::Step::Person(Step::See {
            find: Find::text(b"The server answered 404."),
            within: Duration::from_secs(2),
        }),
    ]));
    assert_eq!(harness::Harness::new(setup).run(&mut [&mut person]), Outcome::Done);
}

// The loader's HTTP failure needs a placeholder regardless of a prepared example bundle.
fn bundle_without_module(scratch: &skein_scratch::Scratch) -> std::path::PathBuf {
    let example = skein_scratch::paths::tree().join("examples/ui/shell-web");
    let inputs = skein_ui_bundle::Inputs {
        gzip: true,
        title: None,
        placeholder: Some(skein_ui_bundle::Placeholder {
            markup: example.join("loading.html"),
            stylesheet: example.join("loading.css"),
        }),
        name: "missing".into(),
        prefix: "/bundle/".into(),
        stylesheets: vec![],
        pictures: vec![],
        page: None,
        files: None,
        noscript: "Notes needs WebAssembly and scripts.".into(),
    };
    let glue = skein_ui_bundle::Glue {
        js: b"export default function init() { throw new Error('missing module reached init'); }".to_vec(),
        wasm: b"\0asm".to_vec(),
        snippets: vec![],
    };
    let out = scratch.path().join("bundle");
    let digest = skein_ui_bundle::bundle(&inputs, &glue, &out).expect("placeholder bundle");
    for name in ["missing_bg.wasm", "missing_bg.wasm.gz"] {
        std::fs::remove_file(out.join(digest.to_string()).join(name)).expect("remove module form");
    }
    out
}
#[path = "support/tab.rs"]
mod tab;

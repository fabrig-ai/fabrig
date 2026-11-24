use skein_fake_person::{Find, Step};
use skein_fake_person_browser::{Outcome, Script, harness};
use skein_lib::Duration;
use skein_scratch::Scratch;
use skein_ui::Role;
use skein_ui_web_tests::{fixture_bundle, fixture_harness};
#[test]
fn the_harness_fails_a_page_whose_module_throws() {
    let scratch = Scratch::new("harness-throws");
    let bundle =
        fixture_bundle(&scratch, b"export default function init() { throw new Error('fixture module failed'); }");
    let mut person = Script::new(Box::new([harness::Step::Person(Step::See {
        find: Find::exactly_named(Role::Heading { level: 1 }, b"Ready"),
        within: Duration::from_secs(1),
    })]));
    let Outcome::Failed { report, .. } = fixture_harness(&bundle).run(&mut [&mut person]) else {
        panic!("a throwing module fails the story");
    };
    assert!(report.contains("unexpected Exception: Uncaught"), "{report}");
}
#[test]
fn the_harness_reads_the_listening_line() {
    let scratch = Scratch::new("harness-listens");
    let bundle = fixture_bundle(&scratch,
        b"export default function init() { const h = document.createElement('h1'); h.textContent = 'Ready'; document.querySelector('#app').append(h); }");
    let mut person = Script::new(Box::new([harness::Step::Person(Step::See {
        find: Find::exactly_named(Role::Heading { level: 1 }, b"Ready"),
        within: Duration::from_secs(1),
    })]));
    assert_eq!(fixture_harness(&bundle).run(&mut [&mut person]), Outcome::Done);
}

#[test]
#[expect(clippy::disallowed_types, reason = "the browser story measures the caller's wall time, outside the harness")]
fn an_unmet_see_reports_its_deadline_and_returns_promptly() {
    let scratch = Scratch::new("harness-see-expiry");
    let bundle = fixture_bundle(&scratch, READY);
    let mut person = Script::new(Box::new([
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Ready"),
            within: Duration::from_secs(1),
        }),
        harness::Step::Person(Step::See { find: Find::text(b"Never shown"), within: Duration::from_secs(1) }),
    ]));
    let mut harness = fixture_harness(&bundle);
    let start = std::time::Instant::now();
    let outcome = harness.run(&mut [&mut person]);
    let elapsed = start.elapsed();
    let Outcome::Failed { person: 0, step: 1, report } = outcome else {
        panic!("the unmet See fails its own step: {outcome:?}");
    };
    assert!(report.contains("Missing") && report.contains("the browser step: patience "), "{report}");
    assert!(elapsed < std::time::Duration::from_secs(15), "the harness returned after {elapsed:?}: {report}");
}

#[test]
fn the_harness_searches_the_next_accessible_role() {
    let scratch = Scratch::new("harness-next-role");
    let bundle = fixture_bundle(&scratch, b"export default function init() { const e = document.createElement('input'); e.type = 'number'; e.setAttribute('aria-label', 'Count'); document.querySelector('#app').append(e); }");
    let mut person = Script::new(Box::new([harness::Step::Person(Step::See {
        find: Find::exactly_named(Role::TextField, b"Count"),
        within: Duration::from_secs(1),
    })]));
    assert_eq!(fixture_harness(&bundle).run(&mut [&mut person]), Outcome::Done);
}

const READY: &[u8] = b"export default function init() { const h = document.createElement('h1'); h.textContent = 'Ready'; document.querySelector('#app').append(h); }";
#[test]
fn a_page_of_another_origin_is_opened_by_its_step() {
    let scratch = Scratch::new("harness-origin");
    let bundle = fixture_bundle(&scratch, READY);
    let origin = harness::Origin {
        host: b"localhost".as_slice().into(),
        pages: vec![skein_ui_web_tests::page(b"/", b"<!doctype html><h1>Elsewhere</h1>")],
    };
    let mut person = Script::new(Box::new([
        harness::Step::Visit { origin: 0, path: b"/".as_slice().into() },
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Elsewhere"),
            within: skein_ui_web_tests::SECOND,
        }),
    ]));
    assert_eq!(skein_ui_web_tests::fixture_harness_with(&bundle, vec![origin]).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn a_framed_page_s_refusal_is_awaited_as_trouble() {
    let scratch = Scratch::new("harness-framed");
    let bundle = fixture_bundle(&scratch, READY);
    let framing = skein_ui_web_tests::page(b"/", b"<!doctype html><iframe src=\"http://{address}/\"></iframe>");
    let origin = harness::Origin { host: b"localhost".as_slice().into(), pages: vec![framing] };
    let mut person = Script::new(Box::new([
        harness::Step::Visit { origin: 0, path: b"/".as_slice().into() },
        harness::Step::Trouble { words: b"frame-ancestors".as_slice().into(), within: skein_ui_web_tests::SECOND },
    ]));
    assert_eq!(skein_ui_web_tests::fixture_harness_with(&bundle, vec![origin]).run(&mut [&mut person]), Outcome::Done);
}

fn command(mode: &[&str]) -> harness::Command {
    let mut args = vec!["command".into()];
    args.extend(mode.iter().map(std::ffi::OsString::from));
    harness::Command {
        program: skein_ui_web_tests::harness_fixture(),
        args,
        env: vec![],
        stdin: harness::Stdin::Closed,
        opens: Some(harness::Opens {
            prefix: b"http://".as_slice().into(),
            on: harness::Output::Stdout,
            keep: b"link".as_slice().into(),
        }),
        within: skein_ui_web_tests::SECOND,
    }
}
fn command_story(command: harness::Command) -> Script {
    Script::new(Box::new([
        harness::Step::Run(command),
        harness::Step::Exited { code: 0, within: skein_ui_web_tests::SECOND },
    ]))
}
fn command_harness(sent: harness::Sent) -> harness::Harness {
    harness::Harness::new(harness::Setup {
        server: skein_ui_web_tests::harness_fixture(),
        args: vec![if sent == harness::Sent::Command { "held-server" } else { "server" }.into()],
        printed: harness::Printed {
            sent,
            prefix: if sent == harness::Sent::Command { b"http://" } else { b"fixture listening " },
            on: harness::Output::Stderr,
            names: if sent == harness::Sent::Command { harness::Names::Url } else { harness::Names::Address },
        },
        restart: vec![],
        downloads: None,
        chromium: skein_ui_web_tests::chromium(),
        expected: vec![],
        origins: vec![],
    })
}
#[test]
fn a_command_receives_the_held_line_and_eof_without_a_page_visit() {
    let mut harness = command_harness(harness::Sent::Command);
    let mut cmd = command(&["stdin"]);
    cmd.stdin = harness::Stdin::Printed;
    let mut person = command_story(cmd);
    assert_eq!(harness.run(&mut [&mut person]), Outcome::Done);
    assert!(harness.kept()[0].value.ends_with(b"/opened"));
}
#[test]
fn an_exited_commands_printed_link_is_opened_and_kept_whole() {
    let mut harness = command_harness(harness::Sent::Page);
    let mut person = command_story(command(&["link", "http://{address}/opened?kept=whole"]));
    assert_eq!(harness.run(&mut [&mut person]), Outcome::Done);
    assert_eq!(harness.kept()[0].person, 0);
    assert_eq!(harness.kept()[0].name.as_ref(), b"link");
    assert!(harness.kept()[0].value.starts_with(b"http://127.0.0.1:"));
    assert!(harness.kept()[0].value.ends_with(b"/opened?kept=whole"));
}
#[test]
fn a_command_waits_for_the_browsers_loopback_signal_before_exiting() {
    let mut person = command_story(command(&["wait", "{address}"]));
    assert_eq!(command_harness(harness::Sent::Page).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn a_person_types_over_words_then_clears_the_field() {
    let scratch = Scratch::new("harness-replace");
    let bundle = fixture_bundle(
        &scratch,
        br"export default function init() {
        const field = document.createElement('input');
        field.setAttribute('aria-label', 'Title'); field.value = 'Old words';
        const heading = document.createElement('h1'); heading.textContent = 'Old words';
        field.addEventListener('input', () => { heading.textContent = field.value || 'Empty'; });
        document.querySelector('#app').append(field, heading);
    }",
    );
    let mut person = Script::new(Box::new([
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Old words"),
            within: Duration::from_secs(2),
        }),
        harness::Step::Person(Step::Type {
            find: Find::exactly_named(Role::TextField, b"Title"),
            words: b"New words".as_slice().into(),
        }),
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"New words"),
            within: Duration::from_secs(2),
        }),
        harness::Step::Person(Step::Type { find: Find::exactly_named(Role::TextField, b"Title"), words: Box::new([]) }),
        harness::Step::Person(Step::See {
            find: Find::exactly_named(Role::Heading { level: 1 }, b"Empty"),
            within: Duration::from_secs(2),
        }),
    ]));
    assert_eq!(fixture_harness(&bundle).run(&mut [&mut person]), Outcome::Done);
}

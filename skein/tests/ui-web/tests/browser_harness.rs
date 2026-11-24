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
fn ready_person() -> Script {
    Script::new(Box::new([harness::Step::Person(Step::See {
        find: Find::exactly_named(Role::Heading { level: 1 }, b"Ready"),
        within: skein_ui_web_tests::SECOND,
    })]))
}

#[test]
fn a_lent_peer_listens_before_the_server_requests_it() {
    let mut peer = skein_ui_web_tests::harness_peer::Peer::new();
    let mut person = ready_person();
    let outcome = skein_ui_web_tests::lending_harness(&["{neighbor}"], vec![])
        .run_lending(&mut [&mut person], &mut [harness::Lent { name: b"neighbor", peer: &mut peer }]);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(peer.observations, [b"GET / HTTP/1.1\r\nHost: fixture\r\nConnection: close\r\n\r\n".to_vec()]);
}

#[test]
fn a_lent_peer_returns_empty_with_its_handle_usable() {
    use skein_world::Host;
    let mut peer = skein_ui_web_tests::harness_peer::Peer::new();
    let handle = peer.handle.clone();
    let mut person = ready_person();
    let outcome = skein_ui_web_tests::lending_harness(&["{neighbor}"], vec![])
        .run_lending(&mut [&mut person], &mut [harness::Lent { name: b"neighbor", peer: &mut peer }]);
    assert_eq!(outcome, Outcome::Done);
    assert!(peer.is_empty());
    assert_eq!(handle.get(), 1);
    assert_eq!(peer.observations.len(), 1);
}

#[test]
fn colliding_peer_tokens_reach_their_own_hosts() {
    use skein_world::Host;
    let mut first = skein_ui_web_tests::harness_peer::Peer::new();
    let mut second = skein_ui_web_tests::harness_peer::Peer::new();
    let mut person = ready_person();
    let outcome = skein_ui_web_tests::lending_harness(&["{first}", "{second}"], vec![]).run_lending(
        &mut [&mut person],
        &mut [harness::Lent { name: b"first", peer: &mut first }, harness::Lent { name: b"second", peer: &mut second }],
    );
    assert_eq!(outcome, Outcome::Done);
    assert_eq!((first.observations.len(), second.observations.len()), (1, 1));
    assert!(first.is_empty() && second.is_empty());
}

#[test]
fn repeated_peer_arguments_each_reach_the_server() {
    let mut peer = skein_ui_web_tests::harness_peer::Peer::new();
    let mut person = ready_person();
    let outcome = skein_ui_web_tests::lending_harness(&["{neighbor}", "{neighbor}"], vec![])
        .run_lending(&mut [&mut person], &mut [harness::Lent { name: b"neighbor", peer: &mut peer }]);
    assert_eq!(outcome, Outcome::Done);
    assert_eq!(peer.handle.get(), 2);
}

fn keep(selector: &[u8], attribute: Option<&[u8]>, name: &[u8]) -> harness::Step {
    harness::Step::Keep { selector: selector.into(), attribute: attribute.map(Box::from), name: name.into() }
}

fn command(mode: &[&str], opens: bool) -> harness::Command {
    let mut args = vec!["command".into()];
    args.extend(mode.iter().map(std::ffi::OsString::from));
    harness::Command {
        program: skein_ui_web_tests::harness_fixture(),
        args,
        env: vec![],
        stdin: harness::Stdin::Closed,
        opens: opens.then(|| harness::Opens {
            prefix: b"http://".as_slice().into(),
            on: harness::Output::Stdout,
            keep: b"link".as_slice().into(),
        }),
        within: skein_ui_web_tests::SECOND,
    }
}
fn exited(code: i32) -> harness::Step {
    harness::Step::Exited { code, within: skein_ui_web_tests::SECOND }
}
fn command_story(command: harness::Command) -> Script {
    Script::new(Box::new([harness::Step::Run(command), exited(0)]))
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
    let mut cmd = command(&["stdin"], true);
    cmd.stdin = harness::Stdin::Printed;
    let mut person = command_story(cmd);
    assert_eq!(harness.run(&mut [&mut person]), Outcome::Done);
    assert!(harness.kept()[0].value.ends_with(b"/opened"));
}
#[test]
fn closed_command_stdin_receives_only_eof() {
    let mut person = command_story(command(&["closed"], false));
    assert_eq!(command_harness(harness::Sent::Command).run(&mut [&mut person]), Outcome::Done);
}
#[test]
fn an_exited_commands_printed_link_is_opened_and_kept_whole() {
    let mut harness = command_harness(harness::Sent::Page);
    let mut person = command_story(command(&["link", "http://{address}/opened?kept=whole"], true));
    assert_eq!(harness.run(&mut [&mut person]), Outcome::Done);
    assert_eq!(harness.kept()[0].person, 0);
    assert_eq!(harness.kept()[0].name.as_ref(), b"link");
    assert!(harness.kept()[0].value.starts_with(b"http://127.0.0.1:"));
    assert!(harness.kept()[0].value.ends_with(b"/opened?kept=whole"));
}
#[test]
fn a_command_without_the_link_fails_naming_the_command() {
    let mut person = command_story(command(&["no-link"], true));
    let Outcome::Failed { step: 0, report, .. } = command_harness(harness::Sent::Page).run(&mut [&mut person]) else {
        panic!("missing command link fails")
    };
    assert!(report.contains("harness_fixture: exited without matching link"), "{report}");
    assert!(report.contains("fixture exits without a link"), "command stderr retained: {report}");
}
#[test]
fn exited_accepts_the_commands_expected_code() {
    let mut person = Script::new(Box::new([harness::Step::Run(command(&["code", "7"], false)), exited(7)]));
    assert_eq!(command_harness(harness::Sent::Page).run(&mut [&mut person]), Outcome::Done);
}
#[test]
fn exited_rejects_a_different_code_naming_the_command() {
    let mut person = Script::new(Box::new([harness::Step::Run(command(&["code", "7"], false)), exited(0)]));
    let Outcome::Failed { step: 1, report, .. } = command_harness(harness::Sent::Page).run(&mut [&mut person]) else {
        panic!("wrong exit code fails")
    };
    assert!(report.contains("harness_fixture: exited with code 7, expected 0"), "{report}");
}
#[test]
fn a_live_command_at_story_end_fails_and_settles() {
    let mut person = Script::new(Box::new([harness::Step::Run(command(&["wait"], false))]));
    let Outcome::Failed { report, .. } = command_harness(harness::Sent::Page).run(&mut [&mut person]) else {
        panic!("live command fails story end")
    };
    assert!(report.contains("harness_fixture still running at story end"), "{report}");
    assert!(!report.contains("deadline passed"), "teardown settles: {report}");
}
#[test]
fn a_person_runs_two_commands_in_turn() {
    let mut person = Script::new(Box::new([
        harness::Step::Run(command(&["code", "7"], false)),
        exited(7),
        harness::Step::Run(command(&["code", "8"], false)),
        exited(8),
    ]));
    assert_eq!(command_harness(harness::Sent::Page).run(&mut [&mut person]), Outcome::Done);
}
#[test]
fn a_second_run_while_the_command_runs_fails() {
    let mut person = Script::new(Box::new([
        harness::Step::Run(command(&["wait"], false)),
        harness::Step::Run(command(&["code", "0"], false)),
    ]));
    let Outcome::Failed { step: 1, report, .. } = command_harness(harness::Sent::Page).run(&mut [&mut person]) else {
        panic!("overlapping commands fail")
    };
    assert!(report.contains("harness_fixture: another Run before it settled"), "{report}");
}
#[test]
fn command_arguments_receive_peer_and_server_addresses() {
    let mut peer = skein_ui_web_tests::harness_peer::Peer::new();
    let mut person = command_story(command(&["args", "{neighbor}", "{address}"], false));
    assert_eq!(
        command_harness(harness::Sent::Page)
            .run_lending(&mut [&mut person], &mut [harness::Lent { name: b"neighbor", peer: &mut peer }]),
        Outcome::Done
    );
    assert_eq!(peer.observations.len(), 1);
}
#[test]
fn command_environment_values_receive_peer_and_server_addresses() {
    let mut peer = skein_ui_web_tests::harness_peer::Peer::new();
    let mut cmd = command(&["env"], false);
    cmd.env = vec![("FIXTURE_PEER".into(), "{neighbor}".into()), ("FIXTURE_SERVER".into(), "{address}".into())];
    let mut person = command_story(cmd);
    assert_eq!(
        command_harness(harness::Sent::Page)
            .run_lending(&mut [&mut person], &mut [harness::Lent { name: b"neighbor", peer: &mut peer }]),
        Outcome::Done
    );
    assert_eq!(peer.observations.len(), 1);
}
#[test]
fn a_link_keep_and_a_selector_keep_append_to_one_sequence() {
    let mut harness = command_harness(harness::Sent::Page);
    let mut person = Script::new(Box::new([
        harness::Step::Run(command(&["link", "http://{address}/opened"], true)),
        keep(b"input", None, b"field"),
        exited(0),
    ]));
    assert_eq!(harness.run(&mut [&mut person]), Outcome::Done);
    assert_eq!(
        harness.kept().iter().map(|k| (k.person, k.name.as_ref())).collect::<Vec<_>>(),
        vec![(0, b"link".as_slice()), (0, b"field".as_slice())]
    );
    assert_eq!(harness.kept()[1].value.as_ref(), b"fixture value");
}
#[test]
fn two_peoples_commands_have_independent_status_and_keeps() {
    let mut harness = command_harness(harness::Sent::Page);
    let mut first = command_story(command(&["link", "http://{address}/first"], true));
    let mut second = Script::new(Box::new([harness::Step::Run(command(&["code", "7"], false)), exited(7)]));
    assert_eq!(harness.run(&mut [&mut first, &mut second]), Outcome::Done);
    assert_eq!(harness.kept().len(), 1);
    assert_eq!(harness.kept()[0].person, 0);
    assert!(harness.kept()[0].value.ends_with(b"/first"));
}
#[test]
fn a_command_waits_for_the_browsers_loopback_signal_before_exiting() {
    let mut person = command_story(command(&["wait", "{address}"], true));
    assert_eq!(command_harness(harness::Sent::Page).run(&mut [&mut person]), Outcome::Done);
}

#[test]
fn a_missing_link_deadline_fails_and_command_teardown_settles() {
    let mut cmd = command(&["wait"], true);
    cmd.within = Duration::from_millis(50);
    let mut person = command_story(cmd);
    let Outcome::Failed { step: 0, report, .. } = command_harness(harness::Sent::Page).run(&mut [&mut person]) else {
        panic!("missing link deadline fails")
    };
    assert!(report.contains("harness_fixture: Run missing-link/start deadline passed"), "{report}");
    assert!(!report.contains("browser harness deadline passed"), "command teardown settles: {report}");
}
#[test]
fn an_exited_deadline_fails_and_command_teardown_settles() {
    let mut person = Script::new(Box::new([
        harness::Step::Run(command(&["wait"], false)),
        harness::Step::Exited { code: 0, within: Duration::from_millis(50) },
    ]));
    let Outcome::Failed { step: 1, report, .. } = command_harness(harness::Sent::Page).run(&mut [&mut person]) else {
        panic!("exit deadline fails")
    };
    assert!(report.contains("harness_fixture: Exited deadline passed"), "{report}");
    assert!(!report.contains("browser harness deadline passed"), "command teardown settles: {report}");
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

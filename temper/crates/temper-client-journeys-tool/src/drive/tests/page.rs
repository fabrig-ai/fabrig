//! Page trouble policy, without a browser (client/drive.md, 2.3, 2.5 and 3.7).
use super::*;

const ROUTE: &str = "http://127.0.0.1:8367/jig/v1/sign-in/link";
const REFUSED: &str = "Failed to load resource: the server responded with a status of 401 (Unauthorized)";
const MISSING: &str = "Failed to load resource: the server responded with a status of 404 (Not Found)";

fn trouble(kind: Trouble, text: &str, url: &str) -> Event {
    Event::PageTrouble { trouble: kind, text: text.as_bytes().into(), url: url.as_bytes().into() }
}
fn link(secret: &str) -> Event {
    Event::BoxLine { output: Output::Stdout, line: format!("{ROUTE}?secret={secret}").into_bytes().into() }
}
fn signing_in() -> Machine {
    let (mut m, _) = Machine::start(plan(), Time::ZERO);
    let actions = step(&mut m, link("first"));
    assert!(matches!(actions.as_slice(), [Action::Log { .. }, Action::Start { process: Process::SignIn, .. }]));
    m
}
fn opening() -> Machine {
    let mut m = signing_in();
    let actions = step(&mut m, exited(Process::SignIn, 0));
    assert!(
        matches!(actions.as_slice(), [Action::Open(url)] if url.as_ref() == format!("{ROUTE}?secret=first").as_bytes())
    );
    m
}
fn classified(m: &mut Machine) -> Vec<Action> {
    assert!(matches!(step(m, Event::FaceDone(Ok(()))).as_slice(), [Action::TerminateBox]));
    assert!(matches!(step(m, exited(Process::Box, 0)).as_slice(), [Action::Classify]));
    step(m, Event::Logs { serve: vec![], forges: vec![], box_tail: vec![] })
}

#[test]
fn spent_refusal_is_ignored_in_every_start_wait() {
    // The two pipes can deliver the fresh link before or after the spent open completes.
    for (wait, fresh_early, phase) in [
        (0, false, Phase::Spent),
        (1, true, Phase::Spent),
        (1, false, Phase::FreshLink),
        (2, false, Phase::Fresh),
        (3, false, Phase::Root),
        (4, false, Phase::Main),
    ] {
        let mut m = opening();
        let mut events = vec![Event::FaceDone(Ok(())), link("fresh")];
        if fresh_early {
            events.swap(0, 1);
        }
        events.extend([Event::FaceDone(Ok(())), Event::FaceDone(Ok(())), Event::Found(Some(NodeId(1)))]);
        for (at, event) in events.into_iter().enumerate() {
            if at == wait {
                assert_eq!(m.phase, phase);
                let until = m.next_deadline();
                assert!(step(&mut m, trouble(Trouble::Log, REFUSED, ROUTE)).is_empty());
                assert_eq!(m.next_deadline(), until, "expected trouble leaves the wait running");
            }
            let actions = step(&mut m, event);
            assert!(
                !actions.iter().any(|a| matches!(a, Action::CancelFace | Action::Finished(_) | Action::Answer(_))),
                "{actions:?}"
            );
            if at == 4 {
                assert!(matches!(actions.as_slice(), [Action::Ready]));
            }
        }
        assert!(m.ready());
    }
}

#[test]
fn other_start_trouble_fails_the_box_without_naming_secrets() {
    for (kind, text, url, words) in [
        (
            Trouble::Log,
            MISSING,
            "http://127.0.0.1:8367/app.js?secret=SECRET#part",
            format!("Log: {MISSING} (http://127.0.0.1:8367/app.js)"),
        ),
        (
            Trouble::Log,
            REFUSED,
            "http://127.0.0.1:8367/jig/v1/stream",
            format!("Log: {REFUSED} (http://127.0.0.1:8367/jig/v1/stream)"),
        ),
        (
            Trouble::Log,
            REFUSED,
            "http://127.0.0.1:8368/jig/v1/sign-in/link",
            format!("Log: {REFUSED} (http://127.0.0.1:8368/jig/v1/sign-in/link)"),
        ),
        (Trouble::Log, MISSING, ROUTE, format!("Log: {MISSING} ({ROUTE})")),
        (Trouble::Exception, REFUSED, ROUTE, format!("Exception: {REFUSED} ({ROUTE})")),
        (Trouble::Exception, "TEXT", "", "Exception: TEXT".into()),
        (
            Trouble::Exception,
            "TEXT",
            "http://127.0.0.1:8367/jig/v1/sign-in/link?secret=SECRET#part",
            format!("Exception: TEXT ({ROUTE})"),
        ),
        (Trouble::Log, "401 without a resource status", ROUTE, format!("Log: 401 without a resource status ({ROUTE})")),
    ] {
        let mut m = opening();
        let actions = step(&mut m, trouble(kind, text, url));
        assert!(
            matches!(actions.as_slice(), [Action::CancelFace, Action::StopCtl, Action::CloseBrowser]),
            "{actions:?}"
        );
        assert!(!format!("{actions:?}").contains("SECRET"));
        let actions = classified(&mut m);
        let exit = finish(&actions);
        assert_eq!(exit.status, 4);
        assert_eq!(exit.shown, [format!("box failed: {words}")]);
        assert_eq!(exit.error, None);
        let answers = format!("{actions:?}");
        assert!(!answers.contains("SECRET") && !answers.contains("secret="), "{answers}");
    }
}

#[test]
fn refusal_before_the_spent_open_is_not_expected() {
    for mut m in [Machine::start(plan(), Time::ZERO).0, signing_in()] {
        let actions = step(&mut m, trouble(Trouble::Log, REFUSED, ROUTE));
        assert!(actions.iter().any(|a| matches!(a, Action::CancelFace)));
        let actions = classified(&mut m);
        let exit = finish(&actions);
        assert_eq!(exit.status, 4);
        assert_eq!(exit.shown, [format!("box failed: Log: {REFUSED} ({ROUTE})")]);
    }
}

#[test]
fn trouble_words_drop_queries_and_fragments_and_keep_lossy_text() {
    for url in [
        "http://127.0.0.1:8367/jig/v1/sign-in/link?secret=SECRET#part",
        "http://127.0.0.1:8367/jig/v1/sign-in/link#SECRET?secret=SECRET",
    ] {
        let words = trouble_words(Trouble::Exception, b"bad \xff", url.as_bytes());
        assert_eq!(words, format!("Exception: bad \u{fffd} ({ROUTE})"));
        assert!(!words.contains("SECRET") && !words.contains("secret="));
    }
    assert_eq!(trouble_words(Trouble::Exception, b"TEXT", b""), "Exception: TEXT");
}

#[test]
fn trouble_after_ready_fails_only_the_current_web_line() {
    let mut m = ready();
    // Even the spent route's 401 counts after ready, but none count between lines.
    let url = "http://127.0.0.1:9/jig/v1/sign-in/link?secret=SECRET#part";
    assert!(step(&mut m, trouble(Trouble::Log, REFUSED, url)).is_empty());
    let source = r#"web press button "Start""#;
    assert!(matches!(line(&mut m, source).as_slice(), [Action::Find(_)]));
    let actions = step(&mut m, trouble(Trouble::Log, REFUSED, url));
    assert!(matches!(actions.first(), Some(Action::CancelFace)));
    assert_eq!(
        answer(&actions).print(),
        format!("failed: line 1: {source}: Log: {REFUSED} (http://127.0.0.1:9/jig/v1/sign-in/link)\n")
    );
    let answers = format!("{actions:?}");
    assert!(!answers.contains("SECRET") && !answers.contains("secret="));
    assert!(m.ready());
    assert!(step(&mut m, trouble(Trouble::Exception, "between lines", "")).is_empty());
    assert!(matches!(line(&mut m, r#"web see "Hello""#).as_slice(), [Action::Find(_)]));
    assert_eq!(answer(&step(&mut m, Event::Found(Some(NodeId(2))))).print(), "ok\n");
}

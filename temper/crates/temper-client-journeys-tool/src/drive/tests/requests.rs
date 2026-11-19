//! Recorded requests and first answers, with event times (client/drive.md, 3.2 and 6).
use super::*;

const ORIGIN: &str = "http://127.0.0.1:8367";
fn at(m: &mut Machine, event: Event, millis: u64) -> Vec<Action> {
    m.step(event, Time::ZERO.saturating_add(Duration::from_millis(millis)))
}
fn sent(id: &str, load: &str, document: bool, method: &str, url: &str) -> Event {
    Event::PageSent {
        id: id.as_bytes().into(),
        load: load.as_bytes().into(),
        document,
        method: method.as_bytes().into(),
        url: url.as_bytes().into(),
    }
}
fn request(id: &str, method: &str, path: &str) -> Event {
    sent(id, "L2", false, method, &format!("{ORIGIN}{path}"))
}
fn document() -> Event {
    sent("D2", "L2", true, "GET", &format!("{ORIGIN}/projects/1/work"))
}
fn answered(id: &str) -> Event {
    Event::PageAnswered { id: id.as_bytes().into() }
}
fn ask(m: &mut Machine, millis: u64) -> Vec<Action> {
    at(m, Event::Line(b"web requests".as_slice().into()), millis)
}
fn loaded() -> Machine {
    let mut m = ready();
    assert!(at(&mut m, document(), 0).is_empty());
    assert!(at(&mut m, answered("D2"), 0).is_empty());
    m
}
fn kept(actions: &[Action]) {
    assert!(matches!(actions.first(), Some(Action::Keep(bytes)) if bytes.as_ref() == b"web requests"));
}

#[test]
fn recorded_load_filters_and_counts_stream_first_answer() {
    let mut m = ready();
    assert!(at(&mut m, document(), 100).is_empty());
    assert!(at(&mut m, answered("D2"), 110).is_empty());
    for (id, path) in [
        ("js", "/bundle/0123456789abcdef/temper.js"),
        ("wasm", "/bundle/0123456789abcdef/temper.wasm"),
        ("css", "/bundle/0123456789abcdef/temper.css"),
        ("sw", "/sw.js"),
    ] {
        assert!(at(&mut m, request(id, "GET", path), 120).is_empty());
        assert!(at(&mut m, answered(id), 130).is_empty());
    }
    // The leaving load's request arrives after the new document and remains unanswered.
    assert!(at(&mut m, sent("old", "L1", false, "POST", &format!("{ORIGIN}/jig/v1/read")), 140).is_empty());
    assert!(at(&mut m, request("stream", "GET", "/jig/v1/stream?version=3"), 150).is_empty());
    assert!(at(&mut m, answered("stream"), 160).is_empty()); // first bytes, not the stream's end
    assert!(at(&mut m, request("batch", "POST", "/jig/v1/batch#part?secret=SECRET"), 170).is_empty());
    assert!(at(&mut m, answered("batch"), 180).is_empty());
    assert!(at(&mut m, sent("other", "L2", false, "GET", "http://localhost:9/x"), 190).is_empty());
    assert!(at(&mut m, answered("other"), 200).is_empty());
    // Already quiet: no further event or operation is needed.
    let actions = ask(&mut m, 2180);
    kept(&actions);
    let text = answer(&actions).print();
    assert_eq!(text, "| GET /jig/v1/stream, wave 1\n| POST /jig/v1/batch, wave 2\n| 2 requests in 2 waves\nok\n");
    assert!(!text.contains('?') && !text.contains('#') && !text.contains("SECRET"));
}

#[test]
fn waves_follow_greatest_answered_wave_when_answers_come_apart() {
    let mut m = loaded();
    at(&mut m, request("A", "GET", "/a"), 100);
    at(&mut m, request("B", "GET", "/b"), 100);
    at(&mut m, answered("A"), 200);
    at(&mut m, request("C", "GET", "/c"), 200);
    at(&mut m, answered("B"), 300);
    at(&mut m, request("D", "GET", "/d"), 300);
    at(&mut m, answered("C"), 400);
    at(&mut m, answered("D"), 400);
    assert_eq!(
        answer(&ask(&mut m, 2400)).print(),
        "| GET /a, wave 1\n| GET /b, wave 1\n| GET /c, wave 2\n| GET /d, wave 2\n| 4 requests in 2 waves\nok\n"
    );
}

#[test]
fn quiet_waits_for_unlisted_answers_and_restarts_on_send() {
    let mut m = ready();
    at(&mut m, document(), 0);
    at(&mut m, request("A", "POST", "/a"), 100);
    at(&mut m, answered("A"), 200);
    assert!(ask(&mut m, 300).is_empty());
    assert_eq!(m.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_millis(30_300))));
    assert!(at(&mut m, Event::Deadline, 2200).is_empty(), "the document is still unanswered");
    assert!(at(&mut m, answered("D2"), 2300).is_empty());
    assert_eq!(m.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_millis(4300))));
    // A bundle file is not listed but both its send and its answer restart the spell.
    assert!(at(&mut m, request("js", "GET", "/bundle/hash/temper.js"), 4000).is_empty());
    assert!(at(&mut m, answered("js"), 4100).is_empty());
    assert!(at(&mut m, Event::Deadline, 4300).is_empty(), "the old quiet deadline cannot answer");
    assert!(at(&mut m, answered("js"), 5000).is_empty(), "a duplicate first answer is inert");
    assert!(at(&mut m, answered("unknown"), 5100).is_empty());
    assert!(at(&mut m, sent("other", "L2", false, "GET", "http://localhost:9/x"), 5500).is_empty());
    assert_eq!(m.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_millis(6100))));
    assert!(at(&mut m, Event::Deadline, 6099).is_empty());
    let actions = at(&mut m, Event::Deadline, 6100);
    kept(&actions);
    assert_eq!(answer(&actions).print(), "| POST /a, wave 1\n| 1 requests in 1 waves\nok\n");
    assert!(m.ready());
    assert_eq!(m.next_deadline(), None);
}

#[test]
fn unanswered_bound_shows_the_log_and_names_each_pending_request() {
    let mut m = loaded();
    at(&mut m, request("stream", "GET", "/jig/v1/stream?version=3"), 100);
    at(&mut m, answered("stream"), 200);
    at(&mut m, request("batch", "POST", "/jig/v1/batch?secret=SECRET"), 300);
    at(&mut m, request("read", "POST", "/jig/v1/read#SECRET"), 400);
    assert!(ask(&mut m, 1000).is_empty());
    assert!(at(&mut m, Event::Deadline, 30_999).is_empty());
    let actions = at(&mut m, Event::Deadline, 31_000);
    kept(&actions);
    assert_eq!(
        answer(&actions).print(),
        concat!(
            "| GET /jig/v1/stream, wave 1\n| POST /jig/v1/batch, wave 2\n| POST /jig/v1/read, wave 2\n",
            "| 3 requests in 2 waves\nfailed: line 1: web requests: unanswered: ",
            "POST /jig/v1/batch, wave 2; POST /jig/v1/read, wave 2\n"
        )
    );
    assert!(m.ready(), "a failed line does not end the session");
    at(&mut m, answered("batch"), 31_100);
    at(&mut m, answered("read"), 31_200);
    assert!(answer(&ask(&mut m, 33_200)).failed.is_none());
}

#[test]
fn continuous_sends_fail_at_the_bound_from_the_line_start() {
    let mut m = loaded();
    assert!(ask(&mut m, 1000).is_empty());
    for second in 2_u64..=30 {
        let id = second.to_string();
        assert!(at(&mut m, request(&id, "GET", "/poll"), second * 1000).is_empty());
        assert!(at(&mut m, answered(&id), second * 1000).is_empty());
    }
    assert_eq!(m.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(31))));
    assert!(at(&mut m, Event::Deadline, 30_999).is_empty());
    let actions = at(&mut m, Event::Deadline, 31_000);
    kept(&actions);
    let answer = answer(&actions);
    assert_eq!(answer.shown.last().expect("summary"), "29 requests in 29 waves");
    assert_eq!(answer.failed.as_deref(), Some("line 1: web requests: the page kept sending"));
}

#[test]
fn no_document_or_listed_requests_gives_zero_waves() {
    let mut m = ready();
    at(&mut m, request("before", "GET", "/before"), 500);
    let actions = ask(&mut m, 2000);
    assert_eq!(answer(&actions).print(), "| 0 requests in 0 waves\nok\n");
    kept(&actions);
    at(&mut m, document(), 2100);
    at(&mut m, answered("D2"), 2200);
    let actions = ask(&mut m, 4200);
    assert_eq!(answer(&actions).print(), "| 0 requests in 0 waves\nok\n");
}

#[test]
fn new_document_resets_load_origin_and_waves_during_the_line() {
    let mut m = loaded();
    at(&mut m, request("old", "POST", "/old"), 100); // unanswered in the old document
    assert!(ask(&mut m, 200).is_empty());
    assert!(at(&mut m, sent("D3", "L3", true, "GET", "https://localhost:9/new"), 300).is_empty());
    assert!(at(&mut m, answered("old"), 400).is_empty());
    assert!(at(&mut m, request("leaving", "POST", "/jig/v1/read"), 500).is_empty());
    assert!(at(&mut m, answered("D3"), 600).is_empty());
    assert!(at(&mut m, sent("new", "L3", false, "GET", "https://localhost:9/new-read?q=x"), 700).is_empty());
    assert!(at(&mut m, answered("new"), 800).is_empty());
    let actions = at(&mut m, Event::Deadline, 2800);
    assert_eq!(answer(&actions).print(), "| GET /new-read, wave 1\n| 1 requests in 1 waves\nok\n");
}

#[test]
fn log_is_kept_before_ready_and_while_another_line_runs() {
    let (mut m, _) = Machine::start(plan(), Time::ZERO);
    // No line, no ready session, and Place is still starting.
    assert!(at(&mut m, document(), 0).is_empty());
    assert!(at(&mut m, answered("D2"), 0).is_empty());
    let link = |secret| Event::BoxLine {
        output: Output::Stdout,
        line: format!("http://127.0.0.1:9/link?secret={secret}\n").into_bytes().into(),
    };
    step(&mut m, link("first"));
    step(&mut m, exited(Process::SignIn, 0));
    step(&mut m, Event::FaceDone(Ok(())));
    step(&mut m, link("fresh"));
    step(&mut m, Event::FaceDone(Ok(())));
    step(&mut m, Event::Found(Some(NodeId(0))));
    step(&mut m, Event::FaceDone(Ok(())));
    step(&mut m, Event::Found(Some(NodeId(0))));
    assert!(m.ready());
    assert!(matches!(line(&mut m, "web see \"Hello\"").as_slice(), [Action::Find(_)]));
    assert!(at(&mut m, request("A", "POST", "/a"), 100).is_empty());
    assert!(at(&mut m, answered("A"), 200).is_empty());
    assert_eq!(answer(&at(&mut m, Event::Found(Some(NodeId(1))), 300)).print(), "ok\n");
    assert_eq!(answer(&ask(&mut m, 2200)).print(), "| POST /a, wave 1\n| 1 requests in 1 waves\nok\n");
}

#[test]
fn filters_compare_the_whole_origin_and_skip_only_bundle_gets() {
    let mut m = loaded();
    for (id, url) in [
        ("port", "http://127.0.0.1:83670/x"),
        ("host", "http://127.0.0.1:8367.example/x"),
        ("scheme", "https://127.0.0.1:8367/x"),
        ("data", "data:text/plain,x"),
        ("blob", "blob:http://127.0.0.1:8367/x"),
    ] {
        at(&mut m, sent(id, "L2", false, "GET", url), 100);
    } // unanswered foreign requests cannot keep the page busy
    for (id, path) in [("robots", "/robots.txt?x=y"), ("manifest", "/manifest.webmanifest#x")] {
        at(&mut m, request(id, "GET", path), 200);
        at(&mut m, answered(id), 300);
    }
    at(&mut m, request("post", "POST", "/sw.js"), 400);
    at(&mut m, answered("post"), 500);
    at(&mut m, request("nested", "GET", "/nested/sw.js"), 600);
    at(&mut m, answered("nested"), 700);
    assert_eq!(
        answer(&ask(&mut m, 2700)).print(),
        "| POST /sw.js, wave 1\n| GET /nested/sw.js, wave 2\n| 2 requests in 2 waves\nok\n"
    );
}

#[test]
fn trouble_during_requests_fails_only_that_line_and_is_kept() {
    let mut m = loaded();
    assert!(ask(&mut m, 100).is_empty());
    let actions = at(
        &mut m,
        Event::PageTrouble {
            trouble: Trouble::Exception,
            text: b"broken".as_slice().into(),
            url: format!("{ORIGIN}/app.js?secret=SECRET").into_bytes().into(),
        },
        200,
    );
    assert_eq!(
        answer(&actions).print(),
        "failed: line 1: web requests: Exception: broken (http://127.0.0.1:8367/app.js)\n"
    );
    assert!(actions.iter().any(|action| matches!(action, Action::Keep(bytes) if bytes.as_ref() == b"web requests")));
    assert!(m.ready());
    assert!(answer(&ask(&mut m, 2000)).failed.is_none());
}

#[test]
fn ctl_session_refuses_requests_without_keeping_it() {
    let mut p = plan();
    p.options.ctl = true;
    let (mut m, _) = Machine::start(p, Time::ZERO);
    step(
        &mut m,
        Event::BoxLine { output: Output::Stdout, line: b"http://127.0.0.1:9/link?secret=first\n".as_slice().into() },
    );
    assert!(matches!(step(&mut m, exited(Process::SignIn, 0)).as_slice(), [Action::RemoveLink, Action::Ready]));
    let actions = ask(&mut m, 100);
    assert!(matches!(actions.as_slice(), [Action::Answer(_)]), "not kept: {actions:?}");
    assert_eq!(answer(&actions).print(), "failed: line 1: this session takes ctl and forge lines alone\n");
    assert!(m.ready());
}

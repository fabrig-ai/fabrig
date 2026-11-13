use super::{answer, line, ready};
use crate::drive::{Action, Event, Machine, io::forge as adapter};
use skein_http::Method;
use skein_lib::{
    Duration, Time, Token,
    stream::{Down, Held, Up},
};
use skein_scratch::Scratch;
use std::{collections::VecDeque, fs, io::Write, os::unix::fs::OpenOptionsExt, path::Path};
const TOKEN: &str = "fake-drive-write-value";
const COMMIT: &str = r#"forge commit forgejo playground/widget "msg" "a.txt" "b.txt""#;
const LOG: &str = "1970-01-01T00:00:00Z box: fake forgejo http://127.0.0.1:8001\n1970-01-01T00:00:01Z box: fake forgejo http://127.0.0.1:8002\n";
const SHA: &str = "1111111111111111111111111111111111111111";
pub(crate) fn fixture() -> (Scratch, Machine) {
    let scratch = Scratch::new("drive-forge-lines");
    let mut m = ready();
    m.plan.options.scratch = scratch.path().into();
    fs::create_dir_all(scratch.path().join("session")).expect("session");
    fs::create_dir_all(scratch.path().join("files")).expect("files");
    m.plan.options.files = Some(scratch.path().join("files"));
    fs::write(scratch.path().join("files/a.txt"), b"x\n").expect("a");
    fs::write(scratch.path().join("files/b.txt"), b"y").expect("b");
    let log = format!(
        "1970-01-01T00:00:00Z box: Forgejo elsewhere admin token file /older/missing.token\n{LOG}1970-01-01T00:00:02Z box: Forgejo elsewhere admin token file {}\n",
        token_file(scratch.path()).display()
    );
    fs::write(scratch.path().join("session/box.log"), log).expect("fake log");
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(token_file(scratch.path()))
        .expect("fake token file");
    writeln!(file, "{TOKEN}").expect("fake token");
    (scratch, m)
}
fn token_file(root: &Path) -> std::path::PathBuf {
    root.join("forgejo-admin.token")
}
fn at(seconds: u64) -> Time {
    Time::ZERO.saturating_add(Duration::from_secs(seconds))
}
fn logged(m: &mut Machine, actions: &[Action]) -> Vec<Action> {
    let [Action::ReadBoxLog { path }] = actions else { panic!("{actions:?}") };
    assert!(path.ends_with("session/box.log"));
    let actions = m.step(Event::BoxLog(adapter::read_log(path)), Time::ZERO);
    if let [Action::ReadForgeFiles { directory, paths }] = actions.as_slice() {
        m.step(Event::ForgeFiles(adapter::read_files(directory, paths)), Time::ZERO)
    } else {
        actions
    }
}
pub(crate) fn begin(m: &mut Machine, source: &str) -> Vec<Action> {
    let actions = line(m, source);
    logged(m, &actions)
}
fn safe(actions: &[Action]) {
    assert!(!format!("{actions:?}").contains(TOKEN), "credential crossed the io boundary");
    assert!(!actions.iter().any(|a| matches!(a, Action::Log { .. })), "forge requests are never logged");
}
fn request(actions: &[Action], expected: &str) -> adapter::Prepared {
    safe(actions);
    let [Action::ForgeRequest { api, token_file, method, path, body, until }] = actions else { panic!("{actions:?}") };
    assert_eq!(api.as_ref(), b"http://127.0.0.1:8002/api/v1");
    assert_eq!(path.as_ref(), expected.as_bytes());
    assert!(token_file.ends_with("forgejo-admin.token"));
    assert!(*until == at(60) || *until == at(300));
    let prepared = adapter::prepare(api, token_file, *method, path, body.clone()).expect("forge request");
    assert_eq!(prepared.address.port(), 8002);
    assert_eq!(prepared.call.target.as_ref(), format!("/api/v1{expected}").as_bytes());
    assert_eq!(
        prepared.call.headers.iter().find(|h| h.is(b"Authorization")).expect("write header").value.as_ref(),
        format!("token {TOKEN}").as_bytes()
    );
    prepared
}
/// A fake socket, not a process or network: it honours each read/room demand.
fn exchange(prepared: adapter::Prepared, status: u16, body: &[u8]) -> Event {
    use skein_io::{Event as IoEvent, Request};
    let owner = Token::new(77);
    let socket = Token::new(78);
    let expected_upload = prepared.call.body;
    let (mut connection, requests) = adapter::Connection::start(owner, prepared);
    let mut pending: VecDeque<_> = requests.into();
    let mut input = Held::new(
        [format!("HTTP/1.1 {status} Test\r\nContent-Length: {}\r\n\r\n", body.len()).as_bytes(), body].concat().into(),
    );
    let mut sent = vec![];
    let mut result = None;
    for _ in 0..10000 {
        let Some(request) = pending.pop_front() else { break };
        let events = match request {
            Request::Connect { .. } => vec![IoEvent::Connecting { owner, socket }, IoEvent::Connected { owner }],
            Request::Stream { down: Down::Send(bytes), .. } => {
                sent.extend_from_slice(&bytes);
                vec![]
            }
            Request::Stream { down: Down::Demand { read, room }, .. } => {
                if room > 0 {
                    vec![IoEvent::Stream { owner, up: Up::Room }]
                } else {
                    input.answer(read).map(|up| IoEvent::Stream { owner, up }).into_iter().collect()
                }
            }
            Request::Stream { down: Down::Finish, .. } => vec![],
            Request::Close { entity } => {
                assert_eq!(entity, socket);
                vec![IoEvent::Closed { owner }]
            }
            other => panic!("unexpected io: {other:?}"),
        };
        for event in events {
            let (event, requests) = connection.event(event, Time::ZERO);
            if let Some(event) = event {
                assert!(result.replace(event).is_none(), "one terminal response");
            }
            pending.extend(requests);
        }
    }
    assert!(connection.settled && pending.is_empty());
    assert!(
        sent.windows(format!("Authorization: token {TOKEN}\r\n").len())
            .any(|part| part == format!("Authorization: token {TOKEN}\r\n").as_bytes())
    );
    if let skein_http::client::Body::Length(length) = expected_upload {
        let at = sent.windows(4).position(|bytes| bytes == b"\r\n\r\n").expect("request head") + 4;
        assert_eq!(u64::try_from(sent.len() - at).expect("body length"), length);
    }
    result.expect("forge response")
}
pub(crate) fn response(
    m: &mut Machine,
    actions: &[Action],
    path: &str,
    status: u16,
    body: &[u8],
    seconds: u64,
) -> Vec<Action> {
    let prepared = request(actions, path);
    let event = exchange(prepared, status, body);
    let actions = m.step(event, at(seconds));
    safe(&actions);
    actions
}
fn status(state: &str) -> Vec<u8> {
    format!(r#"{{"state":"{state}","sha":"{SHA}","statuses":[{{"context":"gate","status":"{state}"}},{{"context":"lint","status":"success"}}]}}"#).into_bytes()
}
#[test]
fn forge_commit_plan_uses_newest_api_and_one_contents_commit() {
    let (_scratch, mut m) = fixture();
    let actions = begin(&mut m, COMMIT);
    let actions = response(&mut m, &actions, "/repos/playground/widget", 200, br#"{"default_branch":"main"}"#, 0);
    let actions = response(&mut m, &actions, "/repos/playground/widget/contents/a.txt?ref=main", 404, b"absent", 0);
    let actions =
        response(&mut m, &actions, "/repos/playground/widget/contents/b.txt?ref=main", 200, br#"{"sha":"blob"}"#, 0);
    let prepared = request(&actions, "/repos/playground/widget/contents");
    assert_eq!(prepared.call.method, Method::Post);
    let [Action::ForgeRequest { body, .. }] = actions.as_slice() else { panic!("POST") };
    assert_eq!(body.as_ref(), br#"{"branch":"main","message":"msg","files":[{"operation":"create","path":"a.txt","content":"eAo="},{"operation":"update","path":"b.txt","sha":"blob","content":"eQ=="}]}"#);
    let event = exchange(prepared, 201, format!(r#"{{"commit":{{"sha":"{SHA}"}}}}"#).as_bytes());
    let actions = m.step(event, Time::ZERO);
    safe(&actions);
    assert_eq!(answer(&actions).print(), format!("| {SHA}\nok\n"));
    assert!(
        matches!(actions.as_slice(), [Action::Keep(kept), Action::Answer(_)] if kept.as_ref() == COMMIT.as_bytes())
    );
    assert!(m.ready());
}
#[test]
fn forge_checks_poll_two_seconds_apart_on_logic_deadlines() {
    let (_scratch, mut m) = fixture();
    let actions = begin(&mut m, "forge checks forgejo playground/widget");
    let mut actions = response(&mut m, &actions, "/repos/playground/widget", 200, br#"{"default_branch":"main"}"#, 0);
    for seconds in [0, 2] {
        actions = response(
            &mut m,
            &actions,
            "/repos/playground/widget/commits/main/status",
            200,
            &status("pending"),
            seconds,
        );
        assert!(actions.is_empty());
        assert_eq!(m.next_deadline(), Some(at(seconds + 2)));
        assert!(m.step(Event::Deadline, at(seconds + 1)).is_empty());
        actions = m.step(Event::Deadline, at(seconds + 2));
    }
    let actions =
        response(&mut m, &actions, "/repos/playground/widget/commits/main/status", 200, &status("success"), 4);
    assert_eq!(answer(&actions).print(), format!("| {SHA}\n| gate success\n| lint success\nok\n"));
    assert!(
        matches!(actions.as_slice(), [Action::Keep(kept), Action::Answer(_)] if kept.as_ref() == b"forge checks forgejo playground/widget")
    );
}
#[test]
fn forge_failed_checks_and_pending_bound_name_the_last_statuses() {
    for (state, empty) in [("failure", false), ("error", false), ("pending", false), ("pending", true)] {
        let (_scratch, mut m) = fixture();
        let actions = begin(&mut m, "forge checks forgejo playground/widget");
        let actions = response(&mut m, &actions, "/repos/playground/widget", 200, br#"{"default_branch":"main"}"#, 0);
        let body = if empty {
            format!(r#"{{"state":"pending","sha":"{SHA}","statuses":[]}}"#).into_bytes()
        } else {
            status(state)
        };
        let mut actions = response(&mut m, &actions, "/repos/playground/widget/commits/main/status", 200, &body, 0);
        if state == "pending" {
            assert!(actions.is_empty());
            actions = m.step(Event::Deadline, at(300));
            assert!(actions.iter().any(|a| matches!(a, Action::CancelForge)));
        }
        let why = answer(&actions).failed.as_deref().expect("failed line");
        assert!(why.contains(SHA), "{why}");
        if empty {
            assert!(why.contains("state pending") && why.contains("no statuses"), "{why}");
        } else {
            assert!(why.contains(&format!("gate {state}")) && why.contains("lint success"), "{why}");
        }
        if state == "pending" {
            assert!(why.contains("deadline"));
        }
        assert!(m.ready());
        assert_eq!(answer(&line(&mut m, "")).print(), "ok\n");
    }
}
#[test]
fn forge_missing_log_values_token_and_files_fail_only_the_line() {
    for missing in ["log", "api", "token", "admin", "files", "missing", "large", "default_branch"] {
        let (scratch, mut m) = fixture();
        match missing {
            "log" => fs::remove_file(scratch.path().join("session/box.log")).expect("remove log"),
            "api" => fs::write(scratch.path().join("session/box.log"), b"no api here\n").expect("log"),
            "token" => fs::remove_file(token_file(scratch.path())).expect("remove fake token"),
            "admin" => fs::write(scratch.path().join("session/box.log"), LOG).expect("log without token path"),
            "files" => m.plan.options.files = None,
            "missing" => fs::remove_file(scratch.path().join("files/a.txt")).expect("remove a"),
            "large" => fs::write(scratch.path().join("files/a.txt"), vec![b'x'; 65537]).expect("large a"),
            "default_branch" => {}
            _ => unreachable!(),
        }
        let mut actions = begin(&mut m, COMMIT);
        if missing == "token" {
            let [Action::ForgeRequest { api, token_file, method, path, body, .. }] = actions.as_slice() else {
                panic!("request")
            };
            let why = adapter::prepare(api, token_file, *method, path, body.clone()).err().expect("missing token");
            actions = m.step(Event::ForgeFailure(why), Time::ZERO);
        } else if missing == "default_branch" {
            actions = response(&mut m, &actions, "/repos/playground/widget", 200, b"{}", 0);
        }
        safe(&actions);
        let why = answer(&actions).failed.as_deref().expect("failed line");
        let expected = match missing {
            "token" => "forgejo-admin.token",
            "admin" => "box: Forgejo elsewhere admin token file",
            "files" => "files directory",
            "missing" | "large" => "a.txt",
            "log" => "session/box.log",
            "api" => "box: fake forgejo",
            _ => "default_branch",
        };
        assert!(why.contains(expected), "{why}");
        if missing == "large" {
            assert!(why.contains("64 KiB"), "{why}");
        }
        assert!(m.ready());
        assert_eq!(answer(&line(&mut m, "")).print(), "ok\n");
    }
}
#[test]
fn forge_escaped_credentials_are_redacted_and_tokens_are_read_afresh() {
    let (scratch, mut m) = fixture();
    let actions = begin(&mut m, "forge checks forgejo playground/widget");
    let actions = response(&mut m, &actions, "/repos/playground/widget", 200, br#"{"default_branch":"main"}"#, 0);
    let prepared = request(&actions, "/repos/playground/widget/commits/main/status");
    let escaped = TOKEN.replacen('-', "\\u002d", 1);
    let body =
        format!(r#"{{"state":"failure","sha":"{SHA}","statuses":[{{"context":"{escaped}","status":"failure"}}]}}"#);
    let event = exchange(prepared, 200, body.as_bytes());
    assert!(!format!("{event:?}").contains(TOKEN));
    let actions = m.step(event, Time::ZERO);
    safe(&actions);
    assert!(answer(&actions).failed.is_some());
    fs::write(token_file(scratch.path()), b"rotated-fake-value\n").expect("rotate fake token");
    let prepared = adapter::prepare(
        b"http://127.0.0.1:8002/api/v1",
        &token_file(scratch.path()),
        Method::Get,
        b"/repos/playground/widget",
        Box::new([]),
    )
    .expect("reread token");
    assert_eq!(
        prepared.call.headers.iter().find(|header| header.is(b"Authorization")).expect("header").value.as_ref(),
        b"token rotated-fake-value"
    );
}

#[test]
fn forge_refused_post_is_cut_and_the_token_never_leaves_io() {
    let (_scratch, mut m) = fixture();
    let source = r#"forge commit forgejo playground/widget "msg" "a.txt""#;
    let actions = begin(&mut m, source);
    let actions = response(&mut m, &actions, "/repos/playground/widget", 200, br#"{"default_branch":"main"}"#, 0);
    let actions = response(&mut m, &actions, "/repos/playground/widget/contents/a.txt?ref=main", 404, b"", 0);
    let body = format!("{TOKEN} {}NOT-KEPT", "x".repeat(5000));
    let actions = response(&mut m, &actions, "/repos/playground/widget/contents", 422, body.as_bytes(), 0);
    let why = answer(&actions).failed.as_deref().expect("refused POST");
    assert!(why.contains("POST /repos/playground/widget/contents: HTTP 422"), "{why}");
    assert_eq!(
        why.split("HTTP 422: ").nth(1).expect("refusal body"),
        format!("[redacted] {}", "x".repeat(4096 - TOKEN.len() - 1))
    );
    assert!(!why.contains("NOT-KEPT") && why.len() < 4200);
    assert!(
        matches!(actions.as_slice(), [Action::Keep(kept), Action::Answer(_)] if kept.as_ref() == source.as_bytes())
    );
    assert!(m.ready());
    assert_eq!(answer(&line(&mut m, "")).print(), "ok\n");
}

#[test]
fn ctl_box_values_resolve_newest_opening_lines_without_changing_kept_bytes() {
    let (scratch, mut m) = fixture();
    let source = "ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello";
    for restarted in [false, true] {
        let mut log = "1970-01-01T00:00:00Z box: effective data /s/data/box/fake-backends/data\n1970-01-01T00:00:00Z box: fake forgejo http://127.0.0.1:4321\n1970-01-01T00:00:00Z box: Forgejo elsewhere admin token file /s/data/box/fake-backends/data/forgejo-admin.token\n".to_owned();
        if restarted {
            log.push_str("1970-01-01T00:00:01Z box: fake forgejo http://127.0.0.1:4322\n1970-01-01T00:00:02Z box: Forgejo elsewhere admin token file /new/admin.token\n1970-01-01T00:00:03Z box: effective data /new/data\n");
        }
        fs::write(scratch.path().join("session/box.log"), log).expect("opening lines");
        let actions = begin(&mut m, source);
        let [Action::Start { process: crate::drive::Process::Ctl, program }] = actions.as_slice() else {
            panic!("{actions:?}")
        };
        let (token, api, git, binary) = if restarted {
            ("/new/admin.token", "127.0.0.1:4322", "http://127.0.0.1:4322", "/new/data/demo/bin/git")
        } else {
            (
                "/s/data/box/fake-backends/data/forgejo-admin.token",
                "127.0.0.1:4321",
                "http://127.0.0.1:4321",
                "/s/data/box/fake-backends/data/demo/bin/git",
            )
        };
        let expected: Vec<std::ffi::OsString> = [
            "ctl",
            &scratch.path().join("session/ctl").to_string_lossy(),
            "forge",
            "connect",
            "forgejo-admin",
            token,
            "--first",
            "work/hello",
            "--project",
            "Hello",
            "--api",
            api,
            "--git",
            git,
            "--git-program",
            binary,
            "--key",
            "qa-hello",
        ]
        .map(Into::into)
        .into();
        assert_eq!(program.args, expected);
        assert_eq!(program.input, crate::drive::Input::Null);
        let actions = m.step(super::exited(crate::drive::Process::Ctl, 0), Time::ZERO);
        assert!(
            matches!(actions.as_slice(), [Action::Keep(kept), Action::Answer(_)] if kept.as_ref() == source.as_bytes())
        );
    }
}

#[test]
fn ctl_missing_box_value_fails_only_its_line_and_names_the_opening_line() {
    let (scratch, mut m) = fixture();
    for (name, expected) in [
        ("{forgejo.api}", "box: fake forgejo"),
        ("{forgejo.git}", "box: fake forgejo"),
        ("{forgejo.admin}", "box: Forgejo elsewhere admin token file"),
        ("{git}", "box: effective data"),
    ] {
        fs::write(scratch.path().join("session/box.log"), b"unrelated\n").expect("log");
        let source = format!("ctl show {name}");
        let actions = begin(&mut m, &source);
        let why = answer(&actions).failed.as_deref().expect("missing opening line");
        assert!(why.contains(expected), "{why}");
        assert!(
            matches!(actions.as_slice(), [Action::Keep(kept), Action::Answer(_)] if kept.as_ref() == source.as_bytes())
        );
        assert!(m.ready());
        assert_eq!(answer(&line(&mut m, "")).print(), "ok\n");
    }
}

pub(crate) fn select_files(machine: &mut Machine, directory: Option<std::path::PathBuf>) {
    machine.plan.options.files = directory;
}

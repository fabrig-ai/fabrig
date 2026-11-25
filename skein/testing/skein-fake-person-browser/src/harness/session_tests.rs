//! Completion sorting keeps io's queue across polls, and Chromium's spawn
//! is prepared without a kernel.
use super::{BROWSER, Opening, PAGE, PERSON, Pump, ROOT_CLOSE, Report, page_report, take_root_close};
use skein_browser::boundary::Trouble;
use skein_browser::{Below, Browser, Event, Limits, up};
use skein_io::{
    self as io,
    kernel::{Complete, Done, Error, Exit, Fd, Op},
};
use skein_lib::{Env, Queue, Time, Token, Wall, stream::Up};
use std::{collections::VecDeque, path::Path};

#[test]
fn page_trouble_reports_kind_text_and_url_without_judging() {
    let text: Box<[u8]> = b"status of 401 \xff".as_slice().into();
    let url: Box<[u8]> = b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=sample#part".as_slice().into();
    let event = Event::Trouble { page: Token::new(100), trouble: Trouble::Log, text: text.clone(), url: url.clone() };
    let report = page_report(event).expect("page observation");
    let Report::Page { trouble, text: reported_text, url: reported_url } = report else { panic!("{report:?}") };
    assert_eq!(trouble, Trouble::Log);
    assert_eq!(reported_text, text);
    assert_eq!(reported_url, url);
    let event = Event::Opened { page: Token::new(100) };
    assert!(matches!(page_report(event), Err(Event::Opened { .. })));
}

#[test]
fn the_session_page_asks_for_requests() {
    let url: Box<[u8]> = b"http://127.0.0.1:9/".as_slice().into();
    let skein_browser::boundary::Request::Page { person, page, url: opened, requests } =
        super::session_page(url.clone())
    else {
        panic!("Page")
    };
    assert_eq!(person, super::PERSON);
    assert_eq!(page, super::PAGE);
    assert_eq!(opened, url);
    assert!(requests);
}

#[test]
fn page_requests_report_every_field_without_judging() {
    let sent = Event::Sent {
        page: Token::new(100),
        id: b"7.1".as_slice().into(),
        load: b"L1".as_slice().into(),
        document: true,
        method: b"POST".as_slice().into(),
        url: b"http://127.0.0.1:9/s?x=1#part".as_slice().into(),
    };
    let Report::Sent { id, load, document, method, url } = page_report(sent).expect("Sent report") else {
        panic!("Sent")
    };
    assert_eq!(id.as_ref(), b"7.1");
    assert_eq!(load.as_ref(), b"L1");
    assert!(document);
    assert_eq!(method.as_ref(), b"POST");
    assert_eq!(url.as_ref(), b"http://127.0.0.1:9/s?x=1#part");
    let answered = Event::Answered { page: Token::new(100), id: b"7.1".as_slice().into() };
    let Report::Answered { id } = page_report(answered).expect("Answered report") else { panic!("Answered") };
    assert_eq!(id.as_ref(), b"7.1");
}

// Three ordinary completions and the root close fill the queue.
const CAPACITY: u32 = 4;

fn completion(op: Token, result: Result<Done, Error>) -> Complete {
    Complete { op, kind: Op::Cancel { target: Token::new(0) }, result }
}

fn ordinary(op: u64) -> Complete {
    completion(Token::new(op), Ok(Done::Nothing))
}

#[test]
fn a_full_queue_keeps_every_ordinary_completion_in_order() {
    let mut complete = Queue::with_capacity(CAPACITY);
    for entry in [ordinary(1), completion(ROOT_CLOSE, Err(Error::Other(5))), ordinary(2), ordinary(3)] {
        complete.push(entry);
    }
    let mut reports = Vec::new();

    take_root_close(&mut complete, &mut reports);

    assert!(complete.iter().eq([ordinary(1), ordinary(2), ordinary(3)].iter()), "ordinary completions stay in order");
    assert_eq!(reports.len(), 1);
    let Report::Trouble { why, chromium_stderr: None } = &reports[0] else {
        panic!("root close failure is Trouble: {reports:?}");
    };
    assert_eq!(why, "closing session root: Other(5)");
}

#[test]
fn a_successful_root_close_makes_no_report() {
    let mut complete = Queue::with_capacity(CAPACITY);
    complete.push(completion(ROOT_CLOSE, Ok(Done::Nothing)));
    let mut reports = Vec::new();

    take_root_close(&mut complete, &mut reports);

    assert!(complete.is_empty());
    assert!(reports.is_empty(), "a successful close is silent: {reports:?}");
}

#[test]
fn another_sort_keeps_leftovers_before_newly_delivered_completions() {
    let mut complete = Queue::with_capacity(CAPACITY);
    complete.push(ordinary(1));
    complete.push(ordinary(2));
    let mut reports = Vec::new();
    take_root_close(&mut complete, &mut reports);

    // io's stage leaves these entries; the next delivery fills only the room left.
    complete.push(completion(ROOT_CLOSE, Ok(Done::Nothing)));
    complete.push(ordinary(3));
    take_root_close(&mut complete, &mut reports);

    assert!(complete.iter().eq([ordinary(1), ordinary(2), ordinary(3)].iter()), "leftovers precede new completions");
    assert!(reports.is_empty());
}

#[test]
fn a_spawn_directory_is_given_beneath_the_root() {
    // The root is `/`, and a spawn opens its directory beneath it, refusing an absolute path.
    for (path, beneath) in [
        ("/home/person/scratch", "home/person/scratch"),
        ("//home/person", "home/person"),
        ("/", ""),
        ("profile", "profile"),
    ] {
        assert_eq!(&*Pump::beneath(Path::new(path)), beneath.as_bytes(), "{path}");
    }
}

#[test]
fn the_kit_ending_and_its_stderr_are_reported_instead_of_the_owner_kill() {
    const CHILD: Token = Token::new(50);
    const OWNER_KILL: Exit = Exit::Signal(9);
    const TAIL: &[u8] = b"late\nlast\n";
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: Limits { stderr: 10, ..Limits::default() } };
    let mut browser = Browser::new(BROWSER, &env.limits);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    for line in [b"discard me\n".as_slice(), b"late\n", b"last\n"] {
        up(&mut browser, &env, Below::Errors(Up::Bytes(line.into())), &mut above, &mut below);
    }
    up(&mut browser, &env, Below::Replies(Up::End), &mut above, &mut below);
    let Some(Event::Ended { why, detail }) = above.pop() else { panic!("kit ending") };

    let mut opening = Opening::Open;
    let mut reports = Vec::new();
    let mut requests = VecDeque::new();
    opening.ended(why, &detail, &browser, &mut reports);
    for owner in [PAGE, PERSON, BROWSER] {
        opening.browser_closed(owner, Some(CHILD), &mut requests, &mut reports);
    }
    assert!(matches!(requests.pop_front(), Some(io::Request::Close { entity: CHILD })));
    assert!(requests.is_empty(), "only the child's close is asked for");
    opening.exited(OWNER_KILL, &browser, &mut reports);

    let [Report::Trouble { why, chromium_stderr: Some(tail) }] = reports.as_slice() else {
        panic!("one browser trouble, got {reports:?}");
    };
    assert_eq!(why, "browser ended: pipe: replies pipe: end");
    assert_eq!(tail.as_ref(), TAIL);
}

#[test]
fn only_an_unasked_chromium_exit_names_the_status() {
    const EXIT: Exit = Exit::Code(19);
    let browser = Browser::new(BROWSER, &Limits::default());
    let mut reports = Vec::new();
    let mut spontaneous = Opening::Open;
    spontaneous.exited(EXIT, &browser, &mut reports);
    let [Report::Trouble { why, chromium_stderr: Some(tail) }] = reports.as_slice() else {
        panic!("one unsolicited exit, got {reports:?}");
    };
    assert_eq!(why, "Chromium exited: Code(19)");
    assert!(tail.is_empty());

    reports.clear();
    let mut closing = Opening::Closing;
    closing.exited(Exit::Code(0), &browser, &mut reports);
    assert!(reports.is_empty(), "an expected close is silent");
}

#[test]
fn chromium_keeps_home_but_not_the_session_tmpdir() {
    let spawn = Pump::chromium_spawn(
        Path::new("/usr/bin/chromium"),
        Path::new("/scratch/session/browser"),
        &[("HOME".into(), "/scratch/session/home".into()), ("TMPDIR".into(), "/scratch/session/tmp".into())],
        Fd::new(0),
    );

    assert!(spawn.env.iter().all(|entry| !entry.starts_with(b"TMPDIR=")), "Chromium uses the host's temp directory");
    assert!(spawn.env.iter().any(|entry| entry.as_ref() == b"HOME=/scratch/session/home"));
}

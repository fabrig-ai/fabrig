//! Tell's filing through the root and its fake engine.
use crate::{Action, Domain, Event, Request, pages::{self, tell::{State, Tell}}, tests::fixtures as f};
use jig_client::{Down, Up, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn setup() -> Domain {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::uncapped());
    super::welcome::deliver(&mut root, me, c::Delivery::Me(Box::new(f::chats::me())));
    root
}
fn tell(root: &mut Domain, words: &[u8]) -> Queue<Request> {
    super::welcome::event(root, Event::Act { action: Action::Pages(pages::Act::Tell { project: 1, words: words.into() }) })
}
fn sent(out: &Queue<Request>) -> (Token, w::FileFinding) {
    let mut sent = List::uncapped();
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down {
            down: Down::Send { attempt, request: jig_client::Ask::Family { scope, record }, .. },
        }} = request {
            assert_eq!(*scope, c::Scope::Project(1));
            assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
            let request = wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap();
            let w::Request::FileFinding(filing) = request.request() else { panic!("FileFinding") };
            sent.push((*attempt, filing.clone())).unwrap();
        }
    }
    assert_eq!(sent.len(), 1, "one filing");
    sent.swap_remove(0).unwrap()
}
fn answer(root: &mut Domain, attempt: Token, outcome: Option<w::Outcome>) {
    let answer = match outcome {
        Some(outcome) => Some(jig_client::Answered::Family(jig_client::Family {
            family: wire::FORGE, version: wire::CLIENT_VERSION,
            bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
        })),
        None => None,
    };
    super::welcome::event(root, Event::Client { event: jig_client::Event::Up { up: Up::Answer {
        attempt, envelope: skein_lib::RequestEnvelope::Final, answer, why: None,
    }}});
}

#[test]
fn tell_files_trimmed_feedback_once_and_keeps_each_outcome() {
    let mut root = setup();
    assert!(root.pages().tell().is_none());
    assert!(tell(&mut root, b" \n\t ").is_empty());
    assert!(root.pages().tell().is_none(), "empty words keep no state");
    let body = b"The export button is grey on dark.\nIt should show.";
    let (attempt, request) = sent(&tell(&mut root, b"  The export button is grey on dark.\nIt should show.  "));
    assert_eq!(request.project(), 1);
    assert_eq!(request.kind(), &w::FindingKind::Feedback);
    assert_eq!(request.severity(), &w::Severity::Medium);
    assert_eq!(request.area(), &None);
    assert_eq!(request.title(), b"The export button is grey on dark.");
    assert_eq!(request.body(), body);
    assert_eq!(root.pages().tell(), Some(&Tell { serial: 1, project: 1, words: body.as_slice().into(), state: State::Sending }));
    assert!(tell(&mut root, b"Another").is_empty(), "one in flight");
    assert_eq!(root.pages().tell().unwrap().serial, 1);
    answer(&mut root, attempt, Some(w::Outcome::Filed(w::Filed::new(&w::CEILINGS, w::FiledParts { number: 12 }).unwrap())));
    assert_eq!(root.pages().tell().unwrap().state, State::Filed { number: 12 });
    assert!(tell(&mut root, b" \n ").is_empty());
    assert_eq!(root.pages().tell().unwrap().serial, 1, "empty words keep last outcome");
    let (attempt, _) = sent(&tell(&mut root, b"Another"));
    assert_eq!(root.pages().tell().unwrap().serial, 2);
    let refusal = w::FindingRefusal::Full(w::FindingsFull::new(&w::CEILINGS, w::FindingsFullParts { count: 128 }).unwrap());
    answer(&mut root, attempt, Some(w::Outcome::FindingRefused(w::FindingRefused::new(&w::CEILINGS, w::FindingRefusedParts { reason: refusal.clone() }).unwrap())));
    assert_eq!(root.pages().tell().unwrap().state, State::Refused(refusal));
}

#[test]
fn tell_loses_an_ended_or_unknown_request_and_can_send_again() {
    for outcome in [None, Some(w::Outcome::Unknown)] {
        let mut root = setup();
        let (attempt, _) = sent(&tell(&mut root, b"Something off"));
        answer(&mut root, attempt, outcome);
        assert_eq!(root.pages().tell().unwrap().state, State::Lost);
        sent(&tell(&mut root, b"Try again"));
        assert_eq!(root.pages().tell().unwrap().serial, 2);
    }
}

#[test]
fn tell_bounds_the_body_and_cuts_its_title_at_a_character() {
    let mut root = setup();
    assert!(tell(&mut root, &[b'x'; 4097]).is_empty());
    let last = root.pages().tell().unwrap();
    assert_eq!(last.serial, 1);
    assert_eq!(last.words.len(), 4097);
    assert_eq!(last.state, State::Long { over: 1 });
    let (attempt, request) = sent(&tell(&mut root, &[b'x'; 4096]));
    assert_eq!(request.body().len(), 4096);
    assert_eq!(request.title().len(), 200);
    assert_eq!(root.pages().tell().unwrap().serial, 2);
    answer(&mut root, attempt, Some(w::Outcome::Unknown));
    for (prefix, length) in [(198, 200), (199, 199)] {
        let body = skein_lib::bytes::concat(&["x".repeat(prefix).as_bytes(), "éz\nNext".as_bytes()]);
        let (attempt, request) = sent(&tell(&mut root, body.as_ref()));
        assert_eq!(request.title().len(), length, "two-byte character at {prefix}");
        assert!(skein_lib::bytes::is_utf8(request.title()));
        assert_eq!(request.body(), body.as_ref());
        answer(&mut root, attempt, Some(w::Outcome::Unknown));
    }
}

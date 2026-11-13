use super::fixtures;
use crate::{Domain, Event, Request, asked::State};
use jig_client::{Approval, ApprovalAct, ApprovalCode, Message, NotAsked};
use jig_client_protocol_wire::v21 as w;
use skein_lib::{List, Queue, Reader, Secret, Token, Wall, Writer};
fn code() -> ApprovalCode {
    ApprovalCode::parse(b"WDJBMJHT").unwrap()
}
fn step(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &fixtures::env(&fixtures::limits()), event, &mut out);
    out
}
fn signed_out(unlinked: bool) -> Domain {
    let mut root =
        if unlinked { crate::unlinked(&fixtures::limits(), 11) } else { crate::new(&fixtures::limits(), 11) };
    step(&mut root, Event::Start { address: crate::Address::Accounts, kept: List::with_capacity(0), with: None });
    if unlinked {
        step(&mut root, Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } });
        step(&mut root, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } });
    }
    assert_eq!(root.client().link().state, jig_client::Link::SignedOut);
    root
}
fn exchange(out: &mut Queue<Request>) -> (Token, Message) {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Exchange { exchange, credential, message, .. } } =
            request
        {
            assert!(credential.is_none());
            assert!(found.replace((exchange, message)).is_none());
        }
    }
    found.expect("one exchange")
}
fn encoded_ask(answer: &w::ApprovalAskAnswer) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut writer).unwrap();
    writer.finish()
}
#[expect(clippy::disallowed_macros, reason = "the test formats root Debug to detect secret bytes")]
fn no_secret(root: &Domain) {
    for debug in [format!("{root:?}"), format!("{root:#?}")] {
        assert!(!debug.contains("7, 7, 7"), "secret's byte list leaked");
        assert!(!debug.contains("\\u{7}\\u{7}\\u{7}"), "secret's escaped text leaked");
        assert!(!debug.contains("\x07\x07\x07"), "secret's text leaked");
    }
}
#[test]
fn an_ask_while_signed_in_or_in_flight_changes_nothing() {
    let mut root = fixtures::on_adoption_page();
    assert!(step(&mut root, Event::AskApproval).is_empty());
    assert_eq!(root.asked(), None);
    let mut root = signed_out(false);
    let mut out = step(&mut root, Event::AskApproval);
    let (_, message) = exchange(&mut out);
    let Message::ApprovalAsk(_) = message else { panic!("approval ask") };
    assert_eq!(root.asked(), Some(State::Asking));
    let shown = crate::shown(&root);
    assert!(step(&mut root, Event::AskApproval).is_empty());
    assert_eq!(crate::shown(&root), shown);
}
#[test]
fn an_answered_ask_signs_in_with_its_secret_and_keeps_the_code_and_expiry() {
    let mut root = signed_out(false);
    let mut out = step(&mut root, Event::AskApproval);
    let (id, message) = exchange(&mut out);
    let Message::ApprovalAsk(body) = message else { panic!("ask") };
    w::ApprovalAsk::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap();
    let answer = w::ApprovalAskAnswer::Asked(
        w::ApprovalAsked::new(
            &w::CEILINGS,
            w::ApprovalAskedParts { secret: Secret::new(Box::new([7; 32])), code: *code().letters(), expires: 300 },
        )
        .unwrap(),
    );
    let mut out = step(
        &mut root,
        Event::Client { event: jig_client::Event::Reply { exchange: id, message: encoded_ask(&answer) } },
    );
    assert_eq!(root.asked(), Some(State::Asked { code: code(), expires: Wall::from_nanos(300) }));
    let (id, message) = exchange(&mut out);
    let Message::SignInApproval(body) = message else { panic!("immediate take") };
    let take = w::ApprovalSignIn::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap();
    assert_eq!(take.secret().expose(), &[7; 32]);
    no_secret(&root);
    let answer = w::ApprovalSignInAnswer::SignedIn(
        w::SignedIn::new(
            &w::CEILINGS,
            w::SignedInParts { token: Some(Secret::new(Box::new([9; 40]))), party: 73, expires: 300 },
        )
        .unwrap(),
    );
    let mut writer = Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut writer).unwrap();
    step(&mut root, Event::Client { event: jig_client::Event::Reply { exchange: id, message: writer.finish() } });
    assert_ne!(root.client().link().state, jig_client::Link::SignedOut);
    assert_eq!(root.asked(), None);
}
#[test]
fn not_asked_is_held_as_it_came() {
    for (answer, expected) in [
        (Some(w::ApprovalAskAnswer::Refused), NotAsked::Refused),
        (Some(w::ApprovalAskAnswer::NotReady), NotAsked::NotReady),
        (Some(w::ApprovalAskAnswer::Busy), NotAsked::Busy),
        (None, NotAsked::Lost),
    ] {
        let mut root = signed_out(false);
        let mut out = step(&mut root, Event::AskApproval);
        let (id, _) = exchange(&mut out);
        let event = match answer {
            Some(answer) => jig_client::Event::Reply { exchange: id, message: encoded_ask(&answer) },
            None => jig_client::Event::Lost { exchange: id },
        };
        let shown = crate::shown(&root);
        step(&mut root, Event::Client { event });
        assert_eq!(root.asked(), Some(State::NotAsked(expected)));
        assert!(crate::shown(&root) > shown);
    }
}
#[test]
fn an_unlinked_root_holds_the_code_and_leaves_the_take_to_its_host() {
    let mut root = signed_out(true);
    let mut out = step(&mut root, Event::AskApproval);
    assert_eq!(
        out.pop(),
        Some(Request::Unlinked { request: jig_client::UnlinkedRequest::ApprovalAct(ApprovalAct::Ask) })
    );
    assert!(out.is_empty());
    assert!(step(&mut root, Event::AskApproval).is_empty());
    let answer =
        Approval::Asked { secret: Secret::new(Box::new([7; 32])), code: code(), expires: Wall::from_nanos(300) };
    let out = step(&mut root, Event::Client { event: jig_client::Event::ApprovalAnswer { answer } });
    assert_eq!(root.asked(), Some(State::Asked { code: code(), expires: Wall::from_nanos(300) }));
    assert!(out.is_empty());
    no_secret(&root);
    step(&mut root, Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
    assert_eq!(root.asked(), None);
}

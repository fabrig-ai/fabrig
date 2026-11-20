use super::fixtures;
use crate::{Domain, Event, Request, asked::State};
use jig_client::{Approval, ApprovalAct, ApprovalCode, Message, NotAsked};
use jig_client_protocol_wire as w;
use skein_lib::{List, Queue, Reader, Secret, Token, Wall, Writer};
fn code() -> ApprovalCode {
    ApprovalCode::parse(b"WDJBMJHT").unwrap()
}
fn step(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &fixtures::env(&fixtures::limits()), event, &mut out);
    crate::settle(root, &fixtures::env(&fixtures::limits()), &mut out);
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
    for port in [None, Some(43210)] {
        let mut root = fixtures::on_adoption_page();
        assert!(step(&mut root, Event::AskApproval { port }).is_empty());
        assert_eq!(root.asked(), None);
        let mut root = signed_out(false);
        let mut out = step(&mut root, Event::AskApproval { port });
        let (_, message) = exchange(&mut out);
        let Message::ApprovalAsk(body) = message else { panic!("approval ask") };
        let ask = w::ApprovalAsk::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap();
        assert_eq!(*ask.port(), port);
        assert_eq!(root.asked(), Some(State::Asking));
        let shown = crate::shown(&root);
        assert!(step(&mut root, Event::AskApproval { port: Some(12345) }).is_empty());
        assert_eq!(crate::shown(&root), shown);
    }
}
#[test]
fn an_answered_ask_signs_in_with_its_secret_and_keeps_the_code_and_expiry() {
    for port in [None, Some(43210)] {
        let mut root = signed_out(false);
        let mut out = step(&mut root, Event::AskApproval { port });
        let (id, message) = exchange(&mut out);
        let Message::ApprovalAsk(body) = message else { panic!("ask") };
        w::ApprovalAsk::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap();
        let mut out = reply_ask(&mut root, id);
        assert_eq!(root.asked(), Some(State::Asked { code: code(), expires: Wall::from_nanos(300), port }));
        let (id, message) = exchange(&mut out);
        let Message::SignInApproval(body) = message else { panic!("immediate take") };
        let take = w::ApprovalSignIn::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap();
        assert_eq!(take.secret().expose(), &[7; 32]);
        assert!(take.grant().is_none());
        assert_eq!(root.asked_secret.is_some(), port.is_some());
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
        assert!(root.asked_secret.is_none());
        ignored_grant(&mut root);
    }
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
        let mut out = step(&mut root, Event::AskApproval { port: None });
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
    let mut out = step(&mut root, Event::AskApproval { port: Some(43210) });
    assert_eq!(
        out.pop(),
        Some(Request::Unlinked {
            request: jig_client::UnlinkedRequest::ApprovalAct(ApprovalAct::Ask { port: Some(43210) })
        })
    );
    assert!(out.is_empty());
    assert!(step(&mut root, Event::AskApproval { port: Some(43210) }).is_empty());
    let answer =
        Approval::Asked { secret: Secret::new(Box::new([7; 32])), code: code(), expires: Wall::from_nanos(300) };
    let out = step(&mut root, Event::Client { event: jig_client::Event::ApprovalAnswer { answer } });
    assert_eq!(root.asked(), Some(State::Asked { code: code(), expires: Wall::from_nanos(300), port: Some(43210) }));
    assert!(out.is_empty());
    no_secret(&root);
    assert!(root.asked_secret.is_none());
    ignored_grant(&mut root);
    step(&mut root, Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
    assert_eq!(root.asked(), None);
}

fn answered(root: &mut Domain, port: Option<u16>) -> Queue<Request> {
    let mut out = step(root, Event::AskApproval { port });
    let (id, _) = exchange(&mut out);
    reply_ask(root, id)
}
fn reply_ask(root: &mut Domain, id: Token) -> Queue<Request> {
    let answer = w::ApprovalAskAnswer::Asked(
        w::ApprovalAsked::new(
            &w::CEILINGS,
            w::ApprovalAskedParts { secret: Secret::new(Box::new([7; 32])), code: *code().letters(), expires: 300 },
        )
        .unwrap(),
    );
    step(root, Event::Client { event: jig_client::Event::Reply { exchange: id, message: encoded_ask(&answer) } })
}
fn ignored_grant(root: &mut Domain) {
    let shown = crate::shown(root);
    let asked = root.asked();
    let link = (root.client().link().state, root.client().link().not_signed_in);
    assert!(step(root, Event::Granted { grant: Secret::new(Box::new([8; 32])) }).is_empty());
    assert_eq!(root.asked(), asked);
    assert_eq!((root.client().link().state, root.client().link().not_signed_in), link);
    assert_eq!(crate::shown(root), shown);
}
#[test]
fn each_grant_takes_again_with_the_port_asks_own_secret() {
    let mut root = signed_out(false);
    let mut out = answered(&mut root, Some(43210));
    exchange(&mut out);
    let asked = root.asked();
    no_secret(&root);
    // The reader's copyable state has no secret, even in its Debug view.
    no_secret_state(asked.unwrap());
    for byte in [8, 10] {
        let mut out = step(&mut root, Event::Granted { grant: Secret::new(Box::new([byte; 32])) });
        let (_, message) = exchange(&mut out);
        let Message::SignInApproval(body) = message else { panic!("take with grant") };
        let take = w::ApprovalSignIn::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap();
        assert_eq!(take.secret().expose(), &[7; 32]);
        assert_eq!(take.grant().as_ref().unwrap().expose(), &[byte; 32]);
        assert_eq!(root.asked(), asked);
        no_secret(&root);
    }
}
#[expect(clippy::disallowed_macros, reason = "the test formats the reader's state to detect secret bytes")]
fn no_secret_state(state: State) {
    assert!(!format!("{state:?}").contains("7, 7, 7"));
}
#[test]
fn grants_without_a_linked_port_ask_do_nothing_and_replacing_it_drops_its_secret() {
    let mut root = signed_out(false);
    ignored_grant(&mut root);
    step(&mut root, Event::AskApproval { port: Some(43210) });
    ignored_grant(&mut root);
    let mut root = signed_out(false);
    answered(&mut root, None);
    assert!(root.asked_secret.is_none());
    ignored_grant(&mut root);
    let mut root = signed_out(false);
    answered(&mut root, Some(43210));
    assert!(root.asked_secret.is_some());
    let mut out = step(&mut root, Event::AskApproval { port: None });
    let (id, _) = exchange(&mut out);
    assert!(root.asked_secret.is_none());
    ignored_grant(&mut root);
    // Answer the replacement, which is a typed-code ask.
    reply_ask(&mut root, id);
    assert_eq!(root.asked(), Some(State::Asked { code: code(), expires: Wall::from_nanos(300), port: None }));
    assert!(root.asked_secret.is_none());
    ignored_grant(&mut root);
}

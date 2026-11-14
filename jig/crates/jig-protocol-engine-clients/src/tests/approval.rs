//! Browser approvals, their refusals and durable takes (jig/docs/design/protocol/client/sign-in.md).
use super::fixtures::{self as f, Exchange};
use crate::{Event, Message, Request, SecretOwner, SecretsAnswer, SecretsRequest, Settings};
use jig_client_protocol_wire::{ApprovalCode, v22 as w};
use jig_domain_people as people;
use skein_lib::{Duration, Reader, ReplyTo, Secret, Token, Wall, Writer, length::usize_of};

const SPAN: Duration = Duration::from_secs(600);
fn new() -> Exchange {
    Exchange::configured(f::env(), Settings { approval: true, ..f::settings() })
}
fn ask_message() -> Message {
    let value = w::ApprovalAsk::new(&f::wire(), w::ApprovalAskParts {}).expect("ask");
    let mut bytes = Writer::new(usize_of(value.measure()));
    value.encode(&mut bytes).expect("ask bytes");
    Message::ApprovalAsk(bytes.finish())
}
fn read_message(code: [u8; 8]) -> Message {
    let value = w::ApprovalRead::new(&f::wire(), w::ApprovalReadParts { code }).expect("read");
    let mut bytes = Writer::new(usize_of(value.measure()));
    value.encode(&mut bytes).expect("read bytes");
    Message::ApprovalRead(bytes.finish())
}
fn decide_message(code: [u8; 8], approve: bool) -> Message {
    let value = w::ApprovalDecide::new(&f::wire(), w::ApprovalDecideParts { code, approve }).expect("decide");
    let mut bytes = Writer::new(usize_of(value.measure()));
    value.encode(&mut bytes).expect("decision bytes");
    Message::ApprovalDecide(bytes.finish())
}
fn take_message(secret: &[u8]) -> Message {
    let value = w::ApprovalSignIn::new(&f::wire(), w::ApprovalSignInParts { secret: Secret::new(secret.into()) })
        .expect("take");
    let mut bytes = Writer::new(usize_of(value.measure()));
    value.encode(&mut bytes).expect("take bytes");
    Message::SignInApproval(bytes.finish())
}
fn finish(e: &mut Exchange, owner: Token) {
    assert!(e.component.up(&e.env, &mut e.out), "deliver response");
    let Some(Request::Transport(crate::ToTransport::Reply { exchange, .. })) = e.out.pop() else {
        panic!("transport response")
    };
    assert_eq!(exchange, owner);
    assert!(e.out.is_empty());
}
fn asked(e: &mut Exchange) -> w::ApprovalAsked {
    let owner = e.send(ask_message());
    let value = w::ApprovalAskAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("ask answer");
    let w::ApprovalAskAnswer::Asked(value) = value else { panic!("approval asked: {value:?}") };
    assert_eq!(value.secret().expose().len(), 32);
    assert!(ApprovalCode::parse(value.code()).is_some());
    assert_eq!(value.expires(), e.env.wall.as_nanos().saturating_add(SPAN.as_nanos()));
    assert!(e.out.is_empty(), "ask sends nothing to core or store");
    finish(e, owner);
    value
}
fn browser(e: &mut Exchange, message: Message) -> Token {
    let owner = e.send_as(message, true, Some(Secret::new(e.token.clone())));
    e.authenticate();
    owner
}
fn read(e: &mut Exchange, code: [u8; 8]) -> w::ApprovalState {
    let owner = browser(e, read_message(code));
    let value = w::ApprovalState::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("read answer");
    finish(e, owner);
    value
}
fn decide(e: &mut Exchange, code: [u8; 8], approve: bool) -> w::ApprovalDecided {
    let owner = browser(e, decide_message(code, approve));
    let value = w::ApprovalDecided::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("decide answer");
    finish(e, owner);
    value
}
fn take_answer(e: &mut Exchange, secret: &[u8]) -> w::ApprovalSignInAnswer {
    let owner = e.send(take_message(secret));
    let value =
        w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("take answer");
    finish(e, owner);
    value
}
fn identity() -> people::Identity {
    people::Identity {
        key: people::IdentityKey { provider: 19, subject: Box::from(b"approver".as_slice()) },
        login: Box::from(b"login".as_slice()),
        name: Box::from(b"Person".as_slice()),
    }
}
fn signer(e: &mut Exchange) -> ReplyTo {
    let to = e.approval_whose();
    e.reply(to, people::Reply::Signer(Some(people::Signer { kind: people::Kind::Person, identity: identity() })));
    e.approval_sign_in(&identity())
}
fn signed_in(e: &mut Exchange, to: ReplyTo, owner: Token) -> w::SignedIn {
    e.reply(to, people::Reply::SignedIn { person: 72, sign_in: 91, expires: Wall::from_nanos(900_000_000_000) });
    assert!(e.component.connections.get(&owner).expect("connection").response.is_none(), "wait for Keep");
    let Some(Request::Secrets(SecretsRequest::Keep { owner: kept_owner, value })) = e.out.pop() else { panic!("Keep") };
    assert_eq!(kept_owner, SecretOwner::SignIn(91));
    assert!(e.out.is_empty());
    e.component.down(&e.env, Event::Secrets(SecretsAnswer::Kept { owner: kept_owner }), &mut e.out);
    let answer = w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer");
    let w::ApprovalSignInAnswer::SignedIn(answer) = answer else { panic!("signed in") };
    let token = answer.token().as_ref().expect("fresh token");
    assert_eq!(token.expose().len(), 40);
    assert_eq!(answer.party(), 72);
    assert_eq!(answer.expires(), value.expires.as_nanos());
    assert_eq!(crate::sign_in::digest(token.expose()), value.digest);
    finish(e, owner);
    answer
}
fn approved(e: &mut Exchange) -> w::ApprovalAsked {
    let value = asked(e);
    assert_eq!(decide(e, *value.code(), true), w::ApprovalDecided::Approved);
    value
}

#[test]
fn an_ask_is_answered_by_the_room_and_the_engine() {
    for case in ["asked", "off", "stopped", "full", "expired frees room", "refused frees room"] {
        let mut e = new();
        match case {
            "asked" => {
                asked(&mut e);
            }
            "off" | "stopped" => {
                if case == "off" {
                    e = Exchange::new();
                } else {
                    e.component.down(&e.env, Event::Stop, &mut e.out);
                }
                let owner = e.send(ask_message());
                let answer =
                    w::ApprovalAskAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer");
                assert_eq!(
                    answer,
                    if case == "off" { w::ApprovalAskAnswer::Refused } else { w::ApprovalAskAnswer::NotReady }
                );
                assert!(e.out.is_empty());
            }
            "full" | "expired frees room" | "refused frees room" => {
                let first = asked(&mut e);
                for _ in 0_u32..3 {
                    let other = asked(&mut e);
                    assert_ne!(first.code(), other.code());
                }
                let owner = e.send(ask_message());
                assert_eq!(
                    w::ApprovalAskAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer"),
                    w::ApprovalAskAnswer::Busy
                );
                finish(&mut e, owner);
                if case == "expired frees room" {
                    e.env.now = e.env.now.saturating_add(SPAN);
                    asked(&mut e);
                } else if case == "refused frees room" {
                    assert_eq!(decide(&mut e, *first.code(), false), w::ApprovalDecided::Refused);
                    asked(&mut e);
                }
            }
            _ => unreachable!(),
        }
    }
}
#[test]
fn only_a_browser_reads_or_decides() {
    for deciding in [false, true] {
        for case in ["bearer", "none", "bad digest", "expired credential"] {
            let mut e = new();
            let message = if deciding { decide_message(*b"BBBBBBBB", true) } else { read_message(*b"BBBBBBBB") };
            let credential = if case == "none" { None } else { Some(Secret::new(e.token.clone())) };
            let owner = e.send_as(message, case != "bearer", credential);
            if case == "bad digest" || case == "expired credential" {
                let Some(Request::Secrets(SecretsRequest::Get { owner })) = e.out.pop() else {
                    panic!("credential lookup")
                };
                let value = if case == "bad digest" {
                    None
                } else {
                    Some(crate::Kept { digest: crate::sign_in::digest(&e.token), expires: e.env.wall })
                };
                e.component.down(&e.env, Event::Secrets(SecretsAnswer::Got { owner, value }), &mut e.out);
            }
            if case == "bearer" {
                assert!(e.out.is_empty(), "bearer must be refused before credential lookup");
            }
            let bytes = e.response(owner).1;
            if deciding {
                assert_eq!(
                    w::ApprovalDecided::decode(&f::wire(), &mut Reader::new(bytes)).expect("answer"),
                    if case == "bearer" { w::ApprovalDecided::NotABrowser } else { w::ApprovalDecided::SignedOut },
                    "{case}"
                );
            } else {
                assert_eq!(
                    w::ApprovalState::decode(&f::wire(), &mut Reader::new(bytes)).expect("answer"),
                    if case == "bearer" { w::ApprovalState::NotABrowser } else { w::ApprovalState::SignedOut },
                    "{case}"
                );
            }
            assert!(e.out.is_empty(), "{case}: no core event or lookup left");
        }
    }
}
#[test]
fn a_read_says_what_the_approval_is() {
    for case in ["waiting", "approved", "taken", "refused", "expired", "fired", "unknown", "forgotten"] {
        let mut e = new();
        e.env.wall = Wall::from_nanos(123);
        let a = asked(&mut e);
        let code = *a.code();
        let expected = match case {
            "waiting" => w::ApprovalState::Waiting(
                w::ApprovalWaiting::new(&f::wire(), w::ApprovalWaitingParts { asked: 123 }).expect("waiting"),
            ),
            "approved" => {
                decide(&mut e, code, true);
                w::ApprovalState::Approved
            }
            "taken" => {
                decide(&mut e, code, true);
                let owner = e.send(take_message(a.secret().expose()));
                let to = signer(&mut e);
                signed_in(&mut e, to, owner);
                w::ApprovalState::Taken
            }
            "refused" => {
                decide(&mut e, code, false);
                w::ApprovalState::Refused
            }
            "expired" | "fired" => {
                e.env.now = e.env.now.saturating_add(SPAN);
                if case == "fired" {
                    e.component.fire(&e.env);
                }
                w::ApprovalState::Expired
            }
            "unknown" => w::ApprovalState::Unknown,
            "forgotten" => {
                decide(&mut e, code, false);
                for _ in 0_u32..8 {
                    let next = asked(&mut e);
                    decide(&mut e, *next.code(), false);
                }
                w::ApprovalState::Unknown
            }
            _ => unreachable!(),
        };
        assert_eq!(read(&mut e, if case == "unknown" { *b"BBBBBBBB" } else { code }), expected, "{case}");
    }
}
#[test]
fn the_first_decision_spends_the_approval() {
    for (first, next, expected) in [
        (true, true, w::ApprovalDecided::Used),
        (true, false, w::ApprovalDecided::Used),
        (false, true, w::ApprovalDecided::Used),
    ] {
        let mut e = new();
        let a = asked(&mut e);
        assert_eq!(
            decide(&mut e, *a.code(), first),
            if first { w::ApprovalDecided::Approved } else { w::ApprovalDecided::Refused }
        );
        assert_eq!(decide(&mut e, *a.code(), next), expected);
    }
    for expired in [true, false] {
        let mut e = new();
        let a = asked(&mut e);
        if expired {
            e.env.now = e.env.now.saturating_add(SPAN);
        }
        assert_eq!(
            decide(&mut e, if expired { *a.code() } else { *b"BBBBBBBB" }, true),
            if expired { w::ApprovalDecided::Expired } else { w::ApprovalDecided::Unknown }
        );
    }
}
#[test]
fn a_take_says_why_it_signs_no_one_in() {
    for case in [
        "waiting",
        "refused",
        "expired",
        "used",
        "unknown",
        "short",
        "empty",
        "busy",
        "stopped",
        "ended",
        "forgotten",
        "core refused",
    ] {
        let mut e = new();
        let a = asked(&mut e);
        let expected = match case {
            "waiting" => w::ApprovalSignInAnswer::Waiting,
            "refused" => {
                decide(&mut e, *a.code(), false);
                w::ApprovalSignInAnswer::Refused
            }
            "expired" => {
                e.env.now = e.env.now.saturating_add(SPAN);
                w::ApprovalSignInAnswer::Expired
            }
            "used" | "busy" | "ended" | "core refused" => {
                decide(&mut e, *a.code(), true);
                let owner = e.send(take_message(a.secret().expose()));
                if case == "busy" {
                    w::ApprovalSignInAnswer::Busy
                } else if case == "ended" || case == "core refused" {
                    let to = if case == "ended" { e.approval_whose() } else { signer(&mut e) };
                    e.reply(to, people::Reply::Signer(None));
                    assert_eq!(
                        w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1))
                            .expect("refusal"),
                        w::ApprovalSignInAnswer::Refused
                    );
                    finish(&mut e, owner);
                    w::ApprovalSignInAnswer::Refused
                } else {
                    let to = signer(&mut e);
                    signed_in(&mut e, to, owner);
                    w::ApprovalSignInAnswer::Used
                }
            }
            "stopped" => {
                decide(&mut e, *a.code(), true);
                e.component.down(&e.env, Event::Stop, &mut e.out);
                w::ApprovalSignInAnswer::NotReady
            }
            "forgotten" => {
                decide(&mut e, *a.code(), false);
                for _ in 0_u32..8 {
                    let next = asked(&mut e);
                    decide(&mut e, *next.code(), false);
                }
                w::ApprovalSignInAnswer::Unknown
            }
            "unknown" | "short" | "empty" => w::ApprovalSignInAnswer::Unknown,
            _ => unreachable!(),
        };
        let secret = match case {
            "unknown" => &[8; 32][..],
            "short" => &[7; 13][..],
            "empty" => &[],
            _ => a.secret().expose(),
        };
        if case == "busy" {
            let _waiting = e.approval_whose();
        }
        assert_eq!(take_answer(&mut e, secret), expected, "{case}");
    }
}
#[test]
fn busy_not_ready_and_full_spend_nothing() {
    for reply in [
        people::Reply::Again(people::Again::Busy),
        people::Reply::Again(people::Again::NotReady),
        people::Reply::Full { sign_ins: 4, earliest: Wall::from_nanos(700) },
    ] {
        let mut e = new();
        let a = approved(&mut e);
        let owner = e.send(take_message(a.secret().expose()));
        let to = signer(&mut e);
        let expected = match &reply {
            people::Reply::Again(people::Again::Busy) => w::ApprovalSignInAnswer::Busy,
            people::Reply::Again(people::Again::NotReady) => w::ApprovalSignInAnswer::NotReady,
            people::Reply::Full { .. } => w::ApprovalSignInAnswer::Full(
                w::ApprovalFull::new(
                    &f::wire(),
                    w::ApprovalFullParts { sign_ins: 4, earliest: 700, until: a.expires() },
                )
                .expect("full"),
            ),
            people::Reply::Signer(_)
            | people::Reply::SignedIn { .. }
            | people::Reply::SignedOut
            | people::Reply::Outcome(_)
            | people::Reply::Again(_)
            | people::Reply::Refused(_) => unreachable!(),
        };
        e.reply(to, reply);
        assert_eq!(
            w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer"),
            expected
        );
        finish(&mut e, owner);
        assert_eq!(read(&mut e, *a.code()), w::ApprovalState::Approved);
        let owner = e.send(take_message(a.secret().expose()));
        let to = signer(&mut e);
        signed_in(&mut e, to, owner);
    }
}
#[test]
fn an_approved_take_signs_in_the_approvers_party_once() {
    let mut e = new();
    let a = asked(&mut e);
    assert_eq!(
        read(&mut e, *a.code()),
        w::ApprovalState::Waiting(
            w::ApprovalWaiting::new(&f::wire(), w::ApprovalWaitingParts { asked: 0 }).expect("waiting")
        )
    );
    assert_eq!(decide(&mut e, *a.code(), true), w::ApprovalDecided::Approved);
    let owner = e.send(take_message(a.secret().expose()));
    let to = signer(&mut e);
    let first = signed_in(&mut e, to, owner);
    assert_eq!(take_answer(&mut e, a.secret().expose()), w::ApprovalSignInAnswer::Used);
    assert_eq!(read(&mut e, *a.code()), w::ApprovalState::Taken);
    assert_eq!(decide(&mut e, *a.code(), true), w::ApprovalDecided::Used);
    let next = approved(&mut e);
    let owner = e.send(take_message(next.secret().expose()));
    let to = signer(&mut e);
    let second = signed_in(&mut e, to, owner);
    assert_ne!(first.token(), second.token(), "a fresh random credential");
}
#[test]
fn an_approval_never_taken_makes_no_sign_in() {
    let mut e = new();
    let a = approved(&mut e);
    assert_eq!(e.component.next_deadline(), Some(e.env.now.saturating_add(SPAN)));
    e.env.now = e.env.now.saturating_add(SPAN);
    e.component.fire(&e.env);
    assert_eq!(e.component.next_deadline(), None);
    assert!(e.out.is_empty());
    assert_eq!(take_answer(&mut e, a.secret().expose()), w::ApprovalSignInAnswer::Expired);
    assert_eq!(read(&mut e, *a.code()), w::ApprovalState::Expired);
    let first = approved(&mut e);
    let first_end = e.env.now.saturating_add(SPAN);
    e.env.now = e.env.now.saturating_add(Duration::from_secs(1));
    let second = approved(&mut e);
    let second_end = e.env.now.saturating_add(SPAN);
    assert_eq!(e.component.next_deadline(), Some(first_end));
    e.env.now = first_end;
    e.component.fire(&e.env);
    assert_eq!(e.component.next_deadline(), Some(second_end));
    assert_eq!(read(&mut e, *first.code()), w::ApprovalState::Expired);
    assert_eq!(read(&mut e, *second.code()), w::ApprovalState::Approved);
    assert!(e.out.is_empty());
}
#[test]
#[expect(clippy::disallowed_macros, reason = "ordinary canary test inspects Debug at each secret boundary")]
fn no_print_of_the_component_shows_a_secret() {
    let mut e = new();
    let owner = e.send(ask_message());
    let w::ApprovalAskAnswer::Asked(a) =
        w::ApprovalAskAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("ask")
    else {
        panic!("asked")
    };
    assert_hidden(&format!("{:?} {:?}", e.component, e.out), a.secret().expose());
    e.component.up(&e.env, &mut e.out);
    assert_hidden(&format!("{:?}", e.out), a.secret().expose());
    e.out.pop().expect("asked answer");
    assert_hidden(&format!("{:?} {:?}", e.component, e.out), a.secret().expose());
    decide(&mut e, *a.code(), true);
    let owner = e.send(take_message(a.secret().expose()));
    assert_hidden(&format!("{:?} {:?}", e.component, e.out), a.secret().expose());
    let whose = e.approval_whose();
    e.reply(whose, people::Reply::Signer(Some(people::Signer { kind: people::Kind::Person, identity: identity() })));
    assert_hidden(&format!("{:?} {:?}", e.component, e.out), a.secret().expose());
    let to = e.approval_sign_in(&identity());
    assert_hidden(&format!("{:?} {:?}", e.component, e.out), a.secret().expose());
    e.reply(to, people::Reply::SignedIn { person: 72, sign_in: 91, expires: Wall::from_nanos(900_000_000_000) });
    let Some(Request::Secrets(SecretsRequest::Keep { owner: kept_owner, .. })) = e.out.pop() else { panic!("Keep") };
    let before = format!("{:?} {:?}", e.component, e.out);
    e.component.down(&e.env, Event::Secrets(SecretsAnswer::Kept { owner: kept_owner }), &mut e.out);
    let after = format!("{:?} {:?}", e.component, e.out);
    let w::ApprovalSignInAnswer::SignedIn(value) =
        w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer")
    else {
        panic!("signed in")
    };
    for printed in [before, after] {
        assert_hidden(&printed, a.secret().expose());
        assert_hidden(&printed, value.token().as_ref().expect("token").expose());
    }
    e.component.up(&e.env, &mut e.out);
    assert_hidden(&format!("{:?}", e.out), value.token().as_ref().expect("token").expose());
    assert_hidden(&format!("{:?}", e.out), a.secret().expose());
    assert_ne!(owner, Token::new(0));
}
#[expect(clippy::disallowed_macros, reason = "ordinary canary test searches Debug byte lists")]
fn assert_hidden(printed: &str, secret: &[u8]) {
    let list = format!("{secret:?}");
    let bytes = list.trim_start_matches('[').trim_end_matches(']');
    assert!(!printed.contains(bytes), "byte list leaked");
    for part in printed.as_bytes().windows(secret.len()) {
        assert_ne!(part, secret, "text leaked");
    }
}
#[test]
fn a_restart_forgets_every_approval() {
    let a = approved(&mut new());
    let mut fresh = new();
    assert_eq!(take_answer(&mut fresh, a.secret().expose()), w::ApprovalSignInAnswer::Unknown);
    assert_eq!(read(&mut fresh, *a.code()), w::ApprovalState::Unknown);
}

#[test]
fn admission_refusals_leave_approvals_alone() {
    for case in ["read busy", "decide busy", "take busy", "read stopped", "decide stopped"] {
        let mut settings = Settings { approval: true, ..f::settings() };
        settings.providers = skein_lib::List::with_capacity(1);
        settings.providers.push(19).expect("application provider");
        let mut env = f::env();
        env.limits.providers = 1;
        let mut e = Exchange::configured(env, settings);
        let a = approved(&mut e);
        let mut pending = None;
        if case.ends_with("busy") {
            for number in 0..2 {
                e.component.down(
                    &e.env,
                    Event::Mint { exchange: Token::new(number), identity: identity() },
                    &mut e.out,
                );
                let _mint = e.approval_sign_in(&identity());
                pending = Some(e.ask());
            }
        } else {
            e.component.down(&e.env, Event::Stop, &mut e.out);
        }
        if case.starts_with("take") {
            assert_eq!(take_answer(&mut e, a.secret().expose()), w::ApprovalSignInAnswer::Busy);
        } else {
            let message =
                if case.starts_with("read") { read_message(*a.code()) } else { decide_message(*a.code(), false) };
            let owner = e.send_as(message, true, Some(Secret::new(e.token.clone())));
            if case.ends_with("busy") {
                e.authenticate();
            }
            if case.starts_with("read") {
                assert_eq!(
                    w::ApprovalState::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer"),
                    if case.ends_with("busy") { w::ApprovalState::Busy } else { w::ApprovalState::NotReady }
                );
            } else {
                assert_eq!(
                    w::ApprovalDecided::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer"),
                    if case.ends_with("busy") { w::ApprovalDecided::Busy } else { w::ApprovalDecided::NotReady }
                );
            }
            finish(&mut e, owner);
        }
        if let Some((owner, to)) = pending {
            e.reply(to, people::Reply::Again(people::Again::Busy));
            finish(&mut e, owner);
            assert_eq!(read(&mut e, *a.code()), w::ApprovalState::Approved);
            let owner = e.send(take_message(a.secret().expose()));
            let to = signer(&mut e);
            signed_in(&mut e, to, owner);
        }
    }
}
#[test]
fn a_take_drains_on_stop_close_or_abort() {
    for stage in ["whose", "core", "keep"] {
        for action in ["stop", "close", "abort", "refuse", "close refused"] {
            let mut e = new();
            let a = approved(&mut e);
            let owner = e.send(take_message(a.secret().expose()));
            let to = if stage == "whose" { Some(e.approval_whose()) } else { Some(signer(&mut e)) };
            let to = if stage == "keep" {
                e.reply(
                    to.expect("core"),
                    people::Reply::SignedIn { person: 72, sign_in: 91, expires: Wall::from_nanos(900_000_000_000) },
                );
                e.out.pop().expect("Keep");
                None
            } else {
                to
            };
            if action == "refuse" || action == "close refused" {
                if action == "close refused" {
                    e.component.down(&e.env, Event::Close, &mut e.out);
                }
                if let Some(to) = to {
                    e.component.down(&e.env, Event::SignInRefused { to }, &mut e.out);
                    assert_eq!(
                        w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1))
                            .expect("answer"),
                        w::ApprovalSignInAnswer::Refused
                    );
                    finish(&mut e, owner);
                    if action != "close refused" {
                        assert_eq!(take_answer(&mut e, a.secret().expose()), w::ApprovalSignInAnswer::Refused);
                    }
                } else {
                    e.component.down(&e.env, Event::SignInEnded { sign_in: 91 }, &mut e.out);
                    e.out.pop().expect("Erase");
                    assert_eq!(
                        w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1))
                            .expect("answer"),
                        w::ApprovalSignInAnswer::Refused
                    );
                    finish(&mut e, owner);
                }
                continue;
            }
            e.component.down(
                &e.env,
                match action {
                    "stop" => Event::Stop,
                    "close" => Event::Close,
                    "abort" => Event::Abort,
                    _ => unreachable!(),
                },
                &mut e.out,
            );
            if let Some(to) = to {
                e.reply(
                    to,
                    if stage == "whose" {
                        people::Reply::Signer(Some(people::Signer { kind: people::Kind::Person, identity: identity() }))
                    } else {
                        people::Reply::Full { sign_ins: 4, earliest: Wall::from_nanos(700) }
                    },
                );
            } else {
                e.component.down(
                    &e.env,
                    Event::Secrets(SecretsAnswer::Kept { owner: SecretOwner::SignIn(91) }),
                    &mut e.out,
                );
            }
            if action == "abort" {
                assert!(e.out.is_empty());
                assert!(e.component.pending.exchanges.is_empty());
            } else if stage == "keep" {
                let w::ApprovalSignInAnswer::SignedIn(_) =
                    w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer")
                else {
                    panic!("signed in after digest kept")
                };
                finish(&mut e, owner);
            } else {
                let answer =
                    w::ApprovalSignInAnswer::decode(&f::wire(), &mut Reader::new(e.response(owner).1)).expect("answer");
                assert_eq!(
                    answer,
                    if action == "stop" && stage == "core" {
                        w::ApprovalSignInAnswer::Full(
                            w::ApprovalFull::new(
                                &f::wire(),
                                w::ApprovalFullParts { sign_ins: 4, earliest: 700, until: a.expires() },
                            )
                            .expect("full"),
                        )
                    } else {
                        w::ApprovalSignInAnswer::NotReady
                    }
                );
                finish(&mut e, owner);
            }
        }
    }
}

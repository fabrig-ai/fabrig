use super::*;
use fixtures::{CREDENTIAL, Client};
use jig_client_protocol_wire::v19 as w;
use skein_lib::{Secret, Time, Token, Wall, Writer};

pub(super) fn code() -> ApprovalCode {
    ApprovalCode::parse(b"BCDF-GHJK").unwrap()
}
pub(super) fn take_answer(answer: &w::ApprovalSignInAnswer) -> Box<[u8]> {
    let mut bytes = Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut bytes).unwrap();
    bytes.finish()
}
pub(super) fn taking() -> With {
    With::Approval { secret: Secret::new(Box::new([7; 32])) }
}
#[test]
fn an_approval_is_asked_read_and_decided_once_each() {
    for held in [false, true] {
        let mut c = Client::new();
        if held { c.step(Event::SignIn { with: With::Held(fixtures::credential(&CREDENTIAL)) }); }
        for (event, expected) in [
            (Event::AskApproval, Approval::NotAsked(NotAsked::Refused)),
            (Event::ReadApproval { code: code() }, Approval::Read { code: code(), state: ApprovalState::Approved }),
            (Event::DecideApproval { code: code(), approve: true }, Approval::Decided { code: code(), decided: Decided::Approved }),
        ] {
            c.step(event);
            let [Request::Exchange { exchange, credential, message }] = c.requests() else { panic!("one approval exchange: {:?}", c.requests()) };
            let exchange = *exchange;
            let body = match message {
                Message::ApprovalAsk(_) => { assert_eq!(credential, &None); Box::from([1]) }
                Message::ApprovalRead(_) => {
                    assert_eq!(credential, &held.then(|| fixtures::credential(&CREDENTIAL)));
                    Box::from([1])
                }
                Message::ApprovalDecide(_) => {
                    assert_eq!(credential, &held.then(|| fixtures::credential(&CREDENTIAL)));
                    Box::from([0])
                }
                other => panic!("approval exchange: {other:?}"),
            };
            c.step(Event::Reply { exchange, message: body });
            assert_eq!(c.requests(), &[Request::Approval(expected)]);
            c.step(Event::Reply { exchange, message: Box::from([1]) });
            assert!(c.requests().is_empty(), "one answer per exchange");
        }
    }
}
#[test]
fn an_approval_lost_or_signed_out_has_one_answer() {
    for decide in [false, true] {
        for signed_out in [false, true] {
            let mut c = Client::signed_in();
            c.step(if decide { Event::DecideApproval { code: code(), approve: false } } else { Event::ReadApproval { code: code() } });
            let exchange = c.exchange();
            c.step(if signed_out { Event::Reply { exchange, message: Box::from([7]) } } else { Event::Lost { exchange } });
            let expected = if decide {
                Approval::Decided { code: code(), decided: if signed_out { Decided::SignedOut } else { Decided::Lost } }
            } else {
                Approval::Read { code: code(), state: if signed_out { ApprovalState::SignedOut } else { ApprovalState::Lost } }
            };
            let answers: Vec<_> = c.requests().iter().filter_map(|request| if let Request::Approval(answer) = request { Some(answer) } else { None }).collect();
            assert_eq!(answers, [&expected]);
            if signed_out {
                assert_eq!(c.machine.link().state, Link::SignedOut);
                assert!(c.machine.link().credential.is_none());
            } else {
                assert_eq!(c.machine.link().state, Link::Live);
            }
        }
    }
}
#[test]
fn no_print_of_an_approval_shows_its_secret() {
    const CANARY: &[u8] = b"approval-secret-canary-0123456789";
    let secret = || Secret::new(Box::from(CANARY));
    let mut c = Client::new();
    let event = Event::SignIn { with: With::Approval { secret: secret() } };
    let mut prints = vec![format!("{event:?}"), format!("{event:#?}")];
    c.step(event);
    prints.extend([format!("{:?}", c.machine), format!("{:#?}", c.machine), format!("{:?}", c.requests()), format!("{:#?}", c.requests())]);
    let answer = Approval::Asked { secret: secret(), code: code(), expires: Wall::EPOCH };
    let event = Event::ApprovalAnswer { answer };
    prints.extend([format!("{event:?}"), format!("{event:#?}")]);
    let exchange = c.exchange();
    let reply = Event::Reply { exchange, message: take_answer(&w::ApprovalSignInAnswer::SignedIn(w::SignedIn::new(&w::CEILINGS, w::SignedInParts { token: Some(secret()), party: 7, expires: 300 }).unwrap())) };
    prints.extend([format!("{reply:?}"), format!("{reply:#?}")]);
    let mut link = jig_client_protocol_link::Link::new(&jig_client_protocol_link::Limits::new(fixtures::domain_limits(), 8).unwrap()).unwrap();
    let mut out = skein_lib::Queue::uncapped();
    let event = jig_client_protocol_link::Event::SignIn { with: With::Approval { secret: secret() } };
    prints.extend([format!("{event:?}"), format!("{event:#?}")]);
    link.step(Time::ZERO, event, &mut out);
    prints.extend([format!("{link:?}"), format!("{link:#?}"), format!("{out:?}"), format!("{out:#?}")]);
    let wire_reply = jig_client_protocol_link::Event::Reply { exchange: Token::new(1), message: match reply { Event::Reply { message, .. } => message, _ => unreachable!() } };
    prints.extend([format!("{wire_reply:?}"), format!("{wire_reply:#?}")]);
    for print in prints {
        assert!(!print.contains(core::str::from_utf8(CANARY).unwrap()), "{print}");
        let normalized = print.replace([' ', '\n', '\r', '\t'], "");
        let numbers = format!("{CANARY:?}").replace(' ', "");
        assert!(!normalized.contains(numbers.trim_matches(['[', ']'])), "secret bytes in {print}");
    }
}

use super::*;
use fixtures::{CREDENTIAL, Client};
use jig_client_protocol_wire::v21 as w;
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
    for (with, held) in
        [(None, false), (Some(With::Held(fixtures::credential(&CREDENTIAL))), true), (Some(With::Transport), false)]
    {
        let mut c = Client::new();
        if let Some(with) = with {
            c.step(Event::SignIn { with });
        }
        for (event, expected) in [
            (Event::AskApproval, Approval::NotAsked(NotAsked::Refused)),
            (Event::ReadApproval { code: code() }, Approval::Read { code: code(), state: ApprovalState::Approved }),
            (
                Event::DecideApproval { code: code(), approve: true },
                Approval::Decided { code: code(), decided: Decided::Approved },
            ),
            (
                Event::DecideApproval { code: code(), approve: false },
                Approval::Decided { code: code(), decided: Decided::Refused },
            ),
        ] {
            c.step(event);
            let [Request::Exchange { exchange, credential, message }] = c.requests() else {
                panic!("one approval exchange: {:?}", c.requests())
            };
            let exchange = *exchange;
            let body = match message {
                Message::ApprovalAsk(_) => {
                    assert_eq!(credential, &None);
                    Box::from([1])
                }
                Message::ApprovalRead(_) => {
                    assert_eq!(credential, &if held { Some(fixtures::credential(&CREDENTIAL)) } else { None });
                    Box::from([1])
                }
                Message::ApprovalDecide(body) => {
                    assert_eq!(credential, &if held { Some(fixtures::credential(&CREDENTIAL)) } else { None });
                    let record = w::ApprovalDecide::decode(&w::CEILINGS, &mut skein_lib::Reader::new(body)).unwrap();
                    assert_eq!(
                        record.approve(),
                        expected == (Approval::Decided { code: code(), decided: Decided::Approved })
                    );
                    if record.approve() { Box::from([0]) } else { Box::from([1]) }
                }
                other @ (Message::Authorize(_)
                | Message::SignInApproval(_)
                | Message::Request(_)
                | Message::Read(_)
                | Message::Watch(_)
                | Message::Unwatch(_)
                | Message::StreamGone(_)
                | Message::SignInLink(_)
                | Message::SignInService
                | Message::SignOut) => panic!("approval exchange: {other:?}"),
            };
            c.step(Event::Reply { exchange, message: body });
            assert_eq!(c.requests(), &[Request::Approval(expected)]);
            c.step(Event::Reply { exchange, message: Box::from([1]) });
            assert!(c.requests().is_empty(), "one answer per exchange");
        }
    }
}
fn signed_out_answer(decide: bool) -> Box<[u8]> {
    let mut bytes = Writer::new(1);
    if decide {
        w::ApprovalDecided::SignedOut.encode(&mut bytes).unwrap();
    } else {
        w::ApprovalState::SignedOut.encode(&mut bytes).unwrap();
    }
    bytes.finish()
}
#[test]
fn an_approval_lost_or_signed_out_has_one_answer() {
    for decide in [false, true] {
        for signed_out in [false, true] {
            let mut c = Client::signed_in();
            c.step(if decide {
                Event::DecideApproval { code: code(), approve: false }
            } else {
                Event::ReadApproval { code: code() }
            });
            let exchange = c.exchange();
            c.step(if signed_out {
                Event::Reply { exchange, message: signed_out_answer(decide) }
            } else {
                Event::Lost { exchange }
            });
            let expected = if decide {
                Approval::Decided { code: code(), decided: if signed_out { Decided::SignedOut } else { Decided::Lost } }
            } else {
                Approval::Read {
                    code: code(),
                    state: if signed_out { ApprovalState::SignedOut } else { ApprovalState::Lost },
                }
            };
            let mut answers = skein_lib::List::with_capacity(1);
            for request in c.requests() {
                if let Request::Approval(answer) = request {
                    answers.push(answer).unwrap();
                }
            }
            assert_eq!(answers.as_slice(), &[&expected]);
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
#[expect(clippy::disallowed_macros, reason = "the canary exercises compact and pretty Debug on client boundaries")]
fn no_print_of_an_approval_shows_its_secret() {
    const CANARY: &[u8] = b"approval-secret-canary-0123456789";
    const CREDENTIAL_CANARY: &[u8; 40] = b"approval-credential-canary-0123456789abc";
    // At most forty compact and pretty boundary snapshots.
    const PRINTS: u32 = 40;
    let secret = || Secret::new(Box::from(CANARY));
    let mut c = Client::new();
    let event = Event::SignIn { with: With::Approval { secret: secret() } };
    let mut prints = skein_lib::List::with_capacity(PRINTS);
    prints.push(format!("{event:?}")).unwrap();
    prints.push(format!("{event:#?}")).unwrap();
    c.step(event);
    for print in [
        format!("{:?}", c.machine),
        format!("{:#?}", c.machine),
        format!("{:?}", c.requests()),
        format!("{:#?}", c.requests()),
    ] {
        prints.push(print).unwrap();
    }
    let answer = Approval::Asked { secret: secret(), code: code(), expires: Wall::EPOCH };
    let event = Event::ApprovalAnswer { answer };
    for print in [format!("{event:?}"), format!("{event:#?}")] {
        prints.push(print).unwrap();
    }
    let exchange = c.exchange();
    let body = take_answer(&w::ApprovalSignInAnswer::SignedIn(
        w::SignedIn::new(
            &w::CEILINGS,
            w::SignedInParts { token: Some(fixtures::credential(CREDENTIAL_CANARY)), party: 7, expires: 300 },
        )
        .unwrap(),
    ));
    let reply = Event::Reply { exchange, message: body.clone() };
    for print in [format!("{reply:?}"), format!("{reply:#?}")] {
        prints.push(print).unwrap();
    }
    c.step(reply);
    for print in [
        format!("{:?}", c.machine),
        format!("{:#?}", c.machine),
        format!("{:?}", c.requests()),
        format!("{:#?}", c.requests()),
    ] {
        prints.push(print).unwrap();
    }
    let asked = || Approval::Asked { secret: secret(), code: code(), expires: Wall::EPOCH };
    let request = Request::Approval(asked());
    let unlinked = UnlinkedRequest::Approval(asked());
    for print in [format!("{request:?}"), format!("{request:#?}"), format!("{unlinked:?}"), format!("{unlinked:#?}")] {
        prints.push(print).unwrap();
    }
    let mut link = jig_client_protocol_link::Link::new(
        &jig_client_protocol_link::Limits::new(fixtures::domain_limits(), 8).unwrap(),
    )
    .unwrap();
    let mut out = skein_lib::Queue::uncapped();
    let event = jig_client_protocol_link::Event::SignIn { with: With::Approval { secret: secret() } };
    for print in [format!("{event:?}"), format!("{event:#?}")] {
        prints.push(print).unwrap();
    }
    link.step(Time::ZERO, event, &mut out);
    for print in [format!("{link:?}"), format!("{link:#?}"), format!("{out:?}"), format!("{out:#?}")] {
        prints.push(print).unwrap();
    }
    let wire_reply = jig_client_protocol_link::Event::Reply { exchange: Token::new(1), message: body.clone() };
    for print in [format!("{wire_reply:?}"), format!("{wire_reply:#?}")] {
        prints.push(print).unwrap();
    }
    link.step(Time::ZERO, wire_reply, &mut out);
    for print in [format!("{link:?}"), format!("{link:#?}"), format!("{out:?}"), format!("{out:#?}")] {
        prints.push(print).unwrap();
    }
    let heard = jig_client_protocol_link::Heard::Reply { exchange, message: body };
    for print in [format!("{heard:?}"), format!("{heard:#?}")] {
        prints.push(print).unwrap();
    }
    for print in &prints {
        for canary in [CANARY, CREDENTIAL_CANARY.as_slice()] {
            assert!(skein_lib::bytes::find(print.as_bytes(), canary).is_none(), "{print}");
            let normalized = print.replace([' ', '\n', '\r', '\t'], "");
            let numbers = format!("{canary:?}").replace(' ', "");
            assert!(!normalized.contains(numbers.trim_matches(['[', ']'])), "secret bytes in {print}");
        }
    }
}

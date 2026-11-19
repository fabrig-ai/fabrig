use super::*;
use fixtures::{CREDENTIAL, Client};
use jig_client_protocol_wire as w;
use skein_lib::{Reader, Secret, Writer};

fn answer(value: &w::LinkAskAnswer) -> Box<[u8]> {
    let mut bytes = Writer::new(usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap());
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
    bytes.finish()
}
fn asked() -> AskedLink {
    AskedLink::Asked { secret: Secret::new(Box::new([7; 32])), expires: 300 }
}
#[test]
fn a_link_is_asked_and_its_answer_told_once() {
    for with in [None, Some(With::Held(fixtures::credential(&CREDENTIAL))), Some(With::Transport)] {
        let held = match &with {
            Some(With::Held(_)) => true,
            Some(With::Transport | With::Service(_) | With::Link { .. } | With::Approval { .. }) | None => false,
        };
        let mut c = Client::new();
        if let Some(with) = with {
            c.step(Event::SignIn { with });
        }
        for (body, expected) in [
            (
                Some(answer(&w::LinkAskAnswer::Asked(
                    w::LinkAsked { secret: Secret::new(Box::new([7; 32])), expires: 300 },
                ))),
                asked(),
            ),
            (None, AskedLink::NotAsked(NotAsked::Lost)),
            (Some(answer(&w::LinkAskAnswer::SignedOut)), AskedLink::SignedOut),
        ] {
            c.step(Event::AskLink);
            let [Request::Exchange { exchange, credential, message: Message::LinkAsk(ask_body) }] = c.requests() else {
                panic!("one link ask: {:?}", c.requests())
            };
            assert_eq!(credential, &if held { Some(fixtures::credential(&CREDENTIAL)) } else { None });
            w::LinkAsk::decode(&w::CEILINGS, &mut Reader::new(ask_body)).unwrap();
            let exchange = *exchange;
            c.step(Event::AskLink);
            // A second ask cannot send an exchange while the first is pending.
            assert_eq!(c.requests(), &[Request::AskedLink(AskedLink::NotAsked(NotAsked::Busy))]);
            c.step(match body {
                Some(body) => Event::Reply { exchange, message: body },
                None => Event::Lost { exchange },
            });
            let mut answers = skein_lib::List::with_capacity(1);
            for request in c.requests() {
                if let Request::AskedLink(answer) = request {
                    answers.push(answer).unwrap();
                }
            }
            assert_eq!(answers.as_slice(), &[&expected]);
            if expected == AskedLink::SignedOut {
                assert_eq!(c.machine.link().state, Link::SignedOut);
                assert!(c.machine.link().credential.is_none());
            }
            c.step(Event::Lost { exchange });
            assert_eq!(c.requests(), []);
        }
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the canary exercises compact and pretty Debug on client boundaries")]
fn no_print_of_an_asked_link_shows_its_secret() {
    const CANARY: &[u8] = b"asked-link-secret-canary-0123456";
    let asked = || AskedLink::Asked { secret: Secret::new(Box::from(CANARY)), expires: 300 };
    let record = w::LinkAskAnswer::Asked(
        w::LinkAsked { secret: Secret::new(Box::from(CANARY)), expires: 300 },
    );
    let body = answer(&record);
    let reply = Event::Reply { exchange: skein_lib::Token::new(1), message: body.clone() };
    let link_reply = jig_client_protocol_link::Event::Reply { exchange: skein_lib::Token::new(1), message: body };
    let request = Request::AskedLink(asked());
    let unlinked = UnlinkedRequest::AskedLink(asked());
    let link_request = jig_client_protocol_link::Request::AskedLink(asked());
    let message = Message::LinkAsk(Box::from(CANARY));
    let asked = asked();
    for print in [
        format!("{asked:?}"),
        format!("{asked:#?}"),
        format!("{reply:?}"),
        format!("{reply:#?}"),
        format!("{link_reply:?}"),
        format!("{link_reply:#?}"),
        format!("{request:?}"),
        format!("{request:#?}"),
        format!("{unlinked:?}"),
        format!("{unlinked:#?}"),
        format!("{link_request:?}"),
        format!("{link_request:#?}"),
        format!("{message:?}"),
        format!("{message:#?}"),
    ] {
        assert!(skein_lib::bytes::find(print.as_bytes(), CANARY).is_none());
        let normalized = print.replace([' ', '\n', '\r', '\t'], "");
        let numbers = format!("{CANARY:?}").replace(' ', "");
        assert!(!normalized.contains(numbers.trim_matches(['[', ']'])), "secret bytes in {print}");
    }
}

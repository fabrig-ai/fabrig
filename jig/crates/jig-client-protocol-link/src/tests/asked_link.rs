use super::*;
use fixtures::{Harness, limits};
use skein_lib::{Secret, Writer};

fn encoded(answer: &w::LinkAskAnswer) -> Box<[u8]> {
    let mut bytes = Writer::new(usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap());
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).unwrap();
    bytes.finish()
}
fn asked() -> AskedLink {
    AskedLink::Asked { secret: Secret::new(Box::new([7; 32])), expires: 300 }
}
fn exchange(h: &mut Harness) -> Token {
    h.link.step(Time::ZERO, Event::AskLink, &mut h.out);
    h.link.settle(&mut h.out);
    let Some(Request::Exchange { exchange, credential, message: Message::LinkAsk(body) }) = h.out.pop() else {
        panic!("one link ask")
    };
    assert_eq!(credential, h.link.credential);
    let mut reader = Reader::new(&body);
    w::LinkAsk::decode(&w::CEILINGS, &mut reader).unwrap();
    assert!(reader.is_empty());
    assert!(h.out.is_empty());
    exchange
}
#[test]
fn each_link_ask_answer_is_decoded_once_with_the_held_credential() {
    for held in [false, true] {
        for (answer, expected) in [
            (
                w::LinkAskAnswer::Asked(
                    w::LinkAsked { secret: Secret::new(Box::new([7; 32])), expires: 300 },
                ),
                asked(),
            ),
            (w::LinkAskAnswer::Refused, AskedLink::NotAsked(NotAsked::Refused)),
            (w::LinkAskAnswer::SignedOut, AskedLink::SignedOut),
            (w::LinkAskAnswer::NotReady, AskedLink::NotAsked(NotAsked::NotReady)),
            (w::LinkAskAnswer::Busy, AskedLink::NotAsked(NotAsked::Busy)),
        ] {
            for trailing in [false, true] {
                let mut h = Harness::new(&limits());
                h.link.credential = if held { Some(Secret::new(Box::new([9; 40]))) } else { None };
                let exchange = exchange(&mut h);
                let body = encoded(&answer);
                let mut bytes = Writer::new(body.len().checked_add(usize::from(trailing)).unwrap());
                bytes.put(&body).unwrap();
                if trailing {
                    bytes.put(&[0]).unwrap();
                }
                let body = bytes.finish();
                h.answer(exchange, &body);
                if !trailing && expected == AskedLink::SignedOut {
                    assert_eq!(h.out.pop(), Some(Request::Signing { signing: Signing::SignedOut }));
                    assert!(h.link.credential.is_none());
                } else {
                    assert_eq!(h.link.credential.is_some(), held);
                }
                let expected = if trailing {
                    AskedLink::NotAsked(NotAsked::Lost)
                } else {
                    match &expected {
                        AskedLink::Asked { .. } => asked(),
                        AskedLink::NotAsked(why) => AskedLink::NotAsked(*why),
                        AskedLink::SignedOut => AskedLink::SignedOut,
                    }
                };
                assert_eq!(h.out.pop(), Some(Request::AskedLink(expected)));
                assert!(h.out.is_empty());
                h.answer(exchange, &body);
                assert!(h.out.is_empty(), "one answer per exchange");
            }
        }
        for body in [
            Some(Box::from([255])),
            None,
            Some(encoded(&w::LinkAskAnswer::Asked(
                w::LinkAsked { secret: Secret::new(Box::new([7; 31])), expires: 300 },
            ))),
        ] {
            let mut h = Harness::new(&limits());
            h.link.credential = if held { Some(Secret::new(Box::new([9; 40]))) } else { None };
            let exchange = exchange(&mut h);
            match body {
                Some(body) => h.answer(exchange, &body),
                None => h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out),
            }
            assert_eq!(h.out.pop(), Some(Request::AskedLink(AskedLink::NotAsked(NotAsked::Lost))));
            assert!(h.out.is_empty());
        }
    }
}
#[test]
fn a_second_link_ask_is_busy_until_the_first_ends() {
    let mut h = Harness::new(&limits());
    let first = exchange(&mut h);
    h.link.step(Time::ZERO, Event::AskLink, &mut h.out);
    assert_eq!(h.out.pop(), Some(Request::AskedLink(AskedLink::NotAsked(NotAsked::Busy))));
    assert!(h.out.is_empty());
    h.link.step(Time::ZERO, Event::Lost { exchange: first }, &mut h.out);
    assert_eq!(h.out.pop(), Some(Request::AskedLink(AskedLink::NotAsked(NotAsked::Lost))));
    exchange(&mut h);
}

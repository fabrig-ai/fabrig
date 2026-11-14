//! A terminal watch uses the same opaque document lifecycle as other jig watches.
use super::fixtures::{self, Client, TASK};
use crate::{Event, Latest, Message, Request, State, Watchable, contract as c};
use jig_client_protocol_wire::v22 as w;
use skein_lib::{Reader, Writer};
#[test]
fn a_terminal_watch_opens_delivers_and_closes() {
    let mut client = Client::signed_in();
    client.step(Event::Watch { handle: TASK, subject: Watchable::Jig(c::Subject::Terminal { terminal: 23 }) });
    let [Request::Exchange { exchange, message: Message::Watch(bytes), .. }] = client.requests() else {
        panic!("one watch opening")
    };
    let exchange = *exchange;
    let opening = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
    assert_eq!(
        w::Subject::decode(&w::CEILINGS, &mut Reader::new(opening.subject().record())),
        Ok(w::Subject::Terminal(
            w::SubjectTerminal::new(&w::CEILINGS, w::SubjectTerminalParts { terminal: 23 }).unwrap()
        ))
    );
    assert_eq!(client.machine.document(TASK).unwrap().state, State::Opening);
    let answer = w::OpenAnswer::Admitted;
    let mut out = Writer::new(skein_lib::length::usize_of(answer.measure()));
    answer.encode(&mut out).unwrap();
    client.step(Event::Reply { exchange, message: out.finish() });
    for cancelled in [false, true] {
        let login = c::TerminalLogin { number: 1, provider: 2, name: b"work".as_slice().into(), cancelled };
        let wire = w::DeliveryValue::Terminal(
            w::Terminal::new(
                &w::CEILINGS,
                w::TerminalParts {
                    login: Some(
                        w::TerminalLogin::new(
                            &w::CEILINGS,
                            w::TerminalLoginParts {
                                number: 1,
                                provider: 2,
                                name: b"work".as_slice().into(),
                                cancelled,
                            },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
        );
        let mut out = Writer::new(skein_lib::length::usize_of(wire.measure()));
        wire.encode(&mut out).unwrap();
        let document =
            w::Owned::new(&w::CEILINGS, w::OwnedParts { owner: w::Owner::Jig, version: 22, record: out.finish() })
                .unwrap();
        client.stream(fixtures::event(&w::StreamEvent::Delivery(
            w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: opening.watch(), document }).unwrap(),
        )));
        let document = client.machine.document(TASK).unwrap();
        assert_eq!(document.state, State::Live);
        assert_eq!(
            document.latest,
            Some(&Latest::Jig(c::Delivery::Terminal(Box::new(c::Terminal { login: Some(login) }))))
        );
    }
    client.step(Event::Unwatch { handle: TASK });
    let [Request::Exchange { message: Message::Unwatch(bytes), .. }] = client.requests() else {
        panic!("one watch closing")
    };
    let closing = w::WatchClose::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
    assert_eq!(closing.watch(), opening.watch());
    assert!(client.machine.document(TASK).is_none());
}

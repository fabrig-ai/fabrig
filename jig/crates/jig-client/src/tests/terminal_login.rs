//! A terminal watch uses the same opaque document lifecycle as other jig watches.
use jig_client_protocol_wire::Record as _;
use super::fixtures::{self, Client, TASK};
use crate::{Event, Latest, Message, Request, State, Watchable, contract as c};
use jig_client_protocol_wire as w;
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
        w::Subject::decode(&w::CEILINGS, &mut Reader::new(opening.subject.record.as_ref())),
        Ok(w::Subject::Terminal(
            w::SubjectTerminal { terminal: 23 }
        ))
    );
    assert_eq!(client.machine.document(TASK).unwrap().state, State::Opening);
    let answer = w::OpenAnswer::Admitted;
    let mut out = Writer::new(skein_lib::length::usize_of(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
    client.step(Event::Reply { exchange, message: out.finish() });
    for cancelled in [false, true] {
        let login = c::TerminalLogin { number: 1, provider: 2, name: b"work".as_slice().into(), cancelled };
        let wire = w::DeliveryValue::Terminal(
            w::Terminal {
                    login: Some(
                        w::TerminalLogin {
                                number: 1,
                                provider: 2,
                                name: b"work".as_slice().into(),
                                cancelled,
                            },
                    ),
                },
        );
        let mut out = Writer::new(skein_lib::length::usize_of(wire.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
        wire.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
        let document = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: out.finish() };
        client.stream(fixtures::event(&w::StreamEvent::Delivery(
            w::Delivery { watch: opening.watch, document },
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
    assert_eq!(closing.watch, opening.watch);
    assert!(client.machine.document(TASK).is_none());
}

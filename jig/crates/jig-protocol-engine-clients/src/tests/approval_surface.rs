//! Approval's disabled surface and stopping answers (protocol/client/sign-in.md).
use super::fixtures::{self as f, Exchange};
use crate::{Event, Message, Settings};
use jig_client_protocol_wire::v22 as w;
use skein_lib::{Secret, Writer, length::usize_of};

type ApprovalCase = (&'static str, fn(Box<[u8]>) -> Message, Box<[u8]>, Box<[u8]>);

fn cases(stopping: bool) -> [ApprovalCase; 4] {
    let limits = f::wire();
    let ask = w::ApprovalAsk::new(&limits, w::ApprovalAskParts {}).expect("ask");
    let mut ask_bytes = Writer::new(usize_of(ask.measure()));
    ask.encode(&mut ask_bytes).expect("ask bytes");
    let read = w::ApprovalRead::new(&limits, w::ApprovalReadParts { code: *b"WDJBMJHT" }).expect("read");
    let mut read_bytes = Writer::new(usize_of(read.measure()));
    read.encode(&mut read_bytes).expect("read bytes");
    let decide = w::ApprovalDecide::new(&limits, w::ApprovalDecideParts { code: *b"WDJBMJHT", approve: true })
        .expect("decision");
    let mut decide_bytes = Writer::new(usize_of(decide.measure()));
    decide.encode(&mut decide_bytes).expect("decision bytes");
    let take = w::ApprovalSignIn::new(&limits, w::ApprovalSignInParts { secret: Secret::new(Box::new([7; 32])) })
        .expect("take");
    let mut take_bytes = Writer::new(usize_of(take.measure()));
    take.encode(&mut take_bytes).expect("take bytes");

    let ask = if stopping { w::ApprovalAskAnswer::NotReady } else { w::ApprovalAskAnswer::Refused };
    let mut ask_answer = Writer::new(usize_of(ask.measure()));
    ask.encode(&mut ask_answer).expect("ask answer");
    let read = if stopping { w::ApprovalState::NotReady } else { w::ApprovalState::Unknown };
    let mut read_answer = Writer::new(usize_of(read.measure()));
    read.encode(&mut read_answer).expect("read answer");
    let decide = if stopping { w::ApprovalDecided::NotReady } else { w::ApprovalDecided::Unknown };
    let mut decide_answer = Writer::new(usize_of(decide.measure()));
    decide.encode(&mut decide_answer).expect("decision answer");
    let take = if stopping { w::ApprovalSignInAnswer::NotReady } else { w::ApprovalSignInAnswer::Unknown };
    let mut take_answer = Writer::new(usize_of(take.measure()));
    take.encode(&mut take_answer).expect("take answer");

    [
        ("ask", Message::ApprovalAsk, ask_bytes.finish(), ask_answer.finish()),
        ("read", Message::ApprovalRead, read_bytes.finish(), read_answer.finish()),
        ("decide", Message::ApprovalDecide, decide_bytes.finish(), decide_answer.finish()),
        ("take", Message::SignInApproval, take_bytes.finish(), take_answer.finish()),
    ]
}

#[test]
fn each_approval_route_is_answered_as_not_enabled() {
    let mut exchange = Exchange::new();
    for (name, message, body, expected) in cases(false) {
        let owner = exchange.send(message(body));
        assert_eq!(exchange.response(owner), (200, expected.as_ref()), "{name}");
        assert!(exchange.out.is_empty(), "{name}: no core event or secrets lookup");
        assert!(exchange.component.pending.exchanges.is_empty(), "{name}: no pending exchange");
        assert!(exchange.component.up(&exchange.env, &mut exchange.out), "{name}: deliver answer");
        exchange.out.pop().expect("transport answer");
        assert!(exchange.out.is_empty(), "{name}: only the answer");
    }
}

#[test]
fn each_approval_route_is_not_ready_while_stopping() {
    for approval in [false, true] {
        let mut exchange = Exchange::configured(f::env(), Settings { approval, ..f::settings() });
        exchange.component.down(&exchange.env, Event::Stop, &mut exchange.out);
        for (name, message, body, expected) in cases(true) {
            let owner = exchange.send_as(message(body), true, None);
            assert_eq!(exchange.response(owner), (200, expected.as_ref()), "{name}, approval {approval}");
            assert!(exchange.out.is_empty(), "{name}: no core event or secrets lookup");
            assert!(exchange.component.pending.exchanges.is_empty(), "{name}: no pending exchange");
            assert!(exchange.component.up(&exchange.env, &mut exchange.out), "{name}: deliver answer");
            exchange.out.pop().expect("transport answer");
            assert!(exchange.out.is_empty(), "{name}: only the answer");
        }
    }
}

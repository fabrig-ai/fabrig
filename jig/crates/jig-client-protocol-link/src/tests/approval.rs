use super::*;
use fixtures::{Harness, limits};
use skein_lib::{Secret, Wall, Writer};

macro_rules! encoded {
    ($record:expr) => {{
        let record = $record;
        let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut bytes).unwrap();
        bytes.finish()
    }};
}
fn code() -> ApprovalCode {
    ApprovalCode::parse(b"BCDF-GHJK").unwrap()
}
fn asked() -> Approval {
    Approval::Asked { secret: Secret::new(Box::new([7; 32])), code: code(), expires: Wall::from_nanos(300) }
}
fn ask_rows() -> Vec<(Box<[u8]>, Approval)> {
    vec![
        (encoded!(w::ApprovalAskAnswer::Asked(w::ApprovalAsked::new(&w::CEILINGS, w::ApprovalAskedParts {
            secret: Secret::new(Box::new([7; 32])), code: *code().letters(), expires: 300,
        }).unwrap())), asked()),
        (encoded!(w::ApprovalAskAnswer::Refused), Approval::NotAsked(NotAsked::Refused)),
        (encoded!(w::ApprovalAskAnswer::NotReady), Approval::NotAsked(NotAsked::NotReady)),
        (encoded!(w::ApprovalAskAnswer::Busy), Approval::NotAsked(NotAsked::Busy)),
    ]
}
fn read_rows() -> Vec<(Box<[u8]>, Approval)> {
    [
        (w::ApprovalState::Waiting(w::ApprovalWaiting::new(&w::CEILINGS, w::ApprovalWaitingParts { asked: 100 }).unwrap()), ApprovalState::Waiting { asked: Wall::from_nanos(100) }),
        (w::ApprovalState::Approved, ApprovalState::Approved),
        (w::ApprovalState::Taken, ApprovalState::Taken),
        (w::ApprovalState::Refused, ApprovalState::Refused),
        (w::ApprovalState::Expired, ApprovalState::Expired),
        (w::ApprovalState::Unknown, ApprovalState::Unknown),
        (w::ApprovalState::NotABrowser, ApprovalState::NotABrowser),
        (w::ApprovalState::SignedOut, ApprovalState::SignedOut),
        (w::ApprovalState::NotReady, ApprovalState::NotReady),
        (w::ApprovalState::Busy, ApprovalState::Busy),
    ].into_iter().map(|(record, state)| (encoded!(record), Approval::Read { code: code(), state })).collect()
}
fn decide_rows() -> Vec<(Box<[u8]>, Approval)> {
    [
        (w::ApprovalDecided::Approved, Decided::Approved),
        (w::ApprovalDecided::Refused, Decided::Refused),
        (w::ApprovalDecided::Used, Decided::Used),
        (w::ApprovalDecided::Expired, Decided::Expired),
        (w::ApprovalDecided::Unknown, Decided::Unknown),
        (w::ApprovalDecided::NotABrowser, Decided::NotABrowser),
        (w::ApprovalDecided::SignedOut, Decided::SignedOut),
        (w::ApprovalDecided::NotReady, Decided::NotReady),
        (w::ApprovalDecided::Busy, Decided::Busy),
    ].into_iter().map(|(record, decided)| (encoded!(record), Approval::Decided { code: code(), decided })).collect()
}
fn event(kind: u8) -> Event {
    match kind {
        0 => Event::AskApproval,
        1 => Event::ReadApproval { code: code() },
        2 => Event::DecideApproval { code: code(), approve: true },
        _ => unreachable!(),
    }
}
fn lost(kind: u8) -> Approval {
    match kind {
        0 => Approval::NotAsked(NotAsked::Lost),
        1 => Approval::Read { code: code(), state: ApprovalState::Lost },
        2 => Approval::Decided { code: code(), decided: Decided::Lost },
        _ => unreachable!(),
    }
}
fn exchange(h: &mut Harness, kind: u8) -> Token {
    h.link.step(Time::ZERO, event(kind), &mut h.out);
    let Some(Request::Exchange { exchange, credential, message }) = h.out.pop() else { panic!("approval exchange") };
    assert!(credential.is_none());
    match message {
        Message::ApprovalAsk(body) => { assert_eq!(kind, 0); assert_eq!(body, encoded!(w::ApprovalAsk::new(&w::CEILINGS, w::ApprovalAskParts {}).unwrap())); }
        Message::ApprovalRead(body) => { assert_eq!(kind, 1); let record = w::ApprovalRead::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap(); assert_eq!(record.code(), code().letters()); }
        Message::ApprovalDecide(body) => { assert_eq!(kind, 2); let record = w::ApprovalDecide::decode(&w::CEILINGS, &mut Reader::new(&body)).unwrap(); assert_eq!(record.code(), code().letters()); assert!(record.approve()); }
        other => panic!("approval message: {other:?}"),
    }
    exchange
}
#[test]
fn each_approval_message_is_encoded_and_its_answer_decoded() {
    for (kind, rows) in [(0, ask_rows()), (1, read_rows()), (2, decide_rows())] {
        for (body, expected) in rows {
            for trailing in [false, true] {
                let mut h = Harness::new(&limits());
                let exchange = exchange(&mut h, kind);
                let mut body = body.to_vec();
                if trailing { body.push(0); }
                h.answer(exchange, &body);
                assert_eq!(h.out.pop(), Some(Request::Approval(if trailing { lost(kind) } else { expected_for(&expected) })));
                assert!(h.out.is_empty());
                h.answer(exchange, &body);
                assert!(h.out.is_empty(), "an answer is told once");
            }
        }
        for body in [Some(&[255][..]), None] {
            let mut h = Harness::new(&limits());
            let exchange = exchange(&mut h, kind);
            match body {
                Some(body) => h.answer(exchange, body),
                None => h.link.step(Time::ZERO, Event::Lost { exchange }, &mut h.out),
            }
            assert_eq!(h.out.pop(), Some(Request::Approval(lost(kind))));
            assert!(h.out.is_empty());
        }
    }
}
// Answers contain a Secret and intentionally do not derive Clone.
fn expected_for(answer: &Approval) -> Approval {
    match answer {
        Approval::Asked { .. } => asked(),
        Approval::NotAsked(why) => Approval::NotAsked(*why),
        Approval::Read { code, state } => Approval::Read { code: *code, state: *state },
        Approval::Decided { code, decided } => Approval::Decided { code: *code, decided: *decided },
    }
}
#[test]
fn approval_admission_is_one_ask_and_one_read_or_decision() {
    for (first, second, expected) in [
        (0, 0, Approval::NotAsked(NotAsked::Busy)),
        (1, 1, Approval::Read { code: code(), state: ApprovalState::Busy }),
        (1, 2, Approval::Decided { code: code(), decided: Decided::Busy }),
        (2, 1, Approval::Read { code: code(), state: ApprovalState::Busy }),
    ] {
        let mut h = Harness::new(&limits());
        exchange(&mut h, first);
        h.link.step(Time::ZERO, event(second), &mut h.out);
        assert_eq!(h.out.pop(), Some(Request::Approval(expected)));
        assert!(h.out.is_empty());
    }
    let mut h = Harness::new(&limits());
    exchange(&mut h, 0);
    exchange(&mut h, 1);
}
#[test]
fn an_invalid_asked_code_is_lost() {
    let mut h = Harness::new(&limits());
    let exchange = exchange(&mut h, 0);
    let body = encoded!(w::ApprovalAskAnswer::Asked(w::ApprovalAsked::new(&w::CEILINGS, w::ApprovalAskedParts {
        secret: Secret::new(Box::new([7; 32])), code: *b"AAAAAAAA", expires: 300,
    }).unwrap()));
    h.answer(exchange, &body);
    assert_eq!(h.out.pop(), Some(Request::Approval(Approval::NotAsked(NotAsked::Lost))));
}

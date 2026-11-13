//! The service passes on the approval setting (temper/docs/design/protocol/configuration/engine.md).
use super::fixtures;
use jig_client_protocol_wire::{ApprovalCode, CODE_ALPHABET, CODE_LETTERS, v20 as w};
use skein_lib::{Reader, Time, Wall, Writer, bytes};

// jig/docs/design/protocol/client/sign-in.md: 32 secret bytes, valid for ten minutes.
const SECRET_BYTES: usize = 32;
const SPAN_NANOS: u64 = 600_000_000_000;

#[test]
fn an_ask_is_answered_as_the_document_enables_the_approval() {
    for (approval, asked) in [(Some(true), true), (None, false), (Some(false), false)] {
        let mut service = match approval {
            Some(enabled) => fixtures::started_with_approval(enabled),
            None => fixtures::started(),
        };
        fixtures::listening(&mut service);
        let engine = service.engine.as_mut().expect("engine");
        let limits = &engine.clients.env.limits.wire;
        let ask = w::ApprovalAsk::new(limits, w::ApprovalAskParts {}).unwrap();
        let mut body = Writer::new(usize::try_from(ask.measure()).unwrap());
        ask.encode(&mut body).unwrap();
        let mut peer = fixtures::http::Peer::new(engine, b"POST", b"/jig/v1/approval/ask", &body.finish(), None);
        let wall = Wall::from_nanos(123_000_000_000);
        peer.pump_at(engine, false, Time::from_nanos(1), wall);
        assert!(peer.received().starts_with(b"HTTP/1.1 200 "));
        let at = bytes::find(peer.received(), b"\r\n\r\n").unwrap().checked_add(4).unwrap();
        let mut body = Reader::new(peer.received().get(at..).unwrap());
        let answer = w::ApprovalAskAnswer::decode(&engine.clients.env.limits.wire, &mut body).unwrap();
        match answer {
            w::ApprovalAskAnswer::Asked(value) => {
                assert!(asked, "approval {approval:?}");
                assert_eq!(value.secret().len(), SECRET_BYTES);
                assert_eq!(value.code().len(), CODE_LETTERS);
                for letter in value.code() {
                    assert!(CODE_ALPHABET.contains(letter));
                }
                assert!(ApprovalCode::parse(value.code()).is_some());
                assert_eq!(value.expires(), wall.as_nanos().checked_add(SPAN_NANOS).unwrap());
            }
            w::ApprovalAskAnswer::Refused => assert!(!asked, "approval {approval:?}"),
            w::ApprovalAskAnswer::NotReady | w::ApprovalAskAnswer::Busy => panic!("the ask can be answered"),
        }
        assert!(body.is_empty());
    }
}

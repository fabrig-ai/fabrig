use super::{credential, mapping};
use crate::{Frames, Limits, exchange, stream};
use jig_client_protocol_link::{Heard, Message};
use skein_http::{Header, Method};
use skein_lib::{Queue, Token};

const EXCHANGE: Token = Token::new(7);
const LIMITS: Limits = Limits { record: 3, chunk: 16 };
const WHOLE: &[u8] = b"\0\0\0\x03abc\0\0\0\x01d";
fn part(bytes: &[u8]) -> Heard {
    Heard::Part { exchange: EXCHANGE, message: bytes.into() }
}

#[test]
fn a_batch_posts_its_record_with_the_bearer() {
    let call = exchange(&mapping(), Some(&credential()), Message::Batch(b"batch record".as_slice().into()));
    assert_eq!(call.method, Method::Post);
    assert_eq!(call.path.as_ref(), b"/jig/v1/batch");
    assert_eq!(call.body.as_ref(), b"batch record");
    assert!(call.headers.as_slice().contains(&Header {
        name: b"Content-Type".as_slice().into(),
        value: b"application/octet-stream".as_slice().into(),
    }));
    assert!(call.headers.as_slice().contains(&Header {
        name: b"Authorization".as_slice().into(),
        value: b"Bearer CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ".as_slice().into(),
    }));
}

#[test]
fn frames_release_parts_as_their_last_byte_arrives() {
    let mut frames = Frames::new(EXCHANGE, &LIMITS);
    let mut out = Queue::uncapped();
    frames.opened(200, &mut out);
    frames.body(b"\0\0\0", &mut out);
    assert!(out.is_empty());
    frames.body(b"\x03ab", &mut out);
    assert!(out.is_empty());
    frames.body(b"c\0\0\0\x01d", &mut out);
    assert_eq!(out.pop(), Some(part(b"abc")));
    assert_eq!(out.pop(), Some(part(b"d")));
    assert!(out.is_empty());
    assert!(frames.active());
    frames.ended(&mut out);
    assert_eq!(out.pop(), Some(Heard::Done { exchange: EXCHANGE }));
    frames.body(WHOLE, &mut out);
    frames.ended(&mut out);
    frames.failed(&mut out);
    assert!(out.is_empty());
}

#[test]
fn a_short_or_long_frame_loses_after_whole_parts() {
    for tail in [b"\0".as_slice(), b"\0\0\0", b"\0\0\0\x03", b"\0\0\0\x03ab", b"\0\0\0\x04abcd"] {
        let mut frames = Frames::new(EXCHANGE, &LIMITS);
        let mut out = Queue::uncapped();
        frames.body(WHOLE, &mut out);
        frames.body(tail, &mut out);
        frames.ended(&mut out);
        assert_eq!(out.pop(), Some(part(b"abc")), "{tail:?}");
        assert_eq!(out.pop(), Some(part(b"d")), "{tail:?}");
        assert_eq!(out.pop(), Some(Heard::Lost { exchange: EXCHANGE }), "{tail:?}");
        frames.failed(&mut out);
        assert!(out.is_empty());
    }
}

#[test]
fn a_refused_or_failed_batch_is_lost_once() {
    for status in [201, 401, 500] {
        let mut frames = Frames::new(EXCHANGE, &LIMITS);
        let mut out = Queue::uncapped();
        frames.opened(status, &mut out);
        frames.body(WHOLE, &mut out);
        frames.ended(&mut out);
        frames.failed(&mut out);
        assert_eq!(out.pop(), Some(Heard::Lost { exchange: EXCHANGE }));
        assert!(out.is_empty());
    }
    let mut frames = Frames::new(EXCHANGE, &LIMITS);
    let mut out = Queue::uncapped();
    frames.body(WHOLE, &mut out);
    frames.failed(&mut out);
    frames.ended(&mut out);
    assert_eq!(out.pop(), Some(part(b"abc")));
    assert_eq!(out.pop(), Some(part(b"d")));
    assert_eq!(out.pop(), Some(Heard::Lost { exchange: EXCHANGE }));
    assert!(out.is_empty());
}

#[test]
fn a_stream_opening_names_the_stream_it_replaces() {
    let name = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10];
    let call = stream(&mapping(), None, Some(name));
    assert_eq!(call.method, Method::Get);
    assert_eq!(call.path.as_ref(), b"/jig/v1/stream?replaces=0123456789abcdeffedcba9876543210");
    assert!(call.body.is_empty());
    assert_eq!(stream(&mapping(), None, None).path.as_ref(), b"/jig/v1/stream");
}

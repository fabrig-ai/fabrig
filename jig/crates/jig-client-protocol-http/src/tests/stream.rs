use super::fixtures;
use crate::Stream;
use alloc::boxed::Box;
use jig_client_protocol_link::{Heard, Streamed};
use skein_lib::Queue;
const CUTS: [usize; 2] = [fixtures::WHOLE_STREAM.len(), 1];
const STATUSES: [u16; 3] = [201, 409, 503];
#[test]
fn the_body_cut_at_each_byte_gives_the_same_records() {
    for cut in 0..=fixtures::WHOLE_STREAM.len() {
        let (mut stream, mut out) = fixtures::reader();
        stream.body(fixtures::WHOLE_STREAM.get(..cut).unwrap(), &mut out);
        stream.body(fixtures::WHOLE_STREAM.get(cut..).unwrap(), &mut out);
        for expected in fixtures::RECORDS {
            assert_eq!(
                out.pop(),
                Some(Heard::Streamed { message: Streamed::Record(Box::from(expected)) }),
                "cut {cut}"
            );
        }
        assert!(out.is_empty(), "cut {cut}");
    }
}
#[test]
fn an_event_s_data_read_as_it_arrives_gives_the_same_record() {
    for size in CUTS {
        let (mut stream, mut out) = fixtures::reader();
        for piece in fixtures::WHOLE_STREAM.chunks(size) {
            stream.body(piece, &mut out);
        }
        for expected in fixtures::RECORDS {
            assert_eq!(
                out.pop(),
                Some(Heard::Streamed { message: Streamed::Record(Box::from(expected)) }),
                "size {size}"
            );
        }
        assert!(out.is_empty(), "size {size}");
    }
}
#[test]
fn malformed_base64_ends_the_stream() {
    let (mut stream, mut out) = fixtures::reader();
    stream.body(b"data: !!!!\n\n", &mut out);
    assert_eq!(out.pop(), Some(Heard::StreamGone));
    assert!(out.is_empty());
}
#[test]
fn a_comment_line_is_a_heartbeat() {
    let (mut stream, mut out) = fixtures::reader();
    stream.body(b": heartbeat\n\n", &mut out);
    assert_eq!(out.pop(), Some(Heard::Streamed { message: Streamed::Heartbeat }));
    assert!(out.is_empty());
}
#[test]
fn a_401_is_the_stream_refused_signed_out() {
    let mut stream = Stream::new(&fixtures::stream_limits());
    let mut out = Queue::with_capacity(4);
    stream.opened(401, &mut out);
    assert_eq!(out.pop(), Some(Heard::Streamed { message: Streamed::SignedOut }));
    stream.body(fixtures::WHOLE_STREAM, &mut out);
    stream.ended(&mut out);
    assert!(out.is_empty());
}
#[test]
fn another_status_other_than_200_ends_the_stream() {
    for status in STATUSES {
        let mut stream = Stream::new(&fixtures::stream_limits());
        let mut out = Queue::with_capacity(4);
        stream.opened(status, &mut out);
        stream.ended(&mut out);
        assert_eq!(out.pop(), Some(Heard::StreamGone), "status {status}");
        assert!(out.is_empty(), "status {status}");
    }
}
#[test]
fn a_piece_after_the_end_is_dropped() {
    let (mut stream, mut out) = fixtures::reader();
    stream.ended(&mut out);
    assert_eq!(out.pop(), Some(Heard::StreamGone));
    stream.body(fixtures::WHOLE_STREAM, &mut out);
    stream.ended(&mut out);
    assert!(out.is_empty());
}

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    let base = fixtures::stream_limits();
    assert!(crate::valid(&base));
    for limits in [
        crate::Limits { record: 0, ..base },
        crate::Limits { chunk: 0, ..base },
        crate::Limits { record: u32::MAX, ..base },
        // Base64 itself fits; adding the SSE framing allowance overflows.
        crate::Limits { record: 3_221_225_400, ..base },
    ] {
        assert!(!crate::valid(&limits), "{limits:?}");
    }
}

#[test]
fn a_409_with_a_bound_record_is_full_at_every_cut() {
    for bound in [jig_client_protocol_wire::StreamBound::Component, jig_client_protocol_wire::StreamBound::Transport] {
        let record = jig_client_protocol_wire::StreamsFull::new(
            &jig_client_protocol_wire::CEILINGS,
            jig_client_protocol_wire::StreamsFullParts { bound: bound.clone(), size: 2 },
        )
        .unwrap();
        let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut writer).unwrap();
        let body = writer.finish();
        for cut in 0..=body.len() {
            let mut stream = Stream::new(&fixtures::stream_limits());
            let mut out = Queue::uncapped();
            stream.opened(409, &mut out);
            stream.body(&body[..cut], &mut out);
            stream.body(&body[cut..], &mut out);
            assert!(out.is_empty());
            stream.ended(&mut out);
            let bound = match bound {
                jig_client_protocol_wire::StreamBound::Component => jig_client_protocol_link::StreamBound::Component,
                jig_client_protocol_wire::StreamBound::Transport => jig_client_protocol_link::StreamBound::Transport,
            };
            assert_eq!(
                out.pop(),
                Some(Heard::Streamed {
                    message: Streamed::Full(jig_client_protocol_link::StreamsFull { bound, size: 2 })
                })
            );
            stream.ended(&mut out);
            assert!(out.is_empty());
        }
    }
}
#[test]
fn a_409_with_a_malformed_or_oversized_record_is_gone() {
    for body in [b"".as_slice(), &[9, 2, 0, 0, 0], &[1, 2, 0], &[1, 2, 0, 0, 0, 0]] {
        let mut stream = Stream::new(&fixtures::stream_limits());
        let mut out = Queue::uncapped();
        stream.opened(409, &mut out);
        stream.body(body, &mut out);
        stream.ended(&mut out);
        assert_eq!(out.pop(), Some(Heard::StreamGone));
        assert!(out.is_empty());
    }
}

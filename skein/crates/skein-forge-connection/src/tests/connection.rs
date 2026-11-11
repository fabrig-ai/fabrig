//! Connection machinery and each forge's start.
use super::fixtures;
use crate::{
    connection::Phase,
    grow::{Full, Grow, LogTail},
};
use alloc::boxed::Box;
use skein_forge::forgejo::plans::{Failure, Heard, plan::Range};
use skein_forge::forgejo::request;
use skein_lib::{
    Duration, Time, Wall,
    stream::{Fault, Up},
};

#[test]
fn the_capped_buffer_doubles_and_refuses_a_write_past_its_cap() {
    let mut grow = Grow::new(4, 10);
    let rows: [(&str, &[u8], u32); 3] = [("first piece", b"abc", 4), ("doubled", b"def", 8), ("capped", b"ghij", 10)];
    for (name, piece, expected) in rows {
        assert_eq!(grow.write(piece), Ok(()), "{name}: piece fits");
        assert_eq!(grow.capacity(), expected, "{name}: capacity");
    }
    assert_eq!(grow.write(b"k"), Err(Full), "one past refused whole");
    assert_eq!(grow.finish().as_ref(), b"abcdefghij", "nothing changed by refusal");
}

#[test]
fn a_reset_before_the_first_byte_is_unseen_and_after_it_seen() {
    for (name, send, expected) in [("before room", false, Failure::Unseen), ("after send", true, Failure::Seen)] {
        let mut h = fixtures::ConnectionHarness::new();
        h.start_forgejo(request::Request::User, None).expect("valid request");
        h.pump();
        if send {
            h.connection.stream(&h.env, Up::Room);
            h.pump();
        }
        h.connection.stream(&h.env, Up::Failed(Fault::Reset));
        h.pump();
        assert_eq!(h.forgejo.heard(&mut h.connection), Some(Heard::Failed(expected)), "{name}");
        assert_eq!(h.forgejo.heard(&mut h.connection), None, "{name}: one failure");
    }
}

#[test]
fn a_failed_connection_holds_link_work_until_close() {
    let mut h = fixtures::ConnectionHarness::new();
    h.start_forgejo(request::Request::User, None).expect("valid request");
    h.pump();
    h.connection.failed(Failure::Unseen);
    h.connection.stream(&h.env, Up::Room);
    h.pump();
    assert!(h.connection.link.has_work(), "closed connection holds link events");
    assert_eq!(h.connection.phase(), Phase::Closed);
    h.connection.close(&h.env, false, &mut h.io);
    h.pump();
    assert!(!h.connection.link.has_work(), "closing drains the link");
}

#[test]
#[expect(clippy::too_many_lines, reason = "one wire table covers validators, replacement and whole-body decoding")]
fn a_json_response_is_decoded_through_its_body_end() {
    let long = fixtures::filled(skein_forge::api::VALIDATOR_BYTES + 1, b'a');
    let at_bound = fixtures::filled(skein_forge::api::VALIDATOR_BYTES, b'a');
    for (label, etag, modified, sent_etag, sent_modified) in [
        ("none", b"".as_slice(), b"".as_slice(), None, None),
        ("ETag", b"\"a\"", b"", Some(b"\"a\"".as_slice()), None),
        ("modified", b"", b"Wed, 01 Jan 2020 00:00:00 GMT", None, Some(b"Wed, 01 Jan 2020 00:00:00 GMT".as_slice())),
        ("both", b"\"a\"", b"yesterday", Some(b"\"a\"".as_slice()), Some(b"yesterday".as_slice())),
        ("bound", at_bound.as_ref(), b"", Some(at_bound.as_ref()), None),
        ("past bound leaves other validator", long.as_ref(), b"yesterday", None, Some(b"yesterday".as_slice())),
        ("control", b"a\tbad", b"", None, None),
        ("non-ASCII", b"a\xff", b"", None, None),
        ("invalid modified leaves ETag", b"\"a\"", b"a\tbad", Some(b"\"a\"".as_slice()), None),
    ] {
        let mut length = 0_usize;
        for (name, value) in [(b"ETag: ".as_slice(), etag), (b"Last-Modified: ".as_slice(), modified)] {
            if !value.is_empty() {
                length = length.saturating_add(name.len()).saturating_add(value.len()).saturating_add(2);
            }
        }
        let mut fields = skein_lib::Writer::new(length);
        for (name, value) in [(b"ETag: ".as_slice(), etag), (b"Last-Modified: ".as_slice(), modified)] {
            if !value.is_empty() {
                fields.put(name).expect("fields fit");
                fields.put(value).expect("fields fit");
                fields.put(b"\r\n").expect("fields fit");
            }
        }
        let fields = fields.finish();
        let mut h = fixtures::ConnectionHarness::new();
        h.start_forgejo(request::Request::User, None).expect("valid request");
        let heard = h.answer(&crate::fixtures::response_with_fields(200, br#"{"id":2,"login":"writer"}"#, &fields));
        assert_eq!(heard.len(), 2, "{label}: head then document");
        assert_eq!(
            heard.get(0),
            Some(&Heard::Head {
                status: 200,
                fields: skein_forge::forgejo::plans::Fields {
                    date: Some(Wall::from_nanos(1_577_836_800_000_000_000)),
                    retry_after: None,
                    has_more: None,
                    content_range: None,
                    close: false
                }
            }),
            "response fields"
        );
        assert_eq!(
            heard.get(1),
            Some(&Heard::Done(skein_forge::forgejo::plans::Body::Document(
                skein_forge::forgejo::document::Document::User(skein_forge::forgejo::document::User {
                    id: 2,
                    login: Box::from(&b"writer"[..]),
                    is_admin: false,
                    full_name: Box::from([])
                })
            ))),
            "decoded whole"
        );
        assert_eq!(h.connection.phase(), Phase::Idle, "reusable");
        h.start_forgejo(request::Request::User, None).expect("repeat request");
        let reusable = sent_etag.is_some() || sent_modified.is_some();
        let next = if reusable {
            b"HTTP/1.1 304 Not Modified\r\nDate: Thu, 02 Jan 2020 00:00:00 GMT\r\n\r\n".as_slice()
        } else {
            b"HTTP/1.1 200 OK\r\nContent-Length: 25\r\n\r\n{\"id\":2,\"login\":\"writer\"}"
        };
        let replayed = h.answer(next);
        for (name, expected) in
            [(b"If-None-Match: ".as_slice(), sent_etag), (b"If-Modified-Since: ".as_slice(), sent_modified)]
        {
            assert_eq!(
                skein_lib::bytes::find(&h.sent, name).is_some(),
                expected.is_some(),
                "{label}: conditional header"
            );
            if let Some(expected) = expected {
                assert_eq!(fixtures::header(&h.sent, name), expected, "{label}");
            }
        }
        assert_eq!(replayed.get(1), heard.get(1), "{label}: decoded through the same body end");
        assert_eq!(h.connection.take_free(), u32::from(reusable), "{label}: reused 304 counted once");
        if reusable {
            let Some(Heard::Head { status: 200, fields }) = replayed.get(0) else { panic!("replayed 200 head") };
            assert_eq!(fields.date, Some(Wall::from_nanos(1_577_923_200_000_000_000)), "{label}: 304's Date");
            for (update, conditional) in [(b"ETag: \"b\"\r\n".as_slice(), true), (b"".as_slice(), false)] {
                let body = br#"{"id":3,"login":"reader"}"#;
                h.start_forgejo(request::Request::User, None).expect("replace read");
                let fresh = h.answer(&crate::fixtures::response_with_fields(200, body, update));
                h.start_forgejo(request::Request::User, None).expect("read replaced answer");
                let replayed =
                    if conditional { h.answer(&fixtures::response(304, b"")) } else { h.answer(&fixtures::ok(body)) };
                assert_eq!(
                    contains(&h.sent, b"If-None-Match:"),
                    conditional,
                    "{label}: 200 replaces or drops validators"
                );
                if conditional {
                    assert_eq!(fixtures::header(&h.sent, b"If-None-Match: "), b"\"b\"");
                }
                assert!(!contains(&h.sent, b"If-Modified-Since:"), "{label}: 200 drops the old modified validator");
                assert_eq!(replayed.get(1), fresh.get(1), "{label}: replacement body, never the old one");
                assert_eq!(h.connection.take_free(), u32::from(conditional));
            }
        }
    }

    for eof in [false, true] {
        let mut h = fixtures::ConnectionHarness::new();
        h.start_forgejo(request::Request::User, None).expect("partial read");
        let partial = b"HTTP/1.1 200 OK\r\nETag: \"partial\"\r\nContent-Length: 100\r\n\r\n{\"id\":2,\"login\":\"wri";
        let heard = h.buffered_answer(partial, partial.len(), eof);
        let Some(Heard::Head { status: 200, .. }) = heard.get(0) else { panic!("validator-bearing head received") };
        h.connection.close(&h.env, true, &mut h.io);
        let mut next = fixtures::ConnectionHarness::new();
        next.kept = h.kept;
        next.start_forgejo(request::Request::User, None).expect("another connection reads whole");
        next.answer(&fixtures::user_ok());
        assert!(!contains(&next.sent, b"If-None-Match:"), "eof{eof}: an incomplete body is never kept");
    }
}

#[test]
fn a_malformed_document_closes_without_reading_more() {
    let mut h = fixtures::ConnectionHarness::new();
    h.start_forgejo(request::Request::User, None).expect("valid request");
    let heard = h.answer(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}");
    let Some(Heard::Done(skein_forge::forgejo::plans::Body::Malformed(_))) = heard.last() else {
        unreachable!("malformed terminal");
    };
    assert_eq!(h.connection.phase(), Phase::Closed, "malformed closes");
}

#[test]
fn a_connection_close_answer_ends_its_connection_after_it() {
    let mut h = fixtures::ConnectionHarness::new();
    h.start_forgejo(request::Request::User, None).expect("valid request");
    let heard =
        h.answer(b"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 25\r\n\r\n{\"id\":2,\"login\":\"writer\"}");
    let Some(Heard::Done(skein_forge::forgejo::plans::Body::Document(_))) = heard.last() else {
        unreachable!("document ends first");
    };
    assert_eq!(h.connection.phase(), Phase::Closed, "answer closed connection");
}

#[test]
fn raw_reads_keep_the_range_and_refuse_bytes_past_remaining_answer_room() {
    let rows: [RawRow; 5] = [
        ("short", b"ab", 3, 10, b"ab", false, Phase::Idle, b"Range: bytes=0-2\r\n"),
        ("exact max", b"abc", 3, 10, b"abc", false, Phase::Idle, b"Range: bytes=0-2\r\n"),
        ("one more", b"abcdef", 3, 10, b"abcd", true, Phase::Closed, b"Range: bytes=0-2\r\n"),
        ("remaining answer", b"abcdef", 5, 3, b"abc", true, Phase::Closed, b"Range: bytes=0-4\r\n"),
        ("empty", b"", 3, 10, b"", false, Phase::Idle, b"Range: bytes=0-2\r\n"),
    ];
    for (name, response, max, room, expected, more, phase, range) in rows {
        let mut h = fixtures::ConnectionHarness::new();
        h.connection.raw_room(room);
        h.start_forgejo(raw_request(), Some(Range::Head(max))).expect("valid raw request");
        let heard = h.answer(&crate::fixtures::response_with_fields(200, response, b"ETag: \"raw\"\r\n"));
        assert_eq!(
            heard.last(),
            Some(&Heard::Done(skein_forge::forgejo::plans::Body::Raw { bytes: Box::from(expected), more })),
            "{name}: raw body"
        );
        assert_eq!(h.connection.phase(), phase, "{name}: connection state");
        assert!(contains(&h.sent, range), "{name}: range header");
        let mut next = fixtures::ConnectionHarness::new();
        next.kept = h.kept;
        if phase == Phase::Idle {
            next.start_forgejo(raw_request(), Some(Range::Tail(max))).expect("different range");
            next.answer(&fixtures::ok(b"x"));
            assert!(!contains(&next.sent, b"If-None-Match:"), "{name}: Range is part of the key");
        }
        next.start_forgejo(raw_request(), Some(Range::Head(max))).expect("repeat head range");
        let replayed = if phase == Phase::Idle {
            next.answer(&fixtures::response(304, b""))
        } else {
            next.answer(&fixtures::ok(b""))
        };
        assert_eq!(contains(&next.sent, b"If-None-Match:"), phase == Phase::Idle, "{name}: cut bodies keep nothing");
        if phase == Phase::Idle {
            assert_eq!(replayed, heard, "{name}: raw retention replays unchanged");
        }
    }
}
#[test]
fn head_ranges_count_bytes_and_report_a_truncated_partial_response() {
    for (max, body, fields, more, range) in [
        (1, b"a".as_slice(), b"Content-Range: bytes 0-0/8\r\n".as_slice(), true, b"Range: bytes=0-0\r\n".as_slice()),
        (4, b"abcd".as_slice(), b"Content-Range: bytes 0-3/8\r\n".as_slice(), true, b"Range: bytes=0-3\r\n".as_slice()),
        (
            4,
            b"abcd".as_slice(),
            b"Content-Range: bytes 0-3/4\r\n".as_slice(),
            false,
            b"Range: bytes=0-3\r\n".as_slice(),
        ),
    ] {
        let mut h = fixtures::ConnectionHarness::new();
        h.start_forgejo(raw_request(), Some(Range::Head(max))).expect("valid head range");
        let response = crate::fixtures::response_with_fields(206, body, fields);
        let heard = h.answer(&response);
        assert!(contains(&h.sent, range), "Head(n) requests exactly n bytes");
        assert_eq!(
            heard.last(),
            Some(&Heard::Done(skein_forge::forgejo::plans::Body::Raw { bytes: Box::from(body), more })),
            "Content-Range reports bytes omitted after the retained prefix"
        );
    }
}

type TailRow = (&'static str, &'static [u8], &'static [u8], bool, Phase);
type RawRow = (&'static str, &'static [u8], u32, u32, &'static [u8], bool, Phase, &'static [u8]);
fn contains(bytes: &[u8], expected: &[u8]) -> bool {
    for piece in bytes.windows(expected.len()) {
        if piece == expected {
            return true;
        }
    }
    false
}
fn raw_request() -> request::Request {
    request::Request::Raw {
        at: request::Repo { owner: Box::from(&b"fixture"[..]), name: Box::from(&b"specimen"[..]) },
        path: Box::from(&b"file"[..]),
        reference: skein_forge::forgejo::id::ObjectId {
            format: skein_forge::forgejo::id::ObjectFormat::Sha1,
            bytes: [0; 32],
        },
    }
}

#[test]
fn a_json_upload_sends_its_body_and_content_headers() {
    let mut h = fixtures::ConnectionHarness::new();
    h.start_forgejo(
        request::Request::RemoveReviewers {
            at: request::Repo { owner: Box::from(&b"fixture"[..]), name: Box::from(&b"specimen"[..]) },
            number: 7,
            reviewers: Box::from([Box::from(&b"alice"[..])]),
        },
        None,
    )
    .expect("valid upload");
    let heard = h.answer(b"HTTP/1.1 204 No Content\r\n\r\n");
    assert_eq!(heard.last(), Some(&Heard::Done(skein_forge::forgejo::plans::Body::Empty)), "bodyless answer");
    assert_eq!(h.sent.as_ref(), b"DELETE /api/v1/repos/fixture/specimen/pulls/7/requested_reviewers HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nAuthorization: token 0123456789abcdef0123456789abcdef01234567\r\nUser-Agent: kit/0.0.0\r\nContent-Type: application/json\r\nContent-Length: 23\r\n\r\n{\"reviewers\":[\"alice\"]}", "encoded upload");
}

#[test]
fn a_chunked_raw_read_keeps_the_last_short_piece() {
    let mut h = fixtures::ConnectionHarness::new();
    h.start_forgejo(raw_request(), Some(Range::Head(3))).expect("valid raw request");
    let heard = h.answer(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nab\r\n0\r\n\r\n");
    assert_eq!(
        heard.last(),
        Some(&Heard::Done(skein_forge::forgejo::plans::Body::Raw { bytes: Box::from(&b"ab"[..]), more: false })),
        "final bytes kept"
    );
    assert_eq!(h.connection.phase(), Phase::Idle, "complete body reusable");
}

#[test]
fn the_tail_keeps_the_last_max_bytes_across_writes() {
    for (name, pieces, expected, dropped) in [
        ("short", [b"ab".as_slice(), b"", b""], b"ab".as_slice(), false),
        ("wrapped", [b"ab".as_slice(), b"cde", b"f"], b"cdef".as_slice(), true),
    ] {
        let mut tail = LogTail::new(4);
        for piece in pieces {
            tail.write(piece);
        }
        assert_eq!(tail.dropped(), dropped, "{name}: Earlier bytes");
        assert_eq!(tail.finish().as_ref(), expected, "{name}: Last bytes");
    }
}
#[test]
fn a_tail_read_keeps_a_200s_last_bytes_and_closes_when_some_were_dropped() {
    let rows: [TailRow; 3] = [
        ("short", b"ab", b"ab", false, Phase::Idle),
        ("max", b"abc", b"abc", false, Phase::Idle),
        ("longer", b"abcdef", b"def", true, Phase::Closed),
    ];
    for (name, response, expected, more, phase) in rows {
        let mut h = fixtures::ConnectionHarness::new();
        h.start_forgejo(raw_request(), Some(Range::Tail(3))).expect("Valid raw request");
        let heard = h.answer(&fixtures::ok(response));
        let kept = Heard::Done(skein_forge::forgejo::plans::Body::Raw { bytes: Box::from(expected), more });
        assert_eq!(heard.last(), Some(&kept), "{name}: The last bytes");
        assert_eq!(h.connection.phase(), phase, "{name}: Connection state");
        assert!(contains(&h.sent, b"Range: bytes=-3\r\n"), "{name}: Suffix range");
    }
}
#[test]
fn a_tail_read_keeps_at_most_the_raw_room() {
    let mut h = fixtures::ConnectionHarness::new();
    h.connection.raw_room(8);
    h.start_forgejo(raw_request(), Some(Range::Tail(16))).expect("Valid raw request");
    let heard = h.answer(&fixtures::ok(b"abcdefghijklmnopqrst"));
    assert_eq!(
        heard.last(),
        Some(&Heard::Done(skein_forge::forgejo::plans::Body::Raw { bytes: Box::from(&b"mnopqrst"[..]), more: true }))
    );
    assert_eq!(h.connection.phase(), Phase::Closed, "Earlier bytes were dropped");
}
#[test]
fn a_chunked_tail_read_keeps_the_last_bytes_across_pieces() {
    let mut h = fixtures::ConnectionHarness::new();
    h.start_forgejo(raw_request(), Some(Range::Tail(3))).expect("Valid raw request");
    let heard = h.answer(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\nab\r\n3\r\ncde\r\n0\r\n\r\n");
    assert_eq!(
        heard.last(),
        Some(&Heard::Done(skein_forge::forgejo::plans::Body::Raw { bytes: Box::from(&b"cde"[..]), more: true }))
    );
    assert_eq!(h.connection.phase(), Phase::Closed, "Earlier bytes were dropped");
}
#[test]
fn a_tail_read_past_body_bytes_in_is_malformed_and_closes() {
    let mut h = fixtures::ConnectionHarness::new();
    h.env.limits.body_bytes_in = 8;
    h.start_forgejo(raw_request(), Some(Range::Tail(3))).expect("Valid raw request");
    let heard = h.answer(&fixtures::ok(b"abcdefghi"));
    assert_eq!(
        heard.last(),
        Some(&Heard::Done(skein_forge::forgejo::plans::Body::Malformed(skein_forge::decode::Malformed::Bound {
            field: b"log"
        })))
    );
    assert_eq!(h.connection.phase(), Phase::Closed, "Body limit closes");
}

#[test]
fn terminal_start_states_ignore_every_start_answer() {
    use crate::start::{self, Start};
    use skein_forge::forgejo::plans::start::Heard as StartHeard;
    // The terminal matrix uses the 23-second backoff horizon and the same limits.
    let now = Time::ZERO.saturating_add(Duration::from_secs(23));
    let limits = fixtures::limits();
    for state in [Start::Unconnected, Start::Ready { cap: 2 }, Start::Stopped(crate::Stopped::OtherUser { user: 1 })] {
        for heard in [
            StartHeard::Unanswered,
            StartHeard::User { id: fixtures::WRITER },
            StartHeard::ApiSettings { max_response_items: 2 },
        ] {
            assert_eq!(start::heard(state, heard, fixtures::WRITER, now, &limits), state, "{state:?}/{heard:?}");
        }
    }
}

#[test]
fn a_failed_step_tail_stops_before_cleanup_across_every_chunk_boundary() {
    let input = b"earlier\nMissing changes/40.md opened timestamp\n2026-11-02T14:01:45Z ##[error]Process completed with exit code 1.\nPost job cleanup.\n";
    let expected = input.strip_prefix(b"earlier\n").unwrap().strip_suffix(b"Post job cleanup.\n").unwrap();
    for chunk in 1..=input.len() {
        let mut tail = LogTail::failed_step(u32::try_from(expected.len()).unwrap());
        for piece in input.chunks(chunk) {
            tail.write(piece);
        }
        assert!(tail.dropped());
        assert_eq!(tail.finish().as_ref(), expected, "chunk {chunk}");
    }
}

#[test]
fn a_failed_step_tail_falls_back_and_reports_omission_at_tiny_bounds() {
    for (input, cap, expected, dropped) in [
        (b"no failure marker\n".as_slice(), 5, b"rker\n".as_slice(), true),
        (b"whole", 5, b"whole", false),
        (b"log", 0, b"", true),
        (b"##[error]Process completed with exit code 1.", 4, b"e 1.", true),
        (b"##[error]Process completed with exit code 1.\ncleanup", 2, b".\n", true),
    ] {
        let mut tail = LogTail::failed_step(cap);
        for byte in input {
            tail.write(core::slice::from_ref(byte));
        }
        assert_eq!(tail.dropped(), dropped);
        assert_eq!(tail.finish().as_ref(), expected);
    }
}

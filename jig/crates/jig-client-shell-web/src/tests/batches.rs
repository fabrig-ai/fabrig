use crate::adapter::{self, Asked, Exchanges};
use jig_client_protocol_http::{Heard, Limits, Mapping};
use jig_client_protocol_link::Message;
use skein_lib::{Queue, Token};
use skein_ui_web::fetch;

const ID: u32 = 7;
const TOKEN: Token = Token::new(17);
const LIMITS: Limits = Limits { record: 3, chunk: 16 };
fn report(exchanges: &mut Exchanges, report: fetch::Report, out: &mut Queue<Heard>) {
    exchanges.heard(ID, adapter::fetched(report), out);
}
fn part(bytes: &[u8]) -> Heard {
    Heard::Part { exchange: TOKEN, message: bytes.into() }
}

#[test]
fn a_pieced_flight_answered_whole_releases_each_part_then_done() {
    let mut exchanges = Exchanges::default();
    exchanges.insert(ID, TOKEN, Some(&LIMITS));
    let mut out = Queue::uncapped();
    report(
        &mut exchanges,
        fetch::Report::Answered {
            exchange: ID,
            status: 200,
            body: b"\0\0\0\x03abc\0\0\0\x01d".as_slice().into(),
            headers: vec![],
        },
        &mut out,
    );
    assert_eq!(out.pop(), Some(part(b"abc")));
    assert_eq!(out.pop(), Some(part(b"d")));
    assert_eq!(out.pop(), Some(Heard::Done { exchange: TOKEN }));
    report(&mut exchanges, fetch::Report::Failed { exchange: ID }, &mut out);
    assert!(out.is_empty(), "the whole answer retires the flight");
}

#[test]
fn a_whole_pieced_refusal_is_lost_once() {
    let mut exchanges = Exchanges::default();
    exchanges.insert(ID, TOKEN, Some(&LIMITS));
    let mut out = Queue::uncapped();
    report(
        &mut exchanges,
        fetch::Report::Answered {
            exchange: ID,
            status: 500,
            body: b"\0\0\0\x03abc\0\0\0\x01d".as_slice().into(),
            headers: vec![],
        },
        &mut out,
    );
    assert_eq!(out.pop(), Some(Heard::Lost { exchange: TOKEN }));
    report(&mut exchanges, fetch::Report::Failed { exchange: ID }, &mut out);
    report(&mut exchanges, fetch::Report::Ended { exchange: ID }, &mut out);
    assert!(out.is_empty(), "a refused whole answer retires the flight");
}

#[test]
fn a_whole_pieced_body_cut_inside_a_frame_keeps_only_whole_parts() {
    let mut exchanges = Exchanges::default();
    exchanges.insert(ID, TOKEN, Some(&LIMITS));
    let mut out = Queue::uncapped();
    report(
        &mut exchanges,
        fetch::Report::Answered {
            exchange: ID,
            status: 200,
            body: b"\0\0\0\x03abc\0\0\0\x02d".as_slice().into(),
            headers: vec![],
        },
        &mut out,
    );
    assert_eq!(out.pop(), Some(part(b"abc")));
    assert_eq!(out.pop(), Some(Heard::Lost { exchange: TOKEN }));
    report(&mut exchanges, fetch::Report::Ended { exchange: ID }, &mut out);
    assert!(out.is_empty(), "a short whole answer retires the flight");
}

#[test]
fn a_batch_fetch_gives_parts_before_its_end() {
    assert_eq!(
        adapter::read_on_exchange(&fetch::Report::Opened { exchange: ID, status: 200, headers: vec![] }),
        Some(ID)
    );
    assert_eq!(adapter::read_on_exchange(&fetch::Report::Piece { exchange: ID, bytes: Box::new([]) }), Some(ID));
    assert_eq!(adapter::read_on_exchange(&fetch::Report::Ended { exchange: ID }), None);
    let mapping = Mapping::new(b"/jig").expect("mapping");
    let call = jig_client_protocol_http::exchange(&mapping, None, Message::Batch(b"batch".as_slice().into()));
    let requests = adapter::requests(vec![Asked::Exchange {
        id: ID,
        call,
        bound: 64,
        pieces: Some(LIMITS.chunk),
        cache: fetch::Cache::Browser,
        keep: &[],
    }]);
    let sent = &requests.exchanges[0];
    assert_eq!(sent.pieces, Some(16));
    assert_eq!(sent.path.as_ref(), b"/jig/v1/batch");
    assert_eq!(sent.body.as_ref(), b"batch");
    assert!(sent.headers.contains(&("Jig-Client".into(), "browser".into())));
    let mut exchanges = Exchanges::default();
    exchanges.insert(ID, TOKEN, Some(&LIMITS));
    let mut out = Queue::uncapped();
    report(&mut exchanges, fetch::Report::Opened { exchange: ID, status: 200, headers: vec![] }, &mut out);
    report(&mut exchanges, fetch::Report::Piece { exchange: ID, bytes: b"\0\0\0".as_slice().into() }, &mut out);
    report(&mut exchanges, fetch::Report::Piece { exchange: ID, bytes: b"\x03ab".as_slice().into() }, &mut out);
    assert!(out.is_empty());
    // An ordinary exchange completes while this batch's frame is incomplete.
    exchanges.insert(8, Token::new(18), None);
    exchanges.heard(
        8,
        adapter::fetched(fetch::Report::Answered {
            exchange: 8,
            status: 200,
            body: b"ordinary".as_slice().into(),
            headers: vec![],
        }),
        &mut out,
    );
    assert_eq!(out.pop(), Some(Heard::Reply { exchange: Token::new(18), message: b"ordinary".as_slice().into() }));
    report(&mut exchanges, fetch::Report::Piece { exchange: ID, bytes: b"c\0\0\0\x01d".as_slice().into() }, &mut out);
    assert_eq!(out.pop(), Some(part(b"abc")));
    assert_eq!(out.pop(), Some(part(b"d")));
    assert!(out.is_empty(), "parts leave before the end");
    report(&mut exchanges, fetch::Report::Ended { exchange: ID }, &mut out);
    assert_eq!(out.pop(), Some(Heard::Done { exchange: TOKEN }));
    report(&mut exchanges, fetch::Report::Failed { exchange: ID }, &mut out);
    assert!(out.is_empty(), "a terminal retires the exchange");
}

#[test]
fn a_pieced_fetch_refusal_is_lost_once() {
    for status in [201, 401, 500] {
        let mut exchanges = Exchanges::default();
        exchanges.insert(ID, TOKEN, Some(&LIMITS));
        let mut out = Queue::uncapped();
        report(&mut exchanges, fetch::Report::Opened { exchange: ID, status, headers: vec![] }, &mut out);
        assert_eq!(out.pop(), Some(Heard::Lost { exchange: TOKEN }));
        report(
            &mut exchanges,
            fetch::Report::Piece { exchange: ID, bytes: b"\0\0\0\x01a".as_slice().into() },
            &mut out,
        );
        report(&mut exchanges, fetch::Report::Ended { exchange: ID }, &mut out);
        report(&mut exchanges, fetch::Report::Failed { exchange: ID }, &mut out);
        assert!(out.is_empty());
    }
}

#[test]
fn a_cut_or_failed_batch_keeps_only_its_whole_parts() {
    for (tail, failed) in
        [(b"\0".as_slice(), false), (b"\0\0\0\x03ab", false), (b"\0\0\0\x04", false), (b"".as_slice(), true)]
    {
        let mut exchanges = Exchanges::default();
        exchanges.insert(ID, TOKEN, Some(&LIMITS));
        let mut out = Queue::uncapped();
        report(&mut exchanges, fetch::Report::Opened { exchange: ID, status: 200, headers: vec![] }, &mut out);
        report(
            &mut exchanges,
            fetch::Report::Piece { exchange: ID, bytes: b"\0\0\0\x01a".as_slice().into() },
            &mut out,
        );
        assert_eq!(out.pop(), Some(part(b"a")));
        report(&mut exchanges, fetch::Report::Piece { exchange: ID, bytes: tail.into() }, &mut out);
        let terminal =
            if failed { fetch::Report::Failed { exchange: ID } } else { fetch::Report::Ended { exchange: ID } };
        report(&mut exchanges, terminal, &mut out);
        assert_eq!(out.pop(), Some(Heard::Lost { exchange: TOKEN }), "{tail:?}, failed {failed}");
        report(&mut exchanges, fetch::Report::Ended { exchange: ID }, &mut out);
        assert!(out.is_empty());
    }
}

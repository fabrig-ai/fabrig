//! A held buffer as a stream (runtime/lib.md, 7): each demand answered at once,
//! by exactly what it reads, or by the end.

use alloc::boxed::Box;

use crate::stream::{Delimiter, Held, Read, Slice, Up};

fn boxed(bytes: &[u8]) -> Box<[u8]> {
    Box::from(bytes)
}

/// Every read and expected answer in these tests exercises both forms.
struct Both<'a> {
    held: Held,
    slice: Slice<'a>,
}

impl<'a> Both<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { held: Held::new(boxed(bytes)), slice: Slice::new(bytes) }
    }

    fn answer(&mut self, read: Read) -> Option<Up> {
        let answer = self.held.answer(read);
        assert_eq!(self.slice.answer(read), answer, "{read:?}");
        answer
    }
}

#[test]
fn fills_and_scans_are_met_from_the_buffer_then_the_end_comes_once() {
    let mut data = Both::new(b"{\"a\":\"bc\"}");
    let quote = Delimiter::new(b"\"").expect("one byte");
    let reads = [
        (Read::Fill(1), Some(Up::Bytes(boxed(b"{")))),
        (Read::Scan { until: quote, max: 4 }, Some(Up::Bytes(boxed(b"\"")))),
        (Read::Scan { until: quote, max: 4 }, Some(Up::Bytes(boxed(b"a\"")))),
        (Read::Scan { until: quote, max: 2 }, Some(Up::Bytes(boxed(b":\"")))),
        (Read::Scan { until: quote, max: 1 }, Some(Up::Bytes(boxed(b"b")))),
        (Read::Fill(3), Some(Up::Bytes(boxed(b"c\"}")))),
        (Read::Fill(0), Some(Up::Bytes(boxed(b"")))),
        (Read::Fill(1), Some(Up::End)),
        (Read::Fill(1), None),
    ];
    for (read, expected) in reads {
        assert_eq!(data.answer(read), expected, "{read:?}");
    }
}

#[test]
fn a_read_larger_than_what_is_left_is_the_end() {
    let mut data = Both::new(b"abc");
    assert_eq!(data.answer(Read::Fill(4)), Some(Up::End));
    let mut data = Both::new(b"abc");
    assert_eq!(
        data.answer(Read::Scan { until: Delimiter::LF, max: 4 }),
        Some(Up::End),
        "no LF, and short of the maximum"
    );
    let mut data = Both::new(b"abc");
    assert_eq!(data.answer(Read::Scan { until: Delimiter::LF, max: 3 }), Some(Up::Bytes(boxed(b"abc"))));
}

#[test]
fn a_withdrawal_is_not_answered() {
    let mut data = Both::new(b"abc");
    assert_eq!(data.answer(Read::Nothing), None);
    assert_eq!(data.answer(Read::Fill(3)), Some(Up::Bytes(boxed(b"abc"))));
}

#[test]
fn a_delimiter_must_end_within_the_maximum() {
    let mut data = Both::new(b"ab\r\ncd");
    assert_eq!(data.answer(Read::Scan { until: Delimiter::CRLF, max: 3 }), Some(Up::Bytes(boxed(b"ab\r"))));
    assert_eq!(data.answer(Read::Scan { until: Delimiter::CRLF, max: 3 }), Some(Up::Bytes(boxed(b"\ncd"))));
}

#[test]
fn held_meets_a_read_of_what_has_arrived_with_what_is_left() {
    let mut held = Both::new(b"hello");
    assert_eq!(held.answer(Read::Arrived { max: 3 }), Some(Up::Bytes(boxed(b"hel"))));
    assert_eq!(held.answer(Read::Arrived { max: 64 }), Some(Up::Bytes(boxed(b"lo"))));
    assert_eq!(held.answer(Read::Arrived { max: 64 }), Some(Up::End));
    assert_eq!(held.answer(Read::Arrived { max: 64 }), None);
}

#[test]
fn slice_reads_each_byte_once_and_leaves_the_buffer_untouched() {
    let bytes = *b"ab\r\ncd\nef";
    let mut data = Both::new(&bytes);
    let reads = [
        (Read::Line { max: 4 }, Some(Up::Bytes(boxed(b"ab\r")))),
        (Read::Line { max: 4 }, Some(Up::Bytes(boxed(b"\n")))),
        (Read::Line { max: 3 }, Some(Up::Bytes(boxed(b"cd\n")))),
        (Read::Line { max: 2 }, Some(Up::Bytes(boxed(b"ef")))),
        (Read::Line { max: 1 }, Some(Up::End)),
        (Read::Fill(0), None),
    ];
    for (read, expected) in reads {
        assert_eq!(data.answer(read), expected, "{read:?}");
    }
    assert_eq!(&bytes, b"ab\r\ncd\nef");
}

use crate::{Reason, piece};
use skein_lib::{List, Overflow, Reader, Writer};

fn write_items(items: &[&[u8]], size: usize) -> Box<[u8]> {
    let mut out = Writer::new(size);
    for item in items {
        piece::write(&mut out, item).expect("the fixture fits its writer");
    }
    out.finish()
}

fn split_items(bytes: &[u8]) -> Result<Box<[Box<[u8]>]>, Reason> {
    let mut reader = Reader::new(bytes);
    let mut items = List::with_capacity(4);
    while let Some(item) = piece::split(&mut reader)? {
        items.push(item).expect("at most four fixture items");
    }
    Ok(items.into_boxed())
}

#[test]
fn each_written_item_has_its_u32_length() {
    assert_eq!(write_items(&[b"ab", b"", b"c"], 15).as_ref(), &[0, 0, 0, 2, b'a', b'b', 0, 0, 0, 0, 0, 0, 0, 1, b'c']);
}

struct WriteRefusal {
    room: usize,
    prior: &'static [&'static [u8]],
    item: &'static [u8],
    written: usize,
    fill: &'static [u8],
    expected: &'static [u8],
}

#[test]
fn an_item_exceeding_writer_room_leaves_output_unchanged() {
    skein_rows::check(
        [
            (
                "an_item_that_exceeds_the_remaining_bound_writes_nothing",
                WriteRefusal {
                    room: 6,
                    prior: &[b"ab"],
                    item: b"",
                    written: 6,
                    fill: b"",
                    expected: &[0, 0, 0, 2, b'a', b'b'],
                },
            ),
            (
                "an_item_header_that_exceeds_the_bound_writes_nothing",
                WriteRefusal { room: 3, prior: &[], item: b"", written: 0, fill: b"abc", expected: b"abc" },
            ),
            (
                "an_item_body_that_exceeds_the_bound_writes_no_header",
                WriteRefusal { room: 5, prior: &[], item: b"ab", written: 0, fill: b"abcde", expected: b"abcde" },
            ),
        ],
        |row| {
            let mut out = Writer::new(row.room);
            for item in row.prior {
                piece::write(&mut out, item).expect("the first item fills the bound");
            }
            assert_eq!(piece::write(&mut out, row.item), Err(Overflow));
            assert_eq!(out.written(), row.written);
            // finish requires a full writer; filling the untouched room also
            // detects any header or body left by the refused item.
            out.put(row.fill).expect("the untouched remainder fits");
            assert_eq!(out.finish().as_ref(), row.expected);
        },
    );
}

#[test]
fn small_and_large_items_fit_the_same_sum_bound() {
    assert_eq!(write_items(&[b"a", b"b"], 10).as_ref(), &[0, 0, 0, 1, b'a', 0, 0, 0, 1, b'b']);
    assert_eq!(write_items(&[b"abcdef"], 10).as_ref(), &[0, 0, 0, 6, b'a', b'b', b'c', b'd', b'e', b'f']);
}

#[test]
fn splitting_complete_pieces_returns_every_item_in_order() {
    skein_rows::check(
        [
            (
                "splitting_ends_only_between_complete_items",
                (
                    &[0, 0, 0, 2, b'a', b'b', 0, 0, 0, 0, 0, 0, 0, 1, b'c'][..],
                    Box::<[Box<[u8]>]>::from([Box::from(&b"ab"[..]), Box::from(&b""[..]), Box::from(&b"c"[..])]),
                ),
            ),
            ("an_empty_piece_has_no_items", (b"".as_slice(), Box::from([]))),
        ],
        |(bytes, expected)| {
            let items = split_items(bytes).expect("complete items");
            assert_eq!(items, expected);
        },
    );
}

#[test]
fn splitting_refuses_incomplete_item_lengths_and_item_bodies() {
    skein_rows::check(
        [
            ("an_item_length_past_the_end_is_malformed: short body", &[0, 0, 0, 2, b'a'][..]),
            ("an_item_length_past_the_end_is_malformed: maximum length", &[255, 255, 255, 255][..]),
            ("a_partial_item_length_at_the_tail_is_malformed: one byte", &b"\0"[..]),
            ("a_partial_item_length_at_the_tail_is_malformed: two bytes", &b"\0\0"[..]),
            ("a_partial_item_length_at_the_tail_is_malformed: three bytes", &b"\0\0\0"[..]),
            ("a_partial_item_length_at_the_tail_is_malformed: after item", &[0, 0, 0, 1, b'a', 0][..]),
        ],
        |bytes| {
            assert_eq!(split_items(bytes), Err(Reason::Malformed));
        },
    );
}

#[test]
fn bounded_splitting_charges_each_item_and_its_length() {
    const MOST: u64 = 10; // Two lengths and the two bytes in "ab".
    const SIZE: usize = 10; // The same encoded piece as MOST bounds.
    const FIRST_SUM: u64 = 6; // "ab" and its four-byte length.
    let bytes = write_items(&[b"ab", b""], SIZE);
    let mut reader = Reader::new(&bytes);
    let mut sum = 0;
    assert_eq!(piece::split_within(&mut reader, &mut sum, MOST), Ok(Some(Box::from(&b"ab"[..]))));
    assert_eq!(sum, FIRST_SUM);
    assert_eq!(piece::split_within(&mut reader, &mut sum, MOST), Ok(Some(Box::from(&b""[..]))));
    assert_eq!(sum, MOST);
}

// "ab" and its four-byte length occupy six bytes. A bound of five is
// one short; MAX - 2 leaves room for the body alone.

#[test]
fn bounded_splitting_refuses_sums_outside_its_byte_bound() {
    skein_rows::check(
        [
            ("bounded_splitting_refuses_one_byte_past_the_sum", (0, 5)),
            ("bounded_splitting_refuses_overflow_in_bytes_or_length: header", (u64::MAX, u64::MAX)),
            ("bounded_splitting_refuses_overflow_in_bytes_or_length: body", (u64::MAX - 2, u64::MAX)),
        ],
        |(mut sum, most)| {
            const SIZE: usize = 6; // "ab" and its four-byte length.
            let bytes = write_items(&[b"ab"], SIZE);
            assert_eq!(piece::split_within(&mut Reader::new(&bytes), &mut sum, most), Err(Reason::Bound));
        },
    );
}

#[test]
fn bounded_splitting_checks_framing_before_the_sum() {
    for bytes in [&b"\0"[..], &b"\0\0"[..], &b"\0\0\0"[..], &b"\0\0\0\x02a"[..]] {
        let mut sum = u64::MAX;
        assert_eq!(piece::split_within(&mut Reader::new(bytes), &mut sum, 0), Err(Reason::Malformed));
        assert_eq!(sum, u64::MAX);
    }
}

#[test]
fn bounded_splitting_at_the_end_charges_nothing() {
    let mut sum = u64::MAX;
    assert_eq!(piece::split_within(&mut Reader::new(b""), &mut sum, 0), Ok(None));
    assert_eq!(sum, u64::MAX);
}

#[test]
fn counting_agrees_with_the_items_written() {
    const SIZE: usize = 15; // Three lengths and the three bytes in "ab" and "c".
    for (items, size, count) in [(&[][..], 0, 0), (&[&b"ab"[..], &b""[..], &b"c"[..]][..], SIZE, 3)] {
        assert_eq!(piece::count(&write_items(items, size)), Ok(count));
    }
}

#[test]
fn counting_refuses_partial_lengths_and_items() {
    for bytes in
        [&b"\0"[..], &b"\0\0"[..], &b"\0\0\0"[..], &b"\0\0\0\x02a"[..], &b"\xff\xff\xff\xff"[..], &b"\0\0\0\0\0"[..]]
    {
        assert_eq!(piece::count(bytes), Err(Reason::Malformed));
    }
}

#[test]
fn counted_splitting_returns_every_item_in_order() {
    const SIZE: usize = 15; // Three lengths and the three bytes in "ab" and "c".
    let bytes = write_items(&[b"ab", b"", b"c"], SIZE);
    let count = piece::count(&bytes).expect("written piece framing");
    let mut reader = Reader::new(&bytes);
    let mut items = List::with_capacity(count);
    for _ in 0..count {
        items.push(piece::counted(&mut reader)).expect("counted items fit");
    }
    assert_eq!(items.into_boxed().as_ref(), &[Box::from(&b"ab"[..]), Box::from(&b""[..]), Box::from(&b"c"[..])]);
    assert!(reader.is_empty());
}

#[test]
#[should_panic(expected = "count accepted the piece framing")]
fn counted_splitting_requires_validated_framing() {
    piece::counted(&mut Reader::new(b"\0"));
}

#[test]
#[should_panic(expected = "count includes the next item")]
fn counted_splitting_requires_a_remaining_item() {
    piece::counted(&mut Reader::new(b""));
}

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

#[test]
fn an_item_that_exceeds_the_remaining_bound_writes_nothing() {
    let mut out = Writer::new(6);
    piece::write(&mut out, b"ab").expect("the first item fills the bound");
    assert_eq!(piece::write(&mut out, b""), Err(Overflow));
    assert_eq!(out.finish().as_ref(), &[0, 0, 0, 2, b'a', b'b']);
}

#[test]
fn an_item_header_that_exceeds_the_bound_writes_nothing() {
    let mut out = Writer::new(3);
    assert_eq!(piece::write(&mut out, b""), Err(Overflow));
    assert_eq!(out.written(), 0);
}

#[test]
fn an_item_body_that_exceeds_the_bound_writes_no_header() {
    let mut out = Writer::new(5);
    assert_eq!(piece::write(&mut out, b"ab"), Err(Overflow));
    assert_eq!(out.written(), 0);
}

#[test]
fn small_and_large_items_fit_the_same_sum_bound() {
    assert_eq!(write_items(&[b"a", b"b"], 10).as_ref(), &[0, 0, 0, 1, b'a', 0, 0, 0, 1, b'b']);
    assert_eq!(write_items(&[b"abcdef"], 10).as_ref(), &[0, 0, 0, 6, b'a', b'b', b'c', b'd', b'e', b'f']);
}

#[test]
fn splitting_ends_only_between_complete_items() {
    let items = split_items(&[0, 0, 0, 2, b'a', b'b', 0, 0, 0, 0, 0, 0, 0, 1, b'c']).expect("complete items");
    assert_eq!(items.as_ref(), &[Box::from(&b"ab"[..]), Box::from(&b""[..]), Box::from(&b"c"[..])]);
}

#[test]
fn an_empty_piece_has_no_items() {
    assert_eq!(split_items(b""), Ok(Box::from([])));
}

#[test]
fn an_item_length_past_the_end_is_malformed() {
    assert_eq!(split_items(&[0, 0, 0, 2, b'a']), Err(Reason::Malformed));
    assert_eq!(split_items(&[255, 255, 255, 255]), Err(Reason::Malformed));
}

#[test]
fn a_partial_item_length_at_the_tail_is_malformed() {
    for tail in [&b"\0"[..], &b"\0\0"[..], &b"\0\0\0"[..]] {
        assert_eq!(split_items(tail), Err(Reason::Malformed));
    }
    assert_eq!(split_items(&[0, 0, 0, 1, b'a', 0]), Err(Reason::Malformed));
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

#[test]
fn bounded_splitting_refuses_one_byte_past_the_sum() {
    const MOST: u64 = 5; // One byte short of "ab" and its length.
    const SIZE: usize = 6; // "ab" and its four-byte length.
    let bytes = write_items(&[b"ab"], SIZE);
    let mut sum = 0;
    assert_eq!(piece::split_within(&mut Reader::new(&bytes), &mut sum, MOST), Err(Reason::Bound));
}

#[test]
fn bounded_splitting_refuses_overflow_in_bytes_or_length() {
    const SIZE: usize = 6; // "ab" and its four-byte length.
    const ROOM_FOR_BODY: u64 = u64::MAX - 2; // Only the two bytes in "ab" fit.
    let bytes = write_items(&[b"ab"], SIZE);
    for mut sum in [u64::MAX, ROOM_FOR_BODY] {
        assert_eq!(piece::split_within(&mut Reader::new(&bytes), &mut sum, u64::MAX), Err(Reason::Bound));
    }
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

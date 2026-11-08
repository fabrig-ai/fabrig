use super::fixtures as f;
use crate::Note;
use skein_lib::{List, Wall, bytes::concat};

type Row<'a> = (&'a str, &'a [u8], &'a [u8], Option<Note>);

#[test]
fn the_sign_in_notes_are_one_bounded_line_for_the_address() {
    let until = Note::Until(Wall::from_nanos(123));
    let mut failures = List::with_capacity(40);
    for (note, line) in [
        (until, b"127.0.0.1:8367 until 123\n".as_slice()),
        (Note::Until(Wall::EPOCH), b"127.0.0.1:8367 until 0\n"),
        (Note::Until(Wall::from_nanos(u64::MAX)), b"127.0.0.1:8367 until 18446744073709551615\n"),
        (Note::SignedOut, b"127.0.0.1:8367 signed-out\n"),
    ] {
        assert_eq!(note.print(f::ADDRESS).as_ref(), line);
        for bytes in [line, &line[..line.len() - 1]] {
            let found = Note::read(bytes, f::ADDRESS);
            if found != Some(note) {
                failures.push(("roundtrip", found, Some(note))).unwrap();
            }
        }
    }
    let bound_address = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:8367";
    let at_bound = concat(&[bound_address, b" until 18446744073709551615\n"]);
    let past_bound = concat(&[b"a", &at_bound]);
    let past_address = concat(&[b"a", bound_address]);
    assert_eq!(Note::MOST, 64);
    assert_eq!(at_bound.len(), 64);
    assert_eq!(past_bound.len(), 65);
    let rows: &[Row<'_>] = &[
        ("64 bytes", &at_bound, bound_address, Some(Note::Until(Wall::from_nanos(u64::MAX)))),
        ("65 bytes", &past_bound, &past_address, None),
        ("64 without line feed", &past_bound[..64], &past_address, Some(Note::Until(Wall::from_nanos(u64::MAX)))),
        ("another address", b"localhost:8367 until 123", f::ADDRESS, None),
        ("another signed-out address", b"localhost:8367 signed-out", f::ADDRESS, None),
        ("unknown word", b"127.0.0.1:8367 unknown", f::ADDRESS, None),
        ("plus sign", b"127.0.0.1:8367 until +1", f::ADDRESS, None),
        ("minus sign", b"127.0.0.1:8367 until -1", f::ADDRESS, None),
        ("leading zero", b"127.0.0.1:8367 until 01", f::ADDRESS, None),
        ("two zeros", b"127.0.0.1:8367 until 00", f::ADDRESS, None),
        ("past u64", b"127.0.0.1:8367 until 18446744073709551616", f::ADDRESS, None),
        ("not digits", b"127.0.0.1:8367 until 1a", f::ADDRESS, None),
        ("second line", b"127.0.0.1:8367 until 1\n127.0.0.1:8367 signed-out", f::ADDRESS, None),
        ("two line feeds", b"127.0.0.1:8367 signed-out\n\n", f::ADDRESS, None),
        ("carriage return", b"127.0.0.1:8367 signed-out\r\n", f::ADDRESS, None),
        ("no line", b"", f::ADDRESS, None),
        ("no word", b"127.0.0.1:8367", f::ADDRESS, None),
        ("no number", b"127.0.0.1:8367 until", f::ADDRESS, None),
        ("empty number", b"127.0.0.1:8367 until ", f::ADDRESS, None),
        ("no address", b"until 1", f::ADDRESS, None),
        ("extra word", b"127.0.0.1:8367 signed-out 1", f::ADDRESS, None),
        ("extra number word", b"127.0.0.1:8367 until 1 more", f::ADDRESS, None),
        ("leading space", b" 127.0.0.1:8367 signed-out", f::ADDRESS, None),
        ("address space", b"127.0.0.1:8367  signed-out", f::ADDRESS, None),
        ("number space", b"127.0.0.1:8367 until  1", f::ADDRESS, None),
        ("trailing space", b"127.0.0.1:8367 signed-out ", f::ADDRESS, None),
        ("number trailing space", b"127.0.0.1:8367 until 1 ", f::ADDRESS, None),
    ];
    for (name, line, address, expected) in rows {
        let found = Note::read(line, address);
        if found != *expected {
            failures.push((*name, found, *expected)).unwrap();
        }
    }
    assert!(failures.is_empty(), "note rows: {failures:?}");
}

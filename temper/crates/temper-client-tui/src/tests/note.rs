use super::fixtures as f;
use crate::Note;
use skein_lib::{List, Wall};

type Row<'a> = (&'a str, &'a [u8], &'a crate::Origin, Option<Note>);

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
    let rows: &[Row<'_>] = &[
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

#[test]
fn notes_name_the_origin_authority_and_fit_the_longest_name() {
    let longest = crate::Origin::Https { name: super::origin::long_name(), port: 65535 };
    let named = super::origin::named(443);
    assert_eq!(Note::MOST, 300);
    assert_eq!(Note::Until(Wall::from_nanos(123)).print(&named).as_ref(), b"dev.fabrig.ai:443 until 123\n");
    for origin in [&named, &longest] {
        for note in [Note::Until(Wall::from_nanos(u64::MAX)), Note::SignedOut] {
            let line = note.print(origin);
            assert!(line.len() <= 300);
            assert_eq!(Note::read(&line, origin), Some(note));
            assert_eq!(Note::read(&line, f::ADDRESS), None);
        }
    }
    assert_eq!(Note::read(&[b'x'; 301], &longest), None);
}

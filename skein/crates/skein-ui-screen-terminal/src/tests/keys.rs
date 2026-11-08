#![expect(clippy::disallowed_methods, reason = "test expectations read literal UTF-8 characters")]
use crate::{Decoder, Key};
use skein_lib::{List, Queue, bytes};
use skein_ui_screen::Rgb;

// Each key row emits one key, so two reads need no more than its
// byte count plus the immediately emitted Escape at the special first split.
const KEY_ROOM: u32 = 64;
fn check(keys: &mut Queue<Key>, expected: &[Key], label: &[u8], split: usize) {
    for key in expected {
        assert_eq!(keys.pop(), Some(key.clone()), "{label:?}, split {split}");
    }
    assert_eq!(keys.pop(), None, "{label:?}, split {split}: trailing key");
}
#[test]
#[expect(
    clippy::too_many_lines,
    clippy::type_complexity,
    reason = "the decoder's key and answer rows share the whole-and-split table"
)]
fn decoding_table_holds_whole_and_at_every_split() {
    let rows: &[(&[u8], Key)] = &[
        (b"a", Key::Char('a')),
        (b"7", Key::Char('7')),
        (b" ", Key::Char(' ')),
        ("é".as_bytes(), Key::Char('é')),
        ("🙂".as_bytes(), Key::Char('🙂')),
        (b"\r", Key::Enter),
        (b"\n", Key::Enter),
        (b"\t", Key::Tab),
        (b"\x1b[Z", Key::BackTab),
        (b"\x7f", Key::Backspace),
        (b"\x08", Key::Backspace),
        (b"\x1b[3~", Key::Delete),
        (b"\x1b[A", Key::Up),
        (b"\x1bOA", Key::Up),
        (b"\x1b[B", Key::Down),
        (b"\x1bOB", Key::Down),
        (b"\x1b[C", Key::Right),
        (b"\x1bOC", Key::Right),
        (b"\x1b[D", Key::Left),
        (b"\x1bOD", Key::Left),
        (b"\x1b[H", Key::Home),
        (b"\x1b[1~", Key::Home),
        (b"\x1bOH", Key::Home),
        (b"\x1b[F", Key::End),
        (b"\x1b[4~", Key::End),
        (b"\x1bOF", Key::End),
        (b"\x1b[5~", Key::PageUp),
        (b"\x1b[6~", Key::PageDown),
        (b"\x01", Key::Control(b'a')),
        (b"\x03", Key::Control(b'c')),
        (b"\x1a", Key::Control(b'z')),
        (b"\x1bx", Key::Alt('x')),
        (b"\x1b7", Key::Alt('7')),
        (b"\x1b]", Key::Alt(']')),
        ("\x1bé".as_bytes(), Key::Alt('é')),
    ];
    for (bytes, expected) in rows {
        for split in 0..=bytes.len() {
            let mut decoder = Decoder::new(0);
            let mut keys = Queue::with_capacity(KEY_ROOM);
            decoder.read(bytes.get(..split).expect("split inside literal"), &mut keys);
            decoder.read(bytes.get(split..).expect("split inside literal"), &mut keys);
            if split == 1 && bytes.first() == Some(&0x1b) {
                assert_eq!(keys.pop(), Some(Key::Escape), "{bytes:?}");
                let tail = core::str::from_utf8(bytes.get(1..).expect("escape prefix")).expect("printable tail");
                for ch in tail.chars() {
                    assert_eq!(keys.pop(), Some(Key::Char(ch)), "{bytes:?}");
                }
                assert_eq!(keys.pop(), None, "{bytes:?}");
            } else {
                check(&mut keys, core::slice::from_ref(expected), bytes, split);
            }
        }
    }
    for control in 1_u8..=26 {
        let expected = match control {
            8 => Key::Backspace,
            9 => Key::Tab,
            10 | 13 => Key::Enter,
            1..=7 | 11..=12 | 14..=26 => Key::Control(b'a'.saturating_add(control.saturating_sub(1))),
            _ => unreachable!("range is the control letters"),
        };
        let mut decoder = Decoder::new(0);
        let mut keys = Queue::with_capacity(KEY_ROOM);
        decoder.read(&[control], &mut keys);
        check(&mut keys, &[expected], &[control], 0);
    }

    let red = Some(Some(Rgb { red: 255, green: 170, blue: 0 }));
    let rows: &[(&[u8], Option<Option<Rgb>>, &[Key])] = &[
        (b"\x1b]11;rgb:f/a/0\x07", red, &[]),
        (b"\x1b]11;rgb:F/A/0\x1b\\", red, &[]),
        (b"\x1b]11;rgb:ff/aa/00\x07", red, &[]),
        (b"\x1b]11;rgb:FF/AA/00\x1b\\", red, &[]),
        (b"\x1b]11;rgb:fff/aaa/000\x07", red, &[]),
        (b"\x1b]11;rgb:FFF/AAA/000\x1b\\", red, &[]),
        (b"\x1b]11;rgb:ffff/aaaa/0000\x07", red, &[]),
        (b"\x1b]11;rgb:FFFF/AAAA/0000\x1b\\", red, &[]),
        (b"\x1b]11;rgb:12f/34ab/5\x1b\\", Some(Some(Rgb { red: 0x12, green: 0x34, blue: 0x55 })), &[]),
        (b"\x1b]11;rgb:fffff/aa/00\x07", Some(None), &[]),
        (b"\x1b]11;rgb:ff/aaaaa/00\x07", Some(None), &[]),
        (b"\x1b]11;rgb:ff/aa/00000\x07", Some(None), &[]),
        (b"\x1b]11;rgb:/aa/00\x07", Some(None), &[]),
        (b"\x1b]11;rgb:ff//00\x07", Some(None), &[]),
        (b"\x1b]11;rgb:ff/aa/\x07", Some(None), &[]),
        (b"\x1b]11;rgba:ff/aa/00/ff\x1b\\", Some(None), &[]),
        (b"\x1b]11;rgb:gg/aa/00\x07", Some(None), &[]),
        (b"\x1b]11;rgb:ff/aa/00/11\x07", Some(None), &[]),
        (b"\x1b]11\x07", Some(None), &[]),
        (b"\x1b]0;title\x07", None, &[]),
        (b"\x1b]110;rgb:ff/aa/00\x1b\\", None, &[]),
        (b"\x1b]0;title\x1b\\\x1b]11;rgb:f/a/0\x07", red, &[]),
        (b"\x1b]11;rgb:f/a/0\x07\x1b]11;unreadable\x07", red, &[]),
        (b"\x1b]11;unreadable\x07\x1b]11;rgb:f/a/0\x07", Some(None), &[]),
        (b"x\x1b]11;rgb:f/a/0\x07q", red, &[Key::Char('x'), Key::Char('q')]),
    ];
    let mut failures = List::uncapped();
    for (source, expected, expected_keys) in rows {
        for split in 0..=source.len() {
            let mut decoder = Decoder::new(0);
            decoder.answers();
            let mut keys = Queue::with_capacity(KEY_ROOM);
            decoder.read(source.get(..split).expect("split inside answer"), &mut keys);
            decoder.read(source.get(split..).expect("split inside answer"), &mut keys);
            if decoder.background() != *expected || !keys.iter().eq(expected_keys.iter()) {
                failures.push((source, split, decoder.background(), keys.pop())).expect("test diagnostics fit");
                break;
            }
        }
    }
    assert!(failures.is_empty(), "answer rows (source, split, background, first key): {failures:?}");
    // Count the complete answer, including its introducer and terminator.
    for terminator in [b"\x07".as_slice(), b"\x1b\\"] {
        for length in [64_usize, 65] {
            let padding =
                length.checked_sub(7).expect("prefix fits").checked_sub(terminator.len()).expect("ending fits");
            let source =
                bytes::concat(&[b"\x1b]11;\x1bX", [b'x'; 60].get(..padding).expect("small padding"), terminator, b"q"]);
            for split in 0..=source.len() {
                let mut decoder = Decoder::new(0);
                decoder.answers();
                let mut keys = Queue::with_capacity(KEY_ROOM);
                decoder.read(source.get(..split).expect("split inside bounded answer"), &mut keys);
                decoder.read(source.get(split..).expect("split inside bounded answer"), &mut keys);
                assert_eq!(
                    decoder.background(),
                    if length == 64 { Some(None) } else { None },
                    "{length}, {split}, {terminator:?}"
                );
                check(&mut keys, &[Key::Char('q')], &source, split);
                decoder.read(b"\x1b]11;rgb:f/a/0\x07", &mut keys);
                assert_eq!(decoder.background(), if length == 64 { Some(None) } else { red });
                check(&mut keys, &[], &source, split);
            }
        }
    }
}
#[test]
fn a_read_ending_in_escape_delivers_it_immediately() {
    for (first, next, expected) in [
        (b"\x1b".as_slice(), b"".as_slice(), &[Key::Escape][..]),
        (b"\x1b", b"[A", &[Key::Escape, Key::Char('['), Key::Char('A')]),
        (b"x\x1b", b"z", &[Key::Char('x'), Key::Escape, Key::Char('z')]),
    ] {
        let mut decoder = Decoder::new(0);
        let mut keys = Queue::with_capacity(KEY_ROOM);
        decoder.read(first, &mut keys);
        // The first read itself must deliver Escape, before a further read.
        let mut prior = Queue::with_capacity(KEY_ROOM);
        for _ in 0..keys.len() {
            prior.push(keys.pop().expect("queued key"));
        }
        assert_eq!(prior.iter().last(), Some(&Key::Escape));
        decoder.read(next, &mut prior);
        check(&mut prior, expected, first, first.len());
    }
}
#[test]
fn unknown_and_overlong_sequences_are_dropped_whole() {
    for sequence in [b"\x1b[99~".as_slice(), b"\x1b[12345678901234~", b"\x1bOQ"] {
        for split in 2..=sequence.len() {
            let mut decoder = Decoder::new(0);
            let mut keys = Queue::with_capacity(KEY_ROOM);
            decoder.read(sequence.get(..split).expect("split in sequence"), &mut keys);
            decoder.read(sequence.get(split..).expect("split in sequence"), &mut keys);
            decoder.read(b"q", &mut keys);
            check(&mut keys, &[Key::Char('q')], sequence, split);
        }
    }
}
#[test]
fn pastes_are_one_key_capped_at_a_character_boundary() {
    let source = "\x1b[200~aé🙂z\x1b[201~q".as_bytes();
    for (most, expected) in [(0, ""), (1, "a"), (2, "a"), (3, "aé"), (6, "aé"), (7, "aé🙂"), (8, "aé🙂z")] {
        for split in 2..=source.len() {
            let mut decoder = Decoder::new(most);
            let mut keys = Queue::with_capacity(KEY_ROOM);
            decoder.read(source.get(..split).expect("split in paste"), &mut keys);
            decoder.read(source.get(split..).expect("split in paste"), &mut keys);
            check(&mut keys, &[Key::Paste(bytes::copy_of(expected.as_bytes())), Key::Char('q')], source, split);
        }
    }
    let mut decoder = Decoder::new(KEY_ROOM);
    let mut keys = Queue::with_capacity(KEY_ROOM);
    decoder.read(b"\x1b[200~a\x1b[20x\x1b\x1b[201~", &mut keys);
    check(&mut keys, &[Key::Paste(bytes::copy_of(b"a\x1b[20x\x1b"))], b"false ending", 0);
}

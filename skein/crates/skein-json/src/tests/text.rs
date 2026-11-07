//! The checks of text: UTF-8 a byte at a time against the standard
//! library's, and a number's grammar against a plain reading of RFC 8259,
//! section 6.

use skein_lib::Intake;

use crate::number::is_number;
use crate::string::{self, Buffer, Piece, Retention, Text};
use crate::tokenizer::Error;
use crate::utf8::Utf8;

/// Whether `bytes` is UTF-8, whole, by the check under test.
fn checked(bytes: &[u8]) -> bool {
    let mut state = Utf8::Between;
    for &byte in bytes {
        match state.next(byte) {
            Some(next) => state = next,
            None => return false,
        }
    }
    state == Utf8::Between
}

#[expect(clippy::disallowed_methods, reason = "the check is compared with the standard library's")]
fn standard(bytes: &[u8]) -> bool {
    core::str::from_utf8(bytes).is_ok()
}

#[test]
fn every_one_and_two_byte_sequence_is_judged_as_the_standard_library_judges_it() {
    for first in 0..=u8::MAX {
        assert_eq!(checked(&[first]), standard(&[first]), "{first:#04x}");
        for second in 0..=u8::MAX {
            let bytes = [first, second];
            assert_eq!(checked(&bytes), standard(&bytes), "{}", bytes.escape_ascii());
        }
    }
}

#[test]
fn three_and_four_byte_sequences_are_judged_at_every_boundary() {
    // Each byte after the first at the edges of the continuation ranges,
    // and just past them: E0's A0, ED's 9F, F0's 90, F4's 8F,
    // and the ordinary 80..=BF, each with its neighbour outside.
    let edges = [0x00, 0x7F, 0x80, 0x8F, 0x90, 0x9F, 0xA0, 0xBF, 0xC0, 0xFF];
    for lead in 0xC0..=u8::MAX {
        for second in edges {
            for third in edges {
                let bytes = [lead, second, third];
                assert_eq!(checked(&bytes), standard(&bytes), "{}", bytes.escape_ascii());
                for fourth in edges {
                    let bytes = [lead, second, third, fourth];
                    assert_eq!(checked(&bytes), standard(&bytes), "{}", bytes.escape_ascii());
                }
            }
        }
    }
}

/// A plain reading of the grammar: an optional minus, an integer without a
/// leading zero, an optional fraction, an optional exponent.
fn naive(text: &[u8]) -> bool {
    fn digits(text: &[u8]) -> usize {
        let mut count: usize = 0;
        for byte in text {
            if !byte.is_ascii_digit() {
                break;
            }
            count = count.checked_add(1).unwrap();
        }
        count
    }
    let rest = text.strip_prefix(b"-").unwrap_or(text);
    let integer = digits(rest);
    if integer == 0 || (integer > 1 && rest.first() == Some(&b'0')) {
        return false;
    }
    let mut rest = &rest[integer..];
    if let Some(fraction) = rest.strip_prefix(b".") {
        let count = digits(fraction);
        if count == 0 {
            return false;
        }
        rest = &fraction[count..];
    }
    if let Some(exponent) = rest.strip_prefix(b"e").or(rest.strip_prefix(b"E")) {
        let exponent = exponent.strip_prefix(b"+").or(exponent.strip_prefix(b"-")).unwrap_or(exponent);
        let count = digits(exponent);
        if count == 0 {
            return false;
        }
        rest = &exponent[count..];
    }
    rest.is_empty()
}

#[test]
fn every_short_text_of_a_numbers_bytes_is_judged_as_the_grammar_reads() {
    let alphabet = b"01.eE+-x";
    let mut text = [0_u8; 6];
    for len in 0..=text.len() {
        let mut indices = [0_usize; 6];
        'texts: loop {
            for (at, &index) in indices[..len].iter().enumerate() {
                text[at] = alphabet[index];
            }
            let candidate = &text[..len];
            assert_eq!(is_number(candidate), naive(candidate), "{}", candidate.escape_ascii());
            // The next text of this length, as a counter in the alphabet.
            for index in &mut indices[..len] {
                *index = index.checked_add(1).unwrap();
                if *index < alphabet.len() {
                    continue 'texts;
                }
                *index = 0;
            }
            break;
        }
    }
}

/// What a string's text leaves, stopping at the first error: each outcome,
/// then the text kept, the length, and the first byte of the last character.
#[derive(Debug, PartialEq, Eq)]
struct Left {
    outcomes: Vec<Result<Piece, Error>>,
    kept: Box<[u8]>,
    length: u64,
    begun: u32,
}

fn by_runs(pieces: &[&[u8]], first: u32, retention: Retention) -> Left {
    leaving(pieces, first, retention, string::decode)
}

fn per_byte(pieces: &[&[u8]], first: u32, retention: Retention) -> Left {
    leaving(pieces, first, retention, byte_piece)
}

fn leaving(
    pieces: &[&[u8]],
    first: u32,
    retention: Retention,
    decode: fn(Text, &[u8], u32, &mut Buffer) -> Result<Piece, Error>,
) -> Left {
    let mut buffer = Buffer { bytes: Intake::with_capacity(1024), length: 0, retention, begun: 71 };
    let mut text = Text::Plain;
    let mut at = first;
    let mut outcomes = Vec::new();
    'pieces: for piece in pieces {
        // The tokenizer's scans stop at every quote, including escaped ones.
        // Preserve empty pieces too: they must leave begun untouched.
        let scans = piece.split_inclusive(|byte| *byte == b'"');
        for scan in scans.chain(piece.is_empty().then_some(*piece)) {
            let outcome = decode(text, scan, at, &mut buffer);
            outcomes.push(outcome);
            match outcome {
                Ok(Piece::More(after)) => text = after,
                Ok(Piece::Closed) | Err(_) => break 'pieces,
            }
            at = at.checked_add(u32::try_from(scan.len()).unwrap()).unwrap();
        }
    }
    Left { outcomes, kept: buffer.take(), length: buffer.length, begun: buffer.begun }
}

/// The byte-at-a-time reading through next, room and put (json.md, 3.4).
fn byte_piece(mut text: Text, piece: &[u8], first: u32, out: &mut Buffer) -> Result<Piece, Error> {
    for (at, &byte) in piece.iter().enumerate() {
        if text == Text::Plain {
            out.begun = first.checked_add(u32::try_from(at).unwrap()).unwrap();
        }
        let decoded = string::next(text, byte)?;
        // A raw character reserves its whole width at its first byte;
        // continuations reserve nothing, and an escape reserves its result.
        let room = match text {
            Text::Plain => match decoded.after {
                Some(Text::Char { more, .. }) => u32::from(more).checked_add(1).unwrap(),
                Some(
                    Text::Plain
                    | Text::Escape
                    | Text::Unicode { .. }
                    | Text::High { .. }
                    | Text::HighEscape { .. }
                    | Text::Low { .. },
                )
                | None => u32::try_from(decoded.bytes().len()).unwrap(),
            },
            Text::Char { .. } => 0,
            Text::Escape | Text::Unicode { .. } | Text::High { .. } | Text::HighEscape { .. } | Text::Low { .. } => {
                u32::try_from(decoded.bytes().len()).unwrap()
            }
        };
        out.room(room)?;
        out.put(decoded.bytes());
        match decoded.after {
            Some(after) => text = after,
            None => {
                assert_eq!(at.checked_add(1), Some(piece.len()));
                return Ok(Piece::Closed);
            }
        }
    }
    Ok(Piece::More(text))
}

fn same(pieces: &[&[u8]], first: u32, retention: Retention) {
    assert_eq!(
        by_runs(pieces, first, retention),
        per_byte(pieces, first, retention),
        "pieces {pieces:?}, first {first}, retention {retention:?}"
    );
}

#[test]
fn a_run_reads_as_its_bytes_one_at_a_time_however_it_is_cut() {
    let texts: &[&[u8]] = &[
        b"abc\"",
        b"\"",
        b"",
        b"abc",
        b"abc\\nxyz\"",
        "abcéxyz\"".as_bytes(),
        b"abc\\ud83d\\ude00xyz\"",
        "éabc\"".as_bytes(),
        b"abc\nxyz\"",
        b"abc\x80xyz\"",
        b"abc\xc0\x80xyz\"",
        b"abc\\xxyz\"",
    ];
    for &text in texts {
        for retention in [
            Retention::Strict(1024),
            Retention::Strict(0),
            Retention::Strict(1),
            Retention::Strict(2),
            Retention::Strict(3),
            Retention::Capped(0),
            Retention::Capped(1),
            Retention::Capped(2),
            Retention::Capped(3),
            Retention::Discard,
        ] {
            for first in [0, 29] {
                same(&[text], first, retention);
                for cut in 0..=text.len() {
                    same(&[&text[..cut], &text[cut..]], first, retention);
                }
                let mut bytes = Vec::new();
                for byte in text.chunks(1) {
                    bytes.push(byte);
                }
                same(&bytes, first, retention);
            }
        }
    }
}

#[test]
fn every_byte_at_a_runs_edge_reads_as_it_does_alone() {
    for byte in 0..=u8::MAX {
        for prefix in [b"aa".as_slice(), b""] {
            let mut text = Vec::from(prefix);
            let edge = text.len();
            text.push(byte);
            text.extend_from_slice(b"aa\"");
            for retention in [
                Retention::Strict(0),
                Retention::Strict(2),
                Retention::Strict(1024),
                Retention::Capped(0),
                Retention::Capped(2),
                Retention::Capped(1024),
                Retention::Discard,
            ] {
                same(&[&text], 13, retention);
                for cut in [edge, edge.checked_add(1).unwrap()] {
                    same(&[&text[..cut], &text[cut..]], 13, retention);
                }
            }
        }
    }
}

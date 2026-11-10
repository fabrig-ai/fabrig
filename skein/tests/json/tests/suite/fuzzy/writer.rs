//! The writer against the tokenizer, swept (testing-strategy.md, 8; data/json.md,
//! 6): many generated documents written and read back the same through the
//! machine world, and text and numbers drawn at random refused by the writer
//! exactly when the standard library and the grammar refuse them.

use skein_json::Token;
use skein_json::tokenizer::Limits;
use skein_json::writer::{self, Refusal};
use skein_json_world::generate::{self, Shape};
use skein_json_world::world::{self, Settings};
use skein_json_world::{Decoded, Outcome, reference, write};
use skein_lib::Rng;

const WRITER: writer::Limits = writer::Limits { depth: 64, length: 1 << 20 };
const READER: Limits = Limits { depth: 64, string: 4096, number: 64, chunk: 32, length: 1 << 20 };

#[test]
fn generated_documents_are_written_and_read_back_the_same_whatever_the_neighbours() {
    for round in 0..5_000 {
        let seed = 0x0057_0000 + round;
        let mut rng = Rng::new(seed);
        let shape = Shape {
            depth: u32::try_from(rng.between(1, 6)).unwrap(),
            width: u32::try_from(rng.between(1, 8)).unwrap(),
            string: u32::try_from(rng.between(0, 24)).unwrap(),
        };
        let tokens = generate::tokens(&mut rng, shape);
        let document = write(&tokens, &WRITER).expect("generated text is UTF-8, its numbers numbers");
        let expected = Decoded { tokens, outcome: Outcome::Done };
        assert_eq!(reference::parse(&document, &READER), expected, "seed {seed}: the reference reads it");
        let mut limits = READER;
        limits.chunk = u32::try_from(rng.between(1, 64)).unwrap();
        let settings = Settings::calm(&mut rng, limits);
        let run = world::check(&document, &settings, seed);
        assert_eq!(run.decoded(), Some(expected), "seed {seed}");
    }
}

/// Bytes drawn from a few that make text valid, invalid, or cut short.
const BYTES: &[u8] = b"a\"\\\n\x00\x1f\x7f\x80\xbf\xc2\xc3\xe0\xe2\xed\xf0\xf4\xf5\xff";

#[test]
fn text_is_written_exactly_when_it_is_utf8_and_reads_back_the_same() {
    let mut written = 0;
    for round in 0..20_000 {
        let mut rng = Rng::new(0x0058_0000 + round);
        let mut text = Vec::new();
        for _ in 0..rng.below(8) {
            text.push(BYTES[usize::try_from(rng.below(BYTES.len() as u64)).unwrap()]);
        }
        let tokens = [Token::ArrayStart, Token::String(text.clone().into_boxed_slice()), Token::ArrayEnd];
        match write(&tokens, &WRITER) {
            Ok(document) => {
                assert!(std::str::from_utf8(&text).is_ok(), "{} is UTF-8", text.escape_ascii());
                let decoded = reference::parse(&document, &READER);
                assert_eq!(decoded, Decoded { tokens: tokens.to_vec(), outcome: Outcome::Done });
                written += 1;
            }
            Err(refusal) => {
                assert_eq!(refusal, Refusal::Text);
                assert!(std::str::from_utf8(&text).is_err(), "{} is not UTF-8", text.escape_ascii());
            }
        }
    }
    assert!(written > 2_000, "much of the text is UTF-8: {written}");
}

#[test]
fn a_number_is_written_exactly_when_the_grammar_reads_one() {
    let mut written = 0;
    for round in 0..20_000 {
        let mut rng = Rng::new(0x0059_0000 + round);
        let mut text = generate::number(&mut rng).into_vec();
        if rng.chance(500) {
            text = generate::mutate(&mut rng, &text);
        }
        let tokens = [Token::Number(text.clone().into_boxed_slice())];
        match write(&tokens, &WRITER) {
            Ok(document) => {
                assert!(reference::is_number(&text), "{} is a number", text.escape_ascii());
                assert_eq!(&*document, &text[..], "a number is written as it is");
                written += 1;
            }
            Err(refusal) => {
                assert_eq!(refusal, Refusal::Number);
                assert!(!reference::is_number(&text), "{} is not a number", text.escape_ascii());
            }
        }
    }
    assert!(written > 10_000, "most numbers are numbers: {written}");
}

fn naive_spelling(text: &[u8]) -> Vec<u8> {
    let mut spelling = Vec::from(b"\"".as_slice());
    for &byte in text {
        match byte {
            b'"' => spelling.extend_from_slice(br#"\""#),
            b'\\' => spelling.extend_from_slice(br"\\"),
            0x08 => spelling.extend_from_slice(br"\b"),
            0x0C => spelling.extend_from_slice(br"\f"),
            b'\n' => spelling.extend_from_slice(br"\n"),
            b'\r' => spelling.extend_from_slice(br"\r"),
            b'\t' => spelling.extend_from_slice(br"\t"),
            0x00..=0x1F => spelling.extend_from_slice(format!("\\u{byte:04x}").as_bytes()),
            _ => spelling.push(byte),
        }
    }
    spelling.push(b'"');
    spelling
}

#[test]
fn text_in_runs_is_written_as_a_naive_reading_has_it() {
    const VALID: &[&[u8]] =
        &[b"\"", b"\\", b"\n", b"\x08", b"\x00", b"\x1f", "é".as_bytes(), "€".as_bytes(), "😀".as_bytes()];
    const MIXED: &[&[u8]] = &[
        b"\"",
        b"\\",
        b"\n",
        b"\x08",
        b"\x00",
        b"\x1f",
        "é".as_bytes(),
        "€".as_bytes(),
        "😀".as_bytes(),
        b"\x80",
        b"\xc3",
        b"\xc0\x80",
        b"\xed\xa0\x80",
        b"\xf5",
    ];
    // data/json.md, 6: a small sweep with over 1,000 accepted and refused texts.
    const ROUNDS: u64 = 5_000;
    let mut written = 0;
    let mut refused = 0;
    for round in 0..ROUNDS {
        let seed = 0x005A_0000 + round;
        let mut rng = Rng::new(seed);
        let pieces = u32::try_from(rng.below(6)).unwrap();
        let longest = u32::try_from(rng.below(48)).unwrap();
        let text = generate::runs(&mut rng, if round % 2 == 0 { VALID } else { MIXED }, pieces, longest);
        let tokens = [Token::String(text.clone().into_boxed_slice())];
        match write(&tokens, &WRITER) {
            Ok(document) => {
                assert!(std::str::from_utf8(&text).is_ok(), "seed {seed}: {text:?}");
                assert_eq!(document.as_ref(), naive_spelling(&text), "seed {seed}: {text:?}");
                written += 1;
            }
            Err(refusal) => {
                assert_eq!(refusal, Refusal::Text, "seed {seed}");
                assert!(std::str::from_utf8(&text).is_err(), "seed {seed}: {text:?}");
                refused += 1;
            }
        }
    }
    assert!(written > 1_000, "valid text fell: {written}");
    assert!(refused > 1_000, "invalid text fell: {refused}");
}

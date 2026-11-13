//! A seeded byte-word reader independent of the quoter (redeploy.md, 2.2, 8).
use temper_redeploy_tool::hosts::quote_bytes;
#[path = "referee_fuzzy.rs"]
mod referee;

fn read_word(text: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        match text[cursor] {
            b'\'' => {
                cursor += 1;
                while cursor < text.len() && text[cursor] != b'\'' {
                    bytes.push(text[cursor]);
                    cursor += 1;
                }
                assert!(cursor < text.len(), "unclosed single quote");
                cursor += 1;
            }
            b'\\' => {
                cursor += 1;
                bytes.push(*text.get(cursor).expect("escaped byte"));
                cursor += 1;
            }
            _ => panic!("not a single POSIX quoted word at {cursor}: {text:?}"),
        }
    }
    bytes
}
#[test]
fn every_non_nul_byte_word_quotes_and_reads_back_seeded() {
    let mut seen = [false; 256];
    for seed in 1_u64..=64 {
        let mut random = seed;
        for length in 0..128 {
            let mut word = Vec::new();
            for _ in 0..length {
                random ^= random << 13;
                random ^= random >> 7;
                random ^= random << 17;
                let byte = u8::try_from(random % 255 + 1).unwrap();
                seen[usize::from(byte)] = true;
                word.push(byte);
            }
            assert_eq!(read_word(&quote_bytes(&word)), word, "seed {seed}, length {length}");
        }
    }
    assert!(seen[1..].iter().all(|seen| *seen), "all non-NUL bytes covered");
}

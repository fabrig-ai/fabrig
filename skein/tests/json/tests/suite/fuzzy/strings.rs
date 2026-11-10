//! Plain runs through the tokenizer (data/json.md, 6): decode is crate-private,
//! and the fuzzy profile selects only the worlds' binaries. The step rows
//! compare decode directly with next; these sweeps compare its tokens,
//! outcomes and positions with the reference's naive byte-at-a-time reader.

use skein_json::tokenizer::{Error, Event, Limits};
use skein_json_world::world::{self, Settings};
use skein_json_world::{Outcome, generate, reference};
use skein_lib::Rng;
use skein_protocol_kit::seen::Seen;

const VALID: &[&[u8]] =
    &[br"\n", br"\\", br#"\""#, "é".as_bytes(), "😀".as_bytes(), "é".as_bytes(), "€".as_bytes(), "😀".as_bytes()];
const MIXED: &[&[u8]] = &[
    br"\n",
    br"\\",
    br#"\""#,
    "é".as_bytes(),
    "😀".as_bytes(),
    "é".as_bytes(),
    "€".as_bytes(),
    "😀".as_bytes(),
    b"\n",
    b"\x00",
    b"\x1f",
    b"\x80",
    b"\xc3",
    b"\xc0\x80",
    b"\xed\xa0\x80",
    b"\xf5",
    br"\x",
    br"\ud83d",
    br"\udc00",
    br"\u12",
];

fn enclosed(body: &[u8], key: bool) -> Vec<u8> {
    let mut document = Vec::from(if key { b"{\"" } else { b"[\"" });
    document.extend_from_slice(body);
    document.extend_from_slice(if key { b"\":1}" } else { b"\"]" });
    document
}

fn plain(byte: u8) -> bool {
    match byte {
        0x20..=0x21 | 0x23..=0x5b | 0x5d..=0x7f => true,
        0x00..=0x1f | b'"' | b'\\' | 0x80..=0xff => false,
    }
}

#[test]
fn strings_in_runs_read_as_the_reference_reads_them_wherever_they_are_cut() {
    let mut tally = Seen::default();
    for round in 0..3_000 {
        let seed = 0x005b_0000 + round;
        let mut rng = Rng::new(seed);
        let pieces = u32::try_from(rng.below(6)).unwrap();
        let longest = u32::try_from(rng.below(48)).unwrap();
        let body = generate::runs(&mut rng, if round % 2 == 0 { VALID } else { MIXED }, pieces, longest);
        let document = enclosed(&body, rng.chance(500));
        let string = if rng.chance(500) { 4096 } else { u32::try_from(rng.between(0, body.len() as u64)).unwrap() };
        let limits =
            Limits { depth: 4, number: 16, length: 1 << 12, string, chunk: u32::try_from(rng.between(1, 32)).unwrap() };
        let settings = Settings::calm(&mut rng, limits);
        let run = world::check(&document, &settings, seed);
        let outcome = run.outcome.expect("calm neighbours finish");
        tally.note(format!("{outcome:?}"));
        if outcome == Outcome::Failed(Error::StringTooLong) {
            let reading = reference::reading(&document, &limits);
            let at = usize::try_from(reading.failed_at.unwrap().offset).unwrap();
            if at > 0 && plain(document[at]) && plain(document[at - 1]) {
                tally.note("a limit passed inside a run".into());
            }
        }
    }
    tally.assert_fell(&[
        "Done",
        "Failed(StringTooLong)",
        "Failed(Utf8)",
        "Failed(Control)",
        "Failed(Escape)",
        "Failed(Surrogate)",
        "a limit passed inside a run",
    ]);
}

#[test]
fn text_caps_and_skips_count_runs_as_the_reference_does() {
    let mut tally = Seen::default();
    for round in 0..1_000 {
        let seed = 0x005c_0000 + round;
        let mut rng = Rng::new(seed);
        let pieces = u32::try_from(rng.below(6)).unwrap();
        // The first run passes every Text cap drawn by check_demands (below 32).
        let mut body = vec![b'a'; 40];
        body.extend(generate::runs(&mut rng, VALID, pieces, 48));
        let document = enclosed(&body, rng.chance(500));
        let limits = Limits {
            depth: 4,
            number: 16,
            length: 1 << 12,
            string: 4096,
            chunk: u32::try_from(rng.between(1, 32)).unwrap(),
        };
        let settings = Settings::calm(&mut rng, limits);
        for event in world::check_demands(&document, &settings, seed).answers {
            match event {
                Event::Token(_) => tally.note("Token".into()),
                Event::Long(_) => tally.note("Long".into()),
                Event::Skipped(_) => tally.note("Skipped".into()),
                Event::Done => {}
                event @ (Event::Failed(_) | Event::Closed) => panic!("seed {seed}: {event:?}"),
            }
        }
    }
    tally.assert_fell(&["Token", "Long", "Skipped"]);
}

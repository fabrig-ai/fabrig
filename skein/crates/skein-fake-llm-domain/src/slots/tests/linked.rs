//! Linked slot draws and line-ended captures (skein/docs/design/testing/fakes/drawn-agent.md, 2).

use super::{
    CONFIG, Domain, Slot, arguments, call, copy_of, history, menu, no_scripts, query, refused, slot, text, user,
};
use crate::api::{Bounds, Error, Query};
use skein_lib::List;

const PAIR: &[u8] = br#"{"target":"{{#asker}}","question":"{{#question}}"}"#;
const ONLY_QUESTION: &[u8] = br#"{"question":"{{#question}}"}"#;
const ASK_FOUR: &[u8] = b"4 asks (question 7; answer with message target=4, form=answer, question=7)";
const ASK_FIVE: &[u8] = b"5 asks (question 9; answer with message target=5, form=answer, question=9)";
const SEED: u64 = 1;

fn numeric(name: &[u8], before: &[u8], after: &[u8]) -> Slot {
    Slot { numeric: Some(Bounds { low: 1, high: u64::MAX }), ..slot(name, before, after, b"0") }
}

fn asker() -> Slot {
    numeric(b"asker", b"target=", b",")
}

fn question(foreign: u32) -> Slot {
    Slot { with: copy_of(b"asker"), foreign, ..numeric(b"question", b"question=", b")") }
}

fn parts(labels: &[&[u8]]) -> Query {
    let mut parts = List::with_capacity(u32::try_from(labels.len()).expect("fixture labels"));
    for label in labels {
        parts.push(text(label)).expect("room for each label");
    }
    query(Box::new([user(parts.into_boxed())]))
}

#[test]
fn linked_draw_chooses_own_or_foreign_by_value() {
    // This fixed span reaches both asker values independently of the second draw.
    const SEEDS: core::ops::RangeInclusive<u64> = 1..=10;
    let labels = parts(&[ASK_FOUR, ASK_FIVE]);
    // Newer mentions win deduplication, but do not hold either question.
    let mentions = parts(&[ASK_FOUR, ASK_FIVE, b"A later mention: target=4, target=5,"]);
    for request in [labels, mentions] {
        for (foreign, with_four, with_five) in [
            (0, br#"{"target":4,"question":7}"#.as_slice(), br#"{"target":5,"question":9}"#.as_slice()),
            (1000, br#"{"target":4,"question":9}"#.as_slice(), br#"{"target":5,"question":7}"#.as_slice()),
        ] {
            let menu = menu(PAIR, Box::new([asker(), question(foreign)]));
            let mut drew_four = false;
            let mut drew_five = false;
            for seed in SEEDS {
                let answer = call(CONFIG, menu.clone(), request.clone(), seed).expect("linked call");
                let pair = arguments(&answer);
                assert!(pair == with_four || pair == with_five, "foreign {foreign}: wrong pair {pair:?}");
                drew_four |= pair == with_four;
                drew_five |= pair == with_five;
            }
            assert!(drew_four && drew_five, "both askers were drawn at foreign weight {foreign}");
        }
    }
}

#[test]
fn linked_draw_uses_the_other_pool_if_empty() {
    for (foreign, labels) in [(0, [b"target=4,".as_slice(), b"question=7)"]), (1000, [ASK_FOUR, b"no other candidate"])]
    {
        let menu = menu(PAIR, Box::new([asker(), question(foreign)]));
        let answer = call(CONFIG, menu, parts(&labels), SEED).expect("fallback call");
        assert_eq!(arguments(&answer), br#"{"target":4,"question":7}"#, "foreign weight {foreign}");
    }
}

#[test]
fn absent_or_unissued_link_leaves_an_independent_draw() {
    for (body, weight, expected) in
        [(ONLY_QUESTION, 0, br#"{"question":7}"#.as_slice()), (PAIR, 1000, br#"{"target":0,"question":9}"#.as_slice())]
    {
        let request = parts(&[b"target=0, question=9)", ASK_FOUR]);
        let menu = menu(body, Box::new([Slot { weight, ..asker() }, question(1000)]));
        let answer = call(CONFIG, menu, request, SEED).expect("independent draw");
        assert_eq!(arguments(&answer), expected, "asker weight {weight}");
    }
}

#[test]
fn malformed_links_are_refused_at_configuration() {
    for (linked, foreign) in [(b"unknown".as_slice(), 0), (b"question", 0), (b"asker", 1001), (b"", 1)] {
        let question = Slot { with: copy_of(linked), ..question(foreign) };
        refused(PAIR, Box::new([asker(), question]));
    }
    refused(PAIR, Box::new([question(0), asker()]));
}

#[test]
fn linked_name_bytes_share_the_slot_byte_cap() {
    // Both names, their delimiters, both never-issued values, and the name in `with`.
    const FIELDS: usize = b"askertarget=,0questionquestion=)0asker".len();
    let menu = menu(PAIR, Box::new([asker(), question(0)]));
    let bytes = 2 * size_of::<Slot>() + FIELDS + size_of::<Box<[u8]>>() + PAIR.len();
    let cap = u32::try_from(bytes).expect("bounded fixture");
    for (script_bytes, error) in [(cap, None), (cap - 1, Some(Error::InvalidRequest))] {
        let config = crate::Config { script_bytes, ..CONFIG };
        assert_eq!(Domain::played(&config, SEED, no_scripts(), menu.clone()).err(), error);
    }
}

#[test]
fn empty_after_ends_at_line_feed_or_part_end() {
    let slot = numeric(b"message", b"Sent message ", b"");
    let menu = menu(br#"["{{#message}}"]"#, Box::new([slot]));
    for (line, expected) in [
        (b"Sent message 5".as_slice(), b"[5]".as_slice()),
        (b"Sent message 6\nnext", b"[6]"),
        (b"Sent message 5.", b"[0]"),
    ] {
        let answer = call(CONFIG, menu.clone(), history(line), SEED).expect("line-ended numeric draw");
        assert_eq!(arguments(&answer), expected, "input {line:?}");
    }
}

#[test]
fn line_ended_values_stop_at_the_256_byte_limit() {
    const MAX_VALUE: usize = 256; // drawn-agent.md, 2
    let slot = slot(b"message", b"Sent message ", b"", b"missing");
    let prefix = slot.before.clone();
    let menu = menu(b"{{message}}", Box::new([slot]));
    let longest = [b'x'; MAX_VALUE];
    for (value, expected) in [(longest.as_slice(), longest.as_slice()), (&[b'x'; MAX_VALUE + 1], b"missing".as_slice())]
    {
        for suffix in [b"".as_slice(), b"\nnext"] {
            let input = skein_lib::bytes::concat(&[&prefix, value, suffix]);
            let answer = call(CONFIG, menu.clone(), history(&input), SEED).expect("line-ended draw");
            assert_eq!(arguments(&answer), expected, "value length {} suffix {suffix:?}", value.len());
        }
    }
}

//! Whole borrowed inputs under the tokenizer's requests (data/json.md, 5.3).

use alloc::vec::Vec;

use skein_lib::Queue;
use skein_lib::stream::Down;

use crate::Token;
use crate::tokenizer::{self, Error, Event, Feed, Limits, Request};

use super::{LIMITS, key, number, string};

const DOCUMENT: &[u8] = b" {\"a\":[\"\\u0061\\n\",-1.2e3,true,false,null,{}]} \n";
// Text(2) keeps two decoded bytes; the rows straddle that bound.
const TEXT_CAP: u32 = 2;

fn read(input: &[u8], limits: &Limits) -> Vec<Event> {
    let mut feed = Feed::new(input, limits);
    let mut events = Vec::new();
    // Every token consumes input, followed by one outcome.
    for _answer in 0..=input.len() {
        let event = feed.answer(Request::Next);
        let over = match &event {
            Event::Token(_) | Event::Skipped(_) | Event::Long(_) => false,
            Event::Done | Event::Failed(_) | Event::Closed => true,
        };
        events.push(event);
        if over {
            return events;
        }
    }
    panic!("the input bounds the answers");
}

#[test]
fn next_answers_each_token_then_the_document_outcome() {
    assert_eq!(
        read(DOCUMENT, &LIMITS),
        [
            Event::Token(Token::ObjectStart),
            Event::Token(key(b"a")),
            Event::Token(Token::ArrayStart),
            Event::Token(string(b"a\n")),
            Event::Token(number(b"-1.2e3")),
            Event::Token(Token::True),
            Event::Token(Token::False),
            Event::Token(Token::Null),
            Event::Token(Token::ObjectStart),
            Event::Token(Token::ObjectEnd),
            Event::Token(Token::ArrayEnd),
            Event::Token(Token::ObjectEnd),
            Event::Done,
        ]
    );
}

#[test]
fn text_keeps_short_strings_and_counts_long_strings() {
    for (input, expected) in [
        (br#""a""#.as_slice(), Event::Token(string(b"a"))),
        (br#""ab""#, Event::Token(string(b"ab"))),
        (br#""abc""#, Event::Long(3)),
        (br#""\u0061\n""#, Event::Token(string(b"a\n"))),
    ] {
        let mut feed = Feed::new(input, &LIMITS);
        assert_eq!(feed.answer(Request::Text(TEXT_CAP)), expected, "{input:?}");
        assert_eq!(feed.answer(Request::Next), Event::Done, "{input:?}");
    }
}

#[test]
fn skip_checks_the_value_then_next_checks_the_end() {
    // The skipped bytes are the complete value, excluding its surrounding whitespace.
    const VALUE_BYTES: u64 = 21;
    let input = b" {\"x\":[\"longer\",null]} \n";
    let limits = Limits { string: 0, ..LIMITS };
    let mut feed = Feed::new(input, &limits);
    assert_eq!(feed.answer(Request::Skip), Event::Skipped(VALUE_BYTES));
    assert_eq!(feed.answer(Request::Next), Event::Done);
}

#[test]
fn a_feed_can_close_or_be_dropped_mid_document() {
    let mut closing = Feed::new(DOCUMENT, &LIMITS);
    assert_eq!(closing.answer(Request::Next), Event::Token(Token::ObjectStart));
    assert_eq!(closing.answer(Request::Close), Event::Closed);
    let mut abandoned = Feed::new(DOCUMENT, &LIMITS);
    assert_eq!(abandoned.answer(Request::Next), Event::Token(Token::ObjectStart));
    drop(abandoned);
}

#[test]
fn malformed_inputs_preserve_the_tokenizers_failure() {
    for (input, expected) in
        [(b"[".as_slice(), Error::Truncated), (b"true false", Error::Trailing), (b"[[[[[]]]]]", Error::TooDeep)]
    {
        assert_eq!(read(input, &LIMITS).last(), Some(&Event::Failed(expected)), "{input:?}");
    }
}

#[test]
fn feed_and_hand_fed_tokenizer_answer_the_same_requests() {
    // Each scan cut crosses different escape and text boundaries.
    for chunk in [1, 2, 4, 64] {
        let limits = Limits { chunk, ..LIMITS };
        assert_eq!(read(DOCUMENT, &limits), super::read(DOCUMENT, limits), "chunk {chunk}");
        let input = br#"["\u0061bc",{"x":[1,true]},null]"#;
        let requests =
            [Request::Next, Request::Text(TEXT_CAP), Request::Skip, Request::Next, Request::Next, Request::Next];
        let mut feed = Feed::new(input, &limits);
        let mut events = Vec::new();
        for request in requests {
            events.push(feed.answer(request));
        }
        let mut hand_fed = Vec::new();
        for (event, _) in super::placed(input, limits, &requests) {
            hand_fed.push(event);
        }
        assert_eq!(events, hand_fed, "chunk {chunk}");
    }
}

#[test]
fn feed_bound_counts_the_tokenizer_and_two_one_entry_queues() {
    for limits in [LIMITS, Limits { string: 0, ..LIMITS }, Limits { chunk: 64, ..LIMITS }] {
        let expected = tokenizer::tokenizer_bound(&limits)
            .unwrap()
            .checked_add(Queue::<Event>::price(1).unwrap())
            .unwrap()
            .checked_add(Queue::<Down>::price(1).unwrap())
            .unwrap();
        assert_eq!(tokenizer::feed_bound(&limits), Some(expected));
        let bounded = Limits { string: 0, ..limits };
        assert_eq!(tokenizer::validation_bound(&limits), tokenizer::feed_bound(&bounded));
    }
    for limits in [Limits { chunk: 0, ..LIMITS }, Limits { length: u32::MAX, ..LIMITS }] {
        assert_eq!(tokenizer::feed_bound(&limits), None);
        assert_eq!(tokenizer::validation_bound(&limits), None);
    }
}

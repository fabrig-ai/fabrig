//! Whole-input parsing, grammar and independent bounds (skein/docs/design/json.md, 5.3).

use crate::{Kind, ParseError, document, parse, tokenizer};

const RETAINED: document::Limits = document::Limits { tokens: 32, text: 64 };

#[test]
fn parses_all_values_and_decoded_text_through_the_stream_end() {
    for chunk in [1, 2, 4, 64] {
        let limits = tokenizer::Limits { chunk, ..super::LIMITS };
        let doc = parse(br#" {"a":["\u0061\n",-1.2e3,true,false,null,{},[]]} "#, &limits, &RETAINED).unwrap();
        assert_eq!(doc.token(0).unwrap().kind, Kind::ObjectStart);
        assert_eq!(doc.text(doc.token(3).unwrap()), Some(b"a\n".as_slice()));
        assert_eq!(doc.text(doc.token(4).unwrap()), Some(b"-1.2e3".as_slice()));
        assert_eq!(doc.value_end(0), Some(doc.len()));
        for input in [b"1".as_slice(), b"true", b"false", b"null", br#""text""#, b"[]", b"{}"] {
            drop(parse(input, &limits, &RETAINED).unwrap());
        }
    }
}

#[test]
fn refuses_empty_truncated_invalid_and_trailing_values() {
    for input in [b"".as_slice(), b" ", b"[", b"{\"x\":", b"tru", b"1 2", b"{}x", br#""\q""#, b"[1,]"] {
        match parse(input, &super::LIMITS, &RETAINED).unwrap_err() {
            ParseError::Tokenizer(_) => {}
            ParseError::Document(error) => panic!("expected grammar refusal for {input:?}, got {error:?}"),
        }
    }
}

#[test]
fn tokenizer_and_retention_limits_are_independent_and_inclusive() {
    let input = br#"{"x":"abc"}"#;
    let limits =
        tokenizer::Limits { depth: 1, string: 3, number: 1, chunk: 1, length: u32::try_from(input.len()).unwrap() };
    let retained = document::Limits { tokens: 4, text: 4 };
    drop(parse(input, &limits, &retained).unwrap());
    assert_eq!(
        parse(input, &limits, &document::Limits { tokens: 3, ..retained }),
        Err(ParseError::Document(document::Error::TooManyTokens))
    );
    assert_eq!(
        parse(input, &limits, &document::Limits { text: 3, ..retained }),
        Err(ParseError::Document(document::Error::TooMuchText))
    );
    assert_eq!(
        parse(input, &tokenizer::Limits { length: limits.length - 1, ..limits }, &retained),
        Err(ParseError::Tokenizer(tokenizer::Error::TooLong))
    );
    assert_eq!(
        parse(input, &tokenizer::Limits { string: 2, ..limits }, &retained),
        Err(ParseError::Tokenizer(tokenizer::Error::StringTooLong))
    );
    assert_eq!(
        parse(input, &tokenizer::Limits { depth: 0, ..limits }, &retained),
        Err(ParseError::Tokenizer(tokenizer::Error::TooDeep))
    );
    assert_eq!(parse(b"12", &limits, &retained), Err(ParseError::Tokenizer(tokenizer::Error::NumberTooLong)));
    drop(parse(b"1", &limits, &document::Limits { tokens: 1, text: 1 }).unwrap());
    assert_eq!(
        parse(b"null", &limits, &document::Limits { tokens: 0, text: 0 }),
        Err(ParseError::Document(document::Error::TooManyTokens))
    );
}

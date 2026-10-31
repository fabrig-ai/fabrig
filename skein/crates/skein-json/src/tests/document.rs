//! Compact records read by index, counts and checked offset edges (json.md, 5.2).

#![expect(clippy::disallowed_macros, reason = "a test supplies its own token sequence with vec!")]

use alloc::boxed::Box;
use alloc::vec;

use crate::document::{self, Error, Limits};
use crate::writer::{self, Encoder};
use crate::{Compact, Document, Kind, Token};

#[test]
fn records_borrow_their_text_from_one_buffer() {
    let tokens = vec![
        Token::ObjectStart,
        Token::Key(Box::from(b"key".as_slice())),
        Token::ArrayStart,
        Token::String(Box::from(b"value".as_slice())),
        Token::Number(Box::from(b"42".as_slice())),
        Token::True,
        Token::False,
        Token::Null,
        Token::ArrayEnd,
        Token::ObjectEnd,
    ];
    let doc = Document::from_tokens(&tokens, &Limits { tokens: 10, text: 10 }).unwrap();
    assert_eq!(doc.len(), 10);
    assert_eq!(doc.text_len(), 10);
    let key = doc.token(1).unwrap();
    let value = doc.token(3).unwrap();
    let number = doc.token(4).unwrap();
    assert_eq!(*key, Compact { kind: Kind::Key, start: 0, len: 3 });
    assert_eq!(*value, Compact { kind: Kind::String, start: 3, len: 5 });
    assert_eq!(*number, Compact { kind: Kind::Number, start: 8, len: 2 });
    assert_eq!(doc.text(key), Some(b"key".as_slice()));
    assert_eq!(doc.text(value), Some(b"value".as_slice()));
    assert_eq!(doc.text(number), Some(b"42".as_slice()));
    assert_eq!(doc.token(10), None);
    assert!(!doc.is_empty());
    let limits = writer::Limits { depth: 2, length: 128 };
    let mut measure = Encoder::measure(&limits);
    measure.document(&doc);
    let mut write = Encoder::write(measure.measured().unwrap(), &limits);
    write.document(&doc);
    assert_eq!(write.finish().as_ref(), br#"{"key":["value",42,true,false,null]}"#);
}

#[test]
fn both_counts_are_checked_at_and_past_their_limits() {
    let tokens = [Token::String(Box::from(b"abc".as_slice()))];
    let limits = Limits { tokens: 1, text: 3 };
    drop(Document::from_tokens(&tokens, &limits).unwrap());
    assert_eq!(Document::from_tokens(&tokens, &Limits { tokens: 0, ..limits }), Err(Error::TooManyTokens));
    assert_eq!(Document::from_tokens(&tokens, &Limits { text: 2, ..limits }), Err(Error::TooMuchText));
    let empty = Document::from_tokens(&[], &Limits { tokens: 0, text: 0 }).unwrap();
    assert!(empty.is_empty());
    assert_eq!(document::document_bound(&Limits { tokens: 0, text: 0 }), Some(0));
}

#[test]
fn offsets_at_the_u32_edge_never_wrap() {
    let edge = Compact { kind: Kind::String, start: u32::MAX, len: 0 };
    assert_eq!(edge.end(), Some(u32::MAX));
    assert_eq!(Compact { len: 1, ..edge }.end(), None);
    let limits = Limits { tokens: 1, text: 0 };
    assert_eq!(Document::from_parts(Box::from([]), Box::from([edge]), &limits), Err(Error::Range));
    assert_eq!(
        Document::from_parts(Box::from([]), Box::from([Compact { len: 1, ..edge }]), &limits),
        Err(Error::Range)
    );
    let long = Compact { kind: Kind::Long, start: 0, len: u32::MAX };
    let document = Document::from_parts(Box::from([]), Box::from([long]), &limits).unwrap();
    assert_eq!(document.text(document.token(0).unwrap()), Some(b"".as_slice()));
    assert_eq!(document.token(0).unwrap().len, u32::MAX);
}

#[test]
#[should_panic(expected = "Long has no text to write")]
fn a_long_string_cannot_be_written_back() {
    let limits = Limits { tokens: 1, text: 0 };
    let long = Compact { kind: Kind::Long, start: 0, len: 42 };
    let document = Document::from_parts(Box::from([]), Box::from([long]), &limits).unwrap();
    Encoder::measure(&writer::Limits { depth: 1, length: 100 }).document(&document);
}

#[test]
fn members_skip_nested_values_and_stop_at_the_object_end() {
    let input = br#"[{"nested":{"needle":false},"items":[{},[]],"needle":42,"needle":7},{"outside":true}]"#;
    let doc = crate::parse(input, &super::LIMITS, &Limits { tokens: 40, text: 80 }).unwrap();
    let index = doc.member(1, b"needle").unwrap();
    assert_eq!(doc.text(doc.token(index).unwrap()), Some(b"42".as_slice()));
    assert_eq!(doc.member(1, b"outside"), None);
    assert_eq!(doc.member(1, b"missing"), None);
    assert_eq!(doc.member(0, b"needle"), None);
    assert_eq!(doc.member(index, b"needle"), None);
    assert_eq!(doc.member(u32::MAX, b"needle"), None);
    let nested = doc.member(1, b"nested").unwrap();
    let needle = doc.member(nested, b"needle").unwrap();
    assert_eq!(doc.token(needle).unwrap().kind, Kind::False);
    assert_eq!(doc.value_end(0), Some(doc.len()));
    assert_eq!(doc.value_end(1), Some(doc.len() - 5));
}

#[test]
fn value_ends_cover_every_kind_and_document_edges() {
    let kinds = [Kind::String, Kind::Number, Kind::True, Kind::False, Kind::Null, Kind::Long];
    for kind in kinds {
        let doc = Document::from_parts(
            Box::from([]),
            Box::from([Compact { kind, start: 0, len: 0 }]),
            &Limits { tokens: 1, text: 0 },
        )
        .unwrap();
        assert_eq!(doc.value_end(0), Some(1), "{kind:?}");
        assert_eq!(doc.value_end(1), None);
        assert_eq!(doc.value_end(u32::MAX), None);
    }
    for token in [Token::Key(Box::from([])), Token::ObjectEnd, Token::ArrayEnd, Token::ObjectStart, Token::ArrayStart] {
        let doc = Document::from_tokens(&[token], &Limits { tokens: 1, text: 0 }).unwrap();
        assert_eq!(doc.value_end(0), None);
        assert_eq!(doc.member(0, b"absent"), None);
    }
    let doc = Document::from_tokens(
        &[Token::ObjectStart, Token::Key(Box::from(b"x".as_slice()))],
        &Limits { tokens: 2, text: 1 },
    )
    .unwrap();
    assert_eq!(doc.member(0, b"x"), None);
    let empty = Document::from_tokens(&[], &Limits { tokens: 0, text: 0 }).unwrap();
    assert_eq!(empty.value_end(0), None);
    assert_eq!(empty.member(0, b"x"), None);
}

#[test]
fn a_long_member_is_a_value_without_retained_text() {
    let records = [
        Compact { kind: Kind::ObjectStart, start: 0, len: 0 },
        Compact { kind: Kind::Key, start: 0, len: 1 },
        Compact { kind: Kind::Long, start: 0, len: u32::MAX },
        Compact { kind: Kind::ObjectEnd, start: 0, len: 0 },
    ];
    let doc =
        Document::from_parts(Box::from(b"x".as_slice()), Box::from(records), &Limits { tokens: 4, text: 1 }).unwrap();
    assert_eq!(doc.member(0, b"x"), Some(2));
    assert_eq!(doc.value_end(2), Some(3));
    assert_eq!(doc.value_end(0), Some(4));
}

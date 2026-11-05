//! Parsed event-wire edits for shapes that typed records cannot represent.
//! Every selected object member must occur exactly once before it is edited.

use skein_json::{Document, Kind, Token, writer::Encoder};

pub fn set(line: &str, path: &[&str], value: &str) -> String {
    let document = parse(line);
    let start = member(&document, path);
    let end = document.value_end(start).expect("complete selected value");
    let mut tokens = tokens(&document);
    tokens.splice(index(start)..index(end), self::tokens(&parse(value)));
    encode(&tokens, line.ends_with('\n'))
}

pub fn remove(line: &str, path: &[&str]) -> String {
    let document = parse(line);
    let start = member(&document, path);
    let end = document.value_end(start).expect("complete selected value");
    let mut tokens = tokens(&document);
    tokens.drain(index(start - 1)..index(end));
    encode(&tokens, line.ends_with('\n'))
}

pub fn insert(line: &str, path: &[&str], key: &str, value: &str) -> String {
    let document = parse(line);
    let object = if path.is_empty() { 0 } else { member(&document, path) };
    assert_eq!(document.token(object).expect("insert object").kind, Kind::ObjectStart);
    assert!(document.member(object, key.as_bytes()).is_none(), "inserted member must be new: {key}");
    let end = document.value_end(object).expect("complete object") - 1;
    let mut tokens = tokens(&document);
    let mut added = vec![Token::Key(key.as_bytes().into())];
    added.extend(self::tokens(&parse(value)));
    tokens.splice(index(end)..index(end), added);
    encode(&tokens, line.ends_with('\n'))
}

pub fn duplicate(line: &str, path: &[&str]) -> String {
    let document = parse(line);
    let start = member(&document, path);
    let end = document.value_end(start).expect("complete selected value");
    let mut tokens = tokens(&document);
    let copy = tokens[index(start - 1)..index(end)].to_vec();
    tokens.splice(index(end)..index(end), copy);
    encode(&tokens, line.ends_with('\n'))
}

pub fn string(value: &str) -> String {
    encode(&[Token::String(value.as_bytes().into())], false)
}

fn index(value: u32) -> usize {
    usize::try_from(value).expect("fixture index fits usize")
}

fn parse(line: &str) -> Document {
    let length = u32::try_from(line.len()).expect("fixture length");
    let limits = skein_json::tokenizer::Limits { depth: 32, string: length, number: length, chunk: 256, length };
    let retained = skein_json::document::Limits { tokens: length, text: length };
    skein_json::parse(line.as_bytes(), &limits, &retained).expect("wire edit starts from JSON")
}

fn member(document: &Document, path: &[&str]) -> u32 {
    assert!(!path.is_empty(), "an edit names a member");
    let mut object = 0;
    for name in path {
        if document.token(object).expect("path parent").kind == Kind::ArrayStart {
            let position: u32 = name.parse().expect("array path segment is an index");
            let mut at = object + 1;
            for _ in 0..position {
                at = document.value_end(at).expect("array item");
            }
            assert!(at < document.value_end(object).expect("array end") - 1, "array edit selects one item");
            object = at;
            continue;
        }
        let mut at = object + 1;
        let end = document.value_end(object).expect("complete parent object") - 1;
        let mut found = Vec::new();
        while at < end {
            let key = document.token(at).expect("member key");
            assert_eq!(key.kind, Kind::Key, "edit path names objects");
            if document.text(key).expect("key bytes") == name.as_bytes() {
                found.push(at + 1);
            }
            at = document.value_end(at + 1).expect("complete member value");
        }
        assert_eq!(found.len(), 1, "wire edit must select one member: {path:?}, {name}");
        object = found[0];
    }
    object
}

fn tokens(document: &Document) -> Vec<Token> {
    (0..document.len())
        .map(|at| {
            let token = document.token(at).expect("document token");
            match token.kind {
                Kind::ObjectStart => Token::ObjectStart,
                Kind::ObjectEnd => Token::ObjectEnd,
                Kind::ArrayStart => Token::ArrayStart,
                Kind::ArrayEnd => Token::ArrayEnd,
                Kind::Key => Token::Key(document.text(token).expect("key bytes").into()),
                Kind::String => Token::String(document.text(token).expect("string bytes").into()),
                Kind::Number => Token::Number(document.text(token).expect("number bytes").into()),
                Kind::True => Token::True,
                Kind::False => Token::False,
                Kind::Null => Token::Null,
                Kind::Long => panic!("fixture text is fully retained"),
            }
        })
        .collect()
}

fn encode(tokens: &[Token], newline: bool) -> String {
    let limits = skein_json::writer::Limits { depth: 32, length: u32::MAX - 1 };
    let mut measure = Encoder::measure(&limits);
    for token in tokens {
        measure.token(token);
    }
    let mut writer = Encoder::write(measure.measured().expect("edited JSON size"), &limits);
    for token in tokens {
        writer.token(token);
    }
    let mut text = String::from_utf8(writer.finish().into_vec()).expect("edited JSON is UTF-8");
    if newline {
        text.push('\n');
    }
    text
}

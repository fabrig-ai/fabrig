//! Ops tool argument reads over compact JSON (jig/docs/design/domain/engine.md, 7.3).

use jig_protocol_tools::Why as Problem;
use skein_json::{
    Document, ParseError,
    document::{self, Kind},
    tokenizer,
};
use skein_lib::Stack;

pub(crate) fn parse(input: &[u8]) -> Result<Document, Problem> {
    if input.len() > 65_536 {
        return Err(Problem::TooLarge);
    }
    let length = u32::try_from(input.len()).or(Err(Problem::TooLarge))?;
    let limits = tokenizer::Limits { depth: 16, string: length, number: 24, chunk: length.max(1), length };
    let retained = document::Limits { tokens: 4096, text: length };
    let doc = match skein_json::parse(input, &limits, &retained) {
        Ok(doc) => doc,
        Err(ParseError::Tokenizer(_)) => return Err(Problem::Malformed),
        Err(ParseError::Document(_)) => return Err(Problem::TooLarge),
    };
    validate(&doc, limits.depth)?;
    if doc.token(0).expect("parsed root").kind != Kind::ObjectStart {
        return Err(Problem::Type);
    }
    Ok(doc)
}

#[derive(Debug)]
struct Container {
    at: u32,
    count: u32,
}

// Grammar belongs to skein-json; tool arguments also refuse repeated keys and
// containers with more than 128 direct values, including in unused fields.
fn validate(doc: &Document, depth: u32) -> Result<(), Problem> {
    let mut stack = Stack::with_capacity(depth);
    for at in 0..doc.len() {
        let record = doc.token(at).expect("admitted token");
        match record.kind {
            Kind::ObjectStart | Kind::ArrayStart => {
                stack.push(Container { at, count: 0 }).expect("tokenizer admitted depth");
                continue;
            }
            Kind::Key => {
                let object = stack.top().expect("parsed object").at;
                let name = doc.text(record).expect("parsed key");
                if doc.member(object, name) != at.checked_add(1) {
                    return Err(Problem::Malformed);
                }
                continue;
            }
            Kind::ObjectEnd | Kind::ArrayEnd => {
                stack.pop().expect("parsed container");
            }
            Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null | Kind::Long => {}
        }
        if let Some(parent) = stack.top_mut() {
            parent.count = parent.count.checked_add(1).expect("admitted token count");
            if parent.count > 128 {
                return Err(Problem::TooLarge);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Node<'a> {
    doc: &'a Document,
    at: u32,
}

impl<'a> Node<'a> {
    pub(crate) fn root(doc: &'a Document) -> Self {
        Self { doc, at: 0 }
    }

    pub(crate) fn required(self, name: &[u8]) -> Result<Self, Problem> {
        if self.doc.token(self.at).expect("value token").kind != Kind::ObjectStart {
            return Err(Problem::Type);
        }
        let at = self.doc.member(self.at, name).ok_or(Problem::Missing)?;
        Ok(Self { at, ..self })
    }

    pub(crate) fn text(self) -> Result<Box<[u8]>, Problem> {
        let record = self.doc.token(self.at).expect("value token");
        if record.kind != Kind::String {
            return Err(Problem::Type);
        }
        Ok(self.doc.text(record).expect("parsed string").into())
    }

    pub(crate) fn number(self) -> Result<u64, Problem> {
        let record = self.doc.token(self.at).expect("value token");
        if record.kind != Kind::Number {
            return Err(Problem::Type);
        }
        super::decimal(self.doc.text(record).expect("parsed number")).ok_or(Problem::Range)
    }

    pub(crate) fn small(self) -> Result<u32, Problem> {
        u32::try_from(self.number()?).or(Err(Problem::Range))
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Problem, parse};

    #[test]
    fn tool_json_refuses_duplicate_keys_in_unused_values() {
        assert_eq!(parse(br#"{"unused":[{"x":1,"x":2}]}"#), Err(Problem::Malformed));
    }

    #[test]
    fn tool_json_keeps_container_limits() {
        for count in [128, 129] {
            let input = format!("{{\"unused\":[{}]}}", vec!["null"; count].join(","));
            if count == 128 {
                assert!(parse(input.as_bytes()).is_ok());
            } else {
                assert_eq!(parse(input.as_bytes()), Err(Problem::TooLarge));
            }
        }
    }

    #[test]
    fn tool_json_reads_direct_members_after_nested_values() {
        let doc = parse(br#"{"unused":{"service":"nested"},"service":"top","operation":7}"#).expect("JSON");
        let node = Node::root(&doc);
        assert_eq!(node.required(b"service").expect("field").text().expect("text").as_ref(), b"top");
        assert_eq!(node.required(b"operation").expect("field").small(), Ok(7));
        assert!(matches!(node.required(b"missing"), Err(Problem::Missing)));
    }
}

//! Tool argument reads over compact JSON documents (jig/docs/design/domain/engine.md, 7.3).
//! The borrowed views live only while one call is decoded.

use alloc::boxed::Box;
use skein_json::{
    Document, ParseError,
    document::{self, Kind},
    tokenizer,
};
use skein_lib::Stack;

use crate::{Limits, Problem, Why};

pub(crate) fn parse(input: &[u8], bounds: &Limits) -> Result<Document, Problem> {
    if input.len() > usize::try_from(bounds.input_bytes).expect("u32 fits usize") {
        return Err(Problem { field: b"", why: Why::TooLarge });
    }
    let Ok(length) = u32::try_from(input.len()) else { return Err(Problem { field: b"", why: Why::TooLarge }) };
    let limits = tokenizer::Limits { depth: bounds.depth, string: length, number: 24, chunk: length.max(1), length };
    let retained = document::Limits { tokens: 4096, text: length };
    let doc = match skein_json::parse(input, &limits, &retained) {
        Ok(doc) => doc,
        Err(ParseError::Tokenizer(reason)) => return Err(tokenizer_problem(reason)),
        Err(ParseError::Document(_)) => return Err(Problem { field: b"", why: Why::TooLarge }),
    };
    validate(&doc, bounds.depth)?;
    if doc.token(0).expect("parsed root").kind != Kind::ObjectStart {
        return Err(Problem { field: b"", why: Why::Type });
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
                    return Err(Problem { field: b"", why: Why::Malformed });
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
                return Err(Problem { field: b"", why: Why::TooLarge });
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Node<'a> {
    doc: &'a Document,
    at: u32,
    field: &'static [u8],
    words: u32,
    elements: &'static [u8],
}

impl<'a> Node<'a> {
    pub(crate) fn root(doc: &'a Document, bounds: &Limits) -> Self {
        Self { doc, at: 0, field: b"", elements: b"", words: bounds.words }
    }

    pub(crate) fn object(self) -> bool {
        self.doc.token(self.at).expect("value token").kind == Kind::ObjectStart
    }

    pub(crate) fn field(self, name: &[u8]) -> Result<Option<Self>, Problem> {
        if !self.object() {
            return Err(Problem { field: self.field, why: Why::Type });
        }
        let Some(at) = self.doc.member(self.at, name) else { return Ok(None) };
        Ok(Some(Self { at, field: field_name(name), elements: field_name(name), ..self }))
    }

    pub(crate) fn required(self, name: &'static [u8]) -> Result<Self, Problem> {
        self.field(name)?.ok_or(Problem { field: name, why: Why::Missing })
    }

    pub(crate) fn string(self) -> Result<&'a [u8], Problem> {
        let record = self.doc.token(self.at).expect("value token");
        if record.kind != Kind::String {
            return Err(Problem { field: self.field, why: Why::Type });
        }
        Ok(self.doc.text(record).expect("parsed string"))
    }

    pub(crate) fn text(self) -> Result<Box<[u8]>, Problem> {
        let text = self.string()?;
        if text.len() > usize::try_from(self.words).expect("u32 fits usize") {
            return Err(Problem { field: self.field, why: Why::TooLarge });
        }
        Ok(skein_lib::bytes::copy_of(text))
    }

    pub(crate) fn number(self) -> Result<u64, Problem> {
        let record = self.doc.token(self.at).expect("value token");
        if record.kind != Kind::Number {
            return Err(Problem { field: self.field, why: Why::Type });
        }
        decimal(self.doc.text(record).expect("parsed number")).ok_or(Problem { field: self.field, why: Why::Range })
    }

    pub(crate) fn boolean(self) -> Result<bool, Problem> {
        match self.doc.token(self.at).expect("value token").kind {
            Kind::True => Ok(true),
            Kind::False => Ok(false),
            Kind::ObjectStart
            | Kind::ObjectEnd
            | Kind::ArrayStart
            | Kind::ArrayEnd
            | Kind::Key
            | Kind::String
            | Kind::Number
            | Kind::Null
            | Kind::Long => Err(Problem { field: self.field, why: Why::Type }),
        }
    }

    pub(crate) fn array(self) -> Result<Array<'a>, Problem> {
        if self.doc.token(self.at).expect("value token").kind != Kind::ArrayStart {
            return Err(Problem { field: self.field, why: Why::Type });
        }
        let end = self.doc.value_end(self.at).expect("parsed array").checked_sub(1).expect("array closing");
        let next = self.at.checked_add(1).expect("admitted token index");
        Ok(Array { node: self, next, end })
    }

    pub(crate) fn small(self) -> Result<u32, Problem> {
        u32::try_from(self.number()?).or(Err(Problem { field: self.field, why: Why::Range }))
    }

    pub(crate) fn narrow(self) -> Result<u16, Problem> {
        u16::try_from(self.number()?).or(Err(Problem { field: self.field, why: Why::Range }))
    }
}

pub(crate) struct Array<'a> {
    node: Node<'a>,
    next: u32,
    end: u32,
}

impl<'a> Array<'a> {
    pub(crate) fn len(&self) -> usize {
        let mut at = self.next;
        let mut count = 0_usize;
        for _ in self.next..self.end {
            if at == self.end {
                break;
            }
            count = count.checked_add(1).expect("admitted token count");
            at = self.node.doc.value_end(at).expect("parsed element");
        }
        count
    }

    pub(crate) fn next(&mut self) -> Option<Node<'a>> {
        if self.next == self.end {
            return None;
        }
        let at = self.next;
        self.next = self.node.doc.value_end(at).expect("parsed element");
        Some(Node { at, field: self.node.elements, elements: b"", ..self.node })
    }
}

fn field_name(key: &[u8]) -> &'static [u8] {
    const NAMES: &[&[u8]] = &[
        b"llm",
        b"main",
        b"model",
        b"effort",
        b"account",
        b"subagents",
        b"action",
        b"age",
        b"amendment",
        b"answers",
        b"at",
        b"authority",
        b"batch",
        b"before",
        b"body",
        b"budget",
        b"change_kind",
        b"charter",
        b"choices",
        b"code",
        b"connector",
        b"contract",
        b"count",
        b"deadline",
        b"decision",
        b"delegation",
        b"dependencies",
        b"depth",
        b"description",
        b"environment",
        b"executor",
        b"fields",
        b"filter",
        b"form",
        b"from",
        b"goal",
        b"grants",
        b"hold",
        b"inputs",
        b"instructions",
        b"kind",
        b"kinds",
        b"last",
        b"left",
        b"max",
        b"max_bytes",
        b"name",
        b"news",
        b"note_resources",
        b"notes",
        b"notices",
        b"number",
        b"operation",
        b"page",
        b"parameters",
        b"pattern",
        b"period",
        b"person",
        b"project",
        b"proposal",
        b"proposer",
        b"query",
        b"question",
        b"questions",
        b"reason",
        b"recalled",
        b"references",
        b"resource",
        b"results",
        b"revision",
        b"right",
        b"role",
        b"scope",
        b"scopes",
        b"segments",
        b"service",
        b"setting",
        b"spec",
        b"spend",
        b"subscription",
        b"symbolic_grants",
        b"target",
        b"task",
        b"tasks",
        b"terminal",
        b"through",
        b"timers",
        b"tools",
        b"value",
        b"wake",
        b"words",
    ];
    for name in NAMES {
        if key == *name {
            return name;
        }
    }
    b""
}

pub(crate) fn decimal(bytes: &[u8]) -> Option<u64> {
    let mut value = 0_u64;
    if bytes.is_empty() {
        return None;
    }
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

fn tokenizer_problem(reason: tokenizer::Error) -> Problem {
    let why = match reason {
        tokenizer::Error::TooLong
        | tokenizer::Error::TooDeep
        | tokenizer::Error::StringTooLong
        | tokenizer::Error::NumberTooLong => Why::TooLarge,
        tokenizer::Error::Unexpected
        | tokenizer::Error::Trailing
        | tokenizer::Error::Number
        | tokenizer::Error::Escape
        | tokenizer::Error::Surrogate
        | tokenizer::Error::Utf8
        | tokenizer::Error::Control
        | tokenizer::Error::Truncated
        | tokenizer::Error::Stream(_) => Why::Malformed,
    };
    Problem { field: b"", why }
}

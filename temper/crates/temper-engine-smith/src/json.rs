//! Bounded JSON-object decoding with Skein's tokenizer. The decoded tree is
//! temporary and never crosses the engine or Smith domain boundary.

use alloc::boxed::Box;
use skein_json::{Document, Kind, ParseError, document, tokenizer};
use skein_lib::List;

use crate::Problem;

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Value {
    Object(Box<[(Box<[u8]>, Value)]>),
    Array(Box<[Value]>),
    String(Box<[u8]>),
    Number(Box<[u8]>),
    Boolean(bool),
    Null,
}

pub(crate) fn parse(input: &[u8]) -> Result<Value, Problem> {
    if input.len() > 65_536 {
        return Err(Problem::TooLarge);
    }
    let Ok(length) = u32::try_from(input.len()) else { return Err(Problem::TooLarge) };
    let limits = tokenizer::Limits { depth: 16, string: length, number: 24, chunk: length.max(1), length };
    let parsed = match skein_json::parse(input, &limits, &document::Limits { tokens: length, text: length }) {
        Ok(parsed) => parsed,
        Err(ParseError::Tokenizer(_)) => return Err(Problem::Malformed),
        Err(ParseError::Document(_)) => return Err(Problem::TooLarge),
    };
    let mut at = 0_u32;
    let value = value(&parsed, &mut at)?;
    if at != parsed.len() {
        return Err(Problem::Malformed);
    }
    match value {
        Value::Object(_) => Ok(value),
        Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
            Err(Problem::Type(b"input"))
        }
    }
}

fn value(document: &Document, at: &mut u32) -> Result<Value, Problem> {
    let record = document.token(*at).ok_or(Problem::Malformed)?;
    *at = at.checked_add(1).ok_or(Problem::TooLarge)?;
    match record.kind {
        Kind::ObjectStart => {
            let mut fields: List<(Box<[u8]>, Value)> = List::with_capacity(document.len());
            for _ in 0..document.len() {
                let record = document.token(*at).ok_or(Problem::Malformed)?;
                match record.kind {
                    Kind::ObjectEnd => {
                        *at = at.checked_add(1).ok_or(Problem::TooLarge)?;
                        return Ok(Value::Object(fields.into_boxed()));
                    }
                    Kind::Key => {
                        let key = document.text(record).ok_or(Problem::Malformed)?;
                        *at = at.checked_add(1).ok_or(Problem::TooLarge)?;
                        for (previous, _) in &fields {
                            if previous.as_ref() == key {
                                return Err(Problem::Malformed);
                            }
                        }
                        let item = value(document, at)?;
                        if fields.push((key.into(), item)).is_err() {
                            return Err(Problem::TooLarge);
                        }
                    }
                    Kind::ObjectStart
                    | Kind::ArrayStart
                    | Kind::ArrayEnd
                    | Kind::String
                    | Kind::Number
                    | Kind::True
                    | Kind::False
                    | Kind::Null
                    | Kind::Long => return Err(Problem::Malformed),
                }
            }
            Err(Problem::Malformed)
        }
        Kind::ArrayStart => {
            let mut items = List::with_capacity(document.len());
            for _ in 0..document.len() {
                match document.token(*at).ok_or(Problem::Malformed)?.kind {
                    Kind::ArrayEnd => {
                        *at = at.checked_add(1).ok_or(Problem::TooLarge)?;
                        return Ok(Value::Array(items.into_boxed()));
                    }
                    Kind::ObjectStart
                    | Kind::ObjectEnd
                    | Kind::ArrayStart
                    | Kind::Key
                    | Kind::String
                    | Kind::Number
                    | Kind::True
                    | Kind::False
                    | Kind::Null
                    | Kind::Long => {
                        let item = value(document, at)?;
                        if items.push(item).is_err() {
                            return Err(Problem::TooLarge);
                        }
                    }
                }
            }
            Err(Problem::Malformed)
        }
        Kind::String => Ok(Value::String(document.text(record).ok_or(Problem::Malformed)?.into())),
        Kind::Number => Ok(Value::Number(document.text(record).ok_or(Problem::Malformed)?.into())),
        Kind::True => Ok(Value::Boolean(true)),
        Kind::False => Ok(Value::Boolean(false)),
        Kind::Null => Ok(Value::Null),
        Kind::ObjectEnd | Kind::ArrayEnd | Kind::Key | Kind::Long => Err(Problem::Malformed),
    }
}

impl Value {
    pub(crate) fn object(&self, member: &'static [u8]) -> Result<&Self, Problem> {
        match self {
            Value::Object(_) => Ok(self),
            Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type(member))
            }
        }
    }

    pub(crate) fn field(&self, name: &'static [u8]) -> Result<Option<&Value>, Problem> {
        match self {
            Value::Object(fields) => {
                for (key, value) in fields {
                    if key.as_ref() == name {
                        return Ok(Some(value));
                    }
                }
                Ok(None)
            }
            Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type(name))
            }
        }
    }

    pub(crate) fn required(&self, name: &'static [u8]) -> Result<&Value, Problem> {
        self.field(name)?.ok_or(Problem::Missing(name))
    }

    pub(crate) fn text(&self, member: &'static [u8]) -> Result<Box<[u8]>, Problem> {
        match self {
            Value::String(text) => Ok(text.clone()),
            Value::Object(_) | Value::Array(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type(member))
            }
        }
    }

    pub(crate) fn number(&self, member: &'static [u8]) -> Result<u64, Problem> {
        match self {
            Value::Number(text) => crate::decimal(text, member),
            Value::Object(_) | Value::Array(_) | Value::String(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type(member))
            }
        }
    }

    pub(crate) fn boolean(&self, member: &'static [u8]) -> Result<bool, Problem> {
        match self {
            Value::Boolean(value) => Ok(*value),
            Value::Object(_) | Value::Array(_) | Value::String(_) | Value::Number(_) | Value::Null => {
                Err(Problem::Type(member))
            }
        }
    }

    pub(crate) fn array(&self, member: &'static [u8]) -> Result<&[Value], Problem> {
        match self {
            Value::Array(items) => Ok(items),
            Value::Object(_) | Value::String(_) | Value::Number(_) | Value::Boolean(_) | Value::Null => {
                Err(Problem::Type(member))
            }
        }
    }

    pub(crate) fn small(&self, member: &'static [u8]) -> Result<u32, Problem> {
        let Ok(value) = u32::try_from(self.number(member)?) else { return Err(Problem::Range(member)) };
        Ok(value)
    }

    pub(crate) fn narrow(&self, member: &'static [u8]) -> Result<u16, Problem> {
        let Ok(value) = u16::try_from(self.number(member)?) else { return Err(Problem::Range(member)) };
        Ok(value)
    }
}

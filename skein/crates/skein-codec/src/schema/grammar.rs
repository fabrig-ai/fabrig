//! The grammar pass, keeping unresolved words for the layout pass.

use super::tokens::Token;
use super::{
    Declaration, Expected, Field, FieldIx, Held, MAX_DECLARATIONS, Schema, SchemaError, Shape, Span, Type, TypeIx,
    Variant, VariantIx, Wrong, add,
};
use alloc::boxed::Box;
use skein_lib::{List, Map};

#[derive(Debug)]
pub(super) struct Parsed {
    pub(super) schema: Schema,
    pub(super) references: List<Span>,
    pub(super) payloads: List<Option<Span>>,
}

#[derive(Debug)]
pub(super) struct Parser {
    parsed: Parsed,
    tokens: List<Token>,
    offset: u32,
    final_line: u32,
}

impl Parser {
    pub(super) fn new(text: Box<[u8]>, tokens: List<Token>, final_line: u32) -> Parser {
        let count = tokens.len();
        let text_len = u32::try_from(text.len()).expect("text ceiling");
        let declarations = count.min(MAX_DECLARATIONS);
        Parser {
            parsed: Parsed {
                schema: Schema {
                    text,
                    family: Span { start: 0, end: 0 },
                    version: 0,
                    declarations: List::with_capacity(declarations),
                    fields: List::with_capacity(count),
                    variants: List::with_capacity(count),
                    types: List::with_capacity(count),
                    names: Map::with_capacity(declarations),
                    minimum: List::with_capacity(declarations),
                    maximum: List::with_capacity(declarations),
                    former: List::with_capacity(count),
                    declaration_former: List::with_capacity(declarations),
                    field_former: List::with_capacity(count),
                    variant_former: List::with_capacity(count),
                    defaults: List::with_capacity(count),
                    encodings: List::with_capacity(count),
                    pool: List::with_capacity(text_len.checked_mul(8).expect("bounded pool")),
                },
                references: List::with_capacity(count),
                payloads: List::with_capacity(count),
            },
            tokens,
            offset: 0,
            final_line,
        }
    }

    fn peek(&self) -> Option<&[u8]> {
        self.ahead(0)
    }

    fn ahead(&self, offset: u32) -> Option<&[u8]> {
        match self.tokens.get(add(self.offset, offset)) {
            Some(token) => Some(self.parsed.schema.spelled(token.at)),
            None => None,
        }
    }

    fn line(&self) -> u32 {
        match self.tokens.get(self.offset) {
            Some(token) => token.line,
            None => self.final_line,
        }
    }

    fn take(&mut self) -> Result<Token, SchemaError> {
        match self.tokens.get(self.offset).copied() {
            Some(token) => {
                self.offset = add(self.offset, 1);
                Ok(token)
            }
            None => {
                let end = u32::try_from(self.parsed.schema.text.len()).expect("text passed its ceiling");
                Err(SchemaError { line: self.final_line, at: Span { start: end, end }, wrong: Wrong::End })
            }
        }
    }

    fn error(token: Token, wrong: Wrong) -> SchemaError {
        SchemaError { line: token.line, at: token.at, wrong }
    }

    fn expect(&mut self, word: &[u8], expected: Expected) -> Result<(), SchemaError> {
        let token = self.take()?;
        if self.parsed.schema.spelled(token.at) == word {
            Ok(())
        } else {
            Err(Self::error(token, Wrong::Expected(expected)))
        }
    }

    fn name(&mut self) -> Result<Token, SchemaError> {
        let token = self.take()?;
        if !valid_name(self.parsed.schema.spelled(token.at)) {
            return Err(Self::error(token, Wrong::Name));
        }
        Ok(token)
    }

    fn wanted(&mut self, expected: Expected) -> Result<Token, SchemaError> {
        match self.take() {
            Ok(token) => Ok(token),
            Err(error) => Err(SchemaError { wrong: Wrong::Expected(expected), ..error }),
        }
    }

    fn former(&mut self) -> Result<Span, SchemaError> {
        let start = self.parsed.schema.former.len();
        for _ in 0..self.tokens.len() {
            if self.peek() != Some(b"was") {
                break;
            }
            self.take()?;
            let token = self.wanted(Expected::Name)?;
            let value = self.parsed.schema.spelled(token.at);
            if !valid_name(value) {
                return Err(Self::error(token, Wrong::Expected(Expected::Name)));
            }
            self.parsed.schema.former.push(token.at).expect("each former name consumes a token");
        }
        Ok(Span { start, end: self.parsed.schema.former.len() })
    }

    fn default(&mut self) -> Result<Option<Span>, SchemaError> {
        if self.peek() != Some(b"=") {
            return Ok(None);
        }
        self.take()?;
        let mut start = None;
        for _ in 0..self.tokens.len() {
            let token = self.wanted(Expected::Default)?;
            let first = *start.get_or_insert(token.at.start);
            let word = self.parsed.schema.spelled(token.at);
            if word == b"some" {
                continue;
            }
            let end = if word == b"[" {
                let close = self.wanted(Expected::Default)?;
                if self.parsed.schema.spelled(close.at) != b"]" {
                    return Err(Self::error(close, Wrong::Expected(Expected::Default)));
                }
                close.at.end
            } else if word.starts_with(b"\"")
                || word.first().expect("nonempty token").is_ascii_alphanumeric()
                || word.starts_with(b"_")
                || word.starts_with(b"-")
            {
                token.at.end
            } else {
                return Err(Self::error(token, Wrong::Expected(Expected::Default)));
            };
            return Ok(Some(Span { start: first, end }));
        }
        unreachable!("a default consumes a token on every round")
    }

    fn family(&mut self) -> Result<Token, SchemaError> {
        let token = self.take()?;
        for &byte in self.parsed.schema.spelled(token.at) {
            if !byte.is_ascii_alphanumeric() && byte != b'_' && byte != b'-' {
                return Err(SchemaError { line: self.line(), at: token.at, wrong: Wrong::Family });
            }
        }
        Ok(token)
    }

    fn number(&mut self, wrong: Wrong, ceiling: u32) -> Result<u32, SchemaError> {
        let token = self.take()?;
        let mut value = 0_u32;
        for &byte in self.parsed.schema.spelled(token.at) {
            if !byte.is_ascii_digit() {
                return Err(Self::error(token, wrong));
            }
            let digit = u32::from(byte.checked_sub(b'0').expect("a decimal digit"));
            value = match value.checked_mul(10) {
                Some(value) => value,
                None => return Err(Self::error(token, wrong)),
            };
            value = match value.checked_add(digit) {
                Some(value) => value,
                None => return Err(Self::error(token, wrong)),
            };
            if value > ceiling {
                return Err(Self::error(token, wrong));
            }
        }
        Ok(value)
    }

    fn bound(&mut self) -> Result<u32, SchemaError> {
        self.number(Wrong::Bound, u32::MAX)
    }

    fn held(&mut self) -> Result<(Held, Span), SchemaError> {
        let empty = Span { start: 0, end: 0 };
        if self.ahead(1) == Some(b":") {
            return Ok((Held::Opaque, empty));
        }
        match self.peek() {
            Some(b"holds") => {
                self.take()?;
                let family = self.family()?.at;
                let name = self.name()?.at;
                Ok((Held::Record { family, name }, empty))
            }
            Some(b"pieces") => {
                self.take()?;
                let name = self.name()?.at;
                Ok((Held::Pieces(super::DeclIx::new(0)), name))
            }
            Some(_) | None => Ok((Held::Opaque, empty)),
        }
    }

    fn ty(&mut self) -> Result<TypeIx, SchemaError> {
        let first = TypeIx::new(self.parsed.schema.types.len());
        // Each round consumes a type word, so the token count bounds the chain.
        for _ in 0..self.tokens.len() {
            let token = self.take()?;
            let next = TypeIx::new(add(self.parsed.schema.types.len(), 1));
            let mut reference = token.at;
            let ty = match self.parsed.schema.spelled(token.at) {
                b"u8" => Type::U8,
                b"u16" => Type::U16,
                b"u32" => Type::U32,
                b"u64" => Type::U64,
                b"bool" => Type::Bool,
                b"duration" => Type::Duration,
                b"fixed" => Type::Fixed(self.bound()?),
                b"secret" => {
                    let bound = self.bound()?;
                    let (held, at) = self.held()?;
                    reference = at;
                    Type::Secret { bound, held }
                }
                b"text" => Type::Text(self.bound()?),
                b"bytes" => {
                    let bound = self.bound()?;
                    let (held, at) = self.held()?;
                    reference = at;
                    Type::Bytes { bound, held }
                }
                b"list" => Type::List { bound: self.bound()?, item: next },
                b"option" => Type::Option(next),
                b"{" | b"}" | b":" | b"," | b"=" | b"[" | b"]" => {
                    return Err(Self::error(token, Wrong::Expected(Expected::Type)));
                }
                _ => Type::Named(super::DeclIx::new(0)),
            };
            self.parsed.schema.types.push(ty).expect("each type consumed a token");
            self.parsed.references.push(reference).expect("one reference span per type");
            match ty {
                Type::List { .. } | Type::Option(_) => {}
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Bool
                | Type::Duration
                | Type::Fixed(_)
                | Type::Secret { .. }
                | Type::Text(_)
                | Type::Bytes { .. }
                | Type::Named(_) => return Ok(first),
            }
        }
        self.take()?;
        unreachable!("a chain consumes a token on every round")
    }

    fn record(&mut self, versioned: bool) -> Result<Declaration, SchemaError> {
        let name = self.name()?;
        let former = self.former()?;
        self.parsed.schema.declaration_former.push(former).expect("one former range per declaration");
        self.expect(b"{", Expected::Open)?;
        let first = self.parsed.schema.fields.len();
        for _ in 0..self.tokens.len() {
            if self.peek() == Some(b"}") {
                break;
            }
            let name = self.name()?;
            let former = self.former()?;
            self.parsed.schema.field_former.push(former).expect("one former range per field");
            self.expect(b":", Expected::Colon)?;
            let ty = self.ty()?;
            let default = self.default()?;
            self.parsed.schema.defaults.push(default).expect("one default per field");
            self.parsed.schema.encodings.push(None).expect("one encoding per field");
            self.parsed
                .schema
                .fields
                .push(Field { name: name.at, line: name.line, ty })
                .expect("each field consumed tokens");
            if self.peek() == Some(b",") {
                self.take()?;
            }
        }
        self.expect(b"}", Expected::Open)?;
        let count = self.parsed.schema.fields.len().checked_sub(first).expect("fields appended after the first");
        Ok(Declaration {
            name: name.at,
            line: name.line,
            shape: Shape::Record { versioned, first: FieldIx::new(first), count },
        })
    }

    fn enumeration(&mut self) -> Result<Declaration, SchemaError> {
        let name = self.name()?;
        let former = self.former()?;
        self.parsed.schema.declaration_former.push(former).expect("one former range per declaration");
        self.expect(b"{", Expected::Open)?;
        let first = self.parsed.schema.variants.len();
        for _ in 0..self.tokens.len() {
            if self.peek() == Some(b"}") {
                break;
            }
            let name = self.name()?;
            let former = self.former()?;
            self.parsed.schema.variant_former.push(former).expect("one former range per variant");
            let mut boxed = false;
            let payload = if self.peek() == Some(b":") {
                self.take()?;
                if self.peek() == Some(b"boxed") {
                    self.take()?;
                    boxed = true;
                }
                Some(self.name()?.at)
            } else {
                None
            };
            self.parsed
                .schema
                .variants
                .push(Variant { name: name.at, line: name.line, record: None, boxed })
                .expect("each variant consumed a token");
            self.parsed.payloads.push(payload).expect("one payload span per variant");
            if self.peek() == Some(b",") {
                self.take()?;
            }
        }
        self.expect(b"}", Expected::Open)?;
        let count = self.parsed.schema.variants.len().checked_sub(first).expect("variants appended after the first");
        Ok(Declaration { name: name.at, line: name.line, shape: Shape::Enum { first: VariantIx::new(first), count } })
    }

    pub(super) fn parse(mut self) -> Result<Parsed, SchemaError> {
        self.expect(b"family", Expected::Family)?;
        self.parsed.schema.family = self.family()?.at;
        let version = self.number(Wrong::Version, u32::from(u16::MAX))?;
        self.parsed.schema.version = u16::try_from(version).expect("version passed the u16 ceiling");
        for _ in 0..self.tokens.len() {
            if self.peek().is_none() {
                break;
            }
            let token = self.take()?;
            if self.parsed.schema.declarations.len() == MAX_DECLARATIONS {
                return Err(Self::error(token, Wrong::Many));
            }
            let declaration = match self.parsed.schema.spelled(token.at) {
                b"record" => self.record(false)?,
                b"enum" => self.enumeration()?,
                b"versioned" => {
                    self.expect(b"record", Expected::Record)?;
                    self.record(true)?
                }
                _ => return Err(Self::error(token, Wrong::Expected(Expected::Declaration))),
            };
            self.parsed.schema.declarations.push(declaration).expect("declaration count passed its ceiling");
        }
        Ok(self.parsed)
    }
}

fn valid_name(value: &[u8]) -> bool {
    let mut bytes = value.iter();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && *first != b'_' {
        return false;
    }
    for byte in bytes {
        if !byte.is_ascii_alphanumeric() && *byte != b'_' {
            return false;
        }
    }
    true
}

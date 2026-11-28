//! File format first, then move the old members before the settings decoder.
use super::{Decoder, Notices, bound, error, message};
use crate::{
    Limits, Text,
    fields::{
        self, Choice, Shape, Value,
        formats::{Conversion, Destination, Formats, Move},
    },
};
use alloc::boxed::Box;
use skein_json::{Document, Kind, Token, document};
use skein_lib::{List, Map};

pub(super) fn read(d: &Decoder<'_>, formats: Formats) -> Result<u64, Box<[u8]>> {
    let Some(at) = d.doc.member(0, b"format") else { return Ok(1) };
    let format = match d.number(at) {
        Ok(number) if number >= 1 => number,
        Ok(_) | Err(_) => return Err(message(None, error(b"format", b"a whole number from 1"))),
    };
    // A second marker is not a choice of which format to read.
    let mut next = end(&d.doc, at);
    while kind(&d.doc, next) == Kind::Key {
        if text(&d.doc, next) == b"format" {
            return Err(message(None, error(b"format", b"duplicate member")));
        }
        next = end(&d.doc, after(next));
    }
    if format <= formats.newest {
        return Ok(format);
    }
    let mut measure = Text::measure();
    newer(format, formats.newest, &mut measure);
    let mut out = Text::writing_exact(measure);
    newer(format, formats.newest, &mut out);
    Err(out.finish())
}

fn newer(format: u64, newest: u64, out: &mut Text) {
    out.put(b"settings file: format ");
    out.number(format);
    out.put(b"; this temper ctl reads ");
    if newest == 1 {
        out.put(b"format 1");
    } else {
        out.put(b"formats 1 to ");
        out.number(newest);
    }
    out.put(b": upgrade it, or write the file at format ");
    out.number(newest);
}

pub(super) fn migrate(
    doc: Document,
    limits: &Limits,
    file: u64,
    formats: Formats,
) -> Result<(Document, Notices), Box<[u8]>> {
    if file == formats.newest {
        return Ok((doc, List::with_capacity(0)));
    }
    let mut walk = Walk {
        doc,
        stage: &[],
        file,
        newest: formats.newest,
        origins: List::with_capacity(0),
        notices: Map::uncapped(),
    };
    let mut moves = formats.moves;
    while let Some(first) = moves.first() {
        let mut count = 0_usize;
        for movement in moves {
            if movement.format != first.format {
                break;
            }
            count = count.checked_add(1).expect("move table length");
        }
        let (stage, rest) = moves.split_at_checked(count).expect("format group in the move table");
        moves = rest;
        if first.format <= file {
            continue;
        }
        let most = u32::try_from(limits.ask_bytes).unwrap_or(u32::MAX);
        let mut rewrite = Rewrite {
            tokens: List::with_capacity(walk.doc.len()),
            origins: List::with_capacity(walk.doc.len()),
            // A conversion may add tokens, still inside the file's bound.
            most,
        };
        walk.stage = stage;
        walk.value(Node { at: 0, shape: Shape::File, path: b"".as_slice().into(), item: None }, &mut rewrite)?;
        walk.doc = Document::from_tokens(rewrite.tokens.as_slice(), &document::Limits { tokens: most, text: most })
            .or(Err(message(None, bound(b"moved file bytes", most))))?;
        walk.origins = rewrite.origins;
    }
    let mut lines = List::uncapped();
    for (_, moved) in &walk.notices {
        for line in moved {
            lines.push(line.clone()).expect("bounded moves");
        }
    }
    Ok((walk.doc, lines))
}

struct Rewrite {
    tokens: List<Token>,
    // Token indices from the input file, retained across format steps so notices follow the file.
    origins: List<u32>,
    most: u32,
}
impl Rewrite {
    fn push(&mut self, token: Token, origin: u32) -> Result<(), Box<[u8]>> {
        if self.tokens.len() == self.tokens.capacity() {
            if self.tokens.len() >= self.most {
                return Err(message(None, bound(b"moved file tokens", self.most)));
            }
            let capacity = self.tokens.len().saturating_mul(2).min(self.most);
            self.tokens.raise(capacity);
            self.origins.raise(capacity);
        }
        self.tokens.push(token).expect("token room");
        self.origins.push(origin).expect("origin room");
        Ok(())
    }
}

struct Node {
    at: u32,
    shape: Shape,
    path: Box<[u8]>,
    item: Option<u32>,
}

struct Walk {
    doc: Document,
    stage: &'static [Move],
    file: u64,
    newest: u64,
    origins: List<u32>,
    notices: Map<u32, Notices>,
}

impl Walk {
    fn origin(&self, at: u32) -> u32 {
        self.origins.get(at).copied().unwrap_or(at)
    }

    fn value(&mut self, node: Node, out: &mut Rewrite) -> Result<(), Box<[u8]>> {
        if kind(&self.doc, node.at) != Kind::ObjectStart {
            if let Choice::List { item, .. } = fields::form(node.shape).choice
                && kind(&self.doc, node.at) == Kind::ArrayStart
            {
                return self.list(&node, item, false, out);
            }
            return self.copy(node.at, out);
        }
        out.push(Token::ObjectStart, self.origin(node.at))?;
        let mut key = after(node.at);
        while kind(&self.doc, key) == Kind::Key {
            self.member(&node, key, out)?;
            key = end(&self.doc, after(key));
        }
        out.push(Token::ObjectEnd, self.origin(key))
    }

    fn list(&mut self, node: &Node, shape: Shape, items: bool, out: &mut Rewrite) -> Result<(), Box<[u8]>> {
        out.push(Token::ArrayStart, self.origin(node.at))?;
        let mut at = after(node.at);
        let mut index = 0;
        while kind(&self.doc, at) != Kind::ArrayEnd {
            let (path, item) = if items {
                (b"".as_slice().into(), Some(after(index)))
            } else {
                (path(&node.path, b"", Some(index)), node.item)
            };
            self.value(Node { at, shape, path, item }, out)?;
            at = end(&self.doc, at);
            index = after(index);
        }
        out.push(Token::ArrayEnd, self.origin(at))
    }

    fn member(&mut self, node: &Node, key: u32, out: &mut Rewrite) -> Result<(), Box<[u8]>> {
        let at = after(key);
        let old = text(&self.doc, key);
        let origin = self.origin(key);
        let name = if let Some(movement) = movement(self.stage, node.shape, old) {
            let path = path(&node.path, old, None);
            if self.doc.member(node.at, old) != Some(at) {
                return Err(message(node.item, error(&path, b"duplicate member")));
            }
            self.note(origin, notice(node.item, &path, self.file, self.newest, &movement.to));
            match movement.to {
                Destination::Renamed { member } => member,
                Destination::Converted { member, conversion, .. } => {
                    let converted = match convert(conversion, &self.doc, at) {
                        Ok(value) => value,
                        Err(rule) => return Err(message(node.item, error(&path, rule))),
                    };
                    out.push(Token::Key(member.into()), origin)?;
                    for at in 0..converted.len() {
                        out.push(token(&converted, at), origin)?;
                    }
                    return Ok(());
                }
                Destination::MovedOut { .. } | Destination::Gone { .. } => return Ok(()),
            }
        } else {
            old
        };
        out.push(Token::Key(name.into()), origin)?;
        let Some(field) = fields::member(node.shape, name) else { return self.copy(at, out) };
        let (shape, member) = match field.value {
            Value::Shape(shape) => (shape, name),
            Value::Setting => {
                let Some(shape) = setting_shape(&self.doc, node.at) else { return self.copy(at, out) };
                (shape, fields::form(shape).name)
            }
            Value::Integer(_)
            | Value::CharterOwner
            | Value::Text(_)
            | Value::Hex(_)
            | Value::Words(_)
            | Value::Boolean
            | Value::Nanoseconds
            | Value::Time => return self.copy(at, out),
        };
        let child = Node { at, shape, path: path(&node.path, member, None), item: node.item };
        if field.list.is_some() && kind(&self.doc, at) == Kind::ArrayStart {
            self.list(&child, shape, node.shape == Shape::File && shape == Shape::Item, out)
        } else {
            self.value(child, out)
        }
    }

    fn copy(&self, at: u32, out: &mut Rewrite) -> Result<(), Box<[u8]>> {
        for at in at..end(&self.doc, at) {
            out.push(token(&self.doc, at), self.origin(at))?;
        }
        Ok(())
    }

    fn note(&mut self, origin: u32, line: Box<[u8]>) {
        if let Some(lines) = self.notices.get_mut(&origin) {
            lines.push(line).expect("bounded moves per member");
        } else {
            let mut lines = List::uncapped();
            lines.push(line).expect("one notice");
            let previous = self.notices.insert(origin, lines).expect("bounded members");
            assert!(previous.is_none(), "notice was absent");
        }
    }
}

fn setting_shape(doc: &Document, item: u32) -> Option<Shape> {
    let selector = doc.member(item, b"setting")?;
    let kind = doc.member(selector, b"kind")?;
    fields::shape_of_kind(text(doc, kind))
}

#[expect(clippy::manual_find, reason = "step code finds a move without a closure")]
fn movement(stage: &'static [Move], shape: Shape, name: &[u8]) -> Option<&'static Move> {
    for movement in stage {
        if movement.member == name && movement.shape == shape {
            return Some(movement);
        }
    }
    None
}

#[cfg_attr(not(test), expect(unused_variables, reason = "format 1 has no conversions"))]
fn convert(conversion: Conversion, doc: &Document, at: u32) -> Result<Document, &'static [u8]> {
    match conversion {
        #[cfg(test)]
        Conversion::LegacySwitch => super::format_tests::legacy_switch(doc, at),
    }
}

fn notice(item: Option<u32>, path: &[u8], file: u64, newest: u64, to: &Destination) -> Box<[u8]> {
    let mut measure = Text::measure();
    notice_line(item, path, file, newest, to, &mut measure);
    let mut out = Text::writing_exact(measure);
    notice_line(item, path, file, newest, to, &mut out);
    out.finish()
}
fn notice_line(item: Option<u32>, path: &[u8], file: u64, newest: u64, to: &Destination, out: &mut Text) {
    out.put(b"settings file");
    if let Some(item) = item {
        out.put(b" item ");
        out.number(u64::from(item));
    }
    out.put(b": ");
    out.put(path);
    out.put(b": ");
    match to {
        Destination::Renamed { member: words } | Destination::Converted { words, .. } => {
            out.put(b"format ");
            out.number(file);
            out.put(b"; format ");
            out.number(newest);
            out.put(b" writes it as ");
            out.put(words);
        }
        Destination::MovedOut { words } => {
            out.put(b"moved: ");
            out.put(words);
        }
        Destination::Gone { release } => {
            out.put(b"ignored; ");
            out.put(release);
            out.put(b" took it away");
        }
    }
}

fn path(parent: &[u8], member: &[u8], index: Option<u32>) -> Box<[u8]> {
    let mut measure = Text::measure();
    path_line(parent, member, index, &mut measure);
    let mut out = Text::writing_exact(measure);
    path_line(parent, member, index, &mut out);
    out.finish()
}
fn path_line(parent: &[u8], member: &[u8], index: Option<u32>, out: &mut Text) {
    out.put(parent);
    if !parent.is_empty() && !member.is_empty() {
        out.put(b".");
    }
    out.put(member);
    if let Some(index) = index {
        out.put(b"[");
        out.number(u64::from(index));
        out.put(b"]");
    }
}
fn after(at: u32) -> u32 {
    at.checked_add(1).expect("bounded JSON tokens")
}
fn end(doc: &Document, at: u32) -> u32 {
    doc.value_end(at).expect("parsed JSON value")
}
fn kind(doc: &Document, at: u32) -> Kind {
    doc.token(at).expect("parsed JSON token").kind
}
fn text(doc: &Document, at: u32) -> &[u8] {
    doc.text(doc.token(at).expect("parsed JSON token")).expect("parsed JSON text")
}
fn token(doc: &Document, at: u32) -> Token {
    match kind(doc, at) {
        Kind::ObjectStart => Token::ObjectStart,
        Kind::ObjectEnd => Token::ObjectEnd,
        Kind::ArrayStart => Token::ArrayStart,
        Kind::ArrayEnd => Token::ArrayEnd,
        Kind::Key => Token::Key(text(doc, at).into()),
        Kind::String => Token::String(text(doc, at).into()),
        Kind::Number => Token::Number(text(doc, at).into()),
        Kind::True => Token::True,
        Kind::False => Token::False,
        Kind::Null => Token::Null,
        Kind::Long => unreachable!("file admits all its strings"),
    }
}

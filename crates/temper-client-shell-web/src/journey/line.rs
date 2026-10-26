//! The recording's strict byte codec (client/journeys.md, 3.2).
use jig_client_shell_web::adapter::Heard;
use skein_lib::{Time, Wall};
use skein_ui::Act;
use skein_ui_web::{mapping::Primary, transport::Header};

/// One line of a recording, its kind first (client/journeys.md, 3.2).
#[derive(Debug, PartialEq, Eq)]
pub enum Line {
    Journey { version: u32 },
    Seed(u64),
    Clocks { now: Time, wall: Wall },
    Primary(Primary),
    Ways { content: Box<[u8]>, origin: Box<[u8]> },
    Forge(Box<[u8]>),
    Path(Box<[u8]>),
    Secret(Box<[u8]>),
    Stored { key: Box<[u8]>, value: Box<[u8]> },
    Turn { now: Time, wall: Wall },
    Start,
    Due,
    Place(Box<[u8]>),
    Heard(Heard),
    Carried { writes: u32 },
    Written(Box<[u8]>),
    Unwritten(Box<[u8]>),
    Lost(u32),
    Act { act: Act, event: Box<[u8]>, name: Box<[u8]> },
    Exchange { id: u32, bound: u32, method: Box<[u8]>, path: Box<[u8]>, body: Box<[u8]>, headers: Vec<Header> },
    Open { piece: u32, path: Box<[u8]>, headers: Vec<Header> },
    Close,
    Save { key: Box<[u8]>, value: Box<[u8]> },
    Erase(Box<[u8]>),
    Writes(u32),
    Navigate(Box<[u8]>),
    Replace(Box<[u8]>),
    Show { path: Box<[u8]>, replace: bool },
    Deadline(Option<Time>),
    Mark(Option<Box<[u8]>>),
    End(Option<Box<[u8]>>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Unreadable {
    pub line: u32,
    pub why: &'static str,
}

impl Line {
    /// The line's text and its newline.
    pub fn write(&self, out: &mut Vec<u8>) {
        let mut w = Text { out, first: true };
        match self {
            Self::Journey { version } => {
                w.word(b"journey");
                w.number(u64::from(*version));
            }
            Self::Seed(seed) => {
                w.word(b"seed");
                w.number(*seed);
            }
            Self::Clocks { now, wall } | Self::Turn { now, wall } => {
                w.word(if matches!(self, Self::Clocks { .. }) { b"clocks" } else { b"turn" });
                w.number(now.as_nanos());
                w.number(wall.as_nanos());
            }
            Self::Primary(primary) => {
                w.word(b"primary");
                w.word(match primary {
                    Primary::Control => b"control",
                    Primary::Meta => b"meta",
                });
            }
            Self::Ways { content, origin } => {
                w.word(b"ways");
                w.bytes(content);
                w.bytes(origin);
            }
            Self::Forge(bytes)
            | Self::Path(bytes)
            | Self::Secret(bytes)
            | Self::Place(bytes)
            | Self::Written(bytes)
            | Self::Unwritten(bytes)
            | Self::Erase(bytes)
            | Self::Navigate(bytes)
            | Self::Replace(bytes) => {
                w.word(self.kind());
                w.bytes(bytes);
            }
            Self::Stored { key, value } | Self::Save { key, value } => {
                w.word(self.kind());
                w.bytes(key);
                w.bytes(value);
            }
            Self::Start | Self::Due | Self::Close => w.word(self.kind()),
            Self::Heard(heard) => {
                w.word(b"heard");
                w.heard(heard);
            }
            Self::Carried { writes } | Self::Writes(writes) => {
                w.word(self.kind());
                w.number(u64::from(*writes));
            }
            Self::Lost(exchange) => {
                w.word(b"lost");
                w.number(u64::from(*exchange));
            }
            Self::Act { act, event, name } => {
                w.word(b"act");
                w.act(act);
                w.bytes(event);
                w.bytes(name);
            }
            Self::Exchange { id, bound, method, path, body, headers } => {
                w.word(b"exchange");
                w.number(u64::from(*id));
                w.number(u64::from(*bound));
                w.word(method);
                w.bytes(path);
                w.bytes(body);
                w.headers(headers);
            }
            Self::Open { piece, path, headers } => {
                w.word(b"open");
                w.number(u64::from(*piece));
                w.bytes(path);
                w.headers(headers);
            }
            Self::Show { path, replace } => {
                w.word(b"show");
                w.bytes(path);
                w.word(if *replace { b"replace" } else { b"push" });
            }
            Self::Deadline(deadline) => {
                w.word(b"deadline");
                if let Some(time) = deadline {
                    w.number(time.as_nanos());
                } else {
                    w.word(b"none");
                }
            }
            Self::Mark(html) | Self::End(html) => {
                w.word(self.kind());
                if let Some(html) = html {
                    w.bytes(html);
                }
            }
        }
        w.out.push(b'\n');
    }
    fn kind(&self) -> &'static [u8] {
        match self {
            Self::Journey { .. } => b"journey",
            Self::Seed(_) => b"seed",
            Self::Clocks { .. } => b"clocks",
            Self::Primary(_) => b"primary",
            Self::Ways { .. } => b"ways",
            Self::Forge(_) => b"forge",
            Self::Path(_) => b"path",
            Self::Secret(_) => b"secret",
            Self::Stored { .. } => b"stored",
            Self::Turn { .. } => b"turn",
            Self::Start => b"start",
            Self::Due => b"due",
            Self::Place(_) => b"place",
            Self::Heard(_) => b"heard",
            Self::Carried { .. } => b"carried",
            Self::Written(_) => b"written",
            Self::Unwritten(_) => b"unwritten",
            Self::Lost(_) => b"lost",
            Self::Act { .. } => b"act",
            Self::Exchange { .. } => b"exchange",
            Self::Open { .. } => b"open",
            Self::Close => b"close",
            Self::Save { .. } => b"save",
            Self::Erase(_) => b"erase",
            Self::Writes(_) => b"writes",
            Self::Navigate(_) => b"navigate",
            Self::Replace(_) => b"replace",
            Self::Show { .. } => b"show",
            Self::Deadline(_) => b"deadline",
            Self::Mark(_) => b"mark",
            Self::End(_) => b"end",
        }
    }
    /// One line, without its newline.
    ///
    /// # Errors
    /// Refuses a line outside the recording grammar.
    pub fn read(line: &[u8]) -> Result<Self, Unreadable> {
        read(line).map_err(|why| Unreadable { line: 1, why })
    }
}

/// Read a whole recording; errors number lines from 1.
///
/// # Errors
/// Refuses malformed lines and a missing final newline.
pub fn lines(recording: &[u8]) -> Result<Vec<Line>, Unreadable> {
    let mut result = Vec::new();
    for (index, part) in recording.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let number = u32::try_from(index + 1).expect("recording line count fits u32");
        let Some(line) = part.strip_suffix(b"\n") else {
            return Err(Unreadable { line: number, why: "missing final newline" });
        };
        let record = Line::read(line).map_err(|error| Unreadable { line: number, ..error })?;
        if result.is_empty() && matches!(record, Line::Journey { version } if version != 2) {
            return Err(Unreadable { line: number, why: "expected ordered start facts for journey 2" });
        }
        result.push(record);
    }
    if result.is_empty() {
        return Err(Unreadable { line: 1, why: "empty recording" });
    }
    Ok(result)
}

struct Text<'a> {
    out: &'a mut Vec<u8>,
    first: bool,
}
impl Text<'_> {
    fn heard(&mut self, heard: &Heard) {
        match heard {
            Heard::Answered { exchange, status, body } => {
                self.word(b"answered");
                self.number(u64::from(*exchange));
                self.number(u64::from(*status));
                self.bytes(body);
            }
            Heard::Failed { exchange } => {
                self.word(b"failed");
                self.number(u64::from(*exchange));
            }
            Heard::StreamOpened => self.word(b"opened"),
            Heard::StreamRefused { status } => {
                self.word(b"refused");
                self.number(u64::from(*status));
            }
            Heard::Piece { bytes } => {
                self.word(b"piece");
                self.bytes(bytes);
            }
            Heard::StreamEnded => self.word(b"ended"),
            Heard::Stored { .. } | Heard::NotStored { .. } => {
                panic!("storage reports are recorded as carried reports")
            }
        }
    }
    fn act(&mut self, act: &Act) {
        match act {
            Act::Press => self.word(b"press"),
            Act::Submit => self.word(b"submit"),
            Act::Toggle => self.word(b"toggle"),
            Act::Input { value } => {
                self.word(b"input");
                self.bytes(value);
            }
            Act::Drawing { data } => {
                self.word(b"drawing");
                self.bytes(data);
            }
        }
    }
    fn space(&mut self) {
        if !self.first {
            self.out.push(b' ');
        }
        self.first = false;
    }
    fn word(&mut self, word: &[u8]) {
        self.space();
        self.out.extend_from_slice(word);
    }
    fn number(&mut self, number: u64) {
        self.word(number.to_string().as_bytes());
    }
    fn bytes(&mut self, bytes: &[u8]) {
        const HEX: &[u8] = b"0123456789abcdef";
        self.space();
        self.out.push(b'"');
        for &byte in bytes {
            match byte {
                b'"' | b'\\' => {
                    self.out.push(b'\\');
                    self.out.push(byte);
                }
                0x20..=0x7e => self.out.push(byte),
                _ => {
                    self.out.extend_from_slice(&[
                        b'\\',
                        b'x',
                        HEX[usize::from(byte >> 4)],
                        HEX[usize::from(byte & 15)],
                    ]);
                }
            }
        }
        self.out.push(b'"');
    }
    fn headers(&mut self, headers: &[Header]) {
        for (name, value) in headers {
            self.bytes(name);
            self.bytes(value);
        }
    }
}

#[derive(Debug)]
enum Field {
    Word(Box<[u8]>),
    Bytes(Box<[u8]>),
}
struct Read {
    fields: std::vec::IntoIter<Field>,
}
impl Read {
    fn word(&mut self) -> Result<Box<[u8]>, &'static str> {
        match self.fields.next() {
            Some(Field::Word(word)) => Ok(word),
            _ => Err("expected word"),
        }
    }
    fn bytes(&mut self) -> Result<Box<[u8]>, &'static str> {
        match self.fields.next() {
            Some(Field::Bytes(bytes)) => Ok(bytes),
            _ => Err("expected quoted bytes"),
        }
    }
    fn number(&mut self) -> Result<u64, &'static str> {
        let word = self.word()?;
        if word.is_empty() || word.len() > 1 && word[0] == b'0' || !word.iter().all(u8::is_ascii_digit) {
            return Err("invalid number");
        }
        word.iter()
            .try_fold(0_u64, |value, digit| value.checked_mul(10)?.checked_add(u64::from(digit - b'0')))
            .ok_or("number overflow")
    }
    fn count(&mut self) -> Result<u32, &'static str> {
        u32::try_from(self.number()?).map_err(|_| "number overflow")
    }
    fn status(&mut self) -> Result<u16, &'static str> {
        u16::try_from(self.number()?).map_err(|_| "number overflow")
    }
    fn headers(&mut self) -> Result<Vec<Header>, &'static str> {
        let mut result = Vec::new();
        while self.fields.len() != 0 {
            result.push((self.bytes()?, self.bytes()?));
        }
        Ok(result)
    }
}
fn fields(mut line: &[u8]) -> Result<Read, &'static str> {
    let mut fields = Vec::new();
    while !line.is_empty() {
        if line[0] == b'"' {
            line = &line[1..];
            let mut bytes = Vec::new();
            loop {
                let Some((&byte, rest)) = line.split_first() else {
                    return Err("unclosed quote");
                };
                line = rest;
                match byte {
                    b'"' => break,
                    b'\\' => {
                        let Some((&escape, rest)) = line.split_first() else {
                            return Err("bad escape");
                        };
                        line = rest;
                        match escape {
                            b'"' | b'\\' => bytes.push(escape),
                            b'x' => {
                                if line.len() < 2 {
                                    return Err("bad escape");
                                }
                                let byte = hex(line[0])?.checked_mul(16).expect("hex nibble") + hex(line[1])?;
                                if (0x20..=0x7e).contains(&byte) {
                                    return Err("escaped printable byte");
                                }
                                bytes.push(byte);
                                line = &line[2..];
                            }
                            _ => return Err("bad escape"),
                        }
                    }
                    0x20..=0x7e => bytes.push(byte),
                    _ => return Err("unescaped byte"),
                }
            }
            fields.push(Field::Bytes(bytes.into()));
        } else {
            let end = line.iter().position(|byte| *byte == b' ').unwrap_or(line.len());
            let word = &line[..end];
            if word.is_empty()
                || !word.iter().all(|byte| (0x21..=0x7e).contains(byte) && *byte != b'"' && *byte != b'\\')
            {
                return Err("invalid word");
            }
            fields.push(Field::Word(word.into()));
            line = &line[end..];
        }
        if line.is_empty() {
            break;
        }
        if line[0] != b' ' {
            return Err("expected space");
        }
        line = &line[1..];
        if line.is_empty() {
            return Err("trailing space");
        }
    }
    Ok(Read { fields: fields.into_iter() })
}
fn hex(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err("bad hex digit"),
    }
}
fn read(line: &[u8]) -> Result<Line, &'static str> {
    let mut r = fields(line)?;
    let line = match r.word()?.as_ref() {
        b"journey" => Line::Journey { version: r.count()? },
        b"seed" => Line::Seed(r.number()?),
        b"clocks" => Line::Clocks { now: Time::from_nanos(r.number()?), wall: Wall::from_nanos(r.number()?) },
        b"turn" => Line::Turn { now: Time::from_nanos(r.number()?), wall: Wall::from_nanos(r.number()?) },
        b"primary" => Line::Primary(match r.word()?.as_ref() {
            b"control" => Primary::Control,
            b"meta" => Primary::Meta,
            _ => return Err("unknown primary"),
        }),
        b"ways" => Line::Ways { content: r.bytes()?, origin: r.bytes()? },
        b"forge" => Line::Forge(r.bytes()?),
        b"path" => Line::Path(r.bytes()?),
        b"secret" => Line::Secret(r.bytes()?),
        b"stored" => Line::Stored { key: r.bytes()?, value: r.bytes()? },
        b"start" => Line::Start,
        b"due" => Line::Due,
        b"place" => Line::Place(r.bytes()?),
        b"heard" => Line::Heard(match r.word()?.as_ref() {
            b"answered" => Heard::Answered { exchange: r.count()?, status: r.status()?, body: r.bytes()? },
            b"failed" => Heard::Failed { exchange: r.count()? },
            b"opened" => Heard::StreamOpened,
            b"refused" => Heard::StreamRefused { status: r.status()? },
            b"piece" => Heard::Piece { bytes: r.bytes()? },
            b"ended" => Heard::StreamEnded,
            _ => return Err("unknown heard"),
        }),
        b"carried" => Line::Carried { writes: r.count()? },
        b"written" => Line::Written(r.bytes()?),
        b"unwritten" => Line::Unwritten(r.bytes()?),
        b"lost" => Line::Lost(r.count()?),
        b"act" => {
            let act = match r.word()?.as_ref() {
                b"press" => Act::Press,
                b"submit" => Act::Submit,
                b"toggle" => Act::Toggle,
                b"input" => Act::Input { value: r.bytes()? },
                b"drawing" => Act::Drawing { data: r.bytes()? },
                _ => return Err("unknown act"),
            };
            Line::Act { act, event: r.bytes()?, name: r.bytes()? }
        }
        b"exchange" => {
            let id = r.count()?;
            let bound = r.count()?;
            let method = r.word()?;
            if !matches!(method.as_ref(), b"GET" | b"HEAD" | b"POST" | b"PUT" | b"PATCH" | b"DELETE" | b"OPTIONS") {
                return Err("unknown method");
            }
            Line::Exchange { id, bound, method, path: r.bytes()?, body: r.bytes()?, headers: r.headers()? }
        }
        b"open" => Line::Open { piece: r.count()?, path: r.bytes()?, headers: r.headers()? },
        b"close" => Line::Close,
        b"save" => Line::Save { key: r.bytes()?, value: r.bytes()? },
        b"erase" => Line::Erase(r.bytes()?),
        b"writes" => Line::Writes(r.count()?),
        b"navigate" => Line::Navigate(r.bytes()?),
        b"replace" => Line::Replace(r.bytes()?),
        b"show" => Line::Show {
            path: r.bytes()?,
            replace: match r.word()?.as_ref() {
                b"push" => false,
                b"replace" => true,
                _ => return Err("unknown history mode"),
            },
        },
        b"deadline" => {
            if matches!(r.fields.as_slice(), [Field::Word(word)] if word.as_ref() == b"none") {
                r.word()?;
                Line::Deadline(None)
            } else {
                Line::Deadline(Some(Time::from_nanos(r.number()?)))
            }
        }
        b"mark" => Line::Mark(if r.fields.len() == 0 { None } else { Some(r.bytes()?) }),
        b"end" => Line::End(if r.fields.len() == 0 { None } else { Some(r.bytes()?) }),
        _ => return Err("unknown kind"),
    };
    if r.fields.len() != 0 {
        return Err("extra field");
    }
    Ok(line)
}

// Act and Heard own bytes but do not implement Clone upstream.
impl Clone for Line {
    fn clone(&self) -> Self {
        match self {
            Self::Journey { version } => Self::Journey { version: *version },
            Self::Seed(value) => Self::Seed(*value),
            Self::Clocks { now, wall } => Self::Clocks { now: *now, wall: *wall },
            Self::Primary(value) => Self::Primary(*value),
            Self::Ways { content, origin } => Self::Ways { content: content.clone(), origin: origin.clone() },
            Self::Forge(value) => Self::Forge(value.clone()),
            Self::Path(value) => Self::Path(value.clone()),
            Self::Secret(value) => Self::Secret(value.clone()),
            Self::Stored { key, value } => Self::Stored { key: key.clone(), value: value.clone() },
            Self::Turn { now, wall } => Self::Turn { now: *now, wall: *wall },
            Self::Start => Self::Start,
            Self::Due => Self::Due,
            Self::Place(value) => Self::Place(value.clone()),
            Self::Heard(value) => Self::Heard(clone_heard(value)),
            Self::Carried { writes } => Self::Carried { writes: *writes },
            Self::Written(value) => Self::Written(value.clone()),
            Self::Unwritten(value) => Self::Unwritten(value.clone()),
            Self::Lost(value) => Self::Lost(*value),
            Self::Act { act, event, name } => {
                Self::Act { act: clone_act(act), event: event.clone(), name: name.clone() }
            }
            Self::Exchange { id, bound, method, path, body, headers } => Self::Exchange {
                id: *id,
                bound: *bound,
                method: method.clone(),
                path: path.clone(),
                body: body.clone(),
                headers: headers.clone(),
            },
            Self::Open { piece, path, headers } => {
                Self::Open { piece: *piece, path: path.clone(), headers: headers.clone() }
            }
            Self::Close => Self::Close,
            Self::Save { key, value } => Self::Save { key: key.clone(), value: value.clone() },
            Self::Erase(value) => Self::Erase(value.clone()),
            Self::Writes(value) => Self::Writes(*value),
            Self::Navigate(value) => Self::Navigate(value.clone()),
            Self::Replace(value) => Self::Replace(value.clone()),
            Self::Show { path, replace } => Self::Show { path: path.clone(), replace: *replace },
            Self::Deadline(value) => Self::Deadline(*value),
            Self::Mark(value) => Self::Mark(value.clone()),
            Self::End(value) => Self::End(value.clone()),
        }
    }
}
pub(super) fn clone_act(act: &Act) -> Act {
    match act {
        Act::Press => Act::Press,
        Act::Submit => Act::Submit,
        Act::Toggle => Act::Toggle,
        Act::Input { value } => Act::Input { value: value.clone() },
        Act::Drawing { data } => Act::Drawing { data: data.clone() },
    }
}
pub(super) fn clone_heard(heard: &Heard) -> Heard {
    match heard {
        Heard::Answered { exchange, status, body } => {
            Heard::Answered { exchange: *exchange, status: *status, body: body.clone() }
        }
        Heard::Failed { exchange } => Heard::Failed { exchange: *exchange },
        Heard::StreamOpened => Heard::StreamOpened,
        Heard::StreamRefused { status } => Heard::StreamRefused { status: *status },
        Heard::Piece { bytes } => Heard::Piece { bytes: bytes.clone() },
        Heard::StreamEnded => Heard::StreamEnded,
        Heard::Stored { key } => Heard::Stored { key: key.clone() },
        Heard::NotStored { key } => Heard::NotStored { key: key.clone() },
    }
}

//! The strict byte codec (temper/docs/design/client/tui/journeys.md, 3.2 and 3.3;
//! temper/docs/design/client/web/journeys.md, 3.2).
use skein_lib::{Time, Wall};
use skein_ui_screen::{Rgb, Size};
use skein_ui_screen_terminal::Key;
use temper_client_tui::{Note, Scheme};

#[derive(Clone, PartialEq, Eq)]
pub enum Line {
    Journey { version: u32 },
    Seed(u64),
    Clocks { now: Time, wall: Wall },
    Screen { size: Size, scheme: Option<Scheme> },
    Background(Rgb),
    Address(Box<[u8]>),
    Local,
    Front,
    Box,
    Stored { key: Box<[u8]>, bytes: Box<[u8]> },
    Credential(Box<[u8]>),
    Noted(Note),
    Turn { now: Time, wall: Wall },
    Start,
    Due,
    Key { key: Key, event: Option<Box<[u8]>> },
    Resize(Size),
    Reply { exchange: u64, bytes: Box<[u8]> },
    Unanswered(u64),
    Streamed(Streamed),
    Gone,
    Saved,
    Unsaved,
    Kept { written: bool },
    Opened { opened: bool },
    Found(Option<Box<[u8]>>),
    BoxLine(Box<[u8]>),
    BoxStage(Box<[u8]>),
    StopBox,
    StopBoxAgain,
    Send { exchange: u64, message: Sent, credential: Option<Box<[u8]>> },
    Stream(Option<Box<[u8]>>),
    Unstream,
    Save { key: Box<[u8]>, bytes: Box<[u8]> },
    Erase(Box<[u8]>),
    Keep(Option<Box<[u8]>>),
    Note(Note),
    Signed(Box<[u8]>),
    Read,
    Browse(Box<[u8]>),
    Quit,
    Deadline(Option<Time>),
    Mark(Option<Box<[u8]>>),
    End(Option<Box<[u8]>>),
}

#[derive(Clone, PartialEq, Eq)]
pub enum Streamed {
    Record(Box<[u8]>),
    Heartbeat,
    Full { transport: bool, size: u32 },
    SignedOut,
}

/// A message as sent; the two bodyless kinds always write empty bytes.
#[derive(Clone, PartialEq, Eq)]
pub enum Sent {
    Authorize(Box<[u8]>),
    ApprovalAsk(Box<[u8]>),
    ApprovalRead(Box<[u8]>),
    ApprovalDecide(Box<[u8]>),
    SignInApproval(Box<[u8]>),
    Request(Box<[u8]>),
    Read(Box<[u8]>),
    Watch(Box<[u8]>),
    Unwatch(Box<[u8]>),
    StreamGone(Box<[u8]>),
    SignInLink(Box<[u8]>),
    SignInService,
    SignOut,
}

impl Sent {
    #[must_use]
    pub fn of(message: &jig_client::Message) -> Self {
        use jig_client::Message;
        match message {
            Message::Authorize(bytes) => Self::Authorize(bytes.clone()),
            Message::ApprovalAsk(bytes) => Self::ApprovalAsk(bytes.clone()),
            Message::ApprovalRead(bytes) => Self::ApprovalRead(bytes.clone()),
            Message::ApprovalDecide(bytes) => Self::ApprovalDecide(bytes.clone()),
            Message::SignInApproval(bytes) => Self::SignInApproval(bytes.clone()),
            Message::Request(bytes) => Self::Request(bytes.clone()),
            Message::Read(bytes) => Self::Read(bytes.clone()),
            Message::Watch(bytes) => Self::Watch(bytes.clone()),
            Message::Unwatch(bytes) => Self::Unwatch(bytes.clone()),
            Message::StreamGone(bytes) => Self::StreamGone(bytes.clone()),
            Message::SignInLink(bytes) => Self::SignInLink(bytes.clone()),
            Message::SignInService => Self::SignInService,
            Message::SignOut => Self::SignOut,
        }
    }

    fn fields(&self) -> (&'static [u8], &[u8]) {
        match self {
            Self::Authorize(bytes) => (b"authorize", bytes),
            Self::ApprovalAsk(bytes) => (b"approval-ask", bytes),
            Self::ApprovalRead(bytes) => (b"approval-read", bytes),
            Self::ApprovalDecide(bytes) => (b"approval-decide", bytes),
            Self::SignInApproval(bytes) => (b"sign-in-approval", bytes),
            Self::Request(bytes) => (b"request", bytes),
            Self::Read(bytes) => (b"read", bytes),
            Self::Watch(bytes) => (b"watch", bytes),
            Self::Unwatch(bytes) => (b"unwatch", bytes),
            Self::StreamGone(bytes) => (b"stream-gone", bytes),
            Self::SignInLink(bytes) => (b"sign-in-link", bytes),
            Self::SignInService => (b"sign-in-service", b""),
            Self::SignOut => (b"sign-out", b""),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Unreadable {
    pub line: u32,
    pub why: &'static str,
}

impl Line {
    /// Append this record and its newline.
    pub fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.kind().as_bytes());
        let mut w = Text { out };
        match self {
            Self::Journey { version } => w.number(u64::from(*version)),
            Self::Seed(seed) | Self::Unanswered(seed) => w.number(*seed),
            Self::Clocks { now, wall } | Self::Turn { now, wall } => {
                w.number(now.as_nanos());
                w.number(wall.as_nanos());
            }
            Self::Screen { size, scheme } => {
                w.size(*size);
                w.word(match scheme {
                    Some(Scheme::Dark) => b"dark",
                    Some(Scheme::Light) => b"light",
                    None => b"none",
                });
            }
            Self::Background(rgb) => {
                w.number(u64::from(rgb.red));
                w.number(u64::from(rgb.green));
                w.number(u64::from(rgb.blue));
            }
            Self::BoxLine(bytes)
            | Self::BoxStage(bytes)
            | Self::Address(bytes)
            | Self::Credential(bytes)
            | Self::Erase(bytes)
            | Self::Signed(bytes)
            | Self::Browse(bytes) => w.bytes(bytes),
            Self::Stored { key, bytes } | Self::Save { key, bytes } => {
                w.bytes(key);
                w.bytes(bytes);
            }
            Self::Front
            | Self::Local
            | Self::Box
            | Self::Read
            | Self::Start
            | Self::Due
            | Self::Gone
            | Self::Saved
            | Self::Unsaved
            | Self::Unstream
            | Self::Quit
            | Self::StopBox
            | Self::StopBoxAgain => {}
            Self::Key { key, event } => {
                w.key(key);
                w.optional(event.as_deref());
            }
            Self::Resize(size) => w.size(*size),
            Self::Reply { exchange, bytes } => {
                w.number(*exchange);
                w.bytes(bytes);
            }
            Self::Streamed(streamed) => match streamed {
                Streamed::Record(bytes) => {
                    w.word(b"record");
                    w.bytes(bytes);
                }
                Streamed::Heartbeat => w.word(b"heartbeat"),
                Streamed::Full { transport, size } => {
                    w.word(b"full");
                    w.word(if *transport { b"transport" } else { b"component" });
                    w.number(u64::from(*size));
                }
                Streamed::SignedOut => w.word(b"signed-out"),
            },
            Self::Kept { written } | Self::Opened { opened: written } => {
                w.word(if *written { b"done" } else { b"failed" });
            }
            Self::Send { exchange, message, credential } => {
                w.number(*exchange);
                let (kind, bytes) = message.fields();
                w.word(kind);
                w.bytes(bytes);
                w.optional(credential.as_deref());
            }
            Self::Noted(note) | Self::Note(note) => w.note(*note),
            Self::Found(bytes) | Self::Stream(bytes) | Self::Keep(bytes) => w.optional(bytes.as_deref()),
            Self::Deadline(deadline) => {
                if let Some(time) = deadline {
                    w.number(time.as_nanos());
                } else {
                    w.word(b"none");
                }
            }
            Self::Mark(print) | Self::End(print) => {
                if let Some(print) = print {
                    w.bytes(print);
                }
            }
        }
        w.out.push(b'\n');
    }

    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Journey { .. } => "journey",
            Self::Seed(_) => "seed",
            Self::Clocks { .. } => "clocks",
            Self::Screen { .. } => "screen",
            Self::Address(_) => "address",
            Self::Local => "local",
            Self::Front => "front",
            Self::BoxLine(_) => "box-line",
            Self::BoxStage(_) => "box-stage",
            Self::StopBox => "stop-box",
            Self::StopBoxAgain => "stop-box-again",
            Self::Box => "box",
            Self::Background(_) => "background",
            Self::Noted(_) => "noted",
            Self::Found(_) => "found",
            Self::Note(_) => "note",
            Self::Read => "read",
            Self::Stored { .. } => "stored",
            Self::Credential(_) => "credential",
            Self::Turn { .. } => "turn",
            Self::Start => "start",
            Self::Due => "due",
            Self::Key { .. } => "key",
            Self::Resize(_) => "resize",
            Self::Reply { .. } => "reply",
            Self::Unanswered(_) => "unanswered",
            Self::Streamed(_) => "streamed",
            Self::Gone => "gone",
            Self::Saved => "saved",
            Self::Unsaved => "unsaved",
            Self::Kept { .. } => "kept",
            Self::Opened { .. } => "opened",
            Self::Send { .. } => "send",
            Self::Stream(_) => "stream",
            Self::Unstream => "unstream",
            Self::Save { .. } => "save",
            Self::Erase(_) => "erase",
            Self::Keep(_) => "keep",
            Self::Signed(_) => "signed",
            Self::Browse(_) => "browse",
            Self::Quit => "quit",
            Self::Deadline(_) => "deadline",
            Self::Mark(_) => "mark",
            Self::End(_) => "end",
        }
    }
}

/// Read records, numbering errors from one. Ordering belongs to replay.
pub fn lines(recording: &[u8]) -> Result<Vec<Line>, Unreadable> {
    let mut result = Vec::new();
    let mut number = 0_u32;
    for part in recording.split_inclusive(|byte| *byte == b'\n') {
        number = number.checked_add(1).ok_or(Unreadable { line: u32::MAX, why: "too many lines" })?;
        let line = part.strip_suffix(b"\n").ok_or(Unreadable { line: number, why: "missing final newline" })?;
        result.push(read(line).map_err(|why| Unreadable { line: number, why })?);
    }
    if result.is_empty() {
        return Err(Unreadable { line: 1, why: "empty recording" });
    }
    Ok(result)
}

struct Text<'a> {
    out: &'a mut Vec<u8>,
}
impl Text<'_> {
    fn word(&mut self, word: &[u8]) {
        self.out.push(b' ');
        self.out.extend_from_slice(word);
    }
    fn number(&mut self, number: u64) {
        self.word(number.to_string().as_bytes());
    }
    fn bytes(&mut self, bytes: &[u8]) {
        const HEX: &[u8] = b"0123456789ABCDEF";
        self.out.extend_from_slice(b" \"");
        for &byte in bytes {
            match byte {
                b'"' | b'\\' => self.out.extend_from_slice(&[b'\\', byte]),
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
    fn optional(&mut self, bytes: Option<&[u8]>) {
        if let Some(bytes) = bytes {
            self.bytes(bytes);
        } else {
            self.word(b"none");
        }
    }
    fn note(&mut self, note: Note) {
        match note {
            Note::Until(until) => {
                self.word(b"until");
                self.number(until.as_nanos());
            }
            Note::SignedOut => self.word(b"signed-out"),
        }
    }
    fn size(&mut self, size: Size) {
        self.number(u64::from(size.rows));
        self.number(u64::from(size.columns));
    }
    fn key(&mut self, key: &Key) {
        match key {
            Key::Char(ch) | Key::Alt(ch) => {
                self.word(if matches!(key, Key::Char(_)) { b"char" } else { b"alt" });
                self.bytes(ch.encode_utf8(&mut [0; 4]).as_bytes());
            }
            Key::Paste(bytes) => {
                self.word(b"paste");
                self.bytes(bytes);
            }
            Key::Control(letter) => {
                self.word(b"control");
                self.bytes(&[*letter]);
            }
            Key::Enter => self.word(b"enter"),
            Key::Tab => self.word(b"tab"),
            Key::BackTab => self.word(b"backtab"),
            Key::Backspace => self.word(b"backspace"),
            Key::Delete => self.word(b"delete"),
            Key::Up => self.word(b"up"),
            Key::Down => self.word(b"down"),
            Key::Left => self.word(b"left"),
            Key::Right => self.word(b"right"),
            Key::Home => self.word(b"home"),
            Key::End => self.word(b"end"),
            Key::PageUp => self.word(b"pageup"),
            Key::PageDown => self.word(b"pagedown"),
            Key::Escape => self.word(b"escape"),
        }
    }
}

struct Read<'a> {
    rest: &'a [u8],
}
impl Read<'_> {
    fn word(&mut self) -> Result<&[u8], &'static str> {
        self.space()?;
        let end = self.rest.iter().position(|byte| *byte == b' ').unwrap_or(self.rest.len());
        let (word, rest) = self.rest.split_at(end);
        if word.is_empty()
            || !word.iter().all(|byte| byte.is_ascii_lowercase() || *byte == b'-' || byte.is_ascii_digit())
        {
            return Err("invalid word");
        }
        self.rest = rest;
        Ok(word)
    }
    fn space(&mut self) -> Result<(), &'static str> {
        self.rest = self.rest.strip_prefix(b" ").ok_or("expected field")?;
        Ok(())
    }
    fn number(&mut self) -> Result<u64, &'static str> {
        let word = self.word()?;
        if word.len() > 1 && word.first() == Some(&b'0') || !word.iter().all(u8::is_ascii_digit) {
            return Err("invalid number");
        }
        word.iter()
            .try_fold(0_u64, |value, digit| value.checked_mul(10)?.checked_add(u64::from(digit - b'0')))
            .ok_or("number overflow")
    }
    fn count(&mut self) -> Result<u32, &'static str> {
        u32::try_from(self.number()?).map_err(|_| "number overflow")
    }
    fn size(&mut self) -> Result<Size, &'static str> {
        Ok(Size {
            rows: u16::try_from(self.number()?).map_err(|_| "number overflow")?,
            columns: u16::try_from(self.number()?).map_err(|_| "number overflow")?,
        })
    }
    fn bytes(&mut self) -> Result<Box<[u8]>, &'static str> {
        self.space()?;
        self.rest = self.rest.strip_prefix(b"\"").ok_or("expected quoted bytes")?;
        let mut bytes = Vec::new();
        loop {
            let byte = self.byte().map_err(|_| "unclosed quote")?;
            match byte {
                b'"' => return Ok(bytes.into_boxed_slice()),
                b'\\' => match self.byte()? {
                    b'"' => bytes.push(b'"'),
                    b'\\' => bytes.push(b'\\'),
                    b'x' => {
                        let high = hex(self.byte()?)?;
                        let low = hex(self.byte()?)?;
                        let byte = (high << 4) | low;
                        if (0x20..=0x7e).contains(&byte) {
                            return Err("escaped printable byte");
                        }
                        bytes.push(byte);
                    }
                    _ => return Err("bad escape"),
                },
                0x20..=0x7e => bytes.push(byte),
                _ => return Err("unescaped byte"),
            }
        }
    }
    fn byte(&mut self) -> Result<u8, &'static str> {
        let (&byte, rest) = self.rest.split_first().ok_or("incomplete escape")?;
        self.rest = rest;
        Ok(byte)
    }
    fn optional(&mut self) -> Result<Option<Box<[u8]>>, &'static str> {
        if self.rest.starts_with(b" \"") {
            Ok(Some(self.bytes()?))
        } else if self.word()? == b"none" {
            Ok(None)
        } else {
            Err("expected bytes or none")
        }
    }
    fn note(&mut self) -> Result<Note, &'static str> {
        match self.word()? {
            b"until" => Ok(Note::Until(Wall::from_nanos(self.number()?))),
            b"signed-out" => Ok(Note::SignedOut),
            _ => Err("unknown note"),
        }
    }
    fn done(&mut self) -> Result<bool, &'static str> {
        match self.word()? {
            b"done" => Ok(true),
            b"failed" => Ok(false),
            _ => Err("unknown outcome"),
        }
    }
    fn character(&mut self) -> Result<char, &'static str> {
        let bytes = self.bytes()?;
        let text = std::str::from_utf8(&bytes).map_err(|_| "invalid character")?;
        let mut chars = text.chars();
        let ch = chars.next().ok_or("expected one character")?;
        if chars.next().is_some() {
            return Err("expected one character");
        }
        Ok(ch)
    }
    fn key(&mut self) -> Result<Key, &'static str> {
        Ok(match self.word()? {
            b"char" => Key::Char(self.character()?),
            b"alt" => Key::Alt(self.character()?),
            b"paste" => Key::Paste(self.bytes()?),
            b"control" => {
                let bytes = self.bytes()?;
                let [letter] = bytes.as_ref() else {
                    return Err("expected ASCII letter");
                };
                if !letter.is_ascii_alphabetic() {
                    return Err("expected ASCII letter");
                }
                Key::Control(*letter)
            }
            b"enter" => Key::Enter,
            b"tab" => Key::Tab,
            b"backtab" => Key::BackTab,
            b"backspace" => Key::Backspace,
            b"delete" => Key::Delete,
            b"up" => Key::Up,
            b"down" => Key::Down,
            b"left" => Key::Left,
            b"right" => Key::Right,
            b"home" => Key::Home,
            b"end" => Key::End,
            b"pageup" => Key::PageUp,
            b"pagedown" => Key::PageDown,
            b"escape" => Key::Escape,
            _ => return Err("unknown key"),
        })
    }
    fn streamed(&mut self) -> Result<Streamed, &'static str> {
        Ok(match self.word()? {
            b"record" => Streamed::Record(self.bytes()?),
            b"heartbeat" => Streamed::Heartbeat,
            b"full" => Streamed::Full {
                transport: match self.word()? {
                    b"transport" => true,
                    b"component" => false,
                    _ => return Err("unknown full kind"),
                },
                size: self.count()?,
            },
            b"signed-out" => Streamed::SignedOut,
            _ => return Err("unknown streamed kind"),
        })
    }
    fn sent(&mut self) -> Result<Sent, &'static str> {
        // The reader borrows its source only while choosing the body constructor.
        let make = match self.word()? {
            b"authorize" => Sent::Authorize,
            b"approval-ask" => Sent::ApprovalAsk,
            b"approval-read" => Sent::ApprovalRead,
            b"approval-decide" => Sent::ApprovalDecide,
            b"sign-in-approval" => Sent::SignInApproval,
            b"request" => Sent::Request,
            b"read" => Sent::Read,
            b"watch" => Sent::Watch,
            b"unwatch" => Sent::Unwatch,
            b"stream-gone" => Sent::StreamGone,
            b"sign-in-link" => Sent::SignInLink,
            b"sign-in-service" => return self.bodyless(Sent::SignInService),
            b"sign-out" => return self.bodyless(Sent::SignOut),
            _ => return Err("unknown message kind"),
        };
        Ok(make(self.bytes()?))
    }
    fn bodyless(&mut self, message: Sent) -> Result<Sent, &'static str> {
        if !self.bytes()?.is_empty() {
            return Err("unexpected message body");
        }
        Ok(message)
    }
}

fn hex(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("bad hex digit"),
    }
}

fn read(line: &[u8]) -> Result<Line, &'static str> {
    let end = line.iter().position(|byte| *byte == b' ').unwrap_or(line.len());
    let (kind, rest) = line.split_at(end);
    let mut r = Read { rest };
    let line = match kind {
        b"journey" => {
            let version = r.count()?;
            if version != 2 {
                return Err("unsupported journey version");
            }
            Line::Journey { version }
        }
        b"seed" => Line::Seed(r.number()?),
        b"clocks" => Line::Clocks { now: Time::from_nanos(r.number()?), wall: Wall::from_nanos(r.number()?) },
        b"screen" => Line::Screen {
            size: r.size()?,
            scheme: match r.word()? {
                b"dark" => Some(Scheme::Dark),
                b"light" => Some(Scheme::Light),
                b"none" => None,
                _ => return Err("unknown scheme"),
            },
        },
        b"background" => Line::Background(Rgb {
            red: u8::try_from(r.number()?).map_err(|_| "number overflow")?,
            green: u8::try_from(r.number()?).map_err(|_| "number overflow")?,
            blue: u8::try_from(r.number()?).map_err(|_| "number overflow")?,
        }),
        b"local" => Line::Local,
        b"front" => Line::Front,
        b"box-line" => Line::BoxLine(r.bytes()?),
        b"box-stage" => Line::BoxStage(r.bytes()?),
        b"stop-box" => Line::StopBox,
        b"stop-box-again" => Line::StopBoxAgain,
        b"box" => Line::Box,
        b"noted" => Line::Noted(r.note()?),
        b"found" => Line::Found(r.optional()?),
        b"note" => Line::Note(r.note()?),
        b"read" => Line::Read,
        b"address" => Line::Address(r.bytes()?),
        b"stored" => Line::Stored { key: r.bytes()?, bytes: r.bytes()? },
        b"credential" => Line::Credential(r.bytes()?),
        b"turn" => Line::Turn { now: Time::from_nanos(r.number()?), wall: Wall::from_nanos(r.number()?) },
        b"start" => Line::Start,
        b"due" => Line::Due,
        b"key" => Line::Key { key: r.key()?, event: r.optional()? },
        b"resize" => Line::Resize(r.size()?),
        b"reply" => Line::Reply { exchange: r.number()?, bytes: r.bytes()? },
        b"unanswered" => Line::Unanswered(r.number()?),
        b"streamed" => Line::Streamed(r.streamed()?),
        b"gone" => Line::Gone,
        b"saved" => Line::Saved,
        b"unsaved" => Line::Unsaved,
        b"kept" => Line::Kept { written: r.done()? },
        b"opened" => Line::Opened { opened: r.done()? },
        b"send" => Line::Send { exchange: r.number()?, message: r.sent()?, credential: r.optional()? },
        b"stream" => Line::Stream(r.optional()?),
        b"unstream" => Line::Unstream,
        b"save" => Line::Save { key: r.bytes()?, bytes: r.bytes()? },
        b"erase" => Line::Erase(r.bytes()?),
        b"keep" => Line::Keep(r.optional()?),
        b"signed" => Line::Signed(r.bytes()?),
        b"browse" => Line::Browse(r.bytes()?),
        b"quit" => Line::Quit,
        b"deadline" => Line::Deadline(if r.rest == b" none" {
            r.word()?;
            None
        } else {
            Some(Time::from_nanos(r.number()?))
        }),
        b"mark" => Line::Mark(if r.rest.is_empty() { None } else { Some(r.bytes()?) }),
        b"end" => Line::End(if r.rest.is_empty() { None } else { Some(r.bytes()?) }),
        _ => return Err("unknown record kind"),
    };
    if !r.rest.is_empty() {
        return Err("extra field");
    }
    Ok(line)
}

#[cfg(test)]
mod tests;

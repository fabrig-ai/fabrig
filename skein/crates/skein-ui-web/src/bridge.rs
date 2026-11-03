//! The bridge's messages between a page and its world's worker (playground.md, 5).
use crate::{fetch, transport::StreamDuty};
use skein_lib::{Duration, Time, Wall};

/// The world's pace: its nanoseconds per browser nanosecond, in thousandths; 0 is paused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Speed(pub u32);
impl Speed {
    pub const PAUSED: Speed = Speed(0);
    pub const ONE: Speed = Speed(1000);
}

/// A visit's start: the application's story, and the seed and the wall the page drew.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Start {
    pub story: Box<[u8]>,
    pub seed: u64,
    pub start: Wall,
}

/// An input for the visitor: fetch's duties (ui-web.md, 10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Exchange(fetch::Exchange),
    Abandon { exchange: u32 },
    Stream(StreamDuty),
    ReadOn,
}

/// The page's messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToWorld {
    Start(Start),
    Input(Input),
    Pace(Speed),
}

/// Which of a process's outputs a line was printed on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Output,
    Error,
}

/// A whole line a process printed, without its newline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Printed {
    pub process: Box<[u8]>,
    pub stream: Stream,
    pub line: Box<[u8]>,
}

/// The moment an input entered the world: its step count and its instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub step: u64,
    pub instant: Time,
}

/// The world's clock, closing each slice's posts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    pub step: u64,
    pub now: Time,
    pub wall: Wall,
    /// Whether the world waits idle.
    pub idle: bool,
    pub next_due: Option<Time>,
    /// The worker's linear memory in bytes, its high water; 0 natively.
    pub memory: u64,
    /// The last slice's duration in browser time; zero natively.
    pub slice: Duration,
}

/// The world's messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToPage {
    Report(fetch::Report),
    Printed(Printed),
    Stamp(Stamp),
    Clock(Clock),
    Stopped,
    Failed { words: Box<[u8]> },
}

/// Bytes that are no message of this build's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Malformed;

/// The worker's ready message: the empty buffer, which no encoded message is.
pub const READY: &[u8] = &[];

impl ToWorld {
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let mut out = Writer::default();
        match self {
            Self::Start(start) => {
                out.tag(0);
                out.bytes(&start.story);
                out.u64(start.seed);
                out.u64(start.start.as_nanos());
            }
            Self::Input(input) => {
                out.tag(1);
                out.input(input);
            }
            Self::Pace(speed) => {
                out.tag(2);
                out.u32(speed.0);
            }
        }
        out.0.into_boxed_slice()
    }

    pub fn decode(bytes: &[u8]) -> Result<ToWorld, Malformed> {
        let mut reader = Reader(bytes);
        let message = match reader.tag()? {
            0 => Self::Start(Start {
                story: reader.bytes()?.into(),
                seed: reader.u64()?,
                start: Wall::from_nanos(reader.u64()?),
            }),
            1 => Self::Input(reader.input()?),
            2 => Self::Pace(Speed(reader.u32()?)),
            _ => return Err(Malformed),
        };
        reader.end()?;
        Ok(message)
    }
}

impl ToPage {
    #[must_use]
    pub fn encode(&self) -> Box<[u8]> {
        let mut out = Writer::default();
        match self {
            Self::Report(report) => {
                out.tag(0);
                out.report(report);
            }
            Self::Printed(printed) => {
                out.tag(1);
                out.bytes(&printed.process);
                out.tag(match printed.stream {
                    Stream::Output => 0,
                    Stream::Error => 1,
                });
                out.bytes(&printed.line);
            }
            Self::Stamp(stamp) => {
                out.tag(2);
                out.u64(stamp.step);
                out.u64(stamp.instant.as_nanos());
            }
            Self::Clock(clock) => {
                out.tag(3);
                out.u64(clock.step);
                out.u64(clock.now.as_nanos());
                out.u64(clock.wall.as_nanos());
                out.tag(u8::from(clock.idle));
                match clock.next_due {
                    None => out.tag(0),
                    Some(due) => {
                        out.tag(1);
                        out.u64(due.as_nanos());
                    }
                }
                out.u64(clock.memory);
                out.u64(clock.slice.as_nanos());
            }
            Self::Stopped => out.tag(4),
            Self::Failed { words } => {
                out.tag(5);
                out.bytes(words);
            }
        }
        out.0.into_boxed_slice()
    }

    pub fn decode(bytes: &[u8]) -> Result<ToPage, Malformed> {
        let mut reader = Reader(bytes);
        let message = match reader.tag()? {
            0 => Self::Report(reader.report()?),
            1 => Self::Printed(Printed {
                process: reader.bytes()?.into(),
                stream: match reader.tag()? {
                    0 => Stream::Output,
                    1 => Stream::Error,
                    _ => return Err(Malformed),
                },
                line: reader.bytes()?.into(),
            }),
            2 => Self::Stamp(Stamp { step: reader.u64()?, instant: Time::from_nanos(reader.u64()?) }),
            3 => Self::Clock(reader.clock()?),
            4 => Self::Stopped,
            5 => Self::Failed { words: reader.bytes()?.into() },
            _ => return Err(Malformed),
        };
        reader.end()?;
        Ok(message)
    }
}

// A header pair's two u32 lengths and 40 zero bytes cover its decoded slots:
// two Strings take 48 bytes on a 64-bit target, two byte boxes take 32.
// The wire size therefore pays for the list's storage even for empty headers.
const HEADER_PADDING: [u8; 40] = [0; 40];
const HEADER_MIN: usize = 48;

#[derive(Default)]
struct Writer(Vec<u8>);
impl Writer {
    fn tag(&mut self, value: u8) {
        self.0.push(value);
    }
    fn u16(&mut self, value: u16) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn u64(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }
    fn len(&mut self, value: usize) {
        self.u32(u32::try_from(value).expect("a bridge field's length fits u32"));
    }
    fn bytes(&mut self, bytes: &[u8]) {
        self.len(bytes.len());
        self.0.extend_from_slice(bytes);
    }
    fn headers<'a>(&mut self, headers: impl ExactSizeIterator<Item = (&'a [u8], &'a [u8])>) {
        self.len(headers.len());
        for (name, value) in headers {
            self.0.extend_from_slice(&HEADER_PADDING);
            self.bytes(name);
            self.bytes(value);
        }
    }
    fn input(&mut self, input: &Input) {
        match input {
            Input::Exchange(exchange) => {
                self.tag(0);
                self.u32(exchange.id);
                self.bytes(&exchange.path);
                self.bytes(exchange.method.as_bytes());
                self.headers(exchange.headers.iter().map(|(name, value)| (name.as_bytes(), value.as_bytes())));
                self.bytes(&exchange.body);
                self.u32(exchange.bound);
            }
            Input::Abandon { exchange } => {
                self.tag(1);
                self.u32(*exchange);
            }
            Input::Stream(stream) => {
                self.tag(2);
                match stream {
                    StreamDuty::Open { path, headers, bound } => {
                        self.tag(0);
                        self.bytes(path);
                        self.headers(headers.iter().map(|(name, value)| (name.as_ref(), value.as_ref())));
                        self.u32(*bound);
                    }
                    StreamDuty::Close => self.tag(1),
                }
            }
            Input::ReadOn => self.tag(3),
        }
    }
    fn report(&mut self, report: &fetch::Report) {
        match report {
            fetch::Report::Answered { exchange, status, body } => {
                self.tag(0);
                self.u32(*exchange);
                self.u16(*status);
                self.bytes(body);
            }
            fetch::Report::Failed { exchange } => {
                self.tag(1);
                self.u32(*exchange);
            }
            fetch::Report::StreamOpened => self.tag(2),
            fetch::Report::StreamRefused { status } => {
                self.tag(3);
                self.u16(*status);
            }
            fetch::Report::StreamPiece { bytes } => {
                self.tag(4);
                self.bytes(bytes);
            }
            fetch::Report::StreamEnded => self.tag(5),
        }
    }
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], Malformed> {
        let (head, tail) = self.0.split_at_checked(len).ok_or(Malformed)?;
        self.0 = tail;
        Ok(head)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Malformed> {
        self.take(N)?.try_into().map_err(|_| Malformed)
    }
    fn tag(&mut self) -> Result<u8, Malformed> {
        let [tag] = self.array()?;
        Ok(tag)
    }
    fn u16(&mut self) -> Result<u16, Malformed> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    fn u32(&mut self) -> Result<u32, Malformed> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    fn u64(&mut self) -> Result<u64, Malformed> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    fn len(&mut self) -> Result<usize, Malformed> {
        usize::try_from(self.u32()?).map_err(|_| Malformed)
    }
    fn bytes(&mut self) -> Result<&'a [u8], Malformed> {
        let len = self.len()?;
        self.take(len)
    }
    fn text(&mut self) -> Result<String, Malformed> {
        Ok(std::str::from_utf8(self.bytes()?).map_err(|_| Malformed)?.to_owned())
    }
    fn headers<T>(&mut self, pair: fn(&mut Self) -> Result<T, Malformed>) -> Result<Vec<T>, Malformed> {
        let len = self.len()?;
        if len > self.0.len().checked_div(HEADER_MIN).ok_or(Malformed)? {
            return Err(Malformed);
        }
        // Validate every pair without allocation before reserving the slots, so
        // payload bytes cannot also pay for a missing pair's storage.
        let mut scan = Self(self.0);
        for _ in 0..len {
            if scan.take(HEADER_PADDING.len())? != HEADER_PADDING {
                return Err(Malformed);
            }
            scan.bytes()?;
            scan.bytes()?;
        }
        // The minimum encoded size bounds the allocation before any reserve.
        assert!(std::mem::size_of::<T>() <= HEADER_MIN, "header slots fit their wire size");
        let mut headers = Vec::with_capacity(len);
        for _ in 0..len {
            if self.take(HEADER_PADDING.len())? != HEADER_PADDING {
                return Err(Malformed);
            }
            headers.push(pair(self)?);
        }
        Ok(headers)
    }
    fn text_pair(&mut self) -> Result<(String, String), Malformed> {
        Ok((self.text()?, self.text()?))
    }
    fn byte_pair(&mut self) -> Result<crate::transport::Header, Malformed> {
        Ok((self.bytes()?.into(), self.bytes()?.into()))
    }
    fn input(&mut self) -> Result<Input, Malformed> {
        Ok(match self.tag()? {
            0 => Input::Exchange(fetch::Exchange {
                id: self.u32()?,
                path: self.bytes()?.into(),
                method: self.text()?,
                headers: self.headers(Self::text_pair)?,
                body: self.bytes()?.into(),
                bound: self.u32()?,
            }),
            1 => Input::Abandon { exchange: self.u32()? },
            2 => Input::Stream(match self.tag()? {
                0 => StreamDuty::Open {
                    path: self.bytes()?.into(),
                    headers: self.headers(Self::byte_pair)?,
                    bound: self.u32()?,
                },
                1 => StreamDuty::Close,
                _ => return Err(Malformed),
            }),
            3 => Input::ReadOn,
            _ => return Err(Malformed),
        })
    }
    fn report(&mut self) -> Result<fetch::Report, Malformed> {
        Ok(match self.tag()? {
            0 => fetch::Report::Answered { exchange: self.u32()?, status: self.u16()?, body: self.bytes()?.into() },
            1 => fetch::Report::Failed { exchange: self.u32()? },
            2 => fetch::Report::StreamOpened,
            3 => fetch::Report::StreamRefused { status: self.u16()? },
            4 => fetch::Report::StreamPiece { bytes: self.bytes()?.into() },
            5 => fetch::Report::StreamEnded,
            _ => return Err(Malformed),
        })
    }
    fn clock(&mut self) -> Result<Clock, Malformed> {
        Ok(Clock {
            step: self.u64()?,
            now: Time::from_nanos(self.u64()?),
            wall: Wall::from_nanos(self.u64()?),
            idle: match self.tag()? {
                0 => false,
                1 => true,
                _ => return Err(Malformed),
            },
            next_due: match self.tag()? {
                0 => None,
                1 => Some(Time::from_nanos(self.u64()?)),
                _ => return Err(Malformed),
            },
            memory: self.u64()?,
            slice: Duration::from_nanos(self.u64()?),
        })
    }
    fn end(&self) -> Result<(), Malformed> {
        if self.0.is_empty() { Ok(()) } else { Err(Malformed) }
    }
}

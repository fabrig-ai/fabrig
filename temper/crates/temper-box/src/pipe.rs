//! The bounded, line based box pipe (protocol/setup.md, 7.1).
use crate::lines::joined;
use alloc::boxed::Box;
use skein_io::kernel::Exit;
use skein_lib::{Decimal, List, bytes};
use temper_forge_setup::Stage;

#[derive(PartialEq, Eq, Debug)]
pub enum Line {
    Whole(Box<[u8]>),
    TooLong,
}
/// Retains at most the configured bound, including the line feed.
#[derive(Debug)]
pub struct Reader {
    bytes: List<u8>,
    discarding: bool,
}
impl Reader {
    #[must_use]
    pub fn new(line_bytes: u32) -> Self {
        Self { bytes: List::with_capacity(line_bytes), discarding: false }
    }
    pub fn byte(&mut self, byte: u8) -> Option<Line> {
        if self.discarding {
            if byte == b'\n' {
                self.discarding = false;
                return Some(Line::TooLong);
            }
            return None;
        }
        self.bytes.push(byte).expect("room checked per byte");
        if byte == b'\n' {
            let line = bytes::copy_of(self.bytes.as_slice());
            self.bytes.clear();
            return Some(Line::Whole(line));
        }
        if self.bytes.room() == 0 {
            self.bytes.clear();
            self.discarding = true;
        }
        None
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    Start(u16),
    Stop(u16),
    Unknown,
}
#[must_use]
pub fn command(line: &[u8]) -> Command {
    let (rest, stop) = if let Some(rest) = line.strip_prefix(b"start ") {
        (rest, false)
    } else if let Some(rest) = line.strip_prefix(b"stop ") {
        (rest, true)
    } else {
        return Command::Unknown;
    };
    let Some(digits) = rest.strip_suffix(b"\n") else {
        return Command::Unknown;
    };
    if digits.is_empty() || (digits.len() > 1 && digits.first() == Some(&b'0')) {
        return Command::Unknown;
    }
    let mut number = 0_u16;
    for byte in digits {
        if !byte.is_ascii_digit() {
            return Command::Unknown;
        }
        let Some(tens) = number.checked_mul(10) else {
            return Command::Unknown;
        };
        let Some(next) = tens.checked_add(u16::from(byte.saturating_sub(b'0'))) else {
            return Command::Unknown;
        };
        number = next;
    }
    if stop { Command::Stop(number) } else { Command::Start(number) }
}
#[must_use]
pub fn step(forge: u16, stage: Stage) -> Box<[u8]> {
    joined(&[b"step ", Decimal::of(u64::from(forge)).as_bytes(), b" ", stage.word(), b"\n"])
}
#[must_use]
pub fn started(forge: u16) -> Box<[u8]> {
    joined(&[b"started ", Decimal::of(u64::from(forge)).as_bytes(), b"\n"])
}
#[must_use]
pub fn stopped(forge: u16) -> Box<[u8]> {
    joined(&[b"stopped ", Decimal::of(u64::from(forge)).as_bytes(), b"\n"])
}
#[must_use]
pub fn failed(forge: u16, why: &[u8], line_bytes: u32) -> Box<[u8]> {
    let prefix = joined(&[b"failed ", Decimal::of(u64::from(forge)).as_bytes(), b" "]);
    let room = usize::try_from(line_bytes).expect("line bound fits").saturating_sub(prefix.len()).saturating_sub(1);
    joined(&[&prefix, why.get(..why.len().min(room)).expect("bounded why"), b"\n"])
}
#[must_use]
pub fn exited(name: &[u8], exit: Exit) -> Box<[u8]> {
    let (word, number) = match exit {
        Exit::Code(code) => (b" exited ".as_slice(), u64::from(code)),
        Exit::Signal(signal) => (b" killed by signal ".as_slice(), u64::from(signal)),
    };
    joined(&[name, word, Decimal::of(number).as_bytes()])
}
#[must_use]
pub fn not_started(name: &[u8]) -> Box<[u8]> {
    joined(&[name, b" did not start"])
}
#[must_use]
pub fn not_written() -> &'static [u8] {
    b"record not written"
}

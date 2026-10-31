//! The bounded box pipe (temper/docs/design/draft/playground-reset.md, 3 and 4).
use crate::lines::joined;
use alloc::boxed::Box;
use skein_io::kernel::Exit;
use skein_lib::{Decimal, List, bytes};
use temper_forge_setup::{Stage, machine::BoxKind};

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
    StartKind { forge: u16, kind: BoxKind },
    Stop(u16),
    Remove(u16),
    Unknown,
}
#[must_use]
pub fn command(line: &[u8]) -> Command {
    let Some(line) = line.strip_suffix(b"\n") else { return Command::Unknown };
    if let Some(rest) = line.strip_prefix(b"start ") {
        if let Some(space) = bytes::find(rest, b" ") {
            let Some(forge) = number(rest.get(..space).expect("number prefix")) else { return Command::Unknown };
            let word = rest.get(space.checked_add(1).expect("kind offset")..).expect("kind suffix");
            let Some(kind) = BoxKind::parse(word) else { return Command::Unknown };
            return Command::StartKind { forge, kind };
        }
        return match number(rest) {
            Some(forge) => Command::Start(forge),
            None => Command::Unknown,
        };
    }
    if let Some(rest) = line.strip_prefix(b"stop ") {
        return match number(rest) {
            Some(forge) => Command::Stop(forge),
            None => Command::Unknown,
        };
    }
    if let Some(rest) = line.strip_prefix(b"remove ") {
        return match number(rest) {
            Some(forge) => Command::Remove(forge),
            None => Command::Unknown,
        };
    }
    Command::Unknown
}

pub(crate) fn number(digits: &[u8]) -> Option<u16> {
    if digits.is_empty() || (digits.len() > 1 && digits.first() == Some(&b'0')) {
        return None;
    }
    let mut number = 0_u16;
    for byte in digits {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
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
pub fn removed(forge: u16) -> Box<[u8]> {
    joined(&[b"removed ", Decimal::of(u64::from(forge)).as_bytes(), b"\n"])
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

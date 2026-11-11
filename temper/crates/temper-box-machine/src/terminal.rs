//! Pure terminal words and targets (protocol/setup/box.md, 2).
use crate::{
    layout::{self, Placed},
    lines::joined,
    machine::Why,
    pipe,
};
use alloc::boxed::Box;
use skein_lib::{Decimal, Duration, List, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fact {
    Starting { step: usize },
    GroupStarting { forge: u16 },
    GroupStarted { forge: u16 },
    GroupFailed { forge: u16, why: Why },
    GroupStopped { forge: u16 },
    Restarting { step: usize, why: Why, delay: Duration },
    GivingUp { step: usize, why: Why },
    Stopping,
    Stopped,
}
/// Where a policy fact is placed (temper/docs/design/protocol/setup/box.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    Terminal,
    Log,
}
impl Fact {
    #[must_use]
    pub fn place(self) -> Place {
        match self {
            Self::GroupFailed { .. } | Self::Restarting { .. } | Self::GivingUp { .. } => Place::Terminal,
            Self::Starting { .. }
            | Self::GroupStarting { .. }
            | Self::GroupStarted { .. }
            | Self::GroupStopped { .. }
            | Self::Stopping
            | Self::Stopped => Place::Log,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    Stdout,
    Stderr,
}
#[derive(PartialEq, Eq, Debug)]
pub struct Item {
    pub target: Target,
    pub bytes: Box<[u8]>,
}
pub const PRELUDE: &[u8] = b"Open this link in a browser on this machine.\n\n";
pub const FOOTER: &[u8] = b"\nIt signs you in as this temper's owner, once, within 2 hours.\nOpened after that, it prints a fresh one here.\n";
pub const FRESH: &[u8] = b"\n";
pub const CTRL_C: &[u8] = b"Ctrl-C stops temper.\n";
pub const REFUSAL: &[u8] = b"box: serve: sign-in link too long\n";
/// Largest fixed item, also allowing the ten-byte UTC prefix.
pub const FIXED_BYTES: u32 = 256;

/// How a sign-in link reaches the box's stdout (temper/docs/design/protocol/setup/box.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinkForm {
    /// The URL alone on its line: a pipe, a file or `TERM=dumb`.
    Bare,
    /// The URL in OSC 8, then its line feed: a terminal.
    Osc8,
}
impl LinkForm {
    /// `terminal`: the box's stdout is one. `term`: `TERM`'s value, if it has one.
    #[must_use]
    pub fn of(terminal: bool, term: Option<&[u8]>) -> Self {
        if terminal && term != Some(b"dumb".as_slice()) { Self::Osc8 } else { Self::Bare }
    }
    /// One link's line, ending in its line feed as `Formatter::block` is given it.
    /// A line with no URL stays as it is.
    #[must_use]
    pub fn line(self, line: Box<[u8]>) -> Box<[u8]> {
        match self {
            Self::Bare => line,
            Self::Osc8 => {
                let url = line.strip_suffix(b"\n").expect("link line ends in LF");
                if url.is_empty() {
                    return line;
                }
                joined(&[b"\x1b]8;;", url, b"\x1b\\", url, b"\x1b]8;;\x1b\\\n"])
            }
        }
    }
}
/// A link's widest line on a terminal: the URL twice, 14 marker bytes and the line feed.
pub const LINK_LINE_BYTES: u32 = crate::host::LINE_BYTES
    .checked_mul(2)
    .expect("two URLs fit u32")
    .checked_add(15)
    .expect("OSC 8 markers and LF fit u32");

#[derive(Debug)]
pub struct Formatter {
    first_printed: bool,
    form: LinkForm,
    word: Box<[u8]>,
}
impl Formatter {
    #[must_use]
    pub fn new(form: LinkForm, word: Box<[u8]>) -> Self {
        Self { first_printed: false, form, word }
    }
    #[must_use]
    pub fn block_entries(&self) -> u32 {
        if self.first_printed { 2 } else { 3 }
    }
    /// Caller reserves `block_entries` before extracting the line. State changes
    /// only after the complete adjacent block has entered the terminal FIFO.
    #[must_use]
    pub fn block(&self, url: Box<[u8]>) -> List<Item> {
        let footer = if self.first_printed { None } else { Some(self.footer(&url)) };
        let mut items = List::with_capacity(self.block_entries());
        let words = if self.first_printed {
            Item { target: Target::Stderr, bytes: skein_lib::bytes::copy_of(FRESH) }
        } else {
            Item { target: Target::Stderr, bytes: skein_lib::bytes::copy_of(PRELUDE) }
        };
        items.push(words).expect("block count");
        items.push(Item { target: Target::Stdout, bytes: self.form.line(url) }).expect("block count");
        if let Some(bytes) = footer {
            items.push(Item { target: Target::Stderr, bytes }).expect("block count");
        }
        items
    }
    fn footer(&self, line: &[u8]) -> Box<[u8]> {
        let url = line.strip_suffix(b"\n").unwrap_or(line);
        let start = match skein_lib::bytes::find(url, b"://") {
            Some(at) => at.checked_add(3).expect("scheme end"),
            None => 0,
        };
        let rest = url.get(start..).expect("authority starts within URL");
        let end = start.checked_add(skein_lib::bytes::find(rest, b"/").unwrap_or(rest.len())).expect("origin end");
        let origin = url.get(..end).expect("origin within URL");
        let prefix = b"In a terminal: ";
        // The complete footer shares the stdout link's 8,207-byte item bound.
        // A bounded URL's origin and the fallback word always fit it.
        let overhead = FOOTER
            .len()
            .checked_add(prefix.len())
            .expect("footer prefix")
            .checked_add(origin.len())
            .expect("origin")
            .checked_add(2)
            .expect("space and LF");
        let room = usize::try_from(LINK_LINE_BYTES).expect("output bound fits usize").saturating_sub(overhead);
        let word = if self.word.len() <= room { self.word.as_ref() } else { b"temper" };
        joined(&[FOOTER, prefix, word, b" ", origin, b"\n"])
    }
    pub fn admitted_block(&mut self) {
        self.first_printed = true;
    }
    #[must_use]
    pub fn fact(&self, fact: Fact, steps: &[Placed], wall: Wall) -> Item {
        let words = match fact {
            Fact::Starting { step } => joined(&[b"Starting ", steps.get(step).expect("step fact").step.name(), b"\n"]),
            Fact::GroupStarting { forge } => {
                joined(&[layout::label(steps, forge).expect("group label"), b": starting\n"])
            }
            Fact::GroupStarted { forge } => {
                joined(&[layout::label(steps, forge).expect("group label"), b": started\n"])
            }
            Fact::GroupFailed { forge, why: reason } => {
                joined(&[layout::label(steps, forge).expect("group label"), b": failed, ", &why(reason, steps), b"\n"])
            }
            Fact::GroupStopped { forge } => {
                joined(&[layout::label(steps, forge).expect("group label"), b": stopped\n"])
            }
            Fact::Restarting { why: reason, delay, .. } => {
                joined(&[&why(reason, steps), b"; restarting in ", &seconds(delay), b" s\n"])
            }
            Fact::GivingUp { why: reason, .. } => joined(&[&why(reason, steps), b"; giving up\n"]),
            Fact::Stopping => skein_lib::bytes::copy_of(b"Stopping\n"),
            Fact::Stopped => skein_lib::bytes::copy_of(b"Stopped\n"),
        };
        self.notice(&words, wall)
    }
    #[must_use]
    pub fn notice(&self, words: &[u8], wall: Wall) -> Item {
        let day = wall.as_secs().rem_euclid(86_400);
        let hours = pair(day.div_euclid(3_600));
        let minutes = pair(day.div_euclid(60).rem_euclid(60));
        let seconds = pair(day.rem_euclid(60));
        let stamp = joined(&[&hours, b":", &minutes, b":", &seconds, b"Z "]);
        let mut parts = List::uncapped();
        let mut rest = words;
        for _ in 0..words.len() {
            let length = match skein_lib::bytes::find(rest, b"\n") {
                Some(at) => at.checked_add(1).expect("line end"),
                None => rest.len(),
            };
            parts.push(stamp.as_ref()).expect("notice parts");
            parts.push(rest.get(..length).expect("notice line")).expect("notice parts");
            rest = rest.get(length..).expect("remaining notice");
            if rest.is_empty() {
                break;
            }
        }
        let bytes = joined(parts.as_slice());
        Item { target: Target::Stderr, bytes }
    }
}
/// The same exact WHY words feed terminal notices and bounded numeric replies.
#[must_use]
pub fn why(why: Why, steps: &[Placed]) -> Box<[u8]> {
    match why {
        Why::Busy => skein_lib::bytes::copy_of(b"busy"),
        Why::NotAsked => skein_lib::bytes::copy_of(b"not asked"),
        Why::Stopping => skein_lib::bytes::copy_of(b"stopping"),
        Why::Exited { step, exit } => pipe::exited(steps.get(step).expect("step reason").step.name(), exit),
        Why::NotStarted { step } => pipe::not_started(steps.get(step).expect("step reason").step.name()),
        Why::NotReady { step } => joined(&[steps.get(step).expect("step reason").step.name(), b" version unavailable"]),
        Why::NotWritten => skein_lib::bytes::copy_of(pipe::not_written()),
        Why::NotUnlinked => skein_lib::bytes::copy_of(b"remove: files not removed"),
        Why::RemoveExited { exit } => {
            let words = pipe::exited(b"", exit);
            joined(&[b"remove:", &words])
        }
        Why::RemoveNotStarted => skein_lib::bytes::copy_of(b"remove: did not start"),
    }
}
/// UTC (10), longest WHY suffix/signal (at most 38), restart words (20),
/// decimal u64 nanoseconds in seconds (at most 21), punctuation and LF all
/// fit 128 extra bytes. Sum independent label/name maxima conservatively.
#[must_use]
pub fn max_bytes(steps: &[Placed]) -> u32 {
    let mut label = 0_u32;
    let mut name = 0_u32;
    for placed in steps {
        name = name.max(u32::try_from(placed.step.name().len()).expect("bounded step name"));
        if let Some(words) = &placed.label {
            label = label.max(u32::try_from(words.len()).expect("bounded label"));
        }
    }
    label.checked_add(name).expect("lifecycle words").checked_add(128).expect("lifecycle line").max(FIXED_BYTES)
}
fn seconds(delay: Duration) -> Box<[u8]> {
    let nanos = delay.as_nanos();
    let whole = Decimal::of(nanos.div_euclid(1_000_000_000));
    let mut part = nanos.rem_euclid(1_000_000_000);
    if part == 0 {
        return skein_lib::bytes::copy_of(whole.as_bytes());
    }
    let mut fraction = [b'0'; 9];
    for digit in fraction.iter_mut().rev() {
        *digit = b'0'.checked_add(u8::try_from(part.rem_euclid(10)).expect("decimal digit")).expect("ASCII digit");
        part = part.div_euclid(10);
    }
    let mut end = fraction.len();
    for digit in fraction.iter().rev() {
        if *digit != b'0' {
            break;
        }
        end = end.checked_sub(1).expect("nonzero fraction");
    }
    joined(&[whole.as_bytes(), b".", fraction.get(..end).expect("trimmed fraction")])
}
fn pair(value: u64) -> [u8; 2] {
    [
        b'0'.checked_add(u8::try_from(value.div_euclid(10)).expect("two digits")).expect("ASCII digit"),
        b'0'.checked_add(u8::try_from(value.rem_euclid(10)).expect("one digit")).expect("ASCII digit"),
    ]
}

/// Last successfully logged diagnostic pieces, already in echo's spelling.
#[derive(Debug)]
pub(crate) struct Tail {
    lines: skein_lib::Queue<Box<[u8]>>,
}
impl Tail {
    pub fn new(lines: u32) -> Self {
        Self { lines: skein_lib::Queue::with_capacity(lines) }
    }
    pub fn push(&mut self, line: Box<[u8]>) {
        if self.lines.room() == 0 {
            drop(self.lines.pop());
        }
        self.lines.push(line);
    }
    pub fn reset(&mut self) {
        while let Some(line) = self.lines.pop() {
            drop(line);
        }
    }
    pub fn append(&self, item: &mut Item, path: &[u8]) {
        if self.lines.is_empty() {
            return;
        }
        let mut length =
            item.bytes.len().checked_add(10).expect("Log words").checked_add(path.len()).expect("log path");
        for line in &self.lines {
            length = length.checked_add(4).expect("indent").checked_add(line.len()).expect("tail line");
        }
        let mut writer = skein_lib::Writer::new(length);
        writer.put(&item.bytes).expect("failure line");
        for line in &self.lines {
            writer.put(b"    ").expect("indent");
            writer.put(line).expect("tail line");
        }
        writer.put(b"    Log: ").expect("Log words");
        writer.put(path).expect("log path");
        writer.put(b"\n").expect("log LF");
        item.bytes = writer.finish();
    }
}
/// Only failures of an actual step's exit or readiness carry a tail.
pub(crate) fn tail_step(fact: Fact) -> Option<usize> {
    let why = match fact {
        Fact::Restarting { why, .. } | Fact::GivingUp { why, .. } | Fact::GroupFailed { why, .. } => why,
        Fact::Starting { .. }
        | Fact::GroupStarting { .. }
        | Fact::GroupStarted { .. }
        | Fact::GroupStopped { .. }
        | Fact::Stopping
        | Fact::Stopped => return None,
    };
    match why {
        Why::Exited { step, .. } | Why::NotReady { step } => Some(step),
        Why::NotStarted { .. }
        | Why::NotWritten
        | Why::Busy
        | Why::NotAsked
        | Why::Stopping
        | Why::NotUnlinked
        | Why::RemoveExited { .. }
        | Why::RemoveNotStarted => None,
    }
}
pub(crate) fn failure_bytes(steps: &[Placed], data: &[u8]) -> u32 {
    let mut name = 0_u32;
    for step in steps {
        name = name.max(u32::try_from(step.step.name().len()).expect("step name"));
    }
    let line =
        name.checked_add(crate::host::LINE_BYTES).expect("piece").checked_add(17).expect("time, indent, prefix, LF");
    max_bytes(steps)
        .checked_add(line.checked_mul(crate::limits::MAX_TAIL_LINES).expect("tail lines"))
        .expect("failure block")
        .checked_add(u32::try_from(data.len()).expect("data path"))
        .expect("log path")
        .checked_add(name)
        .expect("log filename")
        .checked_add(32)
        .expect("Log words and path suffix")
}

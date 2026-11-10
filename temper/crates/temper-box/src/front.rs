//! The box's lines to its TUI in front (temper/docs/design/protocol/setup/box.md, 6).
use crate::{
    layout::Placed,
    lines::joined,
    terminal::{Fact, Formatter, Target},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes};

/// A line's most bytes, its line feed among them (protocol/setup/box.md, 6).
pub const LINE_BYTES: usize = 8192;
/// The most lines the box holds that its front has not taken (protocol/setup/box.md, 6).
pub const HELD: u32 = 256;

/// One of the box's lines to its front (protocol/setup/box.md, 6).
#[derive(Clone, PartialEq, Eq)]
pub enum Line {
    Link { url: Box<[u8]> },
    Log { text: Box<[u8]> },
    Restarting { step: Box<[u8]> },
    Running { step: Box<[u8]> },
    GaveUp { step: Box<[u8]> },
}
impl Line {
    /// `link`, `log`, `restarting`, `running` or `gave-up`.
    #[must_use]
    pub fn word(&self) -> &'static [u8] {
        match self {
            Self::Link { .. } => b"link",
            Self::Log { .. } => b"log",
            Self::Restarting { .. } => b"restarting",
            Self::Running { .. } => b"running",
            Self::GaveUp { .. } => b"gave-up",
        }
    }
    /// The word, a space, the field cut to fit at a UTF-8 boundary, and a line feed.
    #[must_use]
    pub fn bytes(&self) -> Box<[u8]> {
        let field = match self {
            Self::Link { url } => url,
            Self::Log { text } => text,
            Self::Restarting { step } | Self::Running { step } | Self::GaveUp { step } => step,
        };
        let word = self.word();
        let room = LINE_BYTES.checked_sub(word.len()).expect("word fits").checked_sub(2).expect("space and LF fit");
        let mut end = room.min(field.len());
        for _ in 0..room {
            let Some(byte) = field.get(end) else { break };
            if byte & 0b1100_0000 != 0b1000_0000 {
                break;
            }
            end = end.checked_sub(1).expect("continuation follows a leading byte");
        }
        joined(&[word, b" ", field.get(..end).expect("field prefix"), b"\n"])
    }
}
impl core::fmt::Debug for Line {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Link { .. } => formatter.debug_struct("Link").finish_non_exhaustive(),
            Self::Log { text } => formatter.debug_struct("Log").field("text", text).finish(),
            Self::Restarting { step } => formatter.debug_struct("Restarting").field("step", step).finish(),
            Self::Running { step } => formatter.debug_struct("Running").field("step", step).finish(),
            Self::GaveUp { step } => formatter.debug_struct("GaveUp").field("step", step).finish(),
        }
    }
}

/// Each nonempty line of whole timed lines, as `Formatter::notice` and
/// `Tail::append` make them, as a `log` line.
#[must_use]
pub fn logs(input: &[u8]) -> List<Line> {
    let mut lines = List::uncapped();
    let mut rest = input;
    for _ in 0..input.len() {
        let end = bytes::find(rest, b"\n").unwrap_or(rest.len());
        let text = rest.get(..end).expect("line prefix");
        if !text.is_empty() {
            lines.push(Line::Log { text: bytes::copy_of(text) }).expect("uncapped lines");
        }
        let Some(next) = rest.get(end.checked_add(1).expect("line feed end")..) else { break };
        rest = next;
        if rest.is_empty() {
            break;
        }
    }
    lines
}
/// The line a fact gives the front beside its `log` lines.
#[must_use]
pub fn state(fact: Fact, steps: &[Placed]) -> Option<Line> {
    match fact {
        Fact::Restarting { step, .. } => {
            Some(Line::Restarting { step: bytes::copy_of(steps.get(step).expect("step fact").step.name()) })
        }
        Fact::GivingUp { step, .. } => {
            Some(Line::GaveUp { step: bytes::copy_of(steps.get(step).expect("step fact").step.name()) })
        }
        Fact::Starting { .. }
        | Fact::GroupStarting { .. }
        | Fact::GroupStarted { .. }
        | Fact::GroupFailed { .. }
        | Fact::GroupStopped { .. }
        | Fact::Stopping
        | Fact::Stopped => None,
    }
}
/// The first block's lines but its link and its blank lines, for box.log.
/// The URL has no line feed. Call before `Formatter::admitted_block`;
/// after admission the fresh block has no words and the result is empty.
#[must_use]
pub fn block_words(formatter: &Formatter, url: &[u8]) -> Box<[u8]> {
    let block = formatter.block(joined(&[url, b"\n"]));
    let mut parts = List::uncapped();
    for item in &block {
        match item.target {
            Target::Stdout => {}
            Target::Stderr => {
                let mut rest = item.bytes.as_ref();
                for _ in 0..item.bytes.len() {
                    let end = bytes::find(rest, b"\n").unwrap_or(rest.len());
                    let text = rest.get(..end).expect("block line prefix");
                    if !text.is_empty() {
                        parts.push(text).expect("uncapped block words");
                        parts.push(b"\n").expect("uncapped block words");
                    }
                    let Some(next) = rest.get(end.checked_add(1).expect("block line feed end")..) else { break };
                    rest = next;
                    if rest.is_empty() {
                        break;
                    }
                }
            }
        }
    }
    joined(parts.as_slice())
}

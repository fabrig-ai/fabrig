//! Whole Smith turn bytes become the client's own bounded vocabulary. No state,
//! endpoint policy or opaque provider bytes survive this translation. `decode`
//! refuses unsupported or broken records; a receiving limit cuts the displayed
//! turn, marked by `Turn::cut`. See client/domain/client.md, 10, and contract.md, 10.
//!
//! `brief_text` renders the decoded client blocks as plain UTF-8 lines for a fresh
//! run's brief, within `Limits::bytes`; archived system context is excluded,
//! and opaque blocks show only dialect and size.
//! It admits exactly the records `decode` admits.
//!
//! Calls are numbered by their first call/result block's position within the turn.
//! Their inputs remain bytes as written. Results also fill the matching call's
//! outcome line, including results outside the display window. This schema has no jig
//! object references or inbox read position: neither is inferred from tool text.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
pub use jig_client_domain::contract::{Block, Call, Speaker, Turn, TurnLimits};
use jig_client_domain::contract::{Message, ModelEffort, Text, Usage};
use skein_lib::{Decimal, List, Reader, length::len_u64};
use smith_transcript::v7 as smith;

mod brief;
mod summary;
#[cfg(test)]
mod tests;

/// Admission bounds for Smith bytes and receiving bounds for the displayed turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub smith: smith::Limits,
    pub bytes: u32,
    pub turn: TurnLimits,
}

impl Limits {
    #[must_use]
    pub const fn new(bytes: u32, turn: TurnLimits) -> Self {
        Self { smith: smith::CEILINGS, bytes, turn }
    }
}

/// Why one activity turn cannot be shown; its activity line remains in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Undecodable {
    /// The record has an unsupported version.
    Version { found: u16 },
    /// Broken fields or trailing bytes.
    Malformed,
    /// The wire turn exceeds admission bounds.
    TooLarge,
    /// A wire enumeration names an unknown variant.
    Unknown,
}

/// Decodes a whole record; receiving bounds truncate without losing usage/spend.
pub fn decode(limits: &Limits, bytes: &[u8]) -> Result<Turn, Undecodable> {
    Ok(translate(limits.turn, &decode_source(limits, bytes)?, true))
}

/// Decode conversation blocks for an activity page, leaving system inspection to a whole-turn read.
pub fn conversation(limits: &Limits, bytes: &[u8]) -> Result<Turn, Undecodable> {
    Ok(translate(limits.turn, &decode_source(limits, bytes)?, false))
}

fn decode_source(limits: &Limits, bytes: &[u8]) -> Result<smith::Turn, Undecodable> {
    if bytes.len() > skein_lib::length::usize_of(limits.bytes) {
        return Err(Undecodable::TooLarge);
    }
    let version = Reader::new(bytes).u16().ok_or(Undecodable::Malformed)?;
    if version != 6 && u64::from(version) != smith::Turn::FORMAT_VERSION {
        return Err(Undecodable::Version { found: version });
    }
    let source = match smith_transcript::decode_archived(&limits.smith, bytes) {
        Ok(turn) => turn,
        Err(problem) => return Err(classify(problem)),
    };
    summary::validate(&limits.smith, &source)?;
    Ok(source)
}

/// Render one decoded turn as bounded UTF-8 lines for an agent's brief.
pub fn brief_text(limits: &Limits, bytes: &[u8]) -> Result<Box<[u8]>, Undecodable> {
    let turn = decode(limits, bytes)?;
    Ok(brief::render(&turn, limits.bytes))
}

// Smith's codec reason type is intentionally not a client dependency. Obtain
// allocation-free reference reasons through Smith's public limits check/decoder.
fn classify(problem: smith::Problem) -> Undecodable {
    let mut beyond = smith::CEILINGS;
    beyond.turn_endpoint = beyond.turn_endpoint.checked_add(1).expect("small ceiling");
    let bound = smith::check_limits(&beyond).expect_err("limits exceed schema");
    let tag = smith::Role::decode(&smith::CEILINGS, &mut Reader::new(&[255])).expect_err("unknown role tag");
    if problem.reason == bound.reason {
        Undecodable::TooLarge
    } else if problem.reason == tag.reason {
        Undecodable::Unknown
    } else {
        Undecodable::Malformed
    }
}

fn translate(limits: TurnLimits, source: &smith::Turn, inspect_system: bool) -> Turn {
    let mut budget = Budget { left: limits.text, cut: false };
    let supplied = inspect_system && source.system().is_some();
    let system_fits = supplied && source.messages().len() < limits.messages;
    let mut messages =
        List::with_capacity(limits.messages.min(source.messages().len().saturating_add(u32::from(system_fits))));
    if system_fits {
        supplied_message(limits, &mut messages, &mut budget);
    } else if supplied {
        budget.cut = true;
    }
    let offset = usize::from(!messages.is_empty());
    append_messages(limits, source, &mut messages, &mut budget);
    // Names are already priced. Each later pass walks only the retained window.
    for price in [Price::Texts, Price::Outcomes, Price::Inputs, Price::Summaries] {
        for (original, message) in source.messages().iter().zip(messages.iter_mut().skip(offset)) {
            for (original, block) in original.blocks().iter().zip(message.blocks.iter_mut()) {
                fill_block(source, original, block, price, limits.outcome, &mut budget);
            }
        }
    }
    fill_supplied(source, &mut messages, &mut budget);
    let usage = source.usage();
    Turn {
        messages,
        // The contract's counts are plain: one the provider did not report shows
        // as zero, as smith reported it before its counts became optional.
        used: Usage {
            input: usage.input().unwrap_or(0),
            output: usage.output().unwrap_or(0),
            cached: usage.cache_read().unwrap_or(0),
            cache_written: usage.cache_write().unwrap_or(0),
        },
        spent: source.spent(),
        read_through: None,
        llm: llm(source),
        cut: budget.cut,
    }
}

/// The turn's model and effort, outside the text budget as its usage is (client/domain/client.md, 10).
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
fn llm(source: &smith::Turn) -> Option<ModelEffort> {
    if source.model().is_empty() {
        return None;
    }
    let model = Text::new(Box::from(source.model()), smith::CEILINGS.turn_model)
        .expect("decoded model fits its transcript bound");
    let effort = match source.effort() {
        Some(effort) => Some(
            Text::new(Box::from(effort.as_ref()), smith::CEILINGS.turn_effort_some)
                .expect("decoded effort fits its transcript bound"),
        ),
        None => None,
    };
    Some(ModelEffort { model, effort })
}

fn empty_text() -> Text {
    Text::new(Box::new([]), 0).expect("empty text fits")
}

// Build the displayed shape and price every retained name before other text.
fn translate_block(source: &smith::Turn, block: &smith::Block, budget: &mut Budget) -> Block {
    match block {
        smith::Block::Text(_) | smith::Block::Refusal(_) => Block::Text(empty_text()),
        smith::Block::Opaque(opaque) => Block::Opaque { dialect: empty_text(), size: len_u64(opaque.bytes().len()) },
        smith::Block::Call(call) => Block::Call(Call {
            number: call_number(source, call.id()),
            tool: budget.text(call.name()),
            input: empty_text(),
            outcome: None,
            failed: false,
            objects: List::with_capacity(0),
            parts: List::with_capacity(0),
        }),
        smith::Block::Result(result) => {
            Block::Result { call: call_number(source, result.id()), summary: empty_text(), failed: false }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Price {
    Texts,
    Outcomes,
    Inputs,
    Summaries,
}

fn fill_block(
    source: &smith::Turn,
    original: &smith::Block,
    block: &mut Block,
    price: Price,
    outcome: u32,
    budget: &mut Budget,
) {
    match block {
        Block::Text(text) => {
            if price == Price::Texts {
                match original {
                    smith::Block::Text(original) | smith::Block::Refusal(original) => {
                        *text = budget.text(original.text());
                    }
                    smith::Block::Opaque(_) | smith::Block::Call(_) | smith::Block::Result(_) => {
                        unreachable!("retained blocks follow the source")
                    }
                }
            }
        }
        Block::Opaque { dialect, .. } => {
            if price == Price::Texts {
                match original {
                    smith::Block::Opaque(original) => {
                        *dialect = budget.text(Decimal::of(u64::from(original.dialect())).as_bytes());
                    }
                    smith::Block::Text(_)
                    | smith::Block::Refusal(_)
                    | smith::Block::Call(_)
                    | smith::Block::Result(_) => {
                        unreachable!("retained blocks follow the source")
                    }
                }
            }
        }
        Block::Call(call) => match original {
            smith::Block::Call(original) => match price {
                Price::Outcomes => {
                    if let Some(answer) = call_outcome(source, original.id(), outcome, budget) {
                        call.outcome = Some(answer.text);
                        call.failed = answer.failed;
                    }
                }
                Price::Inputs => call.input = budget.text(original.input()),
                Price::Texts | Price::Summaries => {}
            },
            smith::Block::Text(_) | smith::Block::Refusal(_) | smith::Block::Opaque(_) | smith::Block::Result(_) => {
                unreachable!("retained blocks follow the source")
            }
        },
        Block::Result { summary, failed, .. } => {
            if price == Price::Summaries {
                match original {
                    smith::Block::Result(original) => {
                        let answer = result_text(original.returned(), None, budget);
                        *summary = answer.text;
                        *failed = answer.failed;
                    }
                    smith::Block::Text(_)
                    | smith::Block::Refusal(_)
                    | smith::Block::Opaque(_)
                    | smith::Block::Call(_) => {
                        unreachable!("retained blocks follow the source")
                    }
                }
            }
        }
    }
}

fn call_outcome(source: &smith::Turn, id: &[u8], bound: u32, budget: &mut Budget) -> Option<ResultText> {
    for message in source.messages() {
        for candidate in message.blocks() {
            match candidate {
                smith::Block::Result(result) => {
                    if result.id() == id {
                        return Some(result_text(result.returned(), Some(bound), budget));
                    }
                }
                smith::Block::Text(_) | smith::Block::Refusal(_) | smith::Block::Call(_) | smith::Block::Opaque(_) => {}
            }
        }
    }
    None
}

fn call_number(source: &smith::Turn, id: &[u8]) -> u64 {
    let mut position = 0_u64;
    for message in source.messages() {
        for block in message.blocks() {
            position = position.checked_add(1).expect("schema ceilings bound blocks");
            let candidate = match block {
                smith::Block::Call(call) => Some(call.id()),
                smith::Block::Result(result) => Some(result.id()),
                smith::Block::Text(_) | smith::Block::Refusal(_) | smith::Block::Opaque(_) => None,
            };
            if candidate == Some(id) {
                return position;
            }
        }
    }
    unreachable!("the translated call or result occurs in its source turn")
}

#[derive(Debug)]
struct Budget {
    left: u32,
    cut: bool,
}
impl Budget {
    fn text(&mut self, bytes: &[u8]) -> Text {
        self.bounded_text(bytes, self.left)
    }

    fn bounded_text(&mut self, bytes: &[u8], bound: u32) -> Text {
        let maximum = skein_lib::length::usize_of(self.left.min(bound));
        let bytes = if bytes.len() <= maximum {
            Box::from(bytes)
        } else {
            self.cut = true;
            let mark = "…".as_bytes();
            if let Some(room) = maximum.checked_sub(mark.len()) {
                let prefix = skein_lib::bytes::utf8_prefix(bytes, room);
                let length = prefix.len().checked_add(mark.len()).expect("within receiving bound");
                let mut writer = skein_lib::Writer::new(length);
                writer.put(prefix).expect("prefix fits final allocation");
                writer.put(mark).expect("mark fits final allocation");
                writer.finish()
            } else {
                Box::new([])
            }
        };
        let length = u32::try_from(bytes.len()).expect("bounded length");
        self.left = self.left.checked_sub(length).expect("length within budget");
        Text::new(bytes, length).expect("text allocated at receiving length")
    }
}

#[derive(Debug)]
struct ResultText {
    text: Text,
    failed: bool,
}
fn result_text(returned: &smith::Returned, outcome: Option<u32>, budget: &mut Budget) -> ResultText {
    match returned {
        smith::Returned::Said(said) => {
            ResultText { text: returned_text(said.text(), outcome, budget), failed: said.error() }
        }
        smith::Returned::Outcome(_)
        | smith::Returned::Invalid(_)
        | smith::Returned::NotRun(_)
        | smith::Returned::Withdrawn => {
            let summary = summary::summarize(returned);
            ResultText { text: returned_text(&summary.bytes, outcome, budget), failed: summary.failed }
        }
    }
}

fn returned_text(bytes: &[u8], outcome: Option<u32>, budget: &mut Budget) -> Text {
    match outcome {
        Some(bound) => budget.bounded_text(first_line(bytes), bound),
        None => budget.text(bytes),
    }
}

fn first_line(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut words = false;
    for (at, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            if words {
                let line = bytes.get(start..at).expect("line is within input");
                return line.strip_suffix(b"\r").unwrap_or(line);
            }
            start = at.checked_add(1).expect("line position within input");
        } else if !byte.is_ascii_whitespace() {
            words = true;
        }
    }
    if words {
        let line = bytes.get(start..).expect("last line is within input");
        line.strip_suffix(b"\r").unwrap_or(line)
    } else {
        b""
    }
}

fn supplied_message(limits: TurnLimits, messages: &mut List<Message>, budget: &mut Budget) {
    if messages.len() == limits.messages || limits.blocks == 0 {
        budget.cut = true;
        return;
    }
    let mut blocks = List::with_capacity(1);
    blocks.push(Block::Text(empty_text())).expect("one system block");
    messages.push(Message { speaker: Speaker::System, blocks }).expect("system slot reserved");
}

fn append_messages(limits: TurnLimits, source: &smith::Turn, messages: &mut List<Message>, budget: &mut Budget) {
    for message in source.messages() {
        if messages.len() == messages.capacity() {
            budget.cut = true;
            break;
        }
        let speaker = match message.role() {
            smith::Role::User => Speaker::Person,
            smith::Role::Assistant => Speaker::Assistant,
        };
        let mut blocks = List::with_capacity(limits.blocks.min(message.blocks().len()));
        for block in message.blocks() {
            if blocks.len() == blocks.capacity() {
                budget.cut = true;
                break;
            }
            let block = translate_block(source, block, budget);
            blocks.push(block).expect("receiving block capacity checked");
        }
        messages.push(Message { speaker, blocks }).expect("receiving message capacity checked");
    }
}

fn fill_supplied(source: &smith::Turn, messages: &mut List<Message>, budget: &mut Budget) {
    if let Some(system) = source.system()
        && let Some(message) = messages.get_mut(0)
        && message.speaker == Speaker::System
    {
        *message.blocks.get_mut(0).expect("system block retained") = Block::Text(budget.text(system));
    }
}

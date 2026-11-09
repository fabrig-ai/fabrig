//! A turn's private parts belong to its task's requester
//! (`jig/docs/design/domain/engine.md`, 10.1).

use alloc::boxed::Box;
use skein_lib::{Intake, List, Set, Writer, length::usize_of, stream::Read};

use crate::{Limits, Undecodable, classify, decode_source, smith, summary};

/// What a turn shows its task's requester alone (domain/engine.md, 10.1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Private {
    /// The brief section's title as runs' briefs render it; empty withholds no section.
    pub title: Box<[u8]>,
    /// The tools whose calls are withheld, by name.
    pub tools: List<Box<[u8]>>,
}

impl Private {
    /// Nothing withheld.
    #[must_use]
    pub fn none() -> Self {
        Self { title: Box::new([]), tools: List::with_capacity(0) }
    }
}

/// The one line a withheld part becomes.
pub const WITHHELD: &str = "Withheld: shown to its person alone.";

/// The turn's bytes with its private parts withheld, re-encoded as the current version.
pub fn withhold(limits: &Limits, bytes: &[u8], private: &Private) -> Result<Box<[u8]>, Undecodable> {
    let source = decode_source(limits, bytes)?;
    let ids = private_ids(&source, private);
    let mut parts = source.into_parts();
    if let Some(system) = &mut parts.system
        && let Some(projected) = section(system, &private.title, limits.bytes.min(limits.smith.turn_system_some))?
    {
        *system = projected;
    }
    for message in parts.messages.iter_mut() {
        let mut fields = message.clone().into_parts();
        for block in fields.blocks.iter_mut() {
            *block = project_block(limits, block.clone(), private, &ids)?;
        }
        *message = match smith::Message::new(&limits.smith, fields) {
            Ok(message) => message,
            Err(problem) => return Err(classify(problem)),
        };
    }
    let projected = match smith::Turn::new(&limits.smith, parts) {
        Ok(turn) => turn,
        Err(problem) => return Err(classify(problem)),
    };
    let length = projected.measure();
    if length > limits.bytes {
        return Err(Undecodable::TooLarge);
    }
    let mut writer = Writer::new(usize_of(length));
    projected.encode(&mut writer).expect("writer sized by the turn's measured length");
    Ok(writer.finish())
}

fn private_ids(source: &smith::Turn, private: &Private) -> Set<Box<[u8]>> {
    let capacity = smith::CEILINGS
        .turn_messages
        .checked_mul(smith::CEILINGS.message_blocks)
        .expect("schema bounds the number of blocks");
    let mut ids = Set::with_capacity(capacity);
    for message in source.messages() {
        for block in message.blocks() {
            match block {
                smith::Block::Call(call) => {
                    if private_tool(private, call.name()) {
                        ids.insert(Box::from(call.id())).expect("at most one id per block");
                    }
                }
                smith::Block::Text(_)
                | smith::Block::Refusal(_)
                | smith::Block::Result(_)
                | smith::Block::Opaque(_) => {}
            }
        }
    }
    ids
}

fn private_tool(private: &Private, name: &[u8]) -> bool {
    for tool in &private.tools {
        if tool.as_ref() == name {
            return true;
        }
    }
    false
}

fn project_block(
    limits: &Limits,
    block: smith::Block,
    private: &Private,
    ids: &Set<Box<[u8]>>,
) -> Result<smith::Block, Undecodable> {
    Ok(match block {
        smith::Block::Text(text) => {
            let mut fields = text.into_parts();
            if let Some(projected) = section(&fields.text, &private.title, limits.bytes.min(limits.smith.text_text))? {
                fields.text = projected;
                fields.replay = None;
            }
            match smith::Text::new(&limits.smith, fields) {
                Ok(text) => smith::Block::Text(text),
                Err(problem) => return Err(classify(problem)),
            }
        }
        smith::Block::Call(call) => {
            let mut fields = call.into_parts();
            if private_tool(private, &fields.name) {
                fields.input = Box::from(b"{}".as_slice());
                fields.replay = None;
            }
            match smith::Call::new(&limits.smith, fields) {
                Ok(call) => smith::Block::Call(call),
                Err(problem) => return Err(classify(problem)),
            }
        }
        smith::Block::Result(result) => {
            let mut fields = result.into_parts();
            if ids.contains(fields.id.as_ref()) {
                let error = summary::summarize(&fields.returned).failed;
                fields.returned = match smith::Said::new(
                    &limits.smith,
                    smith::SaidParts { text: Box::from(WITHHELD.as_bytes()), error },
                ) {
                    Ok(said) => smith::Returned::Said(said),
                    Err(problem) => return Err(classify(problem)),
                };
            }
            match smith::ToolResult::new(&limits.smith, fields) {
                Ok(result) => smith::Block::Result(result),
                Err(problem) => return Err(classify(problem)),
            }
        }
        smith::Block::Refusal(text) => smith::Block::Refusal(text),
        smith::Block::Opaque(opaque) => smith::Block::Opaque(opaque),
    })
}

fn heading(line: &[u8], title: &[u8]) -> bool {
    !title.is_empty() && line.strip_prefix(b"## ") == Some(title)
}

fn private_line(line: &[u8]) -> bool {
    if line.is_empty() || line == b"None kept." {
        return true;
    }
    let mut digits = 0_usize;
    for byte in line {
        if !byte.is_ascii_digit() {
            break;
        }
        digits = digits.checked_add(1).expect("digit count within line");
    }
    digits > 0 && line.get(digits..).expect("digits within line").starts_with(b". ")
}

fn section(bytes: &[u8], title: &[u8], bound: u32) -> Result<Option<Box<[u8]>>, Undecodable> {
    if title.is_empty() {
        return Ok(None);
    }
    let mut found = false;
    let mut output = Intake::with_capacity(bound);
    let mut skipping = false;
    let mut start = 0;
    for end in 0..=bytes.len() {
        let newline = bytes.get(end) == Some(&b'\n');
        if !newline && end != bytes.len() {
            continue;
        }
        if start == bytes.len() {
            break;
        }
        let bare = bytes.get(start..end).expect("line positions within text");
        let next = end.checked_add(usize::from(newline)).expect("line ending within text");
        let line = bytes.get(start..next).expect("line ending within text");
        let prefix = bytes.get(..start).expect("line start within text");
        start = next;
        if skipping && private_line(bare) {
            continue;
        }
        skipping = heading(bare, title);
        if skipping && !found {
            append(&mut output, prefix)?;
            found = true;
        }
        if !found {
            continue;
        }
        append(&mut output, line)?;
        if skipping {
            if !newline {
                append(&mut output, b"\n")?;
            }
            append(&mut output, b"\n")?;
            append(&mut output, WITHHELD.as_bytes())?;
            append(&mut output, b"\n\n")?;
        }
    }
    if !found {
        return Ok(None);
    }
    let length = output.len();
    Ok(Some(output.meet(Read::Fill(length)).expect("projected text is buffered")))
}

fn append(output: &mut Intake, bytes: &[u8]) -> Result<(), Undecodable> {
    match output.append(bytes) {
        Ok(()) => Ok(()),
        Err(skein_lib::Overflow) => Err(Undecodable::TooLarge),
    }
}

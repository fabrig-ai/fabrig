//! API message assembly (protocol/programs.md, section 5.5). Keeps one
//! message and four waiting completions; never derives usage from a result.
//! `StructuredOutput` waits for its outcome and becomes finish in the same slot;
//! it is never a workspace built-in (protocol/programs.md, section 5.5).

use super::{Bound, Rejection};
use crate::mcp::json;
use alloc::boxed::Box;
use skein_json::{Document, Kind};
use skein_lib::{List, Queue};
use smith_domain::llm;

// Marks adapter-owned finish feedback, which is never an MCP reply.
pub(super) const STRUCTURED_FINISH: &[u8] = b"smith-structured-finish";

pub(super) struct Message {
    id: Box<[u8]>,
    framed: bool,
    stopped: bool,
    expected: u32,
    received: u32,
    blocks: List<llm::Said>,
    usage: llm::Usage,
    bytes: u64,
    raw: u32,
}

pub(super) struct Turns {
    pub message: Option<Message>,
    pub ready: Queue<llm::Completion>,
    pub result: bool,
    pub assistant_events: u64,
    pub closed: u64,
    pub finish: bool,
    pub blocks: u32,
    pub maximum: u64,
    pub maximum_raw: u32,
}

struct AssemblyBounds {
    blocks: u32,
    raw: u32,
    bytes: u64,
    input: u32,
}

impl Message {
    fn content(
        &mut self,
        document: &Document,
        content: u32,
        bounds: AssemblyBounds,
        finish: &mut bool,
    ) -> Result<u32, Rejection> {
        let mut index = content.checked_add(1).ok_or(Rejection::Overflow)?;
        let mut text_bytes = 0_u32;
        for _ in 0..document.len() {
            if document.token(index).ok_or(Rejection::Malformed)?.kind == Kind::ArrayEnd {
                break;
            }
            self.received = self.received.checked_add(1).ok_or(Rejection::Overflow)?;
            check_bound(Bound::Blocks, u64::from(self.received), u64::from(bounds.blocks))?;
            if let Some(block) = block(document, index, bounds.input, finish)? {
                match &block {
                    llm::Said::Text { text, .. } => {
                        text_bytes = text_bytes
                            .checked_add(u32::try_from(text.len()).ok().ok_or(Rejection::Overflow)?)
                            .ok_or(Rejection::Overflow)?;
                    }
                    llm::Said::Refusal { .. } | llm::Said::ToolCall { .. } | llm::Said::Opaque { .. } => {}
                }
                self.retain(block, &bounds)?;
            }
            index = json::after(document, index).ok_or(Rejection::Malformed)?;
        }
        Ok(text_bytes)
    }

    fn retain(&mut self, block: llm::Said, bounds: &AssemblyBounds) -> Result<(), Rejection> {
        let payload = payload(&block)?;
        let raw = u64::from(self.raw).checked_add(payload).ok_or(Rejection::Overflow)?;
        check_bound(Bound::Raw, raw, u64::from(bounds.raw))?;
        let bytes = self
            .bytes
            .checked_add(skein_lib::len_u64(size_of::<llm::Said>()))
            .ok_or(Rejection::Overflow)?
            .checked_add(payload)
            .ok_or(Rejection::Overflow)?;
        check_bound(Bound::Completion, bytes, bounds.bytes)?;
        self.raw = u32::try_from(raw).ok().ok_or(Rejection::Overflow)?;
        self.bytes = bytes;
        self.blocks.push(block).expect("admitted block count");
        Ok(())
    }

    fn structured(&mut self, block: llm::Said, maximum: u32) -> Result<(), Rejection> {
        let mut position = None;
        for (index, held) in self.blocks.iter().enumerate() {
            if structured_call(held) {
                position = Some(index);
            }
        }
        let mut finish = Some(block);
        if position.is_some() {
            let old = core::mem::replace(&mut self.blocks, List::with_capacity(maximum));
            for (index, held) in old.into_boxed().into_iter().enumerate() {
                let kept = if Some(index) == position {
                    finish.take()
                } else if structured_call(&held) {
                    None
                } else {
                    Some(held)
                };
                if let Some(kept) = kept {
                    self.blocks.push(kept).expect("translated blocks fit the admitted message");
                }
            }
        } else {
            check_bound(Bound::Blocks, u64::from(self.blocks.len()).saturating_add(1), u64::from(maximum))?;
            self.blocks.push(finish.take().expect("one structured finish")).expect("admitted structured block");
        }
        Ok(())
    }
}

fn structured_call(block: &llm::Said) -> bool {
    match block {
        llm::Said::ToolCall { id, name, .. } => {
            id.as_ref() == STRUCTURED_FINISH && name.as_ref() == b"StructuredOutput"
        }
        llm::Said::Text { .. } | llm::Said::Refusal { .. } | llm::Said::Opaque { .. } => false,
    }
}

/// Refuse a measured admission value strictly above its configured bound.
pub(super) fn check_bound(which: Bound, value: u64, bound: u64) -> Result<(), Rejection> {
    if value > bound { Err(Rejection::Limit { which, value, bound }) } else { Ok(()) }
}

fn payload(block: &llm::Said) -> Result<u64, Rejection> {
    let length = match block {
        llm::Said::Text { text, .. } | llm::Said::Refusal { text, .. } => text.len(),
        llm::Said::ToolCall { id, name, input, .. } => id
            .len()
            .checked_add(name.len())
            .ok_or(Rejection::Overflow)?
            .checked_add(input.len())
            .ok_or(Rejection::Overflow)?,
        llm::Said::Opaque { bytes } => bytes.len(),
    };
    Ok(skein_lib::len_u64(length))
}

fn block(document: &Document, index: u32, maximum: u32, finish: &mut bool) -> Result<Option<llm::Said>, Rejection> {
    let kind = json::text_in(document, json::field(document, index, b"type")).ok_or(Rejection::Malformed)?;
    match kind {
        b"text" => {
            let text = json::text_in(document, json::field(document, index, b"text")).ok_or(Rejection::Malformed)?;
            Ok(Some(llm::Said::Text { text: text.into(), replay: None }))
        }
        b"tool_use" => tool_block(document, index, maximum, finish),
        b"thinking" | b"redacted_thinking" => Ok(None),
        _ => Err(Rejection::Unsupported),
    }
}

fn tool_block(
    document: &Document,
    index: u32,
    maximum: u32,
    finish: &mut bool,
) -> Result<Option<llm::Said>, Rejection> {
    let name = json::text_in(document, json::field(document, index, b"name")).ok_or(Rejection::Malformed)?;
    let builtin = !name.starts_with(b"mcp__");
    let name = name.strip_prefix(b"mcp__smith__").unwrap_or(name);
    let input_index = json::field(document, index, b"input").ok_or(Rejection::Malformed)?;
    let input = bounded_input(document, input_index, maximum)?;
    if !builtin && name == b"finish" {
        *finish = true;
    }
    let provider_id = json::text_in(document, json::field(document, index, b"id")).ok_or(Rejection::Malformed)?;
    Ok(Some(llm::Said::ToolCall {
        id: if builtin && name == b"StructuredOutput" { STRUCTURED_FINISH.into() } else { provider_id.into() },
        name: name.into(),
        input,
        call: if builtin && name != b"StructuredOutput" {
            llm::Decoded::Builtin {
                effect: match super::policy::Builtin::named(name) {
                    Some(tool) => tool.effect(),
                    None => smith_domain::tools::Effect::Write,
                },
            }
        } else {
            llm::Decoded::Invalid { problem: llm::Problem::UnknownTool }
        },
        replay: None,
    }))
}

/// Measure canonical tool input before allocating it under its receiving bound.
pub(super) fn bounded_input(document: &Document, index: u32, maximum: u32) -> Result<Box<[u8]>, Rejection> {
    let length = json::raw_length(document, index).ok_or(Rejection::Malformed)?;
    check_bound(Bound::Input, u64::from(length), u64::from(maximum))?;
    json::raw(document, index, maximum).ok_or(Rejection::Malformed)
}

impl Turns {
    pub fn open_blocks(&self) -> Option<&[llm::Said]> {
        match &self.message {
            Some(message) => Some(message.blocks.as_slice()),
            None => None,
        }
    }

    pub fn new(bounds: &smith_protocol_llm::Receiving, maximum_raw: u32) -> Self {
        Self {
            message: None,
            ready: Queue::with_capacity(4),
            result: false,
            assistant_events: 0,
            closed: 0,
            finish: false,
            blocks: bounds.max_completion_blocks,
            maximum: bounds.max_completion_bytes,
            maximum_raw,
        }
    }

    fn begin(&mut self, id: &[u8], framed: bool) -> Result<(), Rejection> {
        if id.len() > 1024 {
            return Err(Rejection::Limit { which: Bound::MessageId, value: skein_lib::len_u64(id.len()), bound: 1024 });
        }
        if self.message.is_some() {
            return Err(Rejection::Boundary);
        }
        if self.result {
            self.finish = false;
        }
        self.result = false;
        self.message = Some(Message {
            id: id.into(),
            framed,
            stopped: false,
            expected: 0,
            received: 0,
            blocks: List::with_capacity(self.blocks),
            usage: llm::Usage::NONE,
            bytes: 0,
            raw: 0,
        });
        Ok(())
    }

    pub fn event(&mut self, document: &Document, event: u32) -> Result<(), Rejection> {
        match json::text_in(document, json::field(document, event, b"type")).ok_or(Rejection::Malformed)? {
            b"message_start" => {
                self.close(true)?;
                let id =
                    json::text_in(document, json::field_in(document, json::field(document, event, b"message"), b"id"))
                        .ok_or(Rejection::Malformed)?;
                self.begin(id, true)
            }
            b"content_block_start" => {
                let index = number(document, json::field(document, event, b"index")).ok_or(Rejection::Malformed)?;
                let expected = u32::try_from(index)
                    .ok()
                    .ok_or(Rejection::Malformed)?
                    .checked_add(1)
                    .ok_or(Rejection::Malformed)?;
                if expected > self.blocks {
                    return Err(Rejection::Limit {
                        which: Bound::Blocks,
                        value: u64::from(expected),
                        bound: u64::from(self.blocks),
                    });
                }
                let message = self.message.as_mut().ok_or(Rejection::Boundary)?;
                message.expected = message.expected.max(expected);
                Ok(())
            }
            b"message_stop" => {
                self.message.as_mut().ok_or(Rejection::Boundary)?.stopped = true;
                self.close(false)
            }
            b"message_delta" => {
                // The block records repeat the initial usage; the final delta
                // supplies the message's total, including its thinking tokens.
                if let Some(tokens) = json::field(document, event, b"usage") {
                    self.message.as_mut().ok_or(Rejection::Boundary)?.usage = usage(document, Some(tokens));
                }
                Ok(())
            }
            b"content_block_delta" | b"content_block_stop" => Ok(()),
            _ => Err(Rejection::Unsupported),
        }
    }

    pub fn assistant(&mut self, document: &Document, maximum: u32) -> Result<u32, Rejection> {
        self.assistant_events = self.assistant_events.checked_add(1).ok_or(Rejection::Overflow)?;
        let value = json::field(document, 0, b"message").ok_or(Rejection::Malformed)?;
        let id = json::text_in(document, json::field(document, value, b"id")).ok_or(Rejection::Malformed)?;
        // smith/docs/design/protocol/programs.md 5.5: an assistant ID without message_start is an
        // atomic program-authored message. Framed IDs still wait for stop
        // and every block, including when the full content precedes stop.
        if let Some(message) = &self.message
            && message.id.as_ref() != id
        {
            self.close(true)?;
        }
        if self.message.is_none() {
            self.begin(id, false)?;
        }
        let message = self.message.as_mut().ok_or(Rejection::Boundary)?;
        if message.id.as_ref() != id {
            return Err(Rejection::Malformed);
        }
        let content = json::field(document, value, b"content").ok_or(Rejection::Malformed)?;
        if document.token(content).ok_or(Rejection::Malformed)?.kind != Kind::ArrayStart {
            return Err(Rejection::Malformed);
        }
        let text_bytes = message.content(
            document,
            content,
            AssemblyBounds { blocks: self.blocks, raw: self.maximum_raw, bytes: self.maximum, input: maximum },
            &mut self.finish,
        )?;
        message.usage = usage(document, json::field(document, value, b"usage"));
        if !message.framed {
            message.stopped = true;
            message.expected = message.received;
        }
        self.close(false)?;
        Ok(text_bytes)
    }

    pub fn close(&mut self, boundary: bool) -> Result<(), Rejection> {
        let Some(message) = &self.message else {
            return Ok(());
        };
        if !message.stopped || message.received < message.expected {
            return if boundary { Err(Rejection::Boundary) } else { Ok(()) };
        }
        let mut calls = false;
        let mut structured = false;
        for block in &message.blocks {
            match block {
                llm::Said::ToolCall { .. } => {
                    calls = true;
                    structured |= structured_call(block);
                }
                llm::Said::Text { .. } | llm::Said::Refusal { .. } | llm::Said::Opaque { .. } => {}
            }
        }
        // StructuredOutput's result supplies the finish input for this same
        // model completion (smith/docs/design/protocol/programs.md 5.5), never an extra budgeted turn.
        if structured {
            return if boundary { Err(Rejection::StructuredBoundary) } else { Ok(()) };
        }
        if !calls && !boundary {
            return Ok(());
        }
        self.completion_room()?;
        let message = self.message.take().ok_or(Rejection::Malformed)?;
        self.closed = self.closed.checked_add(1).ok_or(Rejection::Overflow)?;
        self.ready.push(llm::Completion {
            reasoning_dropped: Box::default(),
            content: message.blocks.into_boxed(),
            stop: if calls { llm::Stop::ToolUse } else { llm::Stop::EndTurn },
            usage: message.usage,
        });
        Ok(())
    }

    fn completion_room(&self) -> Result<(), Rejection> {
        if self.ready.room() == 0 {
            let bound = u64::from(self.ready.len());
            return Err(Rejection::Backlog { value: bound.saturating_add(1), bound });
        }
        Ok(())
    }

    // smith/docs/design/protocol/programs.md 5.5: a successful result without an assistant still hands
    // the no-tool turn to the domain, which owns its nudge allowance.
    pub fn end_empty(&mut self) -> Result<(), Rejection> {
        if self.assistant_events == 0 && self.ready.is_empty() {
            self.completion_room()?;
            self.closed = self.closed.checked_add(1).ok_or(Rejection::Overflow)?;
            self.ready.push(llm::Completion {
                reasoning_dropped: Box::default(),
                content: Box::default(),
                stop: llm::Stop::EndTurn,
                usage: llm::Usage::ZERO,
            });
        }
        Ok(())
    }

    pub fn interrupted(&mut self) {
        if self.close(true).is_err() {
            self.message = None;
        }
        if self.ready.is_empty() {
            self.assistant_events = 0;
            self.end_empty().expect("one interrupt terminal has completion room");
        }
        self.result = true;
    }

    pub fn accepting_calls(&self) -> bool {
        match &self.message {
            Some(message) => !message.stopped || message.received < message.expected,
            // A built-in result can start the next MCP call before stdout
            // announces its assistant record. The enclosing result ends this
            // bounded correlation window; closed text still rejects above.
            None => !self.result,
        }
    }

    pub fn matching_floor(&self) -> Option<u64> {
        if self.message.is_some() && !self.accepting_calls() { self.closed.checked_add(1) } else { Some(self.closed) }
    }

    pub fn retry(&mut self) {
        self.message = None;
        self.restart();
    }

    pub fn restart(&mut self) {
        self.result = false;
        self.finish = false;
        self.assistant_events = 0;
    }

    pub fn structured(&mut self, input: Box<[u8]>) -> Result<(), Rejection> {
        if self.finish {
            return self.close(true);
        }
        let block = llm::Said::ToolCall {
            id: STRUCTURED_FINISH.into(),
            name: b"finish".as_slice().into(),
            input,
            call: llm::Decoded::Invalid { problem: llm::Problem::UnknownTool },
            replay: None,
        };
        if let Some(message) = &mut self.message {
            message.structured(block, self.blocks)?;
            self.close(true)?;
        } else {
            self.completion_room()?;
            self.closed = self.closed.checked_add(1).ok_or(Rejection::Overflow)?;
            self.ready.push(llm::Completion {
                reasoning_dropped: Box::default(),
                content: Box::new([block]),
                stop: llm::Stop::ToolUse,
                usage: llm::Usage::ZERO,
            });
        }
        self.finish = true;
        Ok(())
    }
}

pub(super) fn number(document: &Document, index: Option<u32>) -> Option<u64> {
    let record = document.token(index?)?;
    if record.kind != Kind::Number {
        return None;
    }
    skein_lib::bytes::decimal(document.text(record)?)
}

fn usage(document: &Document, index: Option<u32>) -> llm::Usage {
    llm::Usage {
        input_tokens: number(document, json::field_in(document, index, b"input_tokens")),
        output_tokens: number(document, json::field_in(document, index, b"output_tokens")),
        cache_read_tokens: number(document, json::field_in(document, index, b"cache_read_input_tokens")),
        cache_write_tokens: number(document, json::field_in(document, index, b"cache_creation_input_tokens")),
        reasoning_tokens: None,
    }
}

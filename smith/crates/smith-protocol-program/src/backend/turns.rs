//! API message assembly (protocol/programs.md, section 5.5). Keeps one
//! message and four waiting completions; never derives usage from a result.
//! `StructuredOutput` waits for its outcome and becomes finish in the same slot;
//! it is never a workspace built-in (protocol/programs.md, section 5.5).

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
    pub problem: &'static [u8],
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
            problem: b"assembly has no rejection",
        }
    }

    fn begin(&mut self, id: &[u8], framed: bool) -> Option<()> {
        self.problem = b"message id exceeds 1024 bytes";
        if id.len() > 1024 {
            return None;
        }
        self.problem = b"a message is already open";
        if self.message.is_some() {
            return None;
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
        Some(())
    }

    pub fn event(&mut self, document: &Document, event: u32) -> Option<()> {
        self.problem = b"event has no string type";
        match json::text_in(document, json::field(document, event, b"type"))? {
            b"message_start" => {
                self.close(true)?;
                self.problem = b"message_start has no string message id";
                let id =
                    json::text_in(document, json::field_in(document, json::field(document, event, b"message"), b"id"))?;
                self.begin(id, true)
            }
            b"content_block_start" => {
                self.problem = b"content_block_start index is missing or not an integer";
                let index = number(document, json::field(document, event, b"index"))?;
                self.problem = b"content_block_start index exceeds u32 block range";
                let expected = u32::try_from(index).ok()?.checked_add(1)?;
                self.problem = b"content_block_start exceeds block bound";
                if expected > self.blocks {
                    return None;
                }
                self.problem = b"content_block_start has no open message";
                let message = self.message.as_mut()?;
                message.expected = message.expected.max(expected);
                Some(())
            }
            b"message_stop" => {
                self.problem = b"message_stop has no open message";
                self.message.as_mut()?.stopped = true;
                self.close(false)
            }
            b"message_delta" => {
                // The block records repeat the initial usage; the final delta
                // supplies the message's total, including its thinking tokens.
                if let Some(tokens) = json::field(document, event, b"usage") {
                    self.problem = b"message_delta usage has no open message";
                    self.message.as_mut()?.usage = usage(document, Some(tokens));
                }
                Some(())
            }
            b"content_block_delta" | b"content_block_stop" => Some(()),
            _ => {
                self.problem = b"unknown stream event type";
                None
            }
        }
    }

    pub fn assistant(&mut self, document: &Document, maximum: u32) -> Option<u32> {
        self.problem = b"assistant count overflow";
        self.assistant_events = self.assistant_events.checked_add(1)?;
        self.problem = b"assistant has no message";
        let value = json::field(document, 0, b"message")?;
        self.problem = b"assistant has no string message id";
        let id = json::text_in(document, json::field(document, value, b"id"))?;
        if let Some(message) = &self.message
            && message.id.as_ref() != id
        {
            self.close(true)?;
        }
        if self.message.is_none() {
            self.begin(id, false)?;
        }
        self.problem = b"assistant message id differs from open message";
        if self.message.as_ref()?.id.as_ref() != id {
            return None;
        }
        self.problem = b"assistant content is not an array";
        let content = json::field(document, value, b"content")?;
        if document.token(content)?.kind != Kind::ArrayStart {
            return None;
        }
        let text_bytes = self.content(document, content, maximum)?;
        let message = self.message.as_mut().expect("assembled message");
        message.usage = usage(document, json::field(document, value, b"usage"));
        if !message.framed {
            message.stopped = true;
            message.expected = message.received;
        }
        self.close(false)?;
        Some(text_bytes)
    }

    fn content(&mut self, document: &Document, content: u32, maximum: u32) -> Option<u32> {
        self.problem = b"assistant content token is missing";
        let mut index = content.checked_add(1)?;
        let mut text_bytes = 0_u32;
        for _ in 0..document.len() {
            self.problem = b"assistant content token is missing";
            if document.token(index)?.kind == Kind::ArrayEnd {
                break;
            }
            self.problem = b"assistant exceeds block bound";
            let message = self.message.as_mut().expect("content has a message");
            message.received = message.received.checked_add(1)?;
            if message.received > self.blocks {
                return None;
            }
            let block = match decode_block(document, index, maximum, &mut self.finish) {
                Ok(block) => block,
                Err(problem) => {
                    self.problem = problem;
                    return None;
                }
            };
            if let Some(block) = block {
                match &block {
                    llm::Said::Text { text, .. } => {
                        self.problem = b"assistant text byte count overflow";
                        text_bytes = text_bytes.checked_add(u32::try_from(text.len()).ok()?)?;
                    }
                    llm::Said::ToolCall { .. } | llm::Said::Refusal { .. } | llm::Said::Opaque { .. } => {}
                }
                match retain_block(message, block, self.maximum_raw, self.maximum) {
                    Ok(()) => {}
                    Err(problem) => {
                        self.problem = problem;
                        return None;
                    }
                }
            }
            self.problem = b"assistant block has no closing token";
            index = json::after(document, index)?;
        }
        Some(text_bytes)
    }

    pub fn close(&mut self, boundary: bool) -> Option<()> {
        let Some(message) = &self.message else {
            return Some(());
        };
        if !message.stopped || message.received < message.expected {
            self.problem =
                if message.stopped { b"boundary before all assistant blocks" } else { b"boundary before message_stop" };
            return if boundary { None } else { Some(()) };
        }
        let mut calls = false;
        let mut structured = false;
        for block in &message.blocks {
            match block {
                llm::Said::ToolCall { id, name, .. } => {
                    calls = true;
                    structured |= id.as_ref() == STRUCTURED_FINISH && name.as_ref() == b"StructuredOutput";
                }
                llm::Said::Text { .. } | llm::Said::Refusal { .. } | llm::Said::Opaque { .. } => {}
            }
        }
        // StructuredOutput's result supplies the finish input for this same
        // model completion (smith/docs/design/protocol/programs.md 5.5), never an extra budgeted turn.
        if structured && !boundary {
            return Some(());
        }
        if !calls && !boundary {
            return Some(());
        }
        self.problem = b"completion backlog is full";
        if self.ready.room() == 0 {
            return None;
        }
        let message = self.message.take().expect("closing message");
        self.problem = b"closed message count overflow";
        self.closed = self.closed.checked_add(1)?;
        self.ready.push(llm::Completion {
            reasoning_dropped: Box::default(),
            content: message.blocks.into_boxed(),
            stop: if calls { llm::Stop::ToolUse } else { llm::Stop::EndTurn },
            usage: message.usage,
        });
        Some(())
    }

    // smith/docs/design/protocol/programs.md 5.5: a successful result without an assistant still hands
    // the no-tool turn to the domain, which owns its nudge allowance.
    pub fn end_empty(&mut self) -> Option<()> {
        if self.assistant_events == 0 && self.ready.is_empty() {
            self.problem = b"completion backlog is full";
            if self.ready.room() == 0 {
                return None;
            }
            self.problem = b"closed message count overflow";
            self.closed = self.closed.checked_add(1)?;
            self.ready.push(llm::Completion {
                reasoning_dropped: Box::default(),
                content: Box::default(),
                stop: llm::Stop::EndTurn,
                usage: llm::Usage::ZERO,
            });
        }
        Some(())
    }

    pub fn interrupted(&mut self) {
        if self.close(true).is_none() {
            self.message = None;
        }
        self.discard_outcome();
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
        self.discard_outcome();
        self.restart();
    }

    fn discard_outcome(&mut self) {
        let mut discard = false;
        for _ in 0..self.ready.len() {
            let completion = self.ready.pop().expect("queued completion");
            for block in &completion.content {
                discard |= unresolved(block);
            }
            if discard {
                self.closed = self.closed.checked_sub(1).expect("discarded queued message");
            } else {
                self.ready.push(completion);
            }
        }
    }

    pub fn restart(&mut self) {
        self.result = false;
        self.finish = false;
        self.assistant_events = 0;
    }

    pub fn structured(&mut self, input: Box<[u8]>) -> Option<()> {
        if self.finish {
            return self.close(true);
        }
        // Closing at the result puts any trailing message behind the held slot.
        let attach = self.message.is_some();
        self.close(true)?;
        let mut count = 0_u32;
        for completion in &self.ready {
            for block in &completion.content {
                if unresolved(block) {
                    count = count.saturating_add(1);
                }
            }
        }
        self.problem = b"multiple StructuredOutput calls await one outcome";
        if count > 1 {
            return None;
        }
        if count == 1 {
            for _ in 0..self.ready.len() {
                let mut completion = self.ready.pop().expect("queued completion");
                for block in &mut completion.content {
                    if unresolved(block) {
                        *block = finish_block(input.clone());
                    }
                }
                self.problem = b"structured outcome exceeds completion receiving bound";
                let valid = completion_fits(&completion, self.maximum_raw, self.maximum);
                self.ready.push(completion);
                valid?;
            }
        } else {
            self.synthetic_finish(input, attach)?;
        }
        self.finish = true;
        Some(())
    }

    fn synthetic_finish(&mut self, input: Box<[u8]>, attach: bool) -> Option<()> {
        // A result without StructuredOutput attaches to the last text message,
        // or supplies a zero-usage completion when no assistant was sent.
        let last = if attach { self.ready.len().checked_sub(1) } else { None };
        if let Some(last) = last {
            for index in 0..self.ready.len() {
                let mut completion = self.ready.pop().expect("queued completion");
                if index == last {
                    let mut blocks = List::with_capacity(self.blocks);
                    self.problem = b"structured outcome exceeds block bound";
                    for block in completion.content {
                        blocks.push(block).ok()?;
                    }
                    blocks.push(finish_block(input.clone())).ok()?;
                    completion.content = blocks.into_boxed();
                    completion.stop = llm::Stop::ToolUse;
                    self.problem = b"structured outcome exceeds completion receiving bound";
                    completion_fits(&completion, self.maximum_raw, self.maximum)?;
                }
                self.ready.push(completion);
            }
        } else {
            self.problem = b"completion backlog is full";
            if self.ready.room() == 0 {
                return None;
            }
            let completion = llm::Completion {
                reasoning_dropped: Box::default(),
                content: Box::new([finish_block(input)]),
                stop: llm::Stop::ToolUse,
                usage: llm::Usage::ZERO,
            };
            self.problem = b"structured outcome exceeds completion receiving bound";
            completion_fits(&completion, self.maximum_raw, self.maximum)?;
            self.problem = b"closed message count overflow";
            self.closed = self.closed.checked_add(1)?;
            self.ready.push(completion);
        }
        Some(())
    }

    /// Whether the first completion has its outcome and can keep its owner.
    pub fn deliverable(&self) -> bool {
        match self.ready.iter().next() {
            Some(completion) => {
                for block in &completion.content {
                    if unresolved(block) {
                        return false;
                    }
                }
                true
            }
            None => false,
        }
    }
}

fn unresolved(block: &llm::Said) -> bool {
    match block {
        llm::Said::ToolCall { id, name, .. } => {
            id.as_ref() == STRUCTURED_FINISH && name.as_ref() == b"StructuredOutput"
        }
        llm::Said::Text { .. } | llm::Said::Refusal { .. } | llm::Said::Opaque { .. } => false,
    }
}

fn finish_block(input: Box<[u8]>) -> llm::Said {
    llm::Said::ToolCall {
        id: STRUCTURED_FINISH.into(),
        name: b"finish".as_slice().into(),
        input,
        call: llm::Decoded::Invalid { problem: llm::Problem::UnknownTool },
        replay: None,
    }
}

fn decode_block(
    document: &Document,
    index: u32,
    maximum: u32,
    finish: &mut bool,
) -> Result<Option<llm::Said>, &'static [u8]> {
    let kind = json::text_in(document, json::field(document, index, b"type"))
        .ok_or(b"assistant block has no string type".as_slice())?;
    match kind {
        b"text" => {
            let text = json::text_in(document, json::field(document, index, b"text"))
                .ok_or(b"text block has no string text".as_slice())?;
            Ok(Some(llm::Said::Text { text: text.into(), replay: None }))
        }
        b"tool_use" => decode_tool(document, index, maximum, finish),
        b"thinking" | b"redacted_thinking" => Ok(None),
        _ => Err(b"unknown assistant block type"),
    }
}

fn decode_tool(
    document: &Document,
    index: u32,
    maximum: u32,
    finish: &mut bool,
) -> Result<Option<llm::Said>, &'static [u8]> {
    let name = json::text_in(document, json::field(document, index, b"name"))
        .ok_or(b"tool_use has no string name".as_slice())?;
    let builtin = !name.starts_with(b"mcp__");
    let name = name.strip_prefix(b"mcp__smith__").unwrap_or(name);
    let position = json::field(document, index, b"input").ok_or(b"tool_use has no input".as_slice())?;
    let input =
        json::raw(document, position, maximum).ok_or(b"tool_use input exceeds bound or is malformed".as_slice())?;
    if !builtin && name == b"finish" {
        *finish = true;
    }
    let provider_id =
        json::text_in(document, json::field(document, index, b"id")).ok_or(b"tool_use has no string id".as_slice())?;
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

fn retain_block(message: &mut Message, block: llm::Said, maximum_raw: u32, maximum: u64) -> Result<(), &'static [u8]> {
    let payload = block_bytes(&block).ok_or(b"block byte count overflow".as_slice())?;
    message.raw = message
        .raw
        .checked_add(u32::try_from(payload).ok().ok_or(b"block byte count exceeds u32".as_slice())?)
        .ok_or(b"raw byte count overflow".as_slice())?;
    if message.raw > maximum_raw {
        return Err(b"assistant exceeds raw byte bound");
    }
    message.bytes = message
        .bytes
        .checked_add(
            u64::try_from(
                size_of::<llm::Said>().checked_add(payload).ok_or(b"block storage count overflow".as_slice())?,
            )
            .ok()
            .ok_or(b"block storage count exceeds u64".as_slice())?,
        )
        .ok_or(b"completion byte count overflow".as_slice())?;
    if message.bytes > maximum {
        return Err(b"assistant exceeds completion byte bound");
    }
    message.blocks.push(block).ok().ok_or(b"assistant exceeds block bound")
}

fn block_bytes(block: &llm::Said) -> Option<usize> {
    match block {
        llm::Said::Text { text, .. } | llm::Said::Refusal { text, .. } => Some(text.len()),
        llm::Said::ToolCall { id, name, input, .. } => id.len().checked_add(name.len())?.checked_add(input.len()),
        llm::Said::Opaque { bytes } => Some(bytes.len()),
    }
}

fn completion_fits(completion: &llm::Completion, raw_limit: u32, limit: u64) -> Option<()> {
    let mut raw = 0_u64;
    let mut bytes = 0_u64;
    for block in &completion.content {
        let payload = u64::try_from(block_bytes(block)?).ok()?;
        raw = raw.checked_add(payload)?;
        bytes = bytes.checked_add(u64::try_from(size_of::<llm::Said>()).ok()?)?.checked_add(payload)?;
    }
    if raw > u64::from(raw_limit) || bytes > limit {
        return None;
    }
    Some(())
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

//! Conversation boundary driving (protocol/programs.md, sections 5.5–5.7).
//! Keeps correlation separately from delivered domain blocks; the domain alone
//! accepts outcomes; cancellation fails unanswered calls before settling the child.

use super::{Backend, Below, State, calls, stream, turns};
use super::{Complete, Rejection};
use crate::mcp::{self, json};
use alloc::boxed::Box;
use skein_json::Kind;
use skein_lib::{Env, List, Queue};
use smith_domain::{llm, run};
use smith_protocol_llm::{self as protocol, ToDomain};

impl Backend {
    pub(super) fn resume(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        request: Complete,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let owner = request.owner;
        self.begin_prompt(request.observation.is_some());
        self.current.claim(owner, up);
        if self.process.exit().is_some() || self.process.child().is_none() || self.process.pipes().input.is_none() {
            self.fail_program(env, b"smith: the program is no longer accepting input", up, below);
            return;
        }
        self.permitted_message = self
            .turns
            .closed
            .checked_sub(u64::from(self.turns.ready.len()))
            .expect("delivered prefix")
            .checked_add(1)
            .expect("permitted message");
        let Some(mut request) = self.wind_down(env, request, up, below) else {
            return;
        };
        let (prompt, capture) = (&mut request.prompt, request.observation.is_some());
        let messages = core::mem::replace(&mut prompt.messages, Box::new([]));
        let Some(resumed) = super::replies::resume(self, messages, prompt, capture) else {
            self.fail(
                env,
                llm::Failure::Invalid,
                b"smith: a program tool result exceeds its receiving bound",
                up,
                below,
            );
            return;
        };
        self.current.retain(request, up);
        let mut sent = resumed.observed;
        self.completion_deadline(env, true, up);
        self.answer_calls(below);
        self.match_calls(below);
        // A later API message may already be in flight. Only a completed
        // program turn needs the domain's new text (message or nudge) as a user line.
        if let Some(line) = resumed.line {
            if self.turns.result && self.turns.ready.is_empty() && self.turns.message.is_none() {
                self.outgoing = Some(line);
                self.turns.restart();
                self.send_input(below);
            } else {
                sent = None;
            }
        }
        self.prompt_measure(env.now, sent.as_deref());
        self.deliver(env, up, below);
        self.continue_ended(env, up, below);
    }

    pub(super) fn line(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        bytes: &[u8],
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let Some(document) =
            stream::collect(bytes, self.settings.translation.client.depth, self.settings.stream_tokens)
        else {
            self.fail_program(env, b"smith: the program stream does not decode", up, below);
            return;
        };
        let kind = json::text_in(&document, json::field(&document, 0, b"type"));
        if kind == Some(b"control_response") {
            if !self.control_response(&document, below) {
                self.fail(env, llm::Failure::Invalid, b"smith: the program refused the interrupt", up, below);
            }
            return;
        }
        if self.stopping.waiting() {
            if kind == Some(b"result") {
                // Note rendering reuses the parser's workspace term.
                drop(document);
                self.stopped_result(env, up, below);
                self.deliver(env, up, below);
            }
            return;
        }
        self.conversation_line(env, document, bytes, up, below);
    }

    fn conversation_line(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        document: skein_json::Document,
        bytes: &[u8],
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let kind = json::text_in(&document, json::field(&document, 0, b"type"));
        if self.startup.switching() {
            return;
        }
        if kind == Some(b"system") && json::text_in(&document, json::field(&document, 0, b"subtype")) == Some(b"init") {
            drop(document);
            self.later_init(env, bytes, up, below);
            return;
        }
        let result = self.record(env, &document, kind, up, below);
        match result {
            Ok(true) => {}
            Ok(false) => return,
            Err(reason) => {
                self.reject_record(env, reason, record_name(kind), up, below);
                return;
            }
        }
        if self.register_calls().is_none() {
            self.fail(env, llm::Failure::Invalid, b"smith: program call ledger is full", up, below);
            return;
        }
        self.match_calls(below);
        self.completion_deadline(env, false, up);
        self.deliver(env, up, below);
        self.continue_ended(env, up, below);
    }

    fn reject_record(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        reason: Rejection,
        record: &[u8],
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        let failure = match reason {
            Rejection::Backlog { .. } => llm::Failure::Invalid,
            Rejection::Malformed
            | Rejection::Unsupported
            | Rejection::Boundary
            | Rejection::StructuredBoundary
            | Rejection::Limit { .. }
            | Rejection::Overflow => llm::Failure::Program { exit: self.process.exit() },
        };
        let detail = reason.detail(record);
        self.fail(env, failure, &detail, up, below);
    }

    fn record(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        document: &skein_json::Document,
        kind: Option<&[u8]>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) -> Result<bool, Rejection> {
        match kind {
            Some(b"stream_event") => {
                self.flush_prompt_measure(env.now);
                let event = json::field(document, 0, b"event").ok_or(Rejection::Malformed)?;
                self.partial_measure(env.now, document, event)?;
                self.turns.event(document, event)?;
            }
            Some(b"assistant") => {
                self.turns.assistant(document, self.settings.translation.client.input)?;
            }
            Some(b"result") => return self.result_record(env, document, up, below),
            Some(b"user") => self.builtin_results(document, up)?,
            Some(b"system") => return self.system_record(env.now, document),
            Some(b"rate_limit_event") => {
                let info = json::field(document, 0, b"rate_limit_info");
                if json::text_in(document, json::field_in(document, info, b"status")) == Some(b"rejected".as_slice()) {
                    let retry_after = retry_after(document, env.wall);
                    self.fail(env, llm::Failure::Exhausted { retry_after }, b"smith: program usage limit", up, below);
                    return Ok(false);
                }
            }
            Some(_) => return Ok(false),
            None => return Err(Rejection::Malformed),
        }
        Ok(true)
    }

    fn result_record(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        document: &skein_json::Document,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) -> Result<bool, Rejection> {
        if let Some((failure, status)) = result_failure(document, env.wall) {
            self.turns.result = true;
            let report = llm::Report { status, ..llm::Report::none() };
            self.fail_report(env, failure, b"smith: the program reported a failed result", report, up, below);
            return Ok(false);
        }
        if self.turns.result {
            self.fail_program(env, b"smith: a program turn repeated its result", up, below);
            return Ok(false);
        }
        self.turns.result = true;
        match json::field(document, 0, b"structured_output") {
            Some(index) if self.result_contract() => {
                let input = json::outcome_value(document, index).ok_or(Rejection::Malformed)?;
                let input = turns::bounded_input(document, input, self.settings.translation.client.input)?;
                self.turns.structured(input)?;
            }
            Some(_) | None => {
                self.turns.close(true)?;
                if !self.turns.deliverable() && !self.turns.ready.is_empty() {
                    return Err(Rejection::StructuredBoundary);
                }
            }
        }
        self.turns.end_empty()?;
        Ok(true)
    }

    fn system_record(&mut self, at: skein_lib::Time, document: &skein_json::Document) -> Result<bool, Rejection> {
        match json::text_in(document, json::field(document, 0, b"subtype")) {
            Some(b"api_retry") => {
                self.retry_measure(at, document);
                self.turns.retry();
                let old = core::mem::replace(
                    &mut self.ledger,
                    List::with_capacity(self.settings.ledger_slots().expect("call slots")),
                );
                for call in old.into_boxed() {
                    if call.message <= self.turns.closed {
                        assert!(self.ledger.push(call).is_ok(), "retained delivered calls");
                    }
                }
            }
            Some(b"compact_boundary" | b"compaction") => {}
            Some(_) => return Ok(false),
            None => return Err(Rejection::Malformed),
        }
        Ok(true)
    }

    fn continue_ended(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        if self.current.owner().is_some()
            && self.turns.result
            && self.turns.ready.is_empty()
            && self.turns.message.is_none()
        {
            if self.process.exit().is_some() || self.process.child().is_none() || self.process.pipes().input.is_none() {
                self.fail_program(env, b"smith: the program is no longer accepting input", up, below);
                return;
            }
            // Complete already asks for the next API message. The preceding
            // tool message was delivered; resume the program without telling
            // the domain another message or spending a no-tool nudge.
            self.outgoing = stream::text_message(b"Continue with the tool results.", self.settings.input_bytes);
            if self.outgoing.is_none() {
                self.fail(env, llm::Failure::Invalid, b"smith: program input exceeds its bound", up, below);
                return;
            }
            self.prompt_measure(env.now, Some(b"Continue with the tool results."));
            self.turns.restart();
            self.completion_deadline(env, true, up);
            self.send_input(below);
        }
    }

    fn register_calls(&mut self) -> Option<()> {
        let mut message = self.turns.closed.checked_sub(u64::from(self.turns.ready.len()))?.checked_add(1)?;
        for _ in 0..self.turns.ready.len() {
            let completion = self.turns.ready.pop()?;
            let result = register_blocks(&mut self.ledger, &completion.content, message);
            self.turns.ready.push(completion);
            result?;
            message = message.checked_add(1)?;
        }
        if let Some(blocks) = self.turns.open_blocks() {
            register_blocks(&mut self.ledger, blocks, self.turns.closed.checked_add(1)?)?;
        }
        let floor = self.turns.matching_floor()?;
        for index in 0..self.ledger.len() {
            let call = self.ledger.get_mut(index)?;
            call.eligible = call.message >= floor;
        }
        if !self.reply_room(0) {
            return None;
        }
        Some(())
    }

    pub(super) fn deliver(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        if self.native.is_some() {
            return;
        }
        if self.startup.switching() {
            return;
        }
        let Some(owner) = self.current.owner() else {
            return;
        };
        if !self.turns.ready.is_empty() {
            self.flush_deltas(env.now);
            self.flush_prompt_measure(env.now);
            if !self.measures.is_empty() {
                self.intake_due = Some(env.now);
                return;
            }
        }
        if !self.turns.deliverable() {
            return;
        }
        let Some(mut completion) = self.turns.ready.pop() else {
            return;
        };
        self.retain_calls();
        if let Err(reason) = self.translate_completion(&mut completion) {
            self.fail(env, llm::Failure::Invalid, &reason.detail(b"assistant"), up, below);
            return;
        }
        let text = completion_text(&completion);
        self.match_calls(below);
        self.stopping.release_alarm();
        if text > 0 {
            up.push(ToDomain::Text { owner, bytes: text });
        }
        self.completion_measure(env.now, owner, &completion);
        self.terminal_measure(env.now, owner);
        self.builtin_owners(owner, &completion);
        self.current.settle(super::current::Terminal::Completed { completion }, up);
        self.flush_builtins(up);
        if self.compaction() {
            self.startup.switch(&mut self.server_up, below);
            self.stop(env, false, false, up, below);
        }
    }

    fn translate_completion(&self, completion: &mut llm::Completion) -> Result<(), Rejection> {
        let Some(Complete { prompt, outcome, deliver, .. }) = self.current.request() else {
            unreachable!("active completion contract");
        };
        let mut decoded = 0_u64;
        for block in &mut completion.content {
            match block {
                llm::Said::ToolCall { id, name, input, call, .. } => {
                    match call {
                        llm::Decoded::Builtin { .. } => {}
                        llm::Decoded::Owned { .. } | llm::Decoded::Served { .. } | llm::Decoded::Invalid { .. } => {
                            *call = decode_call(
                                id,
                                name,
                                input,
                                prompt,
                                outcome,
                                deliver.as_ref(),
                                &self.settings.translation,
                            );
                        }
                    }
                    decoded = match call.owned_bytes() {
                        Some(bytes) => decoded.checked_add(bytes),
                        None => None,
                    }
                    .unwrap_or(u64::MAX);
                    if decoded > self.settings.receiving.decoded_call_bytes {
                        *call = llm::Decoded::Invalid { problem: llm::Problem::TooLarge };
                    }
                }
                llm::Said::Text { text, .. } | llm::Said::Refusal { text, .. } => self.redact_private(text),
                llm::Said::Opaque { .. } => {}
            }
        }
        completion_room(completion, self.settings.receiving.max_completion_bytes)
    }

    fn retain_calls(&mut self) {
        // Retain unanswered calls while their correlation window or RPC lives.
        // Dropping a call also forgets its allowed native write snapshot.
        let old = core::mem::replace(
            &mut self.ledger,
            List::with_capacity(self.settings.ledger_slots().expect("call slots")),
        );
        let delivered = self.turns.closed.checked_sub(u64::from(self.turns.ready.len())).expect("delivered message");
        for call in old.into_boxed() {
            if !call.answered && (call.eligible || call.rpc.is_some() || (call.builtin && call.message >= delivered)) {
                assert!(self.ledger.push(call).is_ok(), "bounded outstanding calls");
            } else {
                drop(self.native_writes.remove(&call.id));
            }
        }
    }

    pub(super) fn match_calls(&mut self, below: &mut Queue<Below>) {
        if self.native.is_some() {
            return;
        }
        let held = core::mem::replace(&mut self.calls, List::with_capacity(u32::from(self.settings.mcp.calls)));
        for call in held.into_boxed() {
            match call {
                mcp::Up::Call { call, tool, input } => {
                    if self.startup.phase() == State::Ending || self.stopping.waiting() {
                        self.startup.server_mut().expect("stopped MCP call").answer(
                            call,
                            mcp::Answer::Error(super::stop::INTERRUPTED.into()),
                            below,
                        );
                        continue;
                    }
                    match calls::find(
                        &self.ledger,
                        &tool,
                        &input,
                        self.settings.translation.client.depth,
                        self.settings.translation.stated.call_input_bytes,
                    ) {
                        Some(index) => self.ledger.get_mut(index).expect("matched call").rpc = Some(call),
                        None if self.turns.accepting_calls() => {
                            self.calls.push(mcp::Up::Call { call, tool, input }).expect("held MCP calls");
                        }
                        None => self.startup.server_mut().expect("MCP call").answer(
                            call,
                            mcp::Answer::Error(b"smith: this call was not in the model's message".as_slice().into()),
                            below,
                        ),
                    }
                }
                prompt @ mcp::Up::Prompt { .. } => self.permission(prompt, below),
                mcp::Up::Listening { .. } | mcp::Up::Connected | mcp::Up::Ended(_) => {
                    unreachable!("call queue")
                }
            }
        }
        self.answer_calls(below);
    }

    pub(super) fn completion_active(&self) -> bool {
        self.current.owner().is_some()
            && self.startup.phase() == State::Running
            && !self.startup.switching()
            && !self.stopping.waiting()
    }

    pub(super) fn completion_deadline(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        head: bool,
        up: &mut Queue<ToDomain>,
    ) {
        if self.completion_active() {
            self.stopping.release_alarm();
            self.current.progress(env.now, head, up);
        }
    }

    pub(super) fn answer_calls(&mut self, below: &mut Queue<Below>) {
        if self.startup.phase() == State::Ending || self.stopping.waiting() {
            return;
        }
        for index in 0..self.ledger.len() {
            let call = self.ledger.get_mut(index).expect("ledger cell");
            if !call.builtin
                && let Some(rpc) = call.rpc
                && let Some(answer) = call.answer.take()
            {
                self.startup.server_mut().expect("MCP reply").answer(rpc, answer, below);
                call.answered = true;
            }
        }
    }
}

fn completion_room(completion: &llm::Completion, maximum: u64) -> Result<(), Rejection> {
    let mut held = 0_u64;
    for block in &completion.content {
        let payload = match block {
            llm::Said::ToolCall { id, name, input, call, .. } => {
                let extra = match call.owned_bytes() {
                    Some(bytes) => bytes
                        .checked_sub(u64::try_from(size_of::<llm::Decoded>()).expect("decoded cell"))
                        .unwrap_or(u64::MAX),
                    None => u64::MAX,
                };
                skein_lib::len_u64(id.len().saturating_add(name.len()).saturating_add(input.len()))
                    .saturating_add(extra)
            }
            llm::Said::Text { text: bytes, .. } | llm::Said::Refusal { text: bytes, .. } => {
                skein_lib::len_u64(bytes.len())
            }
            llm::Said::Opaque { bytes } => skein_lib::len_u64(bytes.len()),
        };
        held = held.saturating_add(u64::try_from(size_of::<llm::Said>()).expect("block cell")).saturating_add(payload);
    }
    turns::check_bound(super::Bound::Completion, held, maximum)
}

fn completion_text(completion: &llm::Completion) -> u32 {
    let mut text = 0_u32;
    for block in &completion.content {
        match block {
            llm::Said::Text { text: bytes, .. } | llm::Said::Refusal { text: bytes, .. } => {
                text = text.saturating_add(u32::try_from(bytes.len()).unwrap_or(u32::MAX));
            }
            llm::Said::ToolCall { .. } | llm::Said::Opaque { .. } => {}
        }
    }
    text
}

fn register_blocks(ledger: &mut List<calls::Call>, blocks: &[llm::Said], message: u64) -> Option<()> {
    for block in blocks {
        match block {
            llm::Said::ToolCall { id, name, input, call, .. } if id.as_ref() != turns::STRUCTURED_FINISH => {
                let mut known = false;
                for call in ledger.iter() {
                    if call.id == *id {
                        known = true;
                    }
                }
                if !known {
                    ledger
                        .push(calls::Call {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                            rpc: None,
                            answer: None,
                            answered: false,
                            prompted: false,
                            eligible: true,
                            message,
                            builtin: match call {
                                llm::Decoded::Builtin { .. } => true,
                                llm::Decoded::Owned { .. }
                                | llm::Decoded::Served { .. }
                                | llm::Decoded::Invalid { .. } => false,
                            },
                            owner: None,
                        })
                        .ok()?;
                }
            }
            llm::Said::ToolCall { .. }
            | llm::Said::Text { .. }
            | llm::Said::Refusal { .. }
            | llm::Said::Opaque { .. } => {}
        }
    }
    Some(())
}

fn decode_call(
    id: &[u8],
    name: &[u8],
    input: &[u8],
    prompt: &llm::Prompt,
    outcome: &run::outcome::OutcomeSpec,
    deliver: Option<&run::outcome::ChangeSpec>,
    limits: &protocol::Limits,
) -> llm::Decoded {
    let unwrapped;
    let decoded_input = if name == b"finish" && id != turns::STRUCTURED_FINISH {
        unwrapped = match stream::collect(input, limits.client.depth, limits.stated.call_input_bytes) {
            Some(document) => json::outcome_input(&document, 0, limits.client.answer),
            None => None,
        };
        unwrapped.as_deref().unwrap_or(b"null")
    } else {
        input
    };
    decode(name, decoded_input, prompt, outcome, deliver, limits)
}

fn decode(
    name: &[u8],
    input: &[u8],
    prompt: &llm::Prompt,
    outcome: &run::outcome::OutcomeSpec,
    deliver: Option<&run::outcome::ChangeSpec>,
    limits: &protocol::Limits,
) -> llm::Decoded {
    if input.len() > usize::try_from(limits.stated.call_input_bytes).expect("call input cap") {
        return llm::Decoded::Invalid { problem: llm::Problem::TooLarge };
    }
    for served in &prompt.served {
        match served {
            llm::Served::Finish if name == b"finish" => return protocol::decode_finish(input, outcome, limits),
            llm::Served::Deliver if name == b"deliver" => {
                return match deliver {
                    Some(change) => protocol::decode_deliver(input, change, limits),
                    None => llm::Decoded::Invalid { problem: llm::Problem::UnknownTool },
                };
            }
            llm::Served::Host(tool) if tool.name.as_ref() == name => {
                let document = stream::collect(input, limits.client.depth, limits.stated.call_input_bytes);
                let object = match document {
                    Some(document) => token_kind(&document, Some(0)) == Some(Kind::ObjectStart),
                    None => false,
                };
                if !object {
                    return llm::Decoded::Invalid { problem: llm::Problem::NotAnObject };
                }
                return match run::HostInput::attested(input.into()) {
                    Some(input) => llm::Decoded::Served {
                        ask: run::Ask::Host { tool: tool.name.clone(), effect: tool.effect, input },
                    },
                    None => llm::Decoded::Invalid { problem: llm::Problem::TooLarge },
                };
            }
            llm::Served::Finish
            | llm::Served::Save
            | llm::Served::Deliver
            | llm::Served::Host(_)
            | llm::Served::Wait
            | llm::Served::SubAgent => {}
        }
    }
    let decoded = protocol::decode(name, input, prompt.tools, &prompt.served, prompt.asker, limits);
    match &decoded {
        llm::Decoded::Served { ask: run::Ask::SubAgent { families, .. } }
            if families.tools.modify || families.tools.shell =>
        {
            llm::Decoded::Invalid { problem: llm::Problem::BadValue { field: b"tools".as_slice().into() } }
        }
        llm::Decoded::Served { .. }
        | llm::Decoded::Owned { .. }
        | llm::Decoded::Builtin { .. }
        | llm::Decoded::Invalid { .. } => decoded,
    }
}

fn token_kind(document: &skein_json::Document, index: Option<u32>) -> Option<Kind> {
    Some(document.token(index?)?.kind)
}

fn result_failure(document: &skein_json::Document, wall: skein_lib::Wall) -> Option<(llm::Failure, Option<u16>)> {
    if token_kind(document, json::field(document, 0, b"is_error")) != Some(Kind::True) {
        return None;
    }
    let status = turns::number(document, json::field(document, 0, b"api_error_status"));
    let status = match status {
        Some(status) => u16::try_from(status).ok(),
        None => None,
    };
    let reason = json::text_in(document, json::field(document, 0, b"terminal_reason"));
    let retry_after = retry_after(document, wall);
    let failure = match status {
        Some(401) => llm::Failure::SignedOut,
        Some(429) => llm::Failure::Exhausted { retry_after },
        Some(_) | None => match reason {
            Some(b"usage_limit" | b"rate_limit" | b"limited") => llm::Failure::Exhausted { retry_after },
            Some(_) | None => match status {
                Some(403) if !provider_error(document) => llm::Failure::Unavailable,
                Some(529) => llm::Failure::Overloaded,
                Some(500..=599) => llm::Failure::Unavailable,
                Some(_) | None => llm::Failure::Invalid,
            },
        },
    };
    Some((failure, status))
}

fn provider_error(document: &skein_json::Document) -> bool {
    if let Some(text) = json::text_in(document, json::field(document, 0, b"result"))
        && error_object(text)
    {
        return true;
    }
    let Some(errors) = json::field(document, 0, b"errors") else {
        return false;
    };
    if token_kind(document, Some(errors)) != Some(Kind::ArrayStart) {
        return false;
    }
    let mut next = errors.checked_add(1).expect("array start has an end");
    for _ in 0..document.len() {
        if token_kind(document, Some(next)) == Some(Kind::ArrayEnd) {
            break;
        }
        if let Some(text) = json::text(document, next)
            && error_object(text)
        {
            return true;
        }
        let Some(after) = json::after(document, next) else {
            break;
        };
        next = after;
    }
    false
}

// Program error text may prefix its JSON with "API Error: <status>".
// Search the bounded decoded text for an object-valued error key, keeping none of it.
fn error_object(text: &[u8]) -> bool {
    const KEY: &[u8] = b"\"error\"";
    for (offset, candidate) in text.windows(KEY.len()).enumerate() {
        if candidate == KEY {
            let after = offset.checked_add(KEY.len()).expect("matched key fits text");
            let rest = text.get(after..).expect("matched key fits text").trim_ascii_start();
            if let Some(value) = rest.strip_prefix(b":")
                && value.trim_ascii_start().starts_with(b"{")
            {
                return true;
            }
        }
    }
    false
}

fn retry_after(document: &skein_json::Document, wall: skein_lib::Wall) -> Option<skein_lib::Duration> {
    let info = json::field(document, 0, b"rate_limit_info");
    let seconds = turns::number(document, json::field_in(document, info, b"resetsAt"))?;
    Some(skein_lib::Duration::from_nanos(seconds.checked_mul(1_000_000_000)?.saturating_sub(wall.as_nanos())))
}

fn record_name(kind: Option<&[u8]>) -> &'static [u8] {
    match kind {
        Some(b"assistant") => b"assistant",
        Some(b"stream_event") => b"stream_event",
        Some(b"result") => b"result",
        Some(b"user") => b"user",
        Some(b"system") => b"system",
        Some(b"rate_limit_event") => b"rate_limit_event",
        Some(_) | None => b"record",
    }
}

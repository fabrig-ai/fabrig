//! Built-in stream correlation (protocol/programs.md 5.4–5.5).
//! Uses the bounded call ledger: a result waits for its call's Completed owner.

use super::Complete;
use super::{Backend, Below, Bound, Rejection, policy, stream};
use crate::mcp::{self, json};
use alloc::boxed::Box;
use skein_json::{Document, Kind};
use skein_lib::{Env, List, Queue, Token};
use smith_domain::{llm, tools::Grants};
use smith_protocol_llm::{self as protocol, ToDomain};

impl Backend {
    pub(super) fn result_contract(&self) -> bool {
        match self.current.request() {
            Some(Complete { prompt, outcome, .. }) => {
                prompt.served.contains(&llm::Served::Finish)
                    && (outcome.change.is_some()
                        || !outcome.verdicts.is_empty()
                        || outcome.report.is_some()
                        || outcome.failure.is_some())
            }
            None => false,
        }
    }

    pub(super) fn grants(&self) -> Grants {
        match self.current.request() {
            Some(Complete { prompt, .. }) => prompt.tools,
            None => Grants { inspect: false, modify: false, shell: false },
        }
    }

    pub(super) fn admit_init(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        bytes: &[u8],
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        if let Some(given) = &mut self.given {
            given.push(super::Given::Init(skein_lib::bytes::concat(&[bytes, b"\n"])));
        }
        if let Some(detail) = self.init_problem(bytes) {
            self.fail(env, llm::Failure::Program { exit: None }, &detail, up, below);
            return;
        }
        self.startup.initialized(&mut self.server_up, below);
        if !self.compaction() {
            self.remember_session(below);
        }
        self.completion_deadline(env, true, up);
    }

    pub(super) fn later_init(
        &mut self,
        env: &Env<protocol::ComponentLimits>,
        bytes: &[u8],
        up: &mut Queue<ToDomain>,
        below: &mut Queue<Below>,
    ) {
        if let Some(given) = &mut self.given {
            given.push(super::Given::Init(skein_lib::bytes::concat(&[bytes, b"\n"])));
        }
        if let Some(detail) = self.init_problem(bytes) {
            self.fail(env, llm::Failure::Program { exit: None }, &detail, up, below);
        }
    }

    fn init_problem(&self, bytes: &[u8]) -> Option<Box<[u8]>> {
        let depth = self.settings.translation.client.depth;
        let tokens = self.settings.stream_tokens;
        if self.compaction() {
            if super::compact::checked_init(bytes, depth, tokens).is_some() {
                return None;
            }
            return Some(b"smith: summary program offered tools".as_slice().into());
        }
        let result_contract = self.result_contract();
        let grants = if self.settings.workspace.is_none() {
            Grants { inspect: false, modify: false, shell: false }
        } else {
            self.grants()
        };
        let error = init_error(bytes, grants, result_contract, depth, tokens);
        if error.is_none() && stream::checked_init(bytes, &self.tools(), depth, tokens).is_some() {
            None
        } else {
            Some(match error {
                Some(detail) => detail,
                None => b"smith: the program started without smith's tools".as_slice().into(),
            })
        }
    }

    pub(super) fn awaits_answer(&self, id: &[u8]) -> bool {
        for call in &self.ledger {
            if !call.builtin && !call.answered && call.id.as_ref() == id {
                return true;
            }
        }
        false
    }

    pub(super) fn builtin_owners(&mut self, owner: Token, completion: &llm::Completion) {
        for block in &completion.content {
            match block {
                llm::Said::ToolCall { id, .. } => {
                    for slot in 0..self.ledger.len() {
                        let call = self.ledger.get_mut(slot).expect("ledger slot");
                        if call.builtin && call.id == *id {
                            call.owner = Some(owner);
                        }
                    }
                }
                llm::Said::Text { .. } | llm::Said::Opaque { .. } | llm::Said::Refusal { .. } => {}
            }
        }
    }

    pub(super) fn flush_builtins(&mut self, up: &mut Queue<ToDomain>) {
        if self.native.is_some() {
            return;
        }
        for slot in 0..self.ledger.len() {
            let call = self.ledger.get_mut(slot).expect("ledger slot");
            if call.builtin
                && let Some(owner) = call.owner
            {
                let (text, error) = match call.answer.take() {
                    Some(mcp::Answer::Text(text)) => (text, false),
                    Some(mcp::Answer::Error(text)) => (text, true),
                    None => continue,
                };
                call.answered = true;
                let result = ToDomain::BuiltinResult { owner, id: call.id.clone(), text, error };
                match self.native_writes.remove(&call.id) {
                    Some(place) => {
                        self.begin_snapshot(owner, place, super::native::After::Result(result));
                        return;
                    }
                    None => up.push(result),
                }
            }
        }
    }

    pub(super) fn builtin_results(&mut self, document: &Document, up: &mut Queue<ToDomain>) -> Result<(), Rejection> {
        let message = json::field(document, 0, b"message").ok_or(Rejection::Malformed)?;
        let content = json::field(document, message, b"content").ok_or(Rejection::Malformed)?;
        if document.token(content).ok_or(Rejection::Malformed)?.kind != Kind::ArrayStart {
            return Ok(());
        }
        let mut index = content.checked_add(1).ok_or(Rejection::Malformed)?;
        for _ in 0..document.len() {
            if document.token(index).ok_or(Rejection::Malformed)?.kind == Kind::ArrayEnd {
                break;
            }
            if json::text_in(document, json::field(document, index, b"type")) == Some(b"tool_result") {
                let id = json::text_in(document, json::field(document, index, b"tool_use_id"))
                    .ok_or(Rejection::Malformed)?;
                for slot in 0..self.ledger.len() {
                    let call = self.ledger.get(slot).ok_or(Rejection::Malformed)?;
                    if call.builtin && call.id.as_ref() == id && !call.answered && call.answer.is_none() {
                        let text = result_text(
                            document,
                            json::field(document, index, b"content").ok_or(Rejection::Malformed)?,
                            self.settings.line_bytes,
                        )?;
                        let error = match json::field(document, index, b"is_error") {
                            Some(value) => document.token(value).ok_or(Rejection::Malformed)?.kind == Kind::True,
                            None => false,
                        };
                        self.reply_bytes(skein_lib::len_u64(text.len()))?;
                        self.ledger.get_mut(slot).ok_or(Rejection::Malformed)?.answer =
                            Some(if error { mcp::Answer::Error(text) } else { mcp::Answer::Text(text) });
                    }
                }
            }
            index = json::after(document, index).ok_or(Rejection::Malformed)?;
        }
        self.flush_builtins(up);
        Ok(())
    }

    // Share the retained-reply budget. Reserve each MCP slot's full cap so
    // its later answer cannot consume bytes already held by a built-in.
    pub(super) fn reply_room(&self, extra: u64) -> bool {
        self.reply_bytes(extra).is_ok()
    }

    fn reply_bytes(&self, extra: u64) -> Result<(), Rejection> {
        let maximum = u64::from(self.settings.translation.rendered_result)
            .checked_mul(u64::from(self.settings.ledger_slots().expect("call slots")))
            .expect("reply budget");
        let mut retained = extra;
        for call in &self.ledger {
            let bytes = if call.builtin {
                match &call.answer {
                    Some(mcp::Answer::Text(text) | mcp::Answer::Error(text)) => skein_lib::len_u64(text.len()),
                    None => 0,
                }
            } else {
                u64::from(self.settings.translation.rendered_result)
            };
            retained = match retained.checked_add(bytes) {
                Some(bytes) => bytes,
                None => return Err(Rejection::Overflow),
            };
        }
        if retained > maximum {
            Err(Rejection::Limit { which: Bound::Reply, value: retained, bound: maximum })
        } else {
            Ok(())
        }
    }
}

pub(super) fn init_error(
    bytes: &[u8],
    grants: Grants,
    result_contract: bool,
    depth: u32,
    tokens: u32,
) -> Option<Box<[u8]>> {
    let document = stream::collect(bytes, depth, tokens)?;
    let listed = json::field(&document, 0, b"tools")?;
    let mut seen = [false; 6];
    let mut structured_output = false;
    let mut index = listed.checked_add(1)?;
    for _ in 0..document.len() {
        if document.token(index)?.kind == Kind::ArrayEnd {
            break;
        }
        let name = json::text(&document, index)?;
        if !name.starts_with(b"mcp__") {
            let mut granted = false;
            // --json-schema adds the program's own result tool, independently
            // of --tools. Versions that omit it from init remain admissible.
            if name == b"StructuredOutput" && result_contract && !structured_output {
                structured_output = true;
                granted = true;
            }
            for (slot, tool) in policy::BUILTINS.iter().enumerate() {
                if tool.name() == name && tool.granted(grants) && !seen.get(slot).expect("six built-ins") {
                    *seen.get_mut(slot).expect("six built-ins") = true;
                    granted = true;
                }
            }
            if !granted {
                return Some(skein_lib::bytes::concat(&[
                    b"smith: the program offers a built-in tool the run did not grant: ",
                    name,
                ]));
            }
        }
        index = json::after(&document, index)?;
    }
    for (slot, tool) in policy::BUILTINS.iter().enumerate() {
        if tool.granted(grants) && !seen.get(slot).expect("six built-ins") {
            return Some(skein_lib::bytes::concat(&[
                b"smith: the program lacks a granted built-in tool: ",
                tool.name(),
            ]));
        }
    }
    None
}

fn result_text(document: &Document, content: u32, maximum: u32) -> Result<Box<[u8]>, Rejection> {
    if document.token(content).ok_or(Rejection::Malformed)?.kind == Kind::String {
        let text = json::text(document, content).ok_or(Rejection::Malformed)?;
        super::turns::check_bound(Bound::ReplyText, skein_lib::len_u64(text.len()), u64::from(maximum))?;
        return Ok(text.into());
    }
    if document.token(content).ok_or(Rejection::Malformed)?.kind != Kind::ArrayStart {
        return Err(Rejection::Malformed);
    }
    let mut text = List::with_capacity(maximum);
    let mut index = content.checked_add(1).ok_or(Rejection::Overflow)?;
    let mut first = true;
    for _ in 0..document.len() {
        if document.token(index).ok_or(Rejection::Malformed)?.kind == Kind::ArrayEnd {
            break;
        }
        if json::text_in(document, json::field(document, index, b"type")) == Some(b"text") {
            let part = json::text_in(document, json::field(document, index, b"text")).ok_or(Rejection::Malformed)?;
            let value = u64::from(text.len())
                .checked_add(u64::from(!first))
                .ok_or(Rejection::Overflow)?
                .checked_add(skein_lib::len_u64(part.len()))
                .ok_or(Rejection::Overflow)?;
            super::turns::check_bound(Bound::ReplyText, value, u64::from(maximum))?;
            if !first {
                text.push(b'\n').expect("admitted result separator");
            }
            first = false;
            for byte in part {
                text.push(*byte).expect("admitted result bytes");
            }
        }
        index = json::after(document, index).ok_or(Rejection::Malformed)?;
    }
    Ok(text.into_boxed())
}

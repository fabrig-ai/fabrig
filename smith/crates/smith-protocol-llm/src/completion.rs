//! Observed shared completions into root blocks, with immutable declaration lookup.
//! No provider ID, assistant position or declared host effect comes from a caller override.

use alloc::boxed::Box;
use core::mem::size_of;

use skein_lib::{List, bytes};
use smith_domain::{Event, llm, run};

use crate::types::Context;

use crate::Error;

/// Decode the client's blocks under the retained charter and receiving contract.
/// Original provider bytes and replay survive decoding. Decoded ownership beyond
/// the secured aggregate becomes `TooLarge`; invalid replay refuses translation.
pub fn completion(context: Context, completion: skein_llm::Completion) -> Result<Event, Error> {
    let skein_llm::Completion { content, stop, usage } = completion;
    let maximum = usize::try_from(context.receiving.max_completion_blocks.min(context.limits.client.output_items))
        .expect("u32 fits usize");
    if content.len() > maximum {
        return Err(Error::Limit);
    }
    let mut decoded_bytes = decoded_cells(&content)?;
    if decoded_bytes > context.receiving.decoded_call_bytes {
        return Err(Error::Limit);
    }
    let count = u32::try_from(content.len()).or(Err(Error::Limit))?;
    let mut translated = List::with_capacity(count);
    let mut reasoning_dropped = List::with_capacity(count);
    let mut held = 0_u64;
    for block in content {
        let block = match block {
            skein_llm::Block::Dropped { bytes } => {
                held = held.checked_add(u64::try_from(size_of::<u64>()).or(Err(Error::Limit))?).ok_or(Error::Limit)?;
                if held > context.receiving.max_completion_bytes {
                    return Err(Error::Limit);
                }
                reasoning_dropped.push(bytes).expect("one drop per observed block");
                continue;
            }
            block @ (skein_llm::Block::Text { .. }
            | skein_llm::Block::Refusal { .. }
            | skein_llm::Block::Reasoning { .. }
            | skein_llm::Block::ToolCall { .. }
            | skein_llm::Block::ToolResult { .. }
            | skein_llm::Block::Oversize { .. }
            | skein_llm::Block::Cut { .. }) => block,
        };
        let said = block_value(block, &context, &mut decoded_bytes)?;
        held = held.checked_add(said_bytes(&said)?).ok_or(Error::Limit)?;
        if held > context.receiving.max_completion_bytes {
            return Err(Error::Limit);
        }
        translated.push(said).expect("one translated block per observed input block");
    }
    let stop = match stop {
        skein_llm::Stop::EndTurn => llm::Stop::EndTurn,
        skein_llm::Stop::ToolUse => llm::Stop::ToolUse,
        skein_llm::Stop::MaxTokens => llm::Stop::MaxTokens,
        skein_llm::Stop::Refusal => llm::Stop::Refusal,
    };
    let skein_llm::Usage { input, output, cache_read, cache_write, reasoning } = usage;
    let usage = llm::Usage {
        input_tokens: input,
        output_tokens: output,
        cache_read_tokens: cache_read,
        cache_write_tokens: cache_write,
        reasoning_tokens: reasoning,
    };
    Ok(Event::Below(smith_domain::Below::Completed {
        owner: context.owner,
        completion: llm::Completion {
            reasoning_dropped: reasoning_dropped.into_boxed(),
            content: translated.into_boxed(),
            stop,
            usage,
        },
    }))
}

fn block_value(block: skein_llm::Block, context: &Context, decoded_bytes: &mut u64) -> Result<llm::Said, Error> {
    match block {
        skein_llm::Block::Text { text, replay } => {
            Ok(llm::Said::Text { text, replay: encode_replay(replay, context)? })
        }
        skein_llm::Block::Refusal { text, replay } => {
            Ok(llm::Said::Refusal { text, replay: encode_replay(replay, context)? })
        }
        skein_llm::Block::Reasoning { replay } => {
            let limits = crate::limits::replay_limits(&context.limits.client).ok_or(Error::Limit)?;
            let bytes = match replay.to_bytes(&limits) {
                Ok(bytes) => bytes,
                Err(error) => return Err(Error::from_client(error)),
            };
            Ok(llm::Said::Opaque { bytes })
        }
        skein_llm::Block::ToolCall { id, name, arguments, replay } => {
            let call = decode(&name, &arguments, context, *decoded_bytes)?;
            let call = receiving_call(call, decoded_bytes, context.receiving.decoded_call_bytes);
            Ok(llm::Said::ToolCall { id, name, input: arguments, call, replay: encode_replay(replay, context)? })
        }
        skein_llm::Block::Oversize { id, name, bytes } => {
            Ok(invalid_call(id, name, llm::Problem::Oversize { bytes, bound: context.limits.client.input }))
        }
        skein_llm::Block::Cut { id, name, arguments } => {
            let bytes = u64::try_from(arguments.len()).or(Err(Error::Limit))?;
            Ok(invalid_call(id, name, llm::Problem::CutOff { bytes }))
        }
        skein_llm::Block::ToolResult { .. } => Err(Error::Invalid),
        skein_llm::Block::Dropped { .. } => unreachable!("drops are separate metadata"),
    }
}

// Canonical replacement keeps the provider's name and correlation right, but
// carries no incomplete or oversized input or native replay. The typed problem
// supplies the note and error result retained by the session and transcript.
fn invalid_call(id: Box<[u8]>, name: Box<[u8]>, problem: llm::Problem) -> llm::Said {
    llm::Said::ToolCall {
        id,
        name,
        input: bytes::copy_of(b"{}"),
        call: llm::Decoded::Invalid { problem },
        replay: None,
    }
}

fn encode_replay(value: Option<skein_llm::Replay>, context: &Context) -> Result<Option<llm::Replay>, Error> {
    match value {
        Some(replay) => {
            let limits = crate::limits::replay_limits(&context.limits.client).ok_or(Error::Limit)?;
            match replay.to_bytes(&limits) {
                Ok(bytes) => Ok(Some(llm::Replay { bytes })),
                Err(error) => Err(Error::from_client(error)),
            }
        }
        None => Ok(None),
    }
}

fn decode(name: &[u8], input: &[u8], context: &Context, held: u64) -> Result<llm::Decoded, Error> {
    for declaration in &context.served {
        match declaration {
            llm::Served::Host(tool) if tool.name.as_ref() == name => {
                let remaining = context.receiving.decoded_call_bytes.checked_sub(held).ok_or(Error::Limit)?;
                return Ok(host(tool, input, context, remaining));
            }
            llm::Served::Host(_)
            | llm::Served::Deliver
            | llm::Served::Finish
            | llm::Served::SubAgent
            | llm::Served::Wait => {}
        }
    }
    Ok(match name {
        b"finish" if context.served.contains(&llm::Served::Finish) => {
            crate::decode_finish(input, &context.outcome, &context.limits)
        }
        b"deliver" if context.served.contains(&llm::Served::Deliver) => match &context.deliver {
            Some(change) => crate::decode_deliver(input, change, &context.limits),
            None => llm::Decoded::Invalid { problem: llm::Problem::UnknownTool },
        },
        _ => crate::decode(name, input, context.grants, &context.served, context.asker, &context.limits),
    })
}

fn host(declaration: &run::HostTool, input: &[u8], context: &Context, remaining: u64) -> llm::Decoded {
    let attested = match crate::prompt::object(input, &context.limits) {
        Ok(value) => value,
        Err(Error::Limit | Error::PathLimit { .. }) => {
            return llm::Decoded::Invalid { problem: llm::Problem::TooLarge };
        }
        Err(Error::Invalid | Error::Unsupported) => {
            return llm::Decoded::Invalid { problem: llm::Problem::NotAnObject };
        }
    };
    drop(attested);
    let wanted = host_wanted(declaration, input);
    match wanted {
        Some(wanted) if wanted <= remaining && input.len() <= run::HostInput::CAPACITY => {}
        Some(_) | None => return llm::Decoded::Invalid { problem: llm::Problem::TooLarge },
    }
    match run::HostInput::attested(bytes::copy_of(input)) {
        Some(input) => llm::Decoded::Served {
            ask: run::Ask::Host { tool: declaration.name.clone(), effect: declaration.effect, input },
        },
        None => llm::Decoded::Invalid { problem: llm::Problem::TooLarge },
    }
}

fn host_wanted(declaration: &run::HostTool, input: &[u8]) -> Option<u64> {
    // This call's wrapper was reserved with every observed tool-call cell
    // before decoding. Only the new declaration/input payload needs room.
    u64::try_from(declaration.name.len()).ok()?.checked_add(u64::try_from(input.len()).ok()?)
}

fn decoded_cells(content: &[skein_llm::Block]) -> Result<u64, Error> {
    let mut calls = 0_u64;
    for block in content {
        match block {
            skein_llm::Block::ToolCall { .. } | skein_llm::Block::Oversize { .. } | skein_llm::Block::Cut { .. } => {
                calls = calls.checked_add(1).ok_or(Error::Limit)?;
            }
            skein_llm::Block::Text { .. }
            | skein_llm::Block::Refusal { .. }
            | skein_llm::Block::Reasoning { .. }
            | skein_llm::Block::ToolResult { .. }
            | skein_llm::Block::Dropped { .. } => {}
        }
    }
    calls.checked_mul(u64::try_from(size_of::<llm::Decoded>()).or(Err(Error::Limit))?).ok_or(Error::Limit)
}

fn receiving_call(call: llm::Decoded, held: &mut u64, maximum: u64) -> llm::Decoded {
    // Every observed tool call already owns its full fixed fallback cell.
    // Admission charges only additional owned payload; an over-cap result
    // moves to the reserved fixed TooLarge cell without erasing its price.
    let extra = match call.owned_bytes() {
        Some(wanted) => match wanted.checked_sub(u64::try_from(size_of::<llm::Decoded>()).expect("size fits u64")) {
            Some(extra) => extra,
            None => return llm::Decoded::Invalid { problem: llm::Problem::TooLarge },
        },
        None => return llm::Decoded::Invalid { problem: llm::Problem::TooLarge },
    };
    let next = match held.checked_add(extra) {
        Some(next) if next <= maximum => next,
        Some(_) | None => return llm::Decoded::Invalid { problem: llm::Problem::TooLarge },
    };
    *held = next;
    call
}

fn said_bytes(said: &llm::Said) -> Result<u64, Error> {
    let mut held = u64::try_from(size_of::<llm::Said>()).or(Err(Error::Limit))?;
    match said {
        llm::Said::Text { text, replay } | llm::Said::Refusal { text, replay } => {
            held = held.checked_add(size(text.len())?).ok_or(Error::Limit)?;
            held = held.checked_add(replay_bytes(replay.as_ref())?).ok_or(Error::Limit)?;
        }
        llm::Said::Opaque { bytes } => held = held.checked_add(size(bytes.len())?).ok_or(Error::Limit)?,
        llm::Said::ToolCall { id, name, input, call, replay } => {
            for field in [id, name, input] {
                held = held.checked_add(size(field.len())?).ok_or(Error::Limit)?;
            }
            let decoded = call.owned_bytes().ok_or(Error::Limit)?;
            let inline = u64::try_from(size_of::<llm::Decoded>()).or(Err(Error::Limit))?;
            held = held.checked_add(decoded.checked_sub(inline).ok_or(Error::Limit)?).ok_or(Error::Limit)?;
            held = held.checked_add(replay_bytes(replay.as_ref())?).ok_or(Error::Limit)?;
        }
    }
    Ok(held)
}

fn replay_bytes(replay: Option<&llm::Replay>) -> Result<u64, Error> {
    match replay {
        Some(replay) => size(replay.bytes.len()),
        None => Ok(0),
    }
}

fn size(value: usize) -> Result<u64, Error> {
    u64::try_from(value).or(Err(Error::Limit))
}

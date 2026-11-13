//! Whole bounded tool declarations and concrete conversations into Skein.
//! Parsing is generic JSON attestation, never schema-policy validation.

use alloc::boxed::Box;

use skein_json::Kind;
use skein_lib::{List, bytes};
use smith_domain::{llm, run, tools};

use crate::types::{Context, ToolSchema};
use crate::{Error, Limits, valid};

/// Prepare the translated prompt and retained context for the connection
/// component, which prepares the sole skein Client when it admits Start.
pub fn prepare_component(
    owner: skein_lib::Token,
    prompt: llm::Prompt,
    outcome: run::outcome::OutcomeSpec,
    deliver: Option<run::outcome::ChangeSpec>,
    receiving: crate::Receiving,
    limits: &Limits,
) -> Result<(skein_llm::Prompt, Context), Error> {
    if !valid(limits, &receiving) {
        return Err(Error::Limit);
    }
    let schemas =
        crate::tools::schemas_for_contract(&prompt, &limits.stated, &outcome, deliver.as_ref(), limits.client.request)?;
    let asker = prompt.asker;
    let grants = prompt.tools;
    let (translated, served) = translate(prompt, &schemas, limits)?;
    Ok((translated, Context { owner, asker, grants, served, receiving, limits: *limits, outcome, deliver }))
}

/// Translate a prompt and supplied schemas under the component's bounds.
pub fn prompt(input: llm::Prompt, schemas: &[ToolSchema], limits: &Limits) -> Result<skein_llm::Prompt, Error> {
    let (translated, _served) = translate(input, schemas, limits)?;
    Ok(translated)
}

fn translate(
    input: llm::Prompt,
    schemas: &[ToolSchema],
    limits: &Limits,
) -> Result<(skein_llm::Prompt, Box<[llm::Served]>), Error> {
    inventory(&input, schemas, limits)?;
    let llm::Prompt {
        asker: _,
        endpoint: _,
        affinity,
        model,
        effort,
        system,
        tools: _,
        namespace,
        served,
        choice,
        messages,
        max_tokens,
        purpose,
    } = input;
    let mut offered = List::with_capacity(
        u32::try_from(schemas.len().checked_add(served.len()).ok_or(Error::Limit)?).or(Err(Error::Limit))?,
    );
    for descriptor in &served {
        match descriptor {
            llm::Served::Host(tool) => {
                let schema = object(&tool.schema, limits)?;
                offered
                    .push(skein_llm::Tool { name: tool.name.clone(), description: tool.description.clone(), schema })
                    .or(Err(Error::Limit))?;
            }
            llm::Served::Save
            | llm::Served::Deliver
            | llm::Served::Finish
            | llm::Served::SubAgent
            | llm::Served::Wait => {}
        }
    }
    for descriptor in schemas {
        let schema = object(&descriptor.schema, limits)?;
        offered
            .push(skein_llm::Tool {
                name: descriptor.name.clone(),
                description: descriptor.description.clone(),
                schema,
            })
            .or(Err(Error::Limit))?;
    }
    let native_choice = match &choice {
        llm::ToolChoice::Auto => skein_llm::ToolChoice::Auto,
        llm::ToolChoice::None => skein_llm::ToolChoice::None,
        llm::ToolChoice::Only(selected) => {
            let mut names = List::with_capacity(u32::try_from(selected.len()).or(Err(Error::Limit))?);
            for descriptor in selected {
                if !served.contains(descriptor) {
                    return Err(Error::Invalid);
                }
                names.push(bytes::copy_of(served_name(descriptor))).or(Err(Error::Limit))?;
            }
            skein_llm::ToolChoice::Only(names.into_boxed())
        }
    };
    let owed_message = messages.len().checked_sub(2);
    let result_maximum = owed_result_bound(purpose, &messages, limits.rendered_result)?;
    let mut translated: List<skein_llm::Message> =
        List::with_capacity(u32::try_from(messages.len()).or(Err(Error::Limit))?);
    for (index, message) in messages.into_iter().enumerate() {
        let maximum = if Some(index) == owed_message { result_maximum } else { limits.rendered_result };
        let mut content = List::with_capacity(u32::try_from(message.content.len()).or(Err(Error::Limit))?);
        for block in message.content {
            let block = translate_block(block, &namespace, limits, maximum, translated.last())?;
            content.push(block).expect("room for each admitted source block");
        }
        let role = match message.role {
            llm::Role::User => skein_llm::Role::User,
            llm::Role::Assistant => skein_llm::Role::Assistant,
        };
        translated.push(skein_llm::Message { role, content: content.into_boxed() }).expect("admitted message count");
    }
    Ok((
        skein_llm::Prompt {
            model,
            instructions: system,
            tools: offered.into_boxed(),
            choice: native_choice,
            messages: translated.into_boxed(),
            reasoning_effort: effort,
            affinity: Some(skein_llm::Affinity { key: affinity.key, thread: affinity.thread }),
            max_output_tokens: Some(max_tokens),
        },
        served,
    ))
}

fn owed_result_bound(purpose: llm::Purpose, messages: &[llm::Message], maximum: u32) -> Result<u32, Error> {
    let room = match purpose {
        llm::Purpose::Ordinary { .. } => return Ok(maximum),
        llm::Purpose::Compaction { room } => room,
    };
    let message = match messages.len().checked_sub(2) {
        Some(index) => messages.get(index),
        None => None,
    };
    let Some(message) = message else { return Ok(maximum) };
    let mut owed = 0_u64;
    for block in &message.content {
        match block {
            llm::Block::ToolResult { .. } => owed = owed.checked_add(1).ok_or(Error::Limit)?,
            llm::Block::Text { .. }
            | llm::Block::Refusal { .. }
            | llm::Block::Opaque { .. }
            | llm::Block::ToolCall { .. } => {}
        }
    }
    let Some(share) = room.checked_div(owed) else { return Ok(maximum) };
    let maximum = u32::try_from(share).unwrap_or(u32::MAX).min(maximum);
    if maximum < crate::render::CUT_MARKER_BYTES {
        return Err(Error::Limit);
    }
    Ok(maximum)
}

fn served_name(descriptor: &llm::Served) -> &[u8] {
    match descriptor {
        llm::Served::Wait => b"wait",
        llm::Served::Finish => b"finish",
        llm::Served::Save => b"save",
        llm::Served::Deliver => b"deliver",
        llm::Served::SubAgent => b"sub_agent",
        llm::Served::Host(tool) => &tool.name,
    }
}

fn outside_choice(choice: Option<&[Box<[u8]>]>, maximum: u32) -> Result<Box<[u8]>, Error> {
    let prefix = b"not run: allowed calls now: ";
    let mut length = prefix.len();
    match choice {
        Some(names) => {
            for name in names {
                length = length.checked_add(name.len()).ok_or(Error::Limit)?;
                length = length.checked_add(1).ok_or(Error::Limit)?;
            }
        }
        None => length = length.checked_add(4).ok_or(Error::Limit)?,
    }
    if u64::try_from(length).or(Err(Error::Limit))? > u64::from(maximum) {
        return Err(Error::Limit);
    }
    let mut writer = skein_lib::Writer::new(length);
    writer.put(prefix).or(Err(Error::Limit))?;
    match choice {
        Some(names) => {
            for name in names {
                writer.put(name).or(Err(Error::Limit))?;
                writer.put(b" ").or(Err(Error::Limit))?;
            }
        }
        None => writer.put(b"none").or(Err(Error::Limit))?,
    }
    Ok(writer.finish())
}

fn after_failed_write(call: u16, previous: Option<&skein_llm::Message>, maximum: u32) -> Result<Box<[u8]>, Error> {
    let previous = previous.ok_or(Error::Invalid)?;
    if previous.role != skein_llm::Role::Assistant || call == 0 {
        return Err(Error::Invalid);
    }
    let mut position = 0_u16;
    for block in &previous.content {
        match block {
            skein_llm::Block::ToolCall { name, .. } => {
                position = position.checked_add(1).ok_or(Error::Limit)?;
                if position == call {
                    let prefix = b"not run: call ";
                    let suffix = b") failed before it; this call's changes were not made";
                    let digits = skein_lib::Decimal::of(u64::from(call));
                    let length = prefix
                        .len()
                        .checked_add(digits.as_bytes().len())
                        .ok_or(Error::Limit)?
                        .checked_add(2)
                        .ok_or(Error::Limit)?
                        .checked_add(name.len())
                        .ok_or(Error::Limit)?
                        .checked_add(suffix.len())
                        .ok_or(Error::Limit)?;
                    if length > usize::try_from(maximum).or(Err(Error::Limit))? {
                        return Err(Error::Limit);
                    }
                    let mut writer = skein_lib::Writer::new(length);
                    writer.put(prefix).or(Err(Error::Limit))?;
                    writer.put(digits.as_bytes()).or(Err(Error::Limit))?;
                    writer.put(b" (").or(Err(Error::Limit))?;
                    writer.put(name).or(Err(Error::Limit))?;
                    writer.put(suffix).or(Err(Error::Limit))?;
                    return Ok(writer.finish());
                }
            }
            skein_llm::Block::Text { .. }
            | skein_llm::Block::Refusal { .. }
            | skein_llm::Block::Reasoning { .. }
            | skein_llm::Block::Oversize { .. }
            | skein_llm::Block::Cut { .. }
            | skein_llm::Block::Dropped { .. }
            | skein_llm::Block::ToolResult { .. } => {}
        }
    }
    Err(Error::Invalid)
}

fn inventory(input: &llm::Prompt, schemas: &[ToolSchema], limits: &Limits) -> Result<(), Error> {
    let maximum = usize::try_from(limits.client.tools).expect("u32 fits usize");
    if schemas.len() > maximum
        || input.messages.len() > usize::try_from(limits.client.history_items).expect("u32 fits usize")
    {
        return Err(Error::Limit);
    }
    let mut count = schemas.len();
    let mut declaration_bytes = 0_u64;
    for (index, descriptor) in input.served.iter().enumerate() {
        served_unique(descriptor, input.served.get(..index).expect("enumerated served prefix"))?;
        declaration_bytes = declaration_bytes.checked_add(size(size_of::<llm::Served>())?).ok_or(Error::Limit)?;
        match descriptor {
            llm::Served::Host(tool) => {
                count = count.checked_add(1).ok_or(Error::Limit)?;
                for field in [&tool.name, &tool.description, &tool.schema] {
                    declaration_bytes = declaration_bytes.checked_add(size(field.len())?).ok_or(Error::Limit)?;
                }
                for other in schemas {
                    if other.name == tool.name {
                        return Err(Error::Invalid);
                    }
                }
            }
            llm::Served::Save
            | llm::Served::Deliver
            | llm::Served::Finish
            | llm::Served::SubAgent
            | llm::Served::Wait => {}
        }
    }
    if count > maximum || input.served.len() > maximum {
        return Err(Error::Limit);
    }
    for descriptor in schemas {
        declaration_bytes = declaration_bytes.checked_add(size(size_of::<ToolSchema>())?).ok_or(Error::Limit)?;
        for field in [&descriptor.name, &descriptor.description, &descriptor.schema] {
            declaration_bytes = declaration_bytes.checked_add(size(field.len())?).ok_or(Error::Limit)?;
        }
    }
    if declaration_bytes > limits.tool_bytes {
        return Err(Error::Limit);
    }
    history_admitted(&input.messages, limits)
}

fn served_unique(descriptor: &llm::Served, previous: &[llm::Served]) -> Result<(), Error> {
    for prior in previous {
        let repeated = served_name(descriptor) == served_name(prior);
        if repeated {
            return Err(Error::Invalid);
        }
    }
    Ok(())
}

fn history_admitted(messages: &[llm::Message], limits: &Limits) -> Result<(), Error> {
    let maximum = usize::try_from(limits.client.history_items).expect("u32 fits usize");
    let mut blocks = 0_usize;
    let mut documents = 0_u64;
    for message in messages {
        blocks = blocks.checked_add(message.content.len()).ok_or(Error::Limit)?;
        if blocks > maximum {
            return Err(Error::Limit);
        }
        for block in &message.content {
            let replay = match block {
                llm::Block::Text { replay, .. }
                | llm::Block::Refusal { replay, .. }
                | llm::Block::ToolCall { replay, .. } => replay.as_ref(),
                llm::Block::ToolResult { result, .. } => match result {
                    llm::Returned::Text { replay, .. } => replay.as_ref(),
                    llm::Returned::Owned { .. }
                    | llm::Returned::Invalid { .. }
                    | llm::Returned::NotRun { .. }
                    | llm::Returned::Withdrawn
                    | llm::Returned::Served { .. } => None,
                },
                llm::Block::Opaque { bytes } => {
                    documents = documents.checked_add(size(bytes.len())?).ok_or(Error::Limit)?;
                    None
                }
            };
            if let Some(replay) = replay {
                documents = documents.checked_add(size(replay.bytes.len())?).ok_or(Error::Limit)?;
            }
        }
    }
    if documents > limits.history_bytes {
        return Err(Error::Limit);
    }
    Ok(())
}

fn translate_block(
    block: llm::Block,
    namespace: &tools::Namespace,
    limits: &Limits,
    maximum: u32,
    previous: Option<&skein_llm::Message>,
) -> Result<skein_llm::Block, Error> {
    match block {
        llm::Block::Text { text, replay } => Ok(skein_llm::Block::Text { text, replay: replay_value(replay, limits)? }),
        llm::Block::Refusal { text, replay } => {
            Ok(skein_llm::Block::Refusal { text, replay: replay_value(replay, limits)? })
        }
        llm::Block::Opaque { bytes } => {
            let replay_limits = crate::limits::replay_limits(&limits.client).ok_or(Error::Limit)?;
            let replay = match skein_llm::Replay::from_bytes(&bytes, &replay_limits) {
                Ok(replay) => replay,
                Err(error) => return Err(Error::from_client(error)),
            };
            Ok(skein_llm::Block::Reasoning { replay })
        }
        llm::Block::ToolCall { id, name, input, replay } => {
            Ok(skein_llm::Block::ToolCall { id, name, arguments: input, replay: replay_value(replay, limits)? })
        }
        llm::Block::ToolResult { id, result } => {
            let (text, error) = result_text(result, namespace, limits, maximum, previous)?;
            Ok(skein_llm::Block::ToolResult { id, text, is_error: error })
        }
    }
}

fn result_text(
    result: llm::Returned,
    namespace: &tools::Namespace,
    limits: &Limits,
    maximum: u32,
    previous: Option<&skein_llm::Message>,
) -> Result<(Box<[u8]>, bool), Error> {
    match result {
        llm::Returned::Text { text, error, replay } => match replay {
            Some(_) => Err(Error::Unsupported),
            None => Ok((crate::render::cut_text(text, maximum), error)),
        },
        llm::Returned::Withdrawn => Ok((bytes::copy_of(b"withdrawn"), true)),
        llm::Returned::NotRun { why } => match why {
            llm::NotRun::Stopped => Ok((bytes::copy_of(b"not run"), true)),
            llm::NotRun::AfterFailedWrite { call } => Ok((after_failed_write(call, previous, maximum)?, true)),
            llm::NotRun::OutsideChoice { choice } => Ok((outside_choice(choice.as_deref(), maximum)?, true)),
        },
        llm::Returned::Invalid { problem } => Ok((crate::failure::problem(problem, maximum)?, true)),
        llm::Returned::Served { returned, error } => {
            let value = smith_domain::feedback(returned, u64::from(limits.rendered_result)).or(Err(Error::Limit))?;
            if value.error != error {
                return Err(Error::Invalid);
            }
            Ok((crate::render::cut_text(value.text, maximum), value.error))
        }
        llm::Returned::Owned { outcome } => crate::render::render_outcome(&outcome, namespace, maximum),
    }
}

fn replay_value(replay: Option<llm::Replay>, limits: &Limits) -> Result<Option<skein_llm::Replay>, Error> {
    match replay {
        Some(replay) => {
            let replay_limits = crate::limits::replay_limits(&limits.client).ok_or(Error::Limit)?;
            match skein_llm::Replay::from_bytes(&replay.bytes, &replay_limits) {
                Ok(replay) => Ok(Some(replay)),
                Err(error) => Err(Error::from_client(error)),
            }
        }
        None => Ok(None),
    }
}

pub(crate) fn object(data: &[u8], limits: &Limits) -> Result<skein_llm::Json, Error> {
    let bytes = u32::try_from(data.len()).or(Err(Error::Limit))?;
    if bytes > limits.client.request {
        return Err(Error::PathLimit { which: llm::Cap::Request, bound: u64::from(limits.client.request) });
    }
    let value = match skein_llm::Json::from_bytes(data, &crate::limits::document_limits(&limits.client, bytes)) {
        Ok(value) => value,
        Err(error) => return Err(Error::from_client(skein_llm::document_error(error))),
    };
    match value.document().token(0) {
        Some(record) if record.kind == Kind::ObjectStart => {}
        Some(_) | None => return Err(Error::Invalid),
    }
    Ok(value)
}

fn size(value: usize) -> Result<u64, Error> {
    u64::try_from(value).or(Err(Error::Limit))
}

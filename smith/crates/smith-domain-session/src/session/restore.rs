//! Validation and restoration of concrete transcripts. This module knows no live peer identity; restore checks the bounded history before opening a kit.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use super::{
    Block, Box, Conversation, Decoded, Limits, List, Message, Returned, Role, compaction, content_cost, len, mem,
    provider_reserve, size_of, unrun,
};

pub(super) fn recorded_sequence(
    turn: &crate::record::Turn,
    opening: bool,
    previous: u32,
) -> Result<u32, crate::record::Refusal> {
    use crate::record::{Kind, Refusal};
    match turn.kind {
        Kind::First(_) => {
            if !opening || turn.sequence != 1 {
                return Err(Refusal::Malformed);
            }
            Ok(previous)
        }
        Kind::Window(_) => {
            if !opening {
                return Err(Refusal::Malformed);
            }
            turn.sequence.checked_sub(1).ok_or(Refusal::Malformed)
        }
        Kind::Ordinary => {
            if opening {
                return Err(Refusal::Malformed);
            }
            Ok(previous)
        }
    }
}

pub(super) fn recorded_turn_shape(turn: &crate::record::Turn, assistants: u32) -> Result<(), crate::record::Refusal> {
    use crate::record::Refusal;
    match turn.kind {
        crate::record::Kind::First(_) | crate::record::Kind::Ordinary => {
            if assistants != 1 {
                return Err(Refusal::Malformed);
            }
        }
        crate::record::Kind::Window(_) => {
            if turn.messages.len() != 1 || assistants != 0 {
                return Err(Refusal::Malformed);
            }
            let message = turn.messages.first().expect("validated one window message");
            for block in &message.content {
                match block {
                    Block::Text { .. } => {}
                    Block::Refusal { .. }
                    | Block::Opaque { .. }
                    | Block::ToolCall { .. }
                    | Block::ToolResult { .. } => {
                        return Err(Refusal::Malformed);
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn validate_transcript(
    conversation: &Conversation,
    transcript: &crate::record::Transcript,
    answered: &[crate::record::Answered],
    limits: &Limits,
) -> Result<(u32, u64, Resumed), crate::record::Refusal> {
    use crate::record::{Refusal, VERSION};
    if transcript.version != VERSION {
        return Err(Refusal::Version);
    }
    // Check sizes before cloning or allocating history into the bounded list.
    if transcript.turns.len() > usize::try_from(limits.history_messages).expect("u32 fits") {
        return Err(Refusal::TooLarge);
    }
    let mut count = 1_u32;
    let mut blocks = 1_u32;
    let mut bytes = conversation.bytes;
    let mut sequence = 0_u32;
    let mut opening = true;
    let mut previous: Option<&Message> = None;
    let mut withheld: Option<&[u8]> = None;
    for turn in &transcript.turns {
        if turn.version != VERSION {
            return Err(Refusal::Version);
        }
        if turn.model != transcript.turns.first().expect("a turn implies nonempty history").model {
            return Err(Refusal::Malformed);
        }
        sequence = recorded_sequence(turn, opening, sequence)?;
        opening = false;
        sequence = sequence.checked_add(1).ok_or(Refusal::Malformed)?;
        if turn.sequence != sequence || turn.messages.is_empty() {
            return Err(Refusal::Malformed);
        }
        if turn.messages.len() > usize::try_from(limits.history_messages).expect("u32 fits") {
            return Err(Refusal::TooLarge);
        }
        let mut assistants = 0_u32;
        for message in &turn.messages {
            blocks = recorded_blocks(blocks, message, limits)?;
            count = count.checked_add(1).ok_or(Refusal::TooLarge)?;
            if count >= limits.history_messages {
                return Err(Refusal::TooLarge);
            }
            recorded_shape(message, limits)?;
            validate_recorded(message)?;
            match withheld {
                Some(id) => {
                    validate_wait_result(message, id)?;
                    withheld = None;
                }
                None => {
                    let missing = validate_link(previous, message)?;
                    if let Some(position) = missing {
                        let assistant = previous.expect("a missing result follows its assistant call");
                        withheld = match assistant
                            .content
                            .get(usize::try_from(position).expect("u32 fits"))
                            .expect("validated call position")
                        {
                            Block::ToolCall { id, .. } => Some(id),
                            Block::Text { .. }
                            | Block::Opaque { .. }
                            | Block::Refusal { .. }
                            | Block::ToolResult { .. } => {
                                unreachable!("validation returns a call position")
                            }
                        };
                    }
                }
            }
            previous = Some(message);
            if message.role == Role::Assistant {
                assistants = assistants.saturating_add(1);
            }
            bytes =
                bytes.checked_add(content_cost(&message.content).ok_or(Refusal::TooLarge)?).ok_or(Refusal::TooLarge)?;
            if bytes > limits.history_bytes {
                return Err(Refusal::TooLarge);
            }
        }
        recorded_turn_shape(turn, assistants)?;
    }
    if sequence == 0 {
        return Err(Refusal::Malformed);
    }
    let tail = previous.expect("validated nonempty turn");
    validate_answers(tail, answered, sequence)?;
    if count.checked_add(3).ok_or(Refusal::TooLarge)? > limits.history_messages || bytes > limits.history_bytes {
        return Err(Refusal::TooLarge);
    }
    let resumed = resumed_tail(tail, withheld);
    Ok((sequence, bytes, resumed))
}

/// Every recorded block leaves receiving credit for an assistant and its results.
pub(super) fn recorded_blocks(blocks: u32, message: &Message, limits: &Limits) -> Result<u32, crate::record::Refusal> {
    use crate::record::Refusal;
    let blocks = blocks
        .checked_add(u32::try_from(message.content.len()).or(Err(Refusal::TooLarge))?)
        .ok_or(Refusal::TooLarge)?;
    let reserved = limits.completion_blocks.checked_mul(2).ok_or(Refusal::TooLarge)?;
    if blocks.checked_add(reserved).ok_or(Refusal::TooLarge)? > limits.history_blocks {
        return Err(Refusal::TooLarge);
    }
    Ok(blocks)
}

pub(super) fn validate_answers(
    tail: &Message,
    answered: &[crate::record::Answered],
    sequence: u32,
) -> Result<(), crate::record::Refusal> {
    use crate::record::Refusal;
    for (index, answer) in answered.iter().enumerate() {
        if tail.role != Role::Assistant || answer.origin.sequence != sequence {
            return Err(Refusal::Malformed);
        }
        let Ok(position) = usize::try_from(answer.origin.position) else {
            return Err(Refusal::Malformed);
        };
        match tail.content.get(position) {
            Some(Block::ToolCall { .. }) => {}
            Some(Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolResult { .. })
            | None => {
                return Err(Refusal::Malformed);
            }
        }
        for earlier in answered.iter().take(index) {
            if earlier.origin == answer.origin {
                return Err(Refusal::Malformed);
            }
        }
        match answer.result {
            Returned::Text { .. } => {}
            Returned::Owned { .. } | Returned::Invalid { .. } | Returned::NotRun { .. } | Returned::Withdrawn => {
                return Err(Refusal::Malformed);
            }
        }
    }
    Ok(())
}

pub(super) fn resumed_tail(tail: &Message, withheld: Option<&[u8]>) -> Resumed {
    match withheld {
        Some(id) => Resumed::WaitWithheld { id: skein_lib::bytes::copy_of(id) },
        None => match tail.role {
            Role::User => Resumed::Settled,
            Role::Assistant => {
                let mut wait = None;
                for block in &tail.content {
                    match block {
                        Block::ToolCall { id, name, .. } if name.as_ref() == b"wait" => {
                            if wait.is_some() {
                                // A generic assistant tail may hold several unstarted calls.
                                // Only the single own wait tail identifies a held wait.
                                return Resumed::Settled;
                            }
                            wait = Some(id);
                        }
                        Block::ToolCall { .. }
                        | Block::Text { .. }
                        | Block::Opaque { .. }
                        | Block::Refusal { .. }
                        | Block::ToolResult { .. } => {}
                    }
                }
                match wait {
                    Some(id) => Resumed::WaitWithheld { id: id.clone() },
                    None => Resumed::Settled,
                }
            }
        },
    }
}

pub(super) fn restore(
    conversation: &mut Conversation,
    mut transcript: crate::record::Transcript,
    mut answered: Box<[crate::record::Answered]>,
    limits: &Limits,
) -> Result<u32, crate::record::Refusal> {
    use crate::record::Refusal;
    let (sequence, mut bytes, resumed) = validate_transcript(conversation, &transcript, &answered, limits)?;
    conversation.jobs.highest = transcript.turns.last().expect("validated nonempty history").jobs;
    compaction::restore(conversation, &transcript, &mut bytes)?;
    bytes = bytes.checked_sub(compaction::carry_over(conversation, &mut transcript)?).ok_or(Refusal::TooLarge)?;
    conversation.affinity = match &transcript.turns.first().expect("validated nonempty history").kind {
        crate::record::Kind::First(opener) | crate::record::Kind::Window(opener) => opener.affinity,
        crate::record::Kind::Ordinary => unreachable!("validated opening kind"),
    };
    let prompt = conversation.transcript.get(0).expect("admission held the initial prompt");
    let prompt_charge = content_cost(&prompt.content).ok_or(Refusal::TooLarge)?;
    let last =
        transcript.turns.last().expect("validated nonempty history").messages.last().expect("validated nonempty turn");
    let waiting = match &resumed {
        Resumed::Settled => false,
        Resumed::WaitWithheld { .. } => true,
    };
    let added = match last.role {
        Role::Assistant => {
            unrun_cost(&last.content, initial_text(prompt), &answered, waiting).ok_or(Refusal::TooLarge)?
        }
        Role::User => match &resumed {
            Resumed::Settled => prompt_charge,
            Resumed::WaitWithheld { id } => {
                prompt_charge.checked_add(len(id).ok_or(Refusal::TooLarge)?).ok_or(Refusal::TooLarge)?
            }
        },
    };
    // The yielded tail needs one concrete result per call and copied ids.
    // Account all of them before moving history or allocating waking content.
    bytes = bytes.checked_sub(prompt_charge).ok_or(Refusal::TooLarge)?;
    bytes = bytes.checked_add(added).ok_or(Refusal::TooLarge)?;
    let reserve = provider_reserve(limits).ok_or(Refusal::TooLarge)?;
    if bytes.checked_add(reserve).ok_or(Refusal::TooLarge)? > limits.history_bytes {
        return Err(Refusal::TooLarge);
    }
    // Move the waking prompt: cloning its potentially large text before
    // clearing the initial transcript would also exceed the bounded staging.
    let initial = mem::replace(&mut conversation.transcript, List::with_capacity(limits.history_messages));
    let mut initial = initial.into_boxed().into_iter();
    let prompt = initial.next().expect("admission held the initial prompt");
    assert!(initial.next().is_none(), "admission holds exactly one user message");
    drop(initial);
    for turn in transcript.turns {
        for message in turn.messages {
            conversation.transcript.push(message).expect("history counted before allocation");
        }
    }
    conversation.window.last_prompt_bytes = compaction::prompt_prefix_bytes(
        conversation,
        usize::try_from(conversation.window.following.saturating_sub(1)).expect("bounded history"),
    )
    .ok_or(Refusal::TooLarge)?;
    let last = conversation.transcript.last().expect("nonempty history");
    let content = match last.role {
        Role::Assistant => {
            let mut blocks = prompt.content.into_iter();
            let text = match blocks.next().expect("admission's initial text") {
                Block::Text { text, .. } => text,
                Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolCall { .. } | Block::ToolResult { .. } => {
                    unreachable!("admission constructs one text block")
                }
            };
            unrun(conversation, text, &mut answered, waiting)
        }
        Role::User => match resumed {
            Resumed::Settled => prompt.content,
            Resumed::WaitWithheld { id } => {
                let mut parts = prompt.content.into_iter();
                let text = match parts.next().expect("admission's initial text") {
                    Block::Text { text, .. } => text,
                    Block::Refusal { .. }
                    | Block::Opaque { .. }
                    | Block::ToolCall { .. }
                    | Block::ToolResult { .. } => {
                        unreachable!("admission constructs one text block")
                    }
                };
                Box::new([Block::ToolResult { id, result: Returned::Text { text, error: false, replay: None } }])
            }
        },
    };
    conversation.transcript.push(Message { role: Role::User, content }).expect("one prompt reserved");
    conversation.bytes = bytes;
    Ok(sequence)
}

pub(super) fn initial_text(prompt: &Message) -> &[u8] {
    match prompt.content.first().expect("admission's initial text") {
        Block::Text { text, .. } => text,
        Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolCall { .. } | Block::ToolResult { .. } => {
            unreachable!("admission constructs one text block")
        }
    }
}

/// The waking user message plus every saved call's result or `NotRun`.
/// Counts the eventual concrete blocks and copied provider ids without owning
/// any of them, so a refusal allocates no tail-sized scratch.
pub(super) fn unrun_cost(
    content: &[Block],
    text: &[u8],
    answered: &[crate::record::Answered],
    waiting: bool,
) -> Option<u64> {
    let block = u64::try_from(size_of::<Block>()).ok()?;
    let mut cost = len(text)?;
    if !waiting {
        cost = cost.checked_add(block)?;
    }
    for (position, part) in content.iter().enumerate() {
        match part {
            Block::ToolCall { id, .. } => {
                cost = cost.checked_add(block)?.checked_add(len(id)?)?;
                for answer in answered {
                    if usize::try_from(answer.origin.position).ok()? == position {
                        match &answer.result {
                            Returned::Text { text, .. } => cost = cost.checked_add(len(text)?)?,
                            Returned::Owned { .. }
                            | Returned::Invalid { .. }
                            | Returned::NotRun { .. }
                            | Returned::Withdrawn => {
                                return None;
                            }
                        }
                    }
                }
            }
            Block::Opaque { .. } | Block::Text { .. } | Block::Refusal { .. } | Block::ToolResult { .. } => {}
        }
    }
    Some(cost)
}

pub(super) fn validate_recorded(message: &Message) -> Result<(), crate::record::Refusal> {
    use crate::record::Refusal;
    for block in &message.content {
        match block {
            Block::Opaque { .. } | Block::Text { .. } | Block::Refusal { .. } => {}
            Block::ToolCall { call, .. } => {
                if message.role != Role::Assistant {
                    return Err(Refusal::Malformed);
                }
                match call {
                    Decoded::Historical => {}
                    Decoded::Builtin { .. }
                    | Decoded::Owned { .. }
                    | Decoded::Delegated { .. }
                    | Decoded::Invalid { .. } => {
                        return Err(Refusal::Unresolved);
                    }
                }
            }
            Block::ToolResult { .. } => {
                if message.role != Role::User {
                    return Err(Refusal::Malformed);
                }
            }
        }
    }
    Ok(())
}

/// Result ids must resolve to the preceding assistant's calls, in call order.
/// A yielded assistant may be the tail; resuming supplies its `NotRun` results.
pub(super) fn validate_link(
    previous: Option<&Message>,
    message: &Message,
) -> Result<Option<u32>, crate::record::Refusal> {
    use crate::record::Refusal;
    let Some(previous) = previous else {
        if message.role != Role::User {
            return Err(Refusal::Malformed);
        }
        for block in &message.content {
            match block {
                Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } => {}
                Block::ToolCall { .. } | Block::ToolResult { .. } => return Err(Refusal::Malformed),
            }
        }
        return Ok(None);
    };
    match message.role {
        Role::Assistant => {
            if previous.role != Role::User {
                return Err(Refusal::Malformed);
            }
            // Call ids must be unique within a provider completion.
            for (index, block) in message.content.iter().enumerate() {
                match block {
                    Block::ToolCall { id, .. } => {
                        for earlier in message.content.iter().take(index) {
                            match earlier {
                                Block::ToolCall { id: other, .. } => {
                                    if id == other {
                                        return Err(Refusal::Malformed);
                                    }
                                }
                                Block::Text { .. }
                                | Block::Refusal { .. }
                                | Block::Opaque { .. }
                                | Block::ToolResult { .. } => {}
                            }
                        }
                    }
                    Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolResult { .. } => {}
                }
            }
        }
        Role::User => return validate_results(previous, message),
    }
    Ok(None)
}

/// A resumed withheld result must appear before the next assistant turn.
pub(super) fn validate_wait_result(message: &Message, wait: &[u8]) -> Result<(), crate::record::Refusal> {
    if message.role != Role::User {
        return Err(crate::record::Refusal::Malformed);
    }
    let mut found = false;
    for block in &message.content {
        match block {
            Block::ToolResult { id, .. } if id.as_ref() == wait && !found => found = true,
            Block::ToolResult { .. } | Block::ToolCall { .. } => return Err(crate::record::Refusal::Malformed),
            Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } => {}
        }
    }
    if !found {
        return Err(crate::record::Refusal::Malformed);
    }
    Ok(())
}

pub(super) fn validate_results(previous: &Message, message: &Message) -> Result<Option<u32>, crate::record::Refusal> {
    use crate::record::Refusal;
    let mut next = 0_u32;
    let mut seen = next_result(&message.content, &mut next)?;
    let mut withheld = None;
    if previous.role == Role::Assistant {
        for (position, block) in previous.content.iter().enumerate() {
            match block {
                Block::ToolCall { id, name, .. } => {
                    let matches = match seen {
                        Some(position) => {
                            result_id(
                                message
                                    .content
                                    .get(usize::try_from(position).expect("u32 fits"))
                                    .expect("validated result position"),
                            ) == id.as_ref()
                        }
                        None => false,
                    };
                    if matches {
                        seen = next_result(&message.content, &mut next)?;
                    } else if name.as_ref() == b"wait" && withheld.is_none() {
                        withheld = Some(u32::try_from(position).or(Err(Refusal::TooLarge))?);
                    } else {
                        return Err(Refusal::Malformed);
                    }
                }
                Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolResult { .. } => {}
            }
        }
    }
    if seen.is_some() {
        return Err(Refusal::Malformed);
    }
    Ok(withheld)
}

pub(super) fn result_id(block: &Block) -> &[u8] {
    match block {
        Block::ToolResult { id, .. } => id,
        Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolCall { .. } => {
            unreachable!("next_result returns a result position")
        }
    }
}

pub(super) fn next_result(blocks: &[Block], next: &mut u32) -> Result<Option<u32>, crate::record::Refusal> {
    let begin = *next;
    let end = u32::try_from(blocks.len()).or(Err(crate::record::Refusal::TooLarge))?;
    for position in begin..end {
        *next = position.checked_add(1).ok_or(crate::record::Refusal::TooLarge)?;
        match blocks.get(usize::try_from(position).expect("u32 fits")).expect("within blocks") {
            Block::ToolResult { .. } => return Ok(Some(position)),
            Block::ToolCall { .. } => return Err(crate::record::Refusal::Malformed),
            Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } => {}
        }
    }
    Ok(None)
}

pub(super) fn recorded_shape(message: &Message, limits: &Limits) -> Result<(), crate::record::Refusal> {
    let fixed = match u64::try_from(size_of::<Block>()).ok() {
        Some(block) => match u64::try_from(message.content.len()).ok() {
            Some(count) => block.checked_mul(count),
            None => None,
        },
        None => None,
    };
    match fixed {
        Some(fixed) if fixed <= limits.history_bytes => Ok(()),
        Some(_) | None => Err(crate::record::Refusal::TooLarge),
    }
}

/// The restored opening is either fresh user text or the result of the last own wait.
#[derive(Debug)]
pub(super) enum Resumed {
    Settled,
    WaitWithheld { id: Box<[u8]> },
}

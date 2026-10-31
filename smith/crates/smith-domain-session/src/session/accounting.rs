//! Retained bytes, receiving credit and completion spend. This module knows no provider wire form; admission and terminal handlers use its checked sums.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use core::mem;

use super::{
    Block, Box, Call, Conversation, Decoded, Descriptor, End, Entry, Failure, Limits, Outcome, Problem, Queue, Request,
    Returned, Slot, Spending, ToolChoice, Usage, blocks_fit, choice_bytes, chosen, compaction, receiving_reserve,
    size_of, tools,
};

pub(super) fn reservation(
    conversation: &Conversation,
    input_bytes: u64,
    allowance: u64,
    max_tokens: u32,
) -> Option<u64> {
    let input_tokens = input_bytes.checked_add(allowance)?;
    let prices = conversation.spending.prices;
    let prices = crate::record::Prices { input: prices.reserved_input(), ..prices };
    let most = prices.price(Usage {
        input_tokens: Some(input_tokens),
        output_tokens: Some(u64::from(max_tokens)),
        ..Usage::ZERO
    })?;
    let total = conversation.spending.total.checked_add(most)?;
    if total > conversation.spending.limit {
        return None;
    }
    Some(most)
}

pub(super) fn completion_price(spending: &Spending, usage: Usage) -> Option<u64> {
    if usage.input_tokens.is_none() || usage.output_tokens.is_none() {
        return spending.reservation;
    }
    spending.prices.price(usage)
}

/// Counts a completion that came back, and tells the opener.
pub(super) fn used(conversation: &mut Conversation, usage: Usage, out: &mut Queue<Request>) -> Result<(), End> {
    let price = completion_price(&conversation.spending, usage).ok_or(End::PriceOverflow)?;
    let own = conversation.spending.own.checked_add(price).ok_or(End::PriceOverflow)?;
    let inclusive = conversation.spending.total.checked_add(price).ok_or(End::PriceOverflow)?;
    let cumulative = checked_usage(conversation.usage, usage)?;
    let reported = checked_reported(conversation.reported, usage).ok_or(End::UsageOverflow)?;
    let next_turn = conversation.turns.checked_add(1).ok_or(End::UsageOverflow)?;
    conversation.turns = next_turn;
    conversation.usage = cumulative;
    conversation.reported = reported;
    conversation.spending.reservation = None;
    conversation.spending.last_charge = price;
    conversation.spending.own = own;
    conversation.spending.total = inclusive;
    conversation.cursor.pending = Some(usage);
    out.push(Request::Priced { opener: conversation.opener, spent: inclusive, own_spent: own });
    out.push(Request::Used { opener: conversation.opener, usage });
    Ok(())
}

/// A cumulative raw count is known only if every accepted count was reported.
pub(super) fn checked_usage(previous: Usage, received: Usage) -> Result<Usage, End> {
    Ok(Usage {
        input_tokens: sum_count(previous.input_tokens, received.input_tokens)?,
        output_tokens: sum_count(previous.output_tokens, received.output_tokens)?,
        cache_read_tokens: sum_count(previous.cache_read_tokens, received.cache_read_tokens)?,
        cache_write_tokens: sum_count(previous.cache_write_tokens, received.cache_write_tokens)?,
        reasoning_tokens: sum_count(previous.reasoning_tokens, received.reasoning_tokens)?,
    })
}

pub(super) fn sum_count(previous: Option<u64>, received: Option<u64>) -> Result<Option<u64>, End> {
    match previous {
        Some(previous) => match received {
            Some(received) => Ok(Some(previous.checked_add(received).ok_or(End::UsageOverflow)?)),
            None => Ok(None),
        },
        None => Ok(None),
    }
}

/// Budget bookkeeping adds supplied counts across gaps, preserving absence.
pub(super) fn checked_reported(previous: Usage, received: Usage) -> Option<Usage> {
    Some(Usage {
        input_tokens: add_reported_count(previous.input_tokens, received.input_tokens).ok()?,
        output_tokens: add_reported_count(previous.output_tokens, received.output_tokens).ok()?,
        cache_read_tokens: add_reported_count(previous.cache_read_tokens, received.cache_read_tokens).ok()?,
        cache_write_tokens: add_reported_count(previous.cache_write_tokens, received.cache_write_tokens).ok()?,
        reasoning_tokens: add_reported_count(previous.reasoning_tokens, received.reasoning_tokens).ok()?,
    })
}

pub(super) fn add_reported_count(previous: Option<u64>, received: Option<u64>) -> Result<Option<u64>, End> {
    match previous {
        Some(previous) => match received {
            Some(received) => Ok(Some(previous.checked_add(received).ok_or(End::UsageOverflow)?)),
            None => Ok(Some(previous)),
        },
        None => Ok(received),
    }
}

/// Counts `cost` more bytes against the session's limit, or says they do not
/// fit.
#[must_use]
pub(super) fn charge(conversation: &mut Conversation, cost: Option<u64>, limits: &Limits) -> bool {
    let Some(cost) = cost else {
        return false;
    };
    let Some(bytes) = conversation.bytes.checked_add(cost) else {
        return false;
    };
    match bytes.checked_add(conversation.reserved) {
        Some(total) if total <= limits.history_bytes => {}
        Some(_) | None => return false,
    }
    conversation.bytes = bytes;
    true
}

pub(crate) fn provider_reserve(limits: &Limits) -> Option<u64> {
    if limits.completion_bytes == 0 || limits.completion_blocks == 0 {
        return None;
    }
    // All copied result IDs and Invalid details already appear in C's content
    // payload, so their aggregate is <= C. Result wrappers require an additional
    // N*max(Block,Slot), irrespective of enum padding. Slot containers and
    // simultaneous result assembly are measured under scenario ceilings.
    let cell = size_of::<Block>().max(size_of::<Slot>());
    let wrappers = u64::try_from(cell).ok()?.checked_mul(u64::from(limits.completion_blocks))?;
    // Rules 1 and 2: copied IDs/details share one decoded answer total.
    let completion = limits.completion_bytes.checked_add(limits.answer_bytes)?.checked_add(wrappers)?;
    let terminal = u64::try_from(size_of::<Failure>())
        .ok()?
        .checked_add(u64::try_from(size_of::<crate::llm::Evidence>()).ok()?)?
        .checked_add(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?
        .checked_add(u64::from(limits.failure_bytes))?;
    Some(completion.max(terminal))
}

pub(super) fn provider_room(conversation: &Conversation, limits: &Limits) -> bool {
    if conversation.compaction.asking() {
        return provider_reserve(limits).is_some();
    }
    let Some(pending) = limits.completion_blocks.checked_mul(2) else {
        return false;
    };
    if !blocks_fit(conversation, pending, limits) {
        return false;
    }
    let Some(reserve) = receiving_reserve(conversation, limits) else {
        return false;
    };
    if conversation.transcript.room() < 2 {
        return false;
    }
    let Some(held) = conversation.bytes.checked_add(conversation.reserved) else {
        return false;
    };
    match held.checked_add(reserve) {
        Some(total) => total <= limits.history_bytes,
        None => false,
    }
}

pub(super) fn reserve_provider(conversation: &mut Conversation, limits: &Limits) -> bool {
    if !provider_room(conversation, limits) {
        return false;
    }
    // A summary replaces the old window; its input is covered by staging,
    // and no completion or result skeleton is appended to that history.
    let credit = if conversation.compaction.asking() {
        0
    } else {
        receiving_reserve(conversation, limits).expect("checked receiving bounds")
    };
    conversation.provider = match mem::replace(&mut conversation.provider, Provider::Idle) {
        Provider::Idle => Provider::Reserved { credit },
        Provider::Reserved { .. } => unreachable!("one provider right at a time"),
    };
    follow_provider(conversation, 0);
    true
}

pub(super) fn release_provider(conversation: &mut Conversation) {
    let _released: Option<u64> = take_provider(conversation);
}

pub(super) fn receive_completion(
    conversation: &mut Conversation,
    content: &[Block],
    calls: u32,
    limits: &Limits,
) -> bool {
    let cost = if calls == 0 {
        content_cost(content)
    } else {
        held(content, calls, &conversation.choice, conversation.choice_names.as_deref())
    };
    let credit = take_provider(conversation).expect("completion owns receiving credit");
    let Some(content_bytes) = content_cost(content) else {
        return false;
    };
    if content_bytes > limits.completion_bytes || count(content.len()) > limits.completion_blocks {
        return false;
    }
    let Some(cost) = cost else {
        return false;
    };
    assert!(cost <= credit, "complete content and result skeleton obey reserved caps");
    let fits = charge(conversation, Some(cost), limits);
    conversation.window.completed(
        conversation.cursor.pending.expect("ordinary completion usage retained"),
        conversation.output,
        conversation.transcript.len().checked_add(1).expect("reserved assistant slot"),
        compaction::prompt_bytes(conversation).unwrap_or(u64::MAX),
    );
    assert!(fits, "valid completion consumes space reserved before provider work");
    true
}

pub(super) fn owned_credit(call: &Call, limits: &Limits) -> u64 {
    tools::result_bound(call, &limits.tools).expect("admitted result bounds")
}

pub(super) fn delegated_credit(limits: &Limits) -> u64 {
    limits.delegated_result_bytes
}

pub(super) fn receive_result(conversation: &mut Conversation, credit: u64, result: &Returned, limits: &Limits) {
    let cost = returned_cost(result).expect("bounded result cost fits");
    assert!(cost <= credit, "lower terminal obeys its pre-effect result cap");
    conversation.reserved = conversation.reserved.checked_sub(credit).expect("call owned result credit");
    let fits = charge(conversation, Some(cost), limits);
    assert!(fits, "valid result consumes reserved space through close");
}

/// What a spec costs: its names, the tools its opener serves, and its first
/// message.
pub(super) fn spec_cost(model: &[u8], system: &[u8], delegated: &[Descriptor], content: &[Block]) -> Option<u64> {
    let descriptor = u64::try_from(size_of::<Descriptor>()).ok()?;
    let delegated = descriptor.checked_mul(u64::try_from(delegated.len()).ok()?)?;
    len(model)?.checked_add(len(system)?)?.checked_add(delegated)?.checked_add(content_cost(content)?)
}

/// What an assistant message with `calls` tool calls costs while its tools run:
/// the message, a block for each call's result, the copied ID it echoes and
/// the answers to invalid calls, which are known already. Admission secured this whole
/// skeleton before provider work and separately reserves each batch's payloads
/// before effects. The terminal transfers its credit into held bytes.
/// Slot containers and their transient coexistence with assembled Block arrays
/// are measured under scenario ceilings; no Slot/Block size inequality is assumed.
pub(super) fn held(content: &[Block], calls: u32, choice: &ToolChoice, names: Option<&[Box<[u8]>]>) -> Option<u64> {
    let slots = u64::try_from(size_of::<Block>()).ok()?.checked_mul(u64::from(calls))?;
    let mut cost = content_cost(content)?.checked_add(slots)?;
    for block in content {
        if !chosen(choice, block) {
            cost = cost.checked_add(choice_bytes(names))?;
        }
        match block {
            Block::ToolCall { id, name: _, input: _, call: Decoded::Invalid { problem, .. }, .. } => {
                cost = cost.checked_add(len(id)?)?.checked_add(problem_cost(problem)?)?;
            }
            Block::ToolCall {
                id,
                name: _,
                input: _,
                call: Decoded::Builtin { .. } | Decoded::Owned { .. } | Decoded::Delegated { .. } | Decoded::Historical,
                ..
            } => {
                cost = cost.checked_add(len(id)?)?;
            }
            Block::Opaque { .. } | Block::Text { .. } | Block::Refusal { .. } | Block::ToolResult { .. } => {}
        }
    }
    Some(cost)
}

pub(crate) fn content_cost(content: &[Block]) -> Option<u64> {
    let mut cost: u64 = 0;
    for block in content {
        cost = cost.checked_add(block_cost(block)?)?;
    }
    Some(cost)
}

/// A block's fixed size plus its payload.
pub(super) fn block_cost(block: &Block) -> Option<u64> {
    u64::try_from(size_of::<Block>()).ok()?.checked_add(payload_cost(block)?)
}

/// The bytes a block holds beyond its fixed size: its own, and those of the
/// call or outcome it carries.
pub(super) fn payload_cost(block: &Block) -> Option<u64> {
    match block {
        Block::Text { text, replay } | Block::Refusal { text, replay } => {
            len(text)?.checked_add(replay_cost(replay.as_ref())?)
        }
        Block::Opaque { bytes } => len(bytes),
        Block::ToolCall { id, name, input, call, replay } => {
            // A delegated call is the opener's to hold.
            let decoded = match call {
                Decoded::Owned { call } => call_cost(call)?,
                Decoded::Builtin { .. } | Decoded::Delegated { .. } | Decoded::Historical => 0,
                Decoded::Invalid { problem, .. } => problem_cost(problem)?,
            };
            len(id)?
                .checked_add(len(name)?)?
                .checked_add(len(input)?)?
                .checked_add(decoded)?
                .checked_add(replay_cost(replay.as_ref())?)
        }
        Block::ToolResult { id, result } => len(id)?.checked_add(returned_cost(result)?),
    }
}

/// The replay wrapper is inline in Block; its owned envelope is additional.
pub(super) fn replay_cost(replay: Option<&crate::llm::Replay>) -> Option<u64> {
    match replay {
        Some(replay) => len(&replay.bytes),
        None => Some(0),
    }
}

/// The bytes a tool call's result holds beyond its block and its id.
pub(super) fn returned_cost(result: &Returned) -> Option<u64> {
    match result {
        Returned::Owned { outcome } => outcome_cost(outcome),
        Returned::Invalid { problem } => problem_cost(problem),
        Returned::NotRun { why } => match why {
            crate::llm::NotRun::Stopped | crate::llm::NotRun::AfterFailedWrite { .. } => Some(0),
            crate::llm::NotRun::OutsideChoice { choice } => Some(choice_bytes(choice.as_deref())),
        },
        Returned::Withdrawn => Some(0),
        Returned::Text { text, replay, .. } => len(text)?.checked_add(replay_cost(replay.as_ref())?),
    }
}

pub(super) fn call_cost(call: &Call) -> Option<u64> {
    call.owned_bytes()?.checked_sub(u64::try_from(size_of::<Call>()).ok()?)
}

pub(super) fn outcome_cost(outcome: &Outcome) -> Option<u64> {
    match outcome {
        Outcome::NotFound { path }
        | Outcome::NotDirectory { path }
        | Outcome::Outside { path }
        | Outcome::ReadOnly { path } => path.owned_bytes(),
        Outcome::Read { content, .. } => len(content),
        Outcome::Listed { entries, more: _ } => {
            let mut cost = u64::try_from(size_of::<Entry>()).ok()?.checked_mul(u64::try_from(entries.len()).ok()?)?;
            for entry in entries {
                cost = cost.checked_add(len(entry.name.as_ref())?)?;
            }
            Some(cost)
        }
        Outcome::Found { hits, more: _, timed_out: _ } => {
            let mut cost = u64::try_from(size_of::<tools::Hit>()).ok()?.checked_mul(u64::try_from(hits.len()).ok()?)?;
            for hit in hits {
                cost = cost.checked_add(len(&hit.path)?)?.checked_add(len(&hit.text)?)?;
            }
            Some(cost)
        }
        Outcome::Exited { head, tail, .. } | Outcome::Running { head, tail, .. } | Outcome::Job { head, tail, .. } => {
            len(head)?.checked_add(len(tail)?)
        }
        Outcome::EditFailed { edit: _, why } => match why {
            tools::Mismatch::Ambiguous { count: _, lines } => {
                u64::try_from(size_of::<u32>()).ok()?.checked_mul(u64::try_from(lines.len()).ok()?)
            }
            tools::Mismatch::NoMatch | tools::Mismatch::Unchanged => Some(0),
        },
        Outcome::Removed
        | Outcome::NotEmpty
        | Outcome::Written { .. }
        | Outcome::Edited { .. }
        | Outcome::NotGranted
        | Outcome::TooLong
        | Outcome::NotFile
        | Outcome::Linked
        | Outcome::Protected
        | Outcome::TooLarge { .. }
        | Outcome::NotRead
        | Outcome::Stale
        | Outcome::Failed { .. }
        | Outcome::TimedOut
        | Outcome::Cancelled
        | Outcome::Busy
        | Outcome::NulByte => Some(0),
    }
}

pub(super) fn problem_cost(problem: &Problem) -> Option<u64> {
    match problem {
        Problem::UnknownTool
        | Problem::Nesting
        | Problem::NotAnObject
        | Problem::TooLarge
        | Problem::Oversize { .. }
        | Problem::CutOff { .. }
        | Problem::JobsFull { .. }
        | Problem::UnknownJob { .. } => Some(0),
        Problem::Missing { field } | Problem::WrongType { field } | Problem::BadValue { field } => len(field),
    }
}

pub(super) fn len(bytes: &[u8]) -> Option<u64> {
    u64::try_from(bytes.len()).ok()
}

/// A count for a fact, which saturates rather than fails: facts decide
/// nothing.
pub(super) fn count(items: usize) -> u32 {
    u32::try_from(items).unwrap_or(u32::MAX)
}

/// Receiving credit is present only while a provider terminal is owed.
#[derive(Debug)]
pub(super) enum Provider {
    Idle,
    Reserved { credit: u64 },
}

impl Provider {
    pub(super) const fn credit(&self) -> Option<u64> {
        match self {
            Self::Idle => None,
            Self::Reserved { credit } => Some(*credit),
        }
    }
}

fn take_provider(conversation: &mut Conversation) -> Option<u64> {
    let credit = conversation.provider.credit();
    conversation.provider = match mem::replace(&mut conversation.provider, Provider::Idle) {
        Provider::Idle | Provider::Reserved { .. } => Provider::Idle,
    };
    follow_provider(conversation, credit.unwrap_or(0));
    credit
}

fn follow_provider(conversation: &mut Conversation, previous: u64) {
    conversation.reserved = conversation
        .reserved
        .checked_sub(previous)
        .expect("provider right owned credit")
        .checked_add(conversation.provider.credit().unwrap_or(0))
        .expect("checked receiving room");
}

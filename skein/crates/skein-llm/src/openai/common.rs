use crate::openai::{Input, Tool};
use skein_json::{Compact, collector, document, tokenizer, writer};
use skein_lib::List;

pub use crate::wire::{Limits, encoder_bound};

/// Bounds whole-value admission and decoded output.
/// Conservative native whole-value admission and decoded output bounds.
/// Request encoding uses `encoder_bound`, without compact copies.
#[must_use]
pub fn decoded_bound(limits: &Limits) -> Option<u64> {
    // Native temporary values belong to the request, answer or replay
    // budgets. A discarded wire event contributes no document-sized buffer.
    let temporary = limits.request.max(limits.answer).max(limits.input).max(limits.reasoning).max(limits.metadata);
    let tokens = document::document_bound(&document::Limits { tokens: limits.tokens, text: temporary })?;
    // Standalone whole-value request decoding admits caller documents here.
    // Every JSON token costs at least one byte in that request budget.
    // Its collector and the decoded native documents may briefly coexist.
    let request_count = u64::from(limits.request);
    let request_tokens = request_count.checked_mul(u64::try_from(size_of::<Compact>()).ok()?)?.checked_mul(2)?;
    let answer_records = u64::from(limits.output_items)
        .checked_mul(u64::from(limits.tokens))?
        .min(u64::from(limits.answer))
        .checked_mul(u64::try_from(size_of::<Compact>()).ok()?)?
        .checked_mul(2)?;
    // Whole-value collection remains a neutral JSON admission entrance for
    // schemas and replay metadata. The client prices its error body separately.
    let admission = collector::collector_bound(
        &collector::Limits {
            tokenizer: limits.tokenizer_limits(),
            tokens: limits.tokens,
            text: temporary,
            skip: u64::from(temporary),
        },
        &[],
        &collector::Filter { root: collector::Keep::Value },
    )?;
    let request_slots =
        List::<Input>::price(limits.history_items)?.checked_add(List::<Tool>::price(limits.tools)?)?.checked_mul(2)?;
    let documents = u64::from(temporary).checked_mul(8)?;
    let answer = u64::from(limits.answer).checked_mul(4)?;
    tokens
        .checked_mul(4)?
        .checked_add(request_tokens)?
        .checked_add(answer_records)?
        .checked_add(admission)?
        .checked_add(request_slots)?
        .checked_add(documents)?
        .checked_add(answer)?
        .checked_add(u64::from(limits.request))?
        .checked_add(tokenizer::tokenizer_bound(&limits.tokenizer_limits())?)?
        .checked_add(writer::encoder_bound(&writer::Limits { depth: limits.depth, length: temporary })?)?
        .checked_add(crate::openai::response::decoder_bound(limits)?)
}
pub use crate::wire::{DecodeError, ProviderError, RateLimit, classify};

pub(crate) use crate::wire::{append, clipped, limit_detail, measured};

use super::{Bounds, Undecodable, api, w};
use alloc::boxed::Box;

use temper_engine_domain_forge_vocabulary as top;

fn encode_entry(value: &skein_forge_client::Entry) -> w::Entry {
    crate::generated::store::entry_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
fn decode_entry(value: w::Entry) -> skein_forge_client::Entry {
    crate::generated::store::entry_to_domain(value, &super::Context::UNBOUNDED).expect(super::FITS)
}

pub(super) fn encode_entry_row(entry: &skein_forge_client::Entry, held: bool) -> w::EntryRow {
    w::EntryRow { entry: encode_entry(entry), held }
}
pub(super) fn decode_entry_row(value: w::EntryRow) -> top::Stored {
    let w::EntryRow { entry, held } = value;
    top::Stored::Entry { entry: decode_entry(entry), held }
}
pub(super) fn encode_proposed(value: &top::ProposedEffect) -> w::ProposedEffect {
    crate::generated::store::proposed_effect_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
pub(super) fn decode_proposed(value: w::ProposedEffect, bounds: &Bounds) -> Result<top::ProposedEffect, Undecodable> {
    crate::generated::store::proposed_effect_to_domain(value, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))
}

fn encode_payload(value: &top::CallPayload) -> w::CallPayload {
    crate::generated::store::call_payload_to_record(value, &super::Context::UNBOUNDED).expect(super::FITS)
}
fn decode_payload(value: w::CallPayload, bounds: &Bounds) -> Result<top::CallPayload, Undecodable> {
    crate::generated::store::call_payload_to_domain(value, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))
}
pub(super) fn encode_call(key: &top::NamedCall, answer: &top::CallPayload) -> w::Call {
    let key = crate::generated::store::named_call_to_record(key, &super::Context::UNBOUNDED).expect(super::FITS);
    w::Call { key, answer: encode_payload(answer) }
}
pub(super) fn decode_call(value: w::Call, bounds: &Bounds) -> Result<top::Stored, Undecodable> {
    let w::Call { key, answer } = value;
    let key = crate::generated::store::named_call_to_domain(key, &super::Context::UNBOUNDED)
        .or(Err(Undecodable::Malformed))?;
    Ok(top::Stored::Call { key, answer: decode_payload(answer, bounds)? })
}

/// Owner-shaped `ReadResult` (skein/docs/design/data/codec/maps.md, 3.3).
pub fn read_result_to_record(
    value: &Result<skein_forge::api::Answer, skein_forge::api::Error>,
    _context: &super::Context,
) -> Result<w::ReadResult, w::Problem> {
    Ok(match value {
        Ok(value) => w::ReadResult::Answer(Box::new(w::AnswerValue { answer: super::answers::encode(value) })),
        Err(error) => w::ReadResult::Error(w::ErrorValue { error: api::encode_error(*error) }),
    })
}
pub fn read_result_to_domain(
    value: w::ReadResult,
    context: &super::Context,
) -> Result<Result<skein_forge::api::Answer, skein_forge::api::Error>, w::Problem> {
    Ok(match value {
        w::ReadResult::Answer(value) => Ok(super::answers::decode(value.answer, context.bounds())
            .or(Err(w::paths::READ_RESULT_TAG.problem(skein_codec::Reason::Malformed)))?),
        w::ReadResult::Error(value) => Err(api::decode_error(value.error)),
    })
}

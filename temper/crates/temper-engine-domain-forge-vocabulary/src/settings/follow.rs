//! Structure and codec of kind 12 (temper/docs/design/domain/forge/follows.md, 2).
use super::{Bounds, Follow, Refusal, within};
use crate::generated::client_wire::{follow_to_domain, follow_to_record};
use temper_client_protocol_wire::v1 as w;
use w::Record;

pub(super) fn judge(value: &Follow, bounds: &Bounds) -> Result<(), Refusal> {
    if !within(value.source.paths.len(), bounds.follow_paths) {
        return Err(Refusal::FollowPaths);
    }
    for path in &value.source.paths {
        if path.is_empty() || !within(path.len(), 256) || path.first() == Some(&b'/') || path.last() == Some(&b'/') {
            return Err(Refusal::FollowPath);
        }
        for pair in path.windows(2) {
            if pair == b"//" {
                return Err(Refusal::FollowPath);
            }
        }
    }
    if !within(value.source.branch.len(), bounds.name_bytes) || !within(value.target.branch.len(), bounds.name_bytes) {
        return Err(Refusal::FollowBranch);
    }
    if let Some(producer) = &value.producer
        && (!within(producer.len(), 8192) || producer_charter(producer).is_none())
    {
        return Err(Refusal::FollowProducer);
    }
    for member in [&value.check.workflow, &value.check.artifact] {
        if member.is_empty() || !within(member.len(), 256) {
            return Err(Refusal::FollowCheck);
        }
    }
    Ok(())
}

/// The charter in change parameter 11; `None` outside its grammar. An empty
/// value uses the defaults (temper/docs/design/domain/forge/follows.md, 2).
#[must_use]
pub fn producer_charter(text: &[u8]) -> Option<Option<u32>> {
    if text.is_empty() {
        return Some(None);
    }
    let (charter, producing, review) = crate::change_llm_text::parts(text)?;
    for text in [producing, review] {
        if !text.is_empty() {
            crate::change_llm_text::choice(text, None)?;
        }
    }
    Some(charter)
}

/// The kind's wire record, shared by its entrances and store's matching record.
#[must_use]
pub fn follow_record(value: &Follow) -> Option<w::Follow> {
    let record = follow_to_record(value).ok()?;
    record.measure(&w::CEILINGS).ok()?;
    Some(record)
}

/// Move a decoded kind 12 into the domain, leaving judgement to its owner.
#[must_use]
pub fn follow_from(value: w::Follow) -> Follow {
    follow_to_domain(value)
}

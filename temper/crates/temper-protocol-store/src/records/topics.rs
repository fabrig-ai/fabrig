use super::names::{decode_name, encode_name};
use super::{Bounds, Undecodable, w};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::List;
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_names(task: u64, resources: &[d::Name]) -> w::Names {
    let mut names = List::with_capacity(u32::try_from(resources.len()).expect("admitted count"));
    for name in resources {
        names.push(encode_name(name)).expect("same count");
    }
    w::Names { task, resources: names }
}
pub(super) fn decode_names(resources: List<w::Name>, bounds: &Bounds) -> Result<Box<[d::Name]>, Undecodable> {
    let mut names = List::with_capacity(resources.len());
    for name in resources.into_boxed() {
        names.push(decode_name(name, bounds)?).expect("same count");
    }
    Ok(names.into_boxed())
}

pub(super) fn encode_ci(value: api::Ci) -> w::Ci {
    crate::generated::store::ci_to_record(&value)
}
pub(super) fn decode_ci(value: w::Ci) -> api::Ci {
    crate::generated::store::ci_to_domain(value)
}

pub(super) fn encode_state(value: api::State) -> w::ItemState {
    crate::generated::store::item_state_to_record(&value)
}
pub(super) fn decode_state(value: w::ItemState) -> api::State {
    crate::generated::store::item_state_to_domain(value)
}

pub(super) fn encode_branch_head(value: &d::BranchHead) -> w::BranchHead {
    crate::generated::store::branch_head_to_record(value)
}
pub(super) fn decode_branch_head(value: w::BranchHead, bounds: &Bounds) -> Result<d::BranchHead, Undecodable> {
    crate::generated::store::branch_head_to_domain(value, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))
}

pub(super) fn encode_pull_state(value: &d::PullState) -> w::PullState {
    crate::generated::store::pull_state_to_record(value)
}
pub(super) fn decode_pull_state(value: w::PullState, bounds: &Bounds) -> Result<d::PullState, Undecodable> {
    crate::generated::store::pull_state_to_domain(value, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))
}

pub(super) fn encode_ci_state(value: &d::CiState) -> w::CiState {
    crate::generated::store::ci_state_to_record(value)
}
pub(super) fn decode_ci_state(value: w::CiState) -> d::CiState {
    crate::generated::store::ci_state_to_domain(value)
}

pub(super) fn encode_subscriber(value: &d::Subscriber) -> w::Subscriber {
    crate::generated::store::subscriber_to_record(value).expect(super::FITS)
}
pub(super) fn decode_subscriber(value: w::Subscriber) -> d::Subscriber {
    crate::generated::store::subscriber_to_domain(value)
}

pub(super) fn encode_landed(commit: api::Commit, task: u64) -> w::Landed {
    w::Landed { commit, task }
}
pub(super) fn encode_release(value: &d::ReleaseRow) -> w::ReleaseRow {
    crate::generated::store::release_row_to_record(value)
}
pub(super) fn decode_release(value: w::ReleaseRow, bounds: &Bounds) -> Result<d::ReleaseRow, Undecodable> {
    crate::generated::store::release_row_to_domain(value, &super::Context::bounded(bounds))
        .or(Err(Undecodable::Malformed))
}

/// Owner-shaped `PendingRelease` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn pending_release_to_record(value: &(u64, Option<d::Name>)) -> w::PendingRelease {
    w::PendingRelease {
        entry: value.0,
        name: match &value.1 {
            Some(name) => Some(encode_name(name)),
            None => None,
        },
    }
}
pub fn pending_release_to_domain(
    value: w::PendingRelease,
    context: &super::Context,
) -> Result<(u64, Option<d::Name>), w::Problem> {
    Ok((
        value.entry,
        match value.name {
            Some(name) => Some(crate::generated::store::name_to_domain(name, context)?),
            None => None,
        },
    ))
}

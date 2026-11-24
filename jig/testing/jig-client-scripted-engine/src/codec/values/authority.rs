//! Wire projections of owner authority facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
pub(in crate::codec) fn wire_authority(value: &c::Authority, limits: &w::Limits) -> Result<w::Authority, w::Problem> {
    Ok(w::Authority {
        models: value.models.as_ref().map(wire_model_set),
        tools: value.tools,
        grants: pieces(value.grants.iter().map(wire_grant), limits)?,
        delegation: wire_delegation(&value.delegation),
        spend: value.spend,
        deadline: value.deadline.as_ref().map(|item| (item).as_nanos()),
        notes: value.notes,
        note_resources: pieces(value.note_resources.iter().map(wire_resource_scope), limits)?,
    })
}

pub(in crate::codec) fn wire_model_set(value: &c::AuthorityModels) -> w::ModelSet {
    match value {
        c::AuthorityModels::All => w::ModelSet::All,
        c::AuthorityModels::Named(value) => {
            w::ModelSet::Names(w::ModelSetNames { names: list((value).iter().cloned()) })
        }
    }
}

pub(in crate::codec) fn wire_grant(value: &c::Grant) -> w::Grant {
    w::Grant { connector: value.connector, kind: value.kind, pattern: wire_pattern(&value.pattern) }
}

pub(in crate::codec) fn wire_pattern(value: &c::Pattern) -> w::Pattern {
    w::Pattern { segments: list(value.segments.iter().cloned()), last: wire_pattern_end(&value.last) }
}

pub(in crate::codec) fn wire_pattern_end(value: &c::PatternEnd) -> w::PatternEnd {
    match value {
        c::PatternEnd::Exact(value) => w::PatternEnd::Exact(w::PatternEndExact { value: bytes::copy_of(value) }),
        c::PatternEnd::Open(value) => w::PatternEnd::Open(w::PatternEndOpen { value: bytes::copy_of(value) }),
    }
}

pub(in crate::codec) fn wire_delegation(value: &c::Delegation) -> w::Delegation {
    w::Delegation {
        kinds: list(value.kinds.iter().map(wire_authority_executor)),
        tasks: value.tasks,
        depth: value.depth,
    }
}

pub(in crate::codec) fn wire_authority_executor(value: &c::AuthorityExecutor) -> w::AuthorityExecutor {
    match value {
        c::AuthorityExecutor::Job(value) => w::AuthorityExecutor::Job(w::AuthorityExecutorJob { value: *value }),
        c::AuthorityExecutor::Charter(value) => {
            w::AuthorityExecutor::Charter(w::AuthorityExecutorCharter { value: *(value) })
        }
        c::AuthorityExecutor::Procedure(value) => {
            w::AuthorityExecutor::Procedure(w::AuthorityExecutorProcedure { value: *(value) })
        }
        c::AuthorityExecutor::Role(value) => w::AuthorityExecutor::Role(w::AuthorityExecutorRole { value: *(value) }),
    }
}

pub(in crate::codec) fn wire_resource_scope(value: &c::ResourceScope) -> w::ResourceScope {
    w::ResourceScope { connector: value.connector, pattern: wire_pattern(&value.pattern) }
}

pub(in crate::codec) fn wire_authority_document(
    value: &c::AuthorityDocument,
    limits: &w::Limits,
) -> Result<w::AuthorityDocument, w::Problem> {
    Ok(w::AuthorityDocument { authority: wire_authority(&value.authority, limits)?, cut: value.cut })
}

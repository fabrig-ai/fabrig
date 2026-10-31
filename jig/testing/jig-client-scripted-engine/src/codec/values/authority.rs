//! Wire projections of owner authority facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
pub(in crate::codec) fn wire_authority(value: &c::Authority, limits: &w::Limits) -> Result<w::Authority, w::Problem> {
    w::Authority::new(
        limits,
        w::AuthorityParts {
            models: value.models.as_ref().map(|item| wire_model_set(item, limits)).transpose()?,
            tools: value.tools,
            grants: pieces(
                value.grants.iter().map(|item| wire_grant(item, limits)),
                w::Grant::measure,
                w::Grant::encode,
            )?,
            delegation: wire_delegation(&value.delegation, limits)?,
            spend: value.spend,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            notes: value.notes,
            note_resources: pieces(
                value.note_resources.iter().map(|item| wire_resource_scope(item, limits)),
                w::ResourceScope::measure,
                w::ResourceScope::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_model_set(
    value: &c::AuthorityModels,
    limits: &w::Limits,
) -> Result<w::ModelSet, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AuthorityModels::All => w::ModelSet::All,
        c::AuthorityModels::Named(value) => w::ModelSet::Names(w::ModelSetNames::new(
            limits,
            w::ModelSetNamesParts {
                names: list((value).iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))))?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_grant(value: &c::Grant, limits: &w::Limits) -> Result<w::Grant, w::Problem> {
    w::Grant::new(
        limits,
        w::GrantParts { connector: value.connector, kind: value.kind, pattern: wire_pattern(&value.pattern, limits)? },
    )
}

pub(in crate::codec) fn wire_pattern(value: &c::Pattern, limits: &w::Limits) -> Result<w::Pattern, w::Problem> {
    w::Pattern::new(
        limits,
        w::PatternParts {
            segments: list(value.segments.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))))?,
            last: wire_pattern_end(&value.last, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_pattern_end(
    value: &c::PatternEnd,
    limits: &w::Limits,
) -> Result<w::PatternEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PatternEnd::Exact(value) => w::PatternEnd::Exact(w::PatternEndExact::new(
            limits,
            w::PatternEndExactParts { value: bytes::copy_of(value) },
        )?),
        c::PatternEnd::Open(value) => w::PatternEnd::Open(w::PatternEndOpen::new(
            limits,
            w::PatternEndOpenParts { value: bytes::copy_of(value) },
        )?),
    })
}

pub(in crate::codec) fn wire_delegation(
    value: &c::Delegation,
    limits: &w::Limits,
) -> Result<w::Delegation, w::Problem> {
    w::Delegation::new(
        limits,
        w::DelegationParts {
            kinds: list(value.kinds.iter().map(|item| wire_authority_executor(item, limits)))?,
            tasks: value.tasks,
            depth: value.depth,
        },
    )
}

pub(in crate::codec) fn wire_authority_executor(
    value: &c::AuthorityExecutor,
    limits: &w::Limits,
) -> Result<w::AuthorityExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::AuthorityExecutor::Charter(value) => w::AuthorityExecutor::Charter(w::AuthorityExecutorCharter::new(
            limits,
            w::AuthorityExecutorCharterParts { value: *(value) },
        )?),
        c::AuthorityExecutor::Procedure(value) => w::AuthorityExecutor::Procedure(w::AuthorityExecutorProcedure::new(
            limits,
            w::AuthorityExecutorProcedureParts { value: *(value) },
        )?),
        c::AuthorityExecutor::Role(value) => w::AuthorityExecutor::Role(w::AuthorityExecutorRole::new(
            limits,
            w::AuthorityExecutorRoleParts { value: *(value) },
        )?),
    })
}

pub(in crate::codec) fn wire_resource_scope(
    value: &c::ResourceScope,
    limits: &w::Limits,
) -> Result<w::ResourceScope, w::Problem> {
    w::ResourceScope::new(
        limits,
        w::ResourceScopeParts { connector: value.connector, pattern: wire_pattern(&value.pattern, limits)? },
    )
}

pub(in crate::codec) fn wire_authority_document(
    value: &c::AuthorityDocument,
    limits: &w::Limits,
) -> Result<w::AuthorityDocument, w::Problem> {
    w::AuthorityDocument::new(
        limits,
        w::AuthorityDocumentParts { authority: wire_authority(&value.authority, limits)?, cut: value.cut },
    )
}

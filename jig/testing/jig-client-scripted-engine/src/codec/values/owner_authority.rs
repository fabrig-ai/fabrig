//! Wire projections of owner authority facts (client/domain/testing.md, 4).
use super::super::{bytes, c, list, pieces, w};
pub(in crate::codec) fn wire_owner_authority(
    value: &c::OwnerAuthority,
    limits: &w::Limits,
) -> Result<w::OwnerAuthority, w::Problem> {
    w::OwnerAuthority::new(
        limits,
        w::OwnerAuthorityParts {
            models: value.models.as_ref().map(|item| wire_model_set(item, limits)).transpose()?,
            tools: value.tools,
            grants: pieces(
                value.grants.iter().map(|item| wire_owner_grant(item, limits)),
                w::OwnerGrant::measure,
                w::OwnerGrant::encode,
            )?,
            delegation: wire_owner_delegation(&value.delegation, limits)?,
            spend: value.spend,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            notes: value.notes,
            note_resources: pieces(
                value.note_resources.iter().map(|item| wire_owner_resource_scope(item, limits)),
                w::OwnerResourceScope::measure,
                w::OwnerResourceScope::encode,
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

pub(in crate::codec) fn wire_owner_grant(
    value: &c::OwnerGrant,
    limits: &w::Limits,
) -> Result<w::OwnerGrant, w::Problem> {
    w::OwnerGrant::new(
        limits,
        w::OwnerGrantParts {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_pattern(
    value: &c::OwnerPattern,
    limits: &w::Limits,
) -> Result<w::OwnerPattern, w::Problem> {
    w::OwnerPattern::new(
        limits,
        w::OwnerPatternParts {
            segments: list(value.segments.iter().map(|item| Ok::<_, w::Problem>(bytes::copy_of(item))))?,
            last: wire_owner_pattern_end(&value.last, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_pattern_end(
    value: &c::OwnerPatternEnd,
    limits: &w::Limits,
) -> Result<w::OwnerPatternEnd, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPatternEnd::Exact(value) => w::OwnerPatternEnd::Exact(w::OwnerPatternEndExact::new(
            limits,
            w::OwnerPatternEndExactParts { value: bytes::copy_of(value) },
        )?),
        c::OwnerPatternEnd::Open(value) => w::OwnerPatternEnd::Open(w::OwnerPatternEndOpen::new(
            limits,
            w::OwnerPatternEndOpenParts { value: bytes::copy_of(value) },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_delegation(
    value: &c::OwnerDelegation,
    limits: &w::Limits,
) -> Result<w::OwnerDelegation, w::Problem> {
    w::OwnerDelegation::new(
        limits,
        w::OwnerDelegationParts {
            kinds: list(value.kinds.iter().map(|item| wire_owner_authority_executor(item, limits)))?,
            tasks: value.tasks,
            depth: value.depth,
        },
    )
}

pub(in crate::codec) fn wire_owner_authority_executor(
    value: &c::OwnerAuthorityExecutor,
    limits: &w::Limits,
) -> Result<w::OwnerAuthorityExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerAuthorityExecutor::Charter(value) => w::OwnerAuthorityExecutor::Charter(
            w::OwnerAuthorityExecutorCharter::new(limits, w::OwnerAuthorityExecutorCharterParts { value: *(value) })?,
        ),
        c::OwnerAuthorityExecutor::Procedure(value) => {
            w::OwnerAuthorityExecutor::Procedure(w::OwnerAuthorityExecutorProcedure::new(
                limits,
                w::OwnerAuthorityExecutorProcedureParts { value: *(value) },
            )?)
        }
        c::OwnerAuthorityExecutor::Role(value) => w::OwnerAuthorityExecutor::Role(w::OwnerAuthorityExecutorRole::new(
            limits,
            w::OwnerAuthorityExecutorRoleParts { value: *(value) },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_resource_scope(
    value: &c::OwnerResourceScope,
    limits: &w::Limits,
) -> Result<w::OwnerResourceScope, w::Problem> {
    w::OwnerResourceScope::new(
        limits,
        w::OwnerResourceScopeParts { connector: value.connector, pattern: wire_owner_pattern(&value.pattern, limits)? },
    )
}

pub(in crate::codec) fn wire_owner_authority_document(
    value: &c::OwnerAuthorityDocument,
    limits: &w::Limits,
) -> Result<w::OwnerAuthorityDocument, w::Problem> {
    w::OwnerAuthorityDocument::new(
        limits,
        w::OwnerAuthorityDocumentParts { authority: wire_owner_authority(&value.authority, limits)?, cut: value.cut },
    )
}

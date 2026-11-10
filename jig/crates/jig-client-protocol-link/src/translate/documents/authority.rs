//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{Limits, List, Reader, Wall, c, too_large, w};

pub(crate) fn grant_fields(value: &w::Grant) -> c::Grant {
    c::Grant {
        connector: value.connector(),
        kind: value.kind(),
        pattern: crate::translate::answers::pattern(value.pattern()),
    }
}

pub(crate) fn grant_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Grant>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::Grant::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = grant_fields(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn authority_executor_fields(value: &w::AuthorityExecutor) -> c::AuthorityExecutor {
    match value {
        w::AuthorityExecutor::Job(row) => c::AuthorityExecutor::Job(row.value()),
        w::AuthorityExecutor::Charter(row) => c::AuthorityExecutor::Charter(row.value()),
        w::AuthorityExecutor::Procedure(row) => c::AuthorityExecutor::Procedure(row.value()),
        w::AuthorityExecutor::Role(row) => c::AuthorityExecutor::Role(row.value()),
    }
}

pub(crate) fn delegation_fields(value: &w::Delegation, limits: &Limits) -> Result<c::Delegation, w::Problem> {
    if value.kinds().len() > limits.client().authority {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.kinds().len());
    for item in value.kinds() {
        list0.push(authority_executor_fields(item)).expect("reserved supplied list");
    }
    Ok(c::Delegation { kinds: list0, tasks: value.tasks(), depth: value.depth() })
}

pub(crate) fn resource_scope_fields(value: &w::ResourceScope) -> c::ResourceScope {
    c::ResourceScope { connector: value.connector(), pattern: crate::translate::answers::pattern(value.pattern()) }
}

pub(crate) fn resource_scope_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ResourceScope>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::ResourceScope::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = resource_scope_fields(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn authority_fields(value: &w::Authority, limits: &Limits) -> Result<c::Authority, w::Problem> {
    let opt0 = match value.deadline() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::Authority {
        models: crate::translate::model_sets::decode(value.models().as_ref())?,
        tools: value.tools(),
        grants: grant_piece(value.grants(), limits, limits.client().authority)?,
        delegation: delegation_fields(value.delegation(), limits)?,
        spend: value.spend(),
        deadline: opt0,
        notes: value.notes(),
        note_resources: resource_scope_piece(value.note_resources(), limits, limits.client().authority)?,
    })
}

pub(crate) fn authority_document_fields(
    value: &w::AuthorityDocument,
    limits: &Limits,
) -> Result<c::AuthorityDocument, w::Problem> {
    Ok(c::AuthorityDocument { authority: authority_fields(value.authority(), limits)?, cut: value.cut() })
}

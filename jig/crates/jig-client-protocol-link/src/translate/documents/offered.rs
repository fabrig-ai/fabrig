//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1).
use super::{Box, Limits, List, Reader, Wall, authority_document_fields, c, grant_piece, too_large, w};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn goal_procedure_fields(value: &w::CreateProcedure) -> c::CreateProcedure {
    client::create_procedure_to_domain(value.clone())
}

pub(crate) fn kind_implication_fields(value: &w::KindImplication) -> c::KindImplication {
    client::kind_implication_to_domain(value.clone())
}

pub(crate) fn bounds_fields(value: &w::Bounds, limits: &Limits) -> Result<c::Bounds, w::Problem> {
    Ok(match value {
        w::Bounds::Create(row) => c::Bounds::Create(bounds_create_to_domain(row.clone(), limits)?),
        w::Bounds::Amend(row) => {
            if row.implications.len() > limits.client().authority {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.implications.len());
            for item in &row.implications {
                list0.push(kind_implication_fields(item)).expect("reserved supplied list");
            }
            c::Bounds::Amend {
                budget_left: row.budget_left,
                grants: grant_piece(&row.grants, limits, limits.client().authority)?,
                implications: list0,
                authority: Box::new(authority_document_fields(&row.authority, limits)?),
                ceiling: Box::new(authority_document_fields(&row.ceiling, limits)?),
            }
        }
    })
}

pub(crate) fn bounds_create_to_domain(row: w::BoundsCreate, limits: &Limits) -> Result<c::CreateBounds, w::Problem> {
    if row.charters.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(row.charters.len());
    for item in &row.charters {
        list0.push(*item).expect("reserved supplied list");
    }
    if row.procedures.len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(row.procedures.len());
    for item in &row.procedures {
        list1.push(goal_procedure_fields(item)).expect("reserved supplied list");
    }
    let opt2 = match &row.latest {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt3 = match &row.maximum_deadline {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::CreateBounds {
        authority: Box::new(authority_document_fields(&row.authority, limits)?),
        pool_left: row.pool_left,
        charters: list0,
        procedures: list1,
        latest: opt2,
        budget: row.budget,
        maximum_budget: row.maximum_budget,
        maximum_deadline: opt3,
    })
}

pub(crate) fn offered_fields(value: &w::Offered, limits: &Limits) -> Result<c::Offered, w::Problem> {
    client::offered_to_domain(value.clone(), limits)
}

pub(crate) fn offered_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::Offered>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::Offered::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = offered_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[cfg(test)]
pub(crate) fn act_kind_fields(value: &w::ActKind) -> c::ActKind {
    client::act_kind_to_domain(value.clone())
}

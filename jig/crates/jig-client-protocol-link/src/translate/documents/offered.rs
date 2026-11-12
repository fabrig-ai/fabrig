//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{Box, Limits, List, Reader, Wall, authority_document_fields, c, effect_fields, grant_piece, too_large, w};

pub(crate) fn act_kind_fields(value: &w::ActKind) -> c::ActKind {
    match value {
        w::ActKind::Reorder => c::ActKind::Reorder,
        w::ActKind::Talk => c::ActKind::Talk,
        w::ActKind::Leave => c::ActKind::Leave,
        w::ActKind::Message => c::ActKind::Message,
        w::ActKind::Answer => c::ActKind::Answer,
        w::ActKind::Take => c::ActKind::Take,
        w::ActKind::HandBack => c::ActKind::HandBack,
        w::ActKind::Accept => c::ActKind::Accept,
        w::ActKind::Withdraw => c::ActKind::Withdraw,
        w::ActKind::Reject => c::ActKind::Reject,
        w::ActKind::PassUp => c::ActKind::PassUp,
        w::ActKind::Release => c::ActKind::Release,
        w::ActKind::LeaveHeld => c::ActKind::LeaveHeld,
        w::ActKind::Amend => c::ActKind::Amend,
        w::ActKind::Stop => c::ActKind::Stop,
        w::ActKind::Cancel => c::ActKind::Cancel,
        w::ActKind::MoveTask => c::ActKind::Move,
        w::ActKind::Seen => c::ActKind::Seen,
        w::ActKind::EditNote => c::ActKind::EditNote,
        w::ActKind::DeleteNote => c::ActKind::DeleteNote,
        w::ActKind::Adopt => c::ActKind::Adopt,
        w::ActKind::RemoveProject => c::ActKind::RemoveProject,
        w::ActKind::Rename => c::ActKind::Rename,
        w::ActKind::ChangePolicy => c::ActKind::ChangePolicy,
        w::ActKind::SetRoles => c::ActKind::SetRoles,
        w::ActKind::MakeService => c::ActKind::MakeService,
        w::ActKind::SetPool => c::ActKind::SetPool,
    }
}

pub(crate) fn goal_procedure_fields(value: &w::CreateProcedure) -> c::CreateProcedure {
    c::CreateProcedure { connector: value.connector(), code: value.code() }
}

pub(crate) fn kind_implication_fields(value: &w::KindImplication) -> c::KindImplication {
    c::KindImplication { connector: value.connector(), kind: value.kind(), implies: value.implies() }
}

pub(crate) fn bounds_fields(value: &w::Bounds, limits: &Limits) -> Result<c::Bounds, w::Problem> {
    Ok(match value {
        w::Bounds::Create(row) => {
            if row.charters().len() > limits.client().rows {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.charters().len());
            for item in row.charters() {
                list0.push(*item).expect("reserved supplied list");
            }
            if row.procedures().len() > limits.client().rows {
                return Err(too_large());
            }
            let mut list1 = List::with_capacity(row.procedures().len());
            for item in row.procedures() {
                list1.push(goal_procedure_fields(item)).expect("reserved supplied list");
            }
            let opt2 = match row.latest() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            let opt3 = match row.maximum_deadline() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::Bounds::Create(c::CreateBounds {
                authority: Box::new(authority_document_fields(row.authority(), limits)?),
                pool_left: row.pool_left(),
                charters: list0,
                procedures: list1,
                latest: opt2,
                budget: row.budget(),
                maximum_budget: row.maximum_budget(),
                maximum_deadline: opt3,
            })
        }
        w::Bounds::Amend(row) => {
            if row.implications().len() > limits.client().authority {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.implications().len());
            for item in row.implications() {
                list0.push(kind_implication_fields(item)).expect("reserved supplied list");
            }
            c::Bounds::Amend {
                budget_left: row.budget_left(),
                grants: grant_piece(row.grants(), limits, limits.client().authority)?,
                implications: list0,
                authority: Box::new(authority_document_fields(row.authority(), limits)?),
                ceiling: Box::new(authority_document_fields(row.ceiling(), limits)?),
            }
        }
    })
}

pub(crate) fn beyond_fields(value: &w::Beyond, limits: &Limits) -> Result<c::Beyond, w::Problem> {
    Ok(match value {
        w::Beyond::Proposed(row) => c::Beyond::Proposed { to: crate::translate::facts::recipient(row.to()) },
        w::Beyond::Refused(row) => c::Beyond::Refused(crate::translate::answers::refusal(row.value(), limits)?),
    })
}

pub(crate) fn admission_fields(value: &w::Admission, limits: &Limits) -> Result<c::Admission, w::Problem> {
    Ok(match value {
        w::Admission::Allowed => c::Admission::Allowed,
        w::Admission::Proposed(row) => c::Admission::Proposed { to: crate::translate::facts::recipient(row.to()) },
        w::Admission::Bounded(row) => c::Admission::Bounded {
            within: bounds_fields(row.within(), limits)?,
            beyond: beyond_fields(row.beyond(), limits)?,
        },
        w::Admission::Refused(row) => c::Admission::Refused(crate::translate::answers::refusal(row.value(), limits)?),
    })
}

pub(crate) fn offered_fields(value: &w::Offered, limits: &Limits) -> Result<c::Offered, w::Problem> {
    Ok(c::Offered {
        effect: match value.effect() {
            Some(value) => Some(effect_fields(value)),
            None => None,
        },
        act: act_kind_fields(value.act()),
        admission: admission_fields(value.admission(), limits)?,
    })
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

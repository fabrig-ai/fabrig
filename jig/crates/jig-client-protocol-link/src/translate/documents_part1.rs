//! Field-for-field document values (client/domain/contract.md, 5–7).
use super::{Limits, List, Reader, Wall, c, piece_problem, raw, subscription_kind_fields, too_large, w};
pub(crate) fn subscription_fields(value: &w::Subscription) -> c::Subscription {
    c::Subscription { number: value.number(), kind: subscription_kind_fields(value.kind()) }
}

pub(crate) fn subscriptions_fields(value: &w::Subscriptions, limits: &Limits) -> Result<c::Subscriptions, w::Problem> {
    if value.entries().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.entries().len());
    for item in value.entries() {
        list0.push(subscription_fields(item)).expect("reserved supplied list");
    }
    Ok(c::Subscriptions { entries: list0 })
}

pub(crate) fn field_rule_fields(value: &w::FieldRule, limits: &Limits) -> Result<c::FieldRule, w::Problem> {
    Ok(c::FieldRule { name: raw(value.name(), limits)?, max: value.max() })
}

pub(crate) fn verdict_choice_fields(value: &w::VerdictChoice, limits: &Limits) -> Result<c::VerdictChoice, w::Problem> {
    if value.fields().len() > limits.client().fields {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.fields().len());
    for item in value.fields() {
        list0.push(field_rule_fields(item, limits)?).expect("reserved supplied list");
    }
    Ok(c::VerdictChoice { code: value.code(), words: value.words(), followups: value.followups(), fields: list0 })
}

pub(crate) fn verdict_choice_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::VerdictChoice>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::VerdictChoice::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = verdict_choice_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn task_contract_document_fields(
    value: &w::TaskContractDocument,
    limits: &Limits,
) -> Result<c::TaskContractDocument, w::Problem> {
    Ok(match value {
        w::TaskContractDocument::Approval(row) => c::TaskContractDocument::Approval {
            state: raw(row.state(), limits)?,
            part: crate::translate::facts::part(row.part()),
        },
        w::TaskContractDocument::Report(row) => c::TaskContractDocument::Report { words: row.words() },
        w::TaskContractDocument::Verdict(row) => c::TaskContractDocument::Verdict {
            choices: verdict_choice_piece(row.choices(), limits, limits.client().rows)?,
            cut: row.cut(),
        },
        w::TaskContractDocument::Change(row) => {
            c::TaskContractDocument::Change { connector: row.connector(), kind: row.kind(), words: row.words() }
        }
    })
}

pub(crate) fn task_resource_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::ResourceLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::ResourceLine::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = crate::translate::facts::resource_line(&row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

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
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
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
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
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

pub(crate) fn result_value_fields(value: &w::ResultValue, limits: &Limits) -> Result<c::ResultValue, w::Problem> {
    match crate::translate::facts::decode(value.rows(), limits)? {
        crate::translate::facts::Node::ResultValue(value) => Ok(*value),
        crate::translate::facts::Node::Chip(_)
        | crate::translate::facts::Node::ChipPhaseFacts(_)
        | crate::translate::facts::Node::Closing(_)
        | crate::translate::facts::Node::Ending(_)
        | crate::translate::facts::Node::HeldRuntime(_)
        | crate::translate::facts::Node::HoldReason(_)
        | crate::translate::facts::Node::ChipDetails(_)
        | crate::translate::facts::Node::ChipEnding(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

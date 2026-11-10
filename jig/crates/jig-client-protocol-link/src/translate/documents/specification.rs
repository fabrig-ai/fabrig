//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{Limits, List, Reader, c, raw, too_large, w};

pub(crate) fn parameter_document_fields(
    value: &w::ParameterDocument,
    limits: &Limits,
) -> Result<c::ParameterDocument, w::Problem> {
    Ok(match value {
        w::ParameterDocument::Number(row) => c::ParameterDocument::Number { name: row.name(), value: row.value() },
        w::ParameterDocument::Bytes(row) => c::ParameterDocument::Bytes {
            name: row.name(),
            value: crate::translate::facts::document_text(row.value(), limits)?,
        },
        w::ParameterDocument::Resource(row) => {
            c::ParameterDocument::Resource { name: row.name(), connector: row.connector(), resource: row.resource() }
        }
    })
}

pub(crate) fn document_specification_fields(
    value: &w::DocumentSpecification,
    limits: &Limits,
) -> Result<c::DocumentSpecification, w::Problem> {
    if value.parameters().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.parameters().len());
    for item in value.parameters() {
        list0.push(parameter_document_fields(item, limits)?).expect("reserved supplied list");
    }
    if value.inputs().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list1 = List::with_capacity(value.inputs().len());
    for item in value.inputs() {
        list1.push(*item).expect("reserved supplied list");
    }
    Ok(c::DocumentSpecification {
        asks: value.asks(),
        words: crate::translate::facts::document_text(value.words(), limits)?,
        parameters: list0,
        inputs: list1,
    })
}

pub(crate) fn specification_document_fields(
    value: &w::SpecificationDocument,
    limits: &Limits,
) -> Result<c::SpecificationDocument, w::Problem> {
    Ok(c::SpecificationDocument {
        specification: document_specification_fields(value.specification(), limits)?,
        cut: value.cut(),
    })
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
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
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
        w::TaskContractDocument::Job(row) => c::TaskContractDocument::Job { bytes: row.bytes() },
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

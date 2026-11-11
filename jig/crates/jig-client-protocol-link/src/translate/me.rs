//! Nested served me records, bounded by the receiving document.
use super::{Limits, List, Reader, c, project_role_fields, task_contract_document_fields, too_large, w};
pub(crate) fn projects(bytes: &[u8], limits: &Limits) -> Result<List<c::MeProject>, w::Problem> {
    let bound = limits.client().projects;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::MeProject::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = project_role_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn templates(bytes: &[u8], limits: &Limits) -> Result<List<c::Template>, w::Problem> {
    let bound = limits.client().rows;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::Template::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = template(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn template(value: &w::Template, limits: &Limits) -> Result<c::Template, w::Problem> {
    Ok(c::Template {
        resources: symbols(value.resources()),
        number: value.number(),
        name: crate::translate::facts::document_text(value.name(), limits)?,
        executor: crate::translate::operations::restore_executor(value.executor()),
        contract: task_contract_document_fields(value.contract(), limits)?,
        shallower: value.shallower(),
        budget: crate::translate::operations::restore_budget(value.budget()),
        deadline: crate::translate::operations::restore_deadline(value.deadline()),
        tracked: *value.tracked(),
        flags: c::TaskFlags {
            ended_by: match value.flags().ended_by() {
                w::EndedBy::Result => c::EndedBy::Result,
                w::EndedBy::Requester => c::EndedBy::Requester,
                w::EndedBy::Standing => c::EndedBy::Standing,
                w::EndedBy::Closer(closer) => {
                    c::EndedBy::Closer(c::Closer { connector: closer.connector(), code: closer.code() })
                }
            },
            conversation: value.flags().conversation(),
            listed: value.flags().listed(),
            retention: match value.flags().retention() {
                w::Retention::Kept => c::Retention::Kept,
                w::Retention::ForgottenOnceRead => c::Retention::ForgottenOnceRead,
            },
        },
        label: value.label(),
        grace: *value.grace(),
        llm: crate::translate::llm::choice(value.llm().as_ref(), limits)?,
    })
}
pub(crate) fn overrides(
    values: &List<w::TemplateOverride>,
    limits: &Limits,
) -> Result<List<c::TemplateOverride>, w::Problem> {
    if values.len() > limits.client().rows {
        return Err(too_large());
    }
    Ok(crate::translate::operations::restore_template_overrides(values))
}

pub(crate) fn symbols(values: &List<w::Symbol>) -> List<c::Symbol> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(c::Symbol { connector: value.connector(), symbol: value.symbol() }).expect("same capacity");
    }
    rows
}

pub(crate) fn wire_symbols(values: &List<c::Symbol>, limits: &w::Limits) -> Result<List<w::Symbol>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(w::Symbol::new(limits, w::SymbolParts { connector: value.connector, symbol: value.symbol })?)
            .expect("same capacity");
    }
    Ok(rows)
}

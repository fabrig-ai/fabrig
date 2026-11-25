//! Nested served me records, bounded by the receiving document.
use super::{Limits, List, Reader, c, project_role_fields, too_large, w};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;
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
        let row = client::template_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[cfg(test)]
pub(crate) fn template(value: &w::Template, limits: &Limits) -> Result<c::Template, w::Problem> {
    client::template_to_domain(value.clone(), limits)
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
        rows.push(client::symbol_to_domain(value.clone())).expect("same capacity");
    }
    rows
}

pub(crate) fn wire_symbols(values: &List<c::Symbol>) -> List<w::Symbol> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(client::symbol_to_record(value)).expect("same capacity");
    }
    rows
}

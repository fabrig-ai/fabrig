//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use crate::Limits;
use crate::generated::client;
use crate::translate::{piece_problem, too_large};
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, Reader};

pub(crate) fn document_text(value: &w::DocumentText, limits: &Limits) -> Result<c::DocumentText, w::Problem> {
    crate::translate::client::document_text_to_domain(value.clone(), limits)
}

pub(crate) fn proposal_key(value: &w::ProposalKey) -> c::ProposalKey {
    client::proposal_key_to_domain(value.clone())
}

pub(crate) fn executor(value: &w::Executor) -> c::Executor {
    client::executor_to_domain(value.clone())
}

pub(crate) fn part(value: &w::Part) -> c::Part {
    client::part_to_domain(value.clone())
}

pub(crate) fn numbers(value: &w::Numbers) -> c::Numbers {
    client::numbers_to_domain(value.clone())
}

pub(crate) fn funder(value: &w::Funder) -> c::Funder {
    client::funder_to_domain(value.clone())
}

pub(crate) fn budget(value: &w::Budget) -> c::Budget {
    client::budget_to_domain(value.clone())
}

pub(crate) fn resource_line_piece(bytes: &[u8], limits: &Limits) -> Result<List<c::ResourceLine>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(limits.client().rows);
    for _ in 0..=limits.client().rows {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(problem) => return Err(piece_problem(problem)),
        };
        let Some(item) = item else { return Ok(rows) };
        let row = w::ResourceLine::decode(limits.receiving(), &mut Reader::new(&item))?;
        let row = client::resource_line_to_domain(row);
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn dependencies(values: &List<w::Dependency>) -> List<c::Dependency> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        rows.push(client::dependency_to_domain(value.clone())).expect("same dependency count");
    }
    rows
}

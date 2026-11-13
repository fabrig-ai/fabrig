//! Effective retained authorities follow configured names; original requests belong to people.
use super::{domain as d, wire as w};
use skein_lib::List;

/// Only names this row references, sorted independently of configuration order.
pub(super) fn encode(
    row: &d::Stored,
    names: &[jig_domain_core_vocabulary::settings::ModelName],
    limits: &w::Limits,
) -> Result<(List<w::ModelName>, [u64; 64]), w::Problem> {
    let bits = referenced(row);
    let mut moved = [0; 64];
    let mut result = List::with_capacity(bits.count_ones());
    // Pick the next referenced name by byte order without a second owned name list.
    for bit in 0..bits.count_ones() {
        let mut next: Option<usize> = None;
        for (at, name) in names.iter().enumerate() {
            let selected = bits & 1_u64.checked_shl(u32::try_from(at).expect("configured index")).unwrap_or(0) != 0;
            if selected && *moved.get(at).expect("selected bit fits") == 0 {
                match next {
                    Some(previous) => {
                        if name.name < names.get(previous).expect("configured name").name {
                            next = Some(at);
                        }
                    }
                    None => next = Some(at),
                }
            }
        }
        let Some(at) = next else {
            return Err(w::Problem { path: w::Path::RowModels, reason: skein_codec::Reason::Bound });
        };
        *moved.get_mut(at).expect("selected bit fits") = 1_u64.checked_shl(bit).expect("selected models fit");
        result
            .push(w::ModelName::new(
                limits,
                w::ModelNameParts { value: names.get(at).expect("selected name").name.clone() },
            )?)
            .expect("same selected count");
    }
    Ok((result, moved))
}
/// Encoders outside a tasks row preserve their enclosing record's vocabulary.
pub(super) fn selector(value: d::Models, moved: Option<&[u64; 64]>) -> d::Models {
    let Some(moved) = moved else { return value };
    value.remap(moved)
}

fn authority_bits(value: &d::Authority) -> u64 {
    match value.models {
        d::Models::All => 0,
        d::Models::Named(bits) => bits,
    }
}
fn member_bits(batch: &[d::New]) -> u64 {
    let mut bits = 0;
    for member in batch {
        bits |= authority_bits(&member.authority);
        if let Some(recurring) = &member.recurring {
            for nested in &recurring.batch {
                bits |= authority_bits(&nested.authority);
            }
        }
    }
    bits
}
fn amended_bits(value: &d::Amendment) -> u64 {
    match &value.authority {
        Some(value) => authority_bits(value),
        None => 0,
    }
}
fn proposal_bits(value: &d::Proposal) -> u64 {
    match &value.action {
        d::ProposalAction::Batch(batch) => member_bits(batch),
        d::ProposalAction::Amend { amendment, .. } => amended_bits(amendment),
        d::ProposalAction::Widen { authority, .. }
        | d::ProposalAction::Effect { authority, .. }
        | d::ProposalAction::ProcedureEffect { authority, .. } => authority_bits(authority),
        d::ProposalAction::Settings { .. }
        | d::ProposalAction::ReleasePlaced { .. }
        | d::ProposalAction::Release { .. } => 0,
    }
}
fn referenced(row: &d::Stored) -> u64 {
    match row {
        d::Stored::Live(row) | d::Stored::Ended(row) => {
            let mut bits = authority_bits(&row.authority);
            if let Some(proposal) = &row.proposal {
                bits |= proposal_bits(proposal);
            }
            if let Some(recurring) = &row.recurring {
                bits |= member_bits(&recurring.template.batch);
            }
            bits
        }
        d::Stored::PersonProposal(row) | d::Stored::DecidedProposal(row) => match &row.action {
            d::PersonAction::Settings { .. } => 0,
            d::PersonAction::Batch(batch) => member_bits(batch),
            d::PersonAction::Amend { amendment, .. } => amended_bits(amendment),
        },
        d::Stored::History(row) => {
            let mut bits = 0;
            if let Some(proposal) = &row.proposal {
                bits |= proposal_bits(proposal);
            }
            if let Some(amendment) = &row.amendment {
                bits |= authority_bits(&amendment.before.authority) | authority_bits(&amendment.after.authority);
            }
            bits
        }
        d::Stored::Update(_)
        | d::Stored::Result(_)
        | d::Stored::EndedByParty(_)
        | d::Stored::EndedByProject(_)
        | d::Stored::EndedDelegate(_)
        | d::Stored::Delegate(_)
        | d::Stored::Activity(_)
        | d::Stored::Milestone(_)
        | d::Stored::ClosedLedger(_)
        | d::Stored::PlacedHold(_)
        | d::Stored::Stub(_)
        | d::Stored::Ledger(_)
        | d::Stored::Writer(_)
        | d::Stored::Pool(_) => 0,
    }
}
pub(super) fn moves(kept: &List<w::ModelName>, names: &[jig_domain_core_vocabulary::settings::ModelName]) -> [u64; 64] {
    let mut moved = [0; 64];
    for (old, name) in kept.iter().enumerate() {
        *moved.get_mut(old).expect("bounded names") =
            jig_domain_core_vocabulary::settings::model_bit(name.value(), names);
    }
    moved
}

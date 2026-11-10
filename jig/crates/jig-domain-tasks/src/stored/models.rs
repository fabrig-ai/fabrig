//! Effective authorities follow configured names (jig/docs/design/protocol/store/records.md, 1).

fn authority(value: &mut crate::Authority, moved: &[u64; 64]) {
    value.models = value.models.remap(moved);
}
fn members(batch: &mut [crate::New], moved: &[u64; 64]) {
    for member in batch {
        authority(&mut member.authority, moved);
        if let Some(recurring) = &mut member.recurring {
            for nested in &mut recurring.batch {
                authority(&mut nested.authority, moved);
            }
        }
    }
}
fn amendment(value: &mut crate::Amendment, moved: &[u64; 64]) {
    if let Some(value) = &mut value.authority {
        authority(value, moved);
    }
}
fn proposal(value: &mut crate::Proposal, moved: &[u64; 64]) {
    match &mut value.action {
        crate::ProposalAction::Batch(batch) => members(batch, moved),
        crate::ProposalAction::Amend { amendment: value, .. } => amendment(value, moved),
        crate::ProposalAction::Widen { authority: value, .. }
        | crate::ProposalAction::Effect { authority: value, .. }
        | crate::ProposalAction::ProcedureEffect { authority: value, .. } => authority(value, moved),
        crate::ProposalAction::Settings { .. }
        | crate::ProposalAction::ReleasePlaced { .. }
        | crate::ProposalAction::Release { .. } => {}
    }
}
impl crate::Stored {
    /// Remap every retained effective authority (jig/docs/design/protocol/store/records.md, 1).
    pub fn remap_models(&mut self, moved: &[u64; 64]) {
        match self {
            crate::Stored::Live(row) | crate::Stored::Ended(row) => {
                authority(&mut row.authority, moved);
                if let Some(value) = &mut row.proposal {
                    proposal(value, moved);
                }
                if let Some(value) = &mut row.recurring {
                    members(&mut value.template.batch, moved);
                }
            }
            crate::Stored::PersonProposal(row) | crate::Stored::DecidedProposal(row) => match &mut row.action {
                crate::PersonAction::Settings { .. } => {}
                crate::PersonAction::Batch(batch) => members(batch, moved),
                crate::PersonAction::Amend { amendment: value, .. } => amendment(value, moved),
            },
            crate::Stored::History(row) => {
                if let Some(value) = &mut row.proposal {
                    proposal(value, moved);
                }
                if let Some(value) = &mut row.amendment {
                    authority(&mut value.before.authority, moved);
                    authority(&mut value.after.authority, moved);
                }
            }
            crate::Stored::Update(_)
            | crate::Stored::Result(_)
            | crate::Stored::EndedByParty(_)
            | crate::Stored::EndedByProject(_)
            | crate::Stored::EndedDelegate(_)
            | crate::Stored::Delegate(_)
            | crate::Stored::Activity(_)
            | crate::Stored::Milestone(_)
            | crate::Stored::ClosedLedger(_)
            | crate::Stored::PlacedHold(_)
            | crate::Stored::Stub(_)
            | crate::Stored::Ledger(_)
            | crate::Stored::Writer(_)
            | crate::Stored::Pool(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Models;

    #[test]
    fn remap_preserves_all_and_translates_named_bits() {
        let mut moved = [0; 64];
        moved[0] = 2;
        moved[1] = 2;
        moved[63] = 4;
        for (before, after) in [
            (Models::All, Models::All),
            (Models::Named(3), Models::Named(2)),
            (Models::Named(4), Models::Named(0)),
            (Models::Named(1 << 63), Models::Named(4)),
            (Models::Named(0), Models::Named(0)),
        ] {
            assert_eq!(before.remap(&moved), after);
        }
    }
}

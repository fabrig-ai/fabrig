//! Effective authorities follow configured names (jig/docs/design/protocol/store.md, 5.1).

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
    /// Remap every retained effective authority (jig/docs/design/protocol/store.md, 5.1).
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
    use crate::{
        Authority, Budget, Contract, Deadline, Delegation, Domain, Event, Executor, Flags, Funder, Kept, Models, New,
        Numbers, Party, Scopes, Spec, Stored, Tools, WakePolicy, step,
    };
    use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

    #[test]
    fn remap_models_updates_a_live_authority_and_preserves_all() {
        let kept = Kept { revisions: 4, versions: 8 };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::limits::tests::LIMITS };
        let mut domain = Domain::new(&env.limits, kept, 91, 1, crate::tests::charters(), Box::new([0]));
        let authority = Authority {
            tools: Tools(0),
            models: Models::Named(3),
            grants: Box::new([]),
            delegation: Delegation { kinds: Box::new([]), tasks: 16, depth: 4 },
            budget: Budget { spend: 100, deadline: None },
            notes: Scopes(0),
            note_resources: Box::new([]),
        };
        let new = New {
            deadline: Deadline::Named,
            label: 0,
            flags: Flags::PLAIN,
            number: 1,
            project: 1,
            executor: Executor::Agent { charter: 1 },
            llm: None,
            spec: Spec { job: None, asks: false, words: Box::new([1]), parameters: Box::new([]), inputs: Box::new([]) },
            contract: Contract::Report { words: 32 },
            authority,
            numbers: Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
            funder: Funder::Period { project: 1, period: 0 },
            dependencies: Box::new([]),
            holdings: Box::new([]),
            wake: WakePolicy::DEFAULT,
            recurring: None,
            tracked: None,
        };
        let mut out = Queue::uncapped();
        for event in [
            Event::Check,
            Event::Restored,
            Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 1, period: 0, budget: 10_000 },
            Event::Make { reply_to: ReplyTo::new(Token::new(2)), creator: Party::Person(9), batch: Box::new([new]) },
        ] {
            step(&mut domain, &env, event, &mut out);
        }
        let mut row = Stored::Live(Box::new(crate::domain::record(&domain, 1).expect("admitted task").clone()));
        let mut moved = [0; 64];
        moved[0] = 4;
        let mut expected = row.clone();
        let Stored::Live(value) = &mut expected else { panic!("live") };
        value.authority.models = Models::Named(4);
        row.remap_models(&moved);
        assert_eq!(row, expected);
        let Stored::Live(value) = &mut row else { panic!("live") };
        value.authority.models = Models::All;
        let expected = row.clone();
        row.remap_models(&moved);
        assert_eq!(row, expected);
    }
}

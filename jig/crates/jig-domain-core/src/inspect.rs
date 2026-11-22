//! Borrow the core-owned facts an inspection of its children cannot know.
use crate::{Core, routing::flight::Flight};
use jig_domain_tasks::New;
use skein_lib::{Map, ReplyTo, Time, Token};

impl Core {
    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_now(&self) -> Time {
        self.run_now
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_batch(&self, owner: Token) -> Option<(u64, &[New], Time)> {
        for (id, flight) in &self.flights {
            match flight {
                Flight::AwaitingResources(batch) => {
                    if batch.reply_to == ReplyTo::new(owner) {
                        return Some((id.token().raw(), &batch.members, batch.deadline));
                    }
                }
                Flight::Party { .. }
                | Flight::PartyPart { .. }
                | Flight::PersonTask { .. }
                | Flight::Goal { .. }
                | Flight::PersonProposal(_)
                | Flight::Call { .. }
                | Flight::Made { .. }
                | Flight::PersonEscalation(_)
                | Flight::ProcedureProposal { .. }
                | Flight::Internal(_)
                | Flight::Moving { .. }
                | Flight::EndedMark { .. }
                | Flight::Key { .. }
                | Flight::Application { .. }
                | Flight::Watch { .. }
                | Flight::Delegating { .. }
                | Flight::Saying { .. }
                | Flight::ResultRead(_)
                | Flight::CreatingInputs { .. }
                | Flight::Creating { .. }
                | Flight::CreatingDelegates { .. }
                | Flight::Workspace { .. }
                | Flight::ProcedureCreation { .. }
                | Flight::EffectStaging
                | Flight::Effect { .. }
                | Flight::Note(_)
                | Flight::SettingsOffering { .. }
                | Flight::Retired => {}
            }
        }
        None
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub fn inspection_placement(&self, task: u64) -> Option<Token> {
        self.proofs.get(&task)?.placement
    }

    /// For its worlds' inspection adapter (skein/docs/design/testing/inspection.md, 2).
    #[must_use]
    pub const fn inspection_closing_connectors(&self) -> &Map<(u64, u16), ()> {
        &self.closing_connectors
    }
}

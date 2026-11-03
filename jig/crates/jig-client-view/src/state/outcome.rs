//! Routing facts from retained outcomes (client/domain/contract.md, 4).
//! Classification never rewrites the whole durable answer or joins missing facts.
use jig_client::contract::{DocumentOutcome, Made, Outcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Created { task: u64 },
    Words { task: u64, message: u64 },
    Proposed,
    Other,
}
pub(crate) fn route(outcome: &Outcome) -> Route {
    match outcome {
        Outcome::Document(value) => match value {
            DocumentOutcome::Created { first, last, .. } => {
                if first == last {
                    Route::Created { task: *first }
                } else {
                    Route::Other
                }
            }
            DocumentOutcome::Said { task, message, .. } => Route::Words { task: *task, message: *message },
            DocumentOutcome::Proposed { .. } => Route::Proposed,
            DocumentOutcome::Withdrawn { .. }
            | DocumentOutcome::Compound { .. }
            | DocumentOutcome::MarkedSeen { .. }
            | DocumentOutcome::NoteEdited { .. }
            | DocumentOutcome::ServiceMade { .. }
            | DocumentOutcome::Adopted { .. }
            | DocumentOutcome::Stopped { .. }
            | DocumentOutcome::Cancelled { .. }
            | DocumentOutcome::Lifted { .. }
            | DocumentOutcome::Released { .. }
            | DocumentOutcome::PersonTaken { .. }
            | DocumentOutcome::PersonHandedBack { .. }
            | DocumentOutcome::PersonAnswered { .. }
            | DocumentOutcome::Moved { .. }
            | DocumentOutcome::ProposalDecided { .. }
            | DocumentOutcome::QuestionAnswered { .. }
            | DocumentOutcome::Amended { .. }
            | DocumentOutcome::RolesSet { .. }
            | DocumentOutcome::PolicyChanged { .. }
            | DocumentOutcome::AccountProgramDeclared { .. }
            | DocumentOutcome::AccountKeyKept { .. }
            | DocumentOutcome::AccountFinishing { .. }
            | DocumentOutcome::AccountMovedIn { .. }
            | DocumentOutcome::AccountChanged
            | DocumentOutcome::AccountRemoved { .. }
            | DocumentOutcome::AccountSignInStarted { .. }
            | DocumentOutcome::AccountSignedOut { .. }
            | DocumentOutcome::ProjectRemoved { .. }
            | DocumentOutcome::Renamed { .. }
            | DocumentOutcome::ProjectMade { .. }
            | DocumentOutcome::PoolSet { .. }
            | DocumentOutcome::EscalationDecided { .. } => Route::Other,
        },
        Outcome::Made { what } => match what {
            Made::Tasks { tasks } => {
                if tasks.len() == 1 {
                    Route::Created { task: *tasks.get(0).expect("one created task") }
                } else {
                    Route::Other
                }
            }
            Made::MovingOut { .. } | Made::Service { .. } | Made::PersonResult { .. } | Made::Note { .. } => {
                Route::Other
            }
        },
        Outcome::SettingsChanged(_) | Outcome::MovedToNewest(_) => Route::Other,
        Outcome::Restored { .. } => Route::Proposed,
    }
}

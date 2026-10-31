//! Retained answers outlive project deletion
//! (temper/docs/design/draft/playground-reset.md, 5).
use super::Domain;
use crate::{Outcome, Refusal};

pub(super) fn answer(domain: &Domain, project: u32, outcome: &Outcome) -> Outcome {
    if let Some(project) = named_project(project, outcome)
        && !domain.roles.contains_key(&project)
    {
        return Outcome::Refused(Refusal::Unknown);
    }
    outcome.clone()
}

fn named_project(subject: u32, outcome: &Outcome) -> Option<u32> {
    let project = match outcome {
        Outcome::ProjectRemoved { .. } => return None,
        Outcome::ProjectMade { project }
        | Outcome::RolesSet { project }
        | Outcome::Renamed { project }
        | Outcome::PolicyChanged { project }
        | Outcome::PoolSet { project, .. }
        | Outcome::Adopted { project }
        | Outcome::Compound { project, .. } => *project,
        Outcome::AccountFinishing { .. }
        | Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::CompoundRefused { .. }
        | Outcome::Created { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::Again(_)
        | Outcome::SignedOut
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Proposed { .. }
        | Outcome::Withdrawn { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Released { .. }
        | Outcome::Lifted { .. }
        | Outcome::PersonTaken { .. }
        | Outcome::PersonHandedBack { .. }
        | Outcome::PersonAnswered { .. }
        | Outcome::Moved { .. }
        | Outcome::ProposalDecided { .. }
        | Outcome::Said { .. }
        | Outcome::QuestionAnswered { .. }
        | Outcome::Amended { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::AccountSignInStarted { .. }
        | Outcome::AccountSignedOut { .. }
        | Outcome::AccountKeyKept { .. }
        | Outcome::AccountProgramDeclared { .. }
        | Outcome::AccountChanged
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::AccountRemoved { .. }
        | Outcome::Refused(_) => subject,
    };
    if project == crate::DEPLOYMENT { None } else { Some(project) }
}

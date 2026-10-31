//! Project closure and retained deletion progress (jig/docs/design/domain/people.md, 5.2).
use super::Domain;
use crate::{Act, Ask, DeletionStep, Key, Outcome, Request, Stored};
use skein_lib::{List, Queue};

pub(super) fn keep(domain: &mut Domain, project: u32, step: Option<DeletionStep>, out: &mut Queue<Request>) {
    match step {
        Some(step) => {
            domain.deletions.insert(project, step).expect("deletions fit the project room");
            out.push(Request::Save { record: Stored::Deletion { project, step } });
        }
        None => {
            domain.deletions.remove(&project);
            out.push(Request::Erase { key: Key::Deletion(project) });
        }
    }
}

pub(super) fn close(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    domain.roles.remove(&project);
    super::prune_unheld_roles(domain, project);
    out.push(Request::Erase { key: Key::Roles(project) });
}

pub(super) fn forget(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut answers = List::with_capacity(domain.answers.len());
    for (key, answer) in &domain.answers {
        if ask_project(&answer.ask) == Some(project) || made_project(&answer.outcome) == Some(project) {
            answers.push(*key).expect("bounded retained answers");
        }
    }
    for key in answers.into_boxed() {
        domain.answers.remove(&key);
        domain.answer_alarms.cancel(key);
        out.push(Request::Erase { key: Key::Answer(key) });
    }
    let mut acts = List::with_capacity(domain.acted.len());
    for (key, answer) in &domain.acted {
        if answer.project() == Some(project) {
            acts.push(*key).expect("bounded retained acts");
        }
    }
    for key in acts.into_boxed() {
        domain.acted.remove(&key);
        domain.act_alarms.cancel(key);
        out.push(Request::Erase { key: Key::Acted(key) });
    }
}

pub(super) fn made_project(outcome: &Outcome) -> Option<u32> {
    match outcome {
        Outcome::ProjectMade { project } => Some(*project),
        Outcome::Again(_)
        | Outcome::SignedOut
        | Outcome::Compound { .. }
        | Outcome::CompoundRefused { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::NoteEdited { .. }
        | Outcome::ServiceMade { .. }
        | Outcome::Adopted { .. }
        | Outcome::RolesSet { .. }
        | Outcome::PolicyChanged { .. }
        | Outcome::Renamed { .. }
        | Outcome::ProjectRemoved { .. }
        | Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::PoolSet { .. }
        | Outcome::Said { .. }
        | Outcome::QuestionAnswered { .. }
        | Outcome::Amended { .. }
        | Outcome::Moved { .. }
        | Outcome::Stopped { .. }
        | Outcome::Cancelled { .. }
        | Outcome::Lifted { .. }
        | Outcome::Released { .. }
        | Outcome::Created { .. }
        | Outcome::Proposed { .. }
        | Outcome::Withdrawn { .. }
        | Outcome::PersonTaken { .. }
        | Outcome::PersonHandedBack { .. }
        | Outcome::PersonAnswered { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::ProposalDecided { .. }
        | Outcome::AccountSignInStarted { .. }
        | Outcome::AccountProgramDeclared { .. }
        | Outcome::AccountKeyKept { .. }
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountFinishing { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::AccountChanged
        | Outcome::AccountRemoved { .. }
        | Outcome::AccountSignedOut { .. }
        | Outcome::Refused(_) => None,
    }
}

pub(super) fn act_project(act: &Act) -> Option<u32> {
    match act {
        Act::Close { project, .. } | Act::Adopt { project, .. } | Act::RemoveProject { project } => Some(*project),
        Act::MakeProject { .. } | Act::AddKey { .. } | Act::RemoveAccount { .. } => None,
    }
}

pub(super) fn ask_project(ask: &Ask) -> Option<u32> {
    match ask {
        Ask::Application { scope, .. } => match scope {
            crate::ApplicationScope::Project(project) => Some(*project),
            crate::ApplicationScope::Own | crate::ApplicationScope::Deployment => None,
        },
        Ask::Lift { .. }
        | Ask::MarkSeen { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. } => None,
        Ask::RemoveProject { .. }
        | Ask::Rename { .. }
        | Ask::Compound { .. }
        | Ask::EditNote { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::SetPool { .. }
        | Ask::DecideEscalation { .. }
        | Ask::DecideProposal { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::Move { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Release { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. } => Some(super::project(ask)),
    }
}

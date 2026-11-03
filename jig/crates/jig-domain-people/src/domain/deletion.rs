//! Project closure and retained deletion progress (jig/docs/design/domain/people.md, 5.2).
use super::Domain;
use crate::{Act, Ask, DeletionStep, KeptDeletion, Key, Outcome, Request, Stored};
use skein_lib::{Duration, List, Queue};

pub(super) fn keep(
    domain: &mut Domain,
    project: u32,
    step: Option<DeletionStep>,
    grace: Option<Duration>,
    out: &mut Queue<Request>,
) {
    match step {
        Some(step) => {
            domain.deletions.insert(project, KeptDeletion { step, grace }).expect("deletions fit the project room");
            out.push(Request::Save { record: Stored::Deletion { project, step, grace } });
        }
        None => {
            domain.deletions.remove(&project);
            forget(domain, project, out);
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
    forget_census(domain, project);
    let mut answers = List::with_capacity(domain.answers.len());
    for (key, answer) in &domain.answers {
        if kept_answer(domain, &answer.outcome) {
            continue;
        }
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
        if kept_answer(domain, answer.outcome()) {
            continue;
        }
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

fn forget_census(domain: &mut Domain, project: u32) {
    let mut parties = List::with_capacity(domain.chat_updates.len());
    for (&(party, scope), _) in &domain.chat_updates {
        if scope == project {
            parties.push(party).expect("project reply census subset");
        }
    }
    for party in parties.into_boxed() {
        domain.chat_updates.remove(&(party, project));
    }
    let mut parties = List::with_capacity(domain.result_census.len());
    for (&(party, scope), _) in &domain.result_census {
        if scope == project {
            parties.push(party).expect("project result census subset");
        }
    }
    for party in parties.into_boxed() {
        drop(domain.result_census.remove(&(party, project)));
    }
}

/// A removal's terminal stays until the deletion finishes (jig/docs/design/domain/people.md, 5.2).
pub(super) fn kept_answer(domain: &Domain, outcome: &Outcome) -> bool {
    for (&project, _) in &domain.deletions {
        if *outcome == (Outcome::ProjectRemoved { project }) {
            return true;
        }
    }
    false
}

pub(super) fn made_project(outcome: &Outcome) -> Option<u32> {
    match outcome {
        Outcome::ProjectMade { project } => Some(*project),
        Outcome::Restored { .. }
        | Outcome::Again(_)
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
        Act::Close { project, .. } | Act::Adopt { project, .. } | Act::RemoveProject { project, .. } => Some(*project),
        Act::MakeProject { .. } | Act::AddKey { .. } | Act::RemoveAccount { .. } => None,
    }
}

pub(super) fn ask_project(ask: &Ask) -> Option<u32> {
    match ask {
        Ask::Application { scope, .. } => match scope {
            crate::ApplicationScope::Project(project) => Some(*project),
            crate::ApplicationScope::Own | crate::ApplicationScope::Deployment => None,
        },
        Ask::Restore { .. }
        | Ask::Lift { .. }
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

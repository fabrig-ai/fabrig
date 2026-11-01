//! Project deletion retires people rows in the deciding step
//! (draft 5; jig/docs/design/domain/people.md, 5.2).
use super::{Domain, acts, end_signin, inbox};
use crate::{Ask, Key, Kind, Limits, Outcome, Request};
use skein_lib::{Env, List, Queue};

pub(super) fn closed(domain: &mut Domain, env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    let mut services = List::with_capacity(env.limits.people);
    if let Some(holdings) = domain.roles.get(&project) {
        for holding in holdings {
            if domain.kind(holding.person) == Some(Kind::Service) {
                let mut elsewhere = false;
                for (other, roster) in &domain.roles {
                    for row in roster {
                        elsewhere |= *other != project && row.person == holding.person;
                    }
                }
                if !elsewhere {
                    services.push(holding.person).expect("services are a subset of the project's roster");
                }
            }
        }
    }
    drop(domain.roles.remove(&project));
    out.push(Request::Erase { key: Key::Roles(project) });
    out.push(Request::Erase { key: Key::Policy(project) });
    inbox::project_closed(domain, env, project, out);
    let mut answers = List::with_capacity(domain.answers.len());
    for (key, answer) in &domain.answers {
        if services.as_slice().contains(&key.person)
            || removed_reference(&answer.ask, &answer.outcome, project, &services)
        {
            answers.push(*key).expect("one key per retained answer");
        }
    }
    for key in answers.into_boxed() {
        erase_answer(domain, key, out);
    }
    for person in services.into_boxed() {
        let mut sign_ins = List::with_capacity(domain.sign_ins.len());
        for (number, sign_in) in &domain.sign_ins {
            if sign_in.person == person {
                sign_ins.push(*number).expect("one entry per sign-in");
            }
        }
        for number in sign_ins.into_boxed() {
            end_signin(domain, number, out);
        }
        inbox::party_removed(domain, env, person, out);
        if let Some(identity) = domain.people.remove(&person) {
            domain.identities.remove(&identity.key);
            out.push(Request::Erase { key: Key::Person(person) });
        }
    }
}

pub(super) fn gone(domain: &mut Domain, _env: &Env<Limits>, project: u32, out: &mut Queue<Request>) {
    let mut answers = List::with_capacity(domain.answers.len());
    for (key, answer) in &domain.answers {
        if names_project(&answer.ask, project) {
            answers.push(*key).expect("one key per retained answer");
        }
    }
    for key in answers.into_boxed() {
        erase_answer(domain, key, out);
    }
    acts::erase_project(domain, project, out);
}

fn erase_answer(domain: &mut Domain, key: crate::RequestKey, out: &mut Queue<Request>) {
    drop(domain.answers.remove(&key));
    domain.answer_alarms.cancel(key);
    out.push(Request::Erase { key: Key::Answer(key) });
}

fn removed_reference(ask: &Ask, outcome: &Outcome, project: u32, services: &List<u64>) -> bool {
    match outcome {
        Outcome::ServiceMade { person } => services.as_slice().contains(person),
        Outcome::RolesSet { project: roster } => {
            if *roster == project {
                return true;
            }
            match ask {
                Ask::SetRoles { holdings, .. } => {
                    for holding in holdings {
                        if services.as_slice().contains(&holding.person) {
                            return true;
                        }
                    }
                    false
                }
                Ask::ChangeSettings { .. }
                | Ask::MoveToNewest { .. }
                | Ask::Compound { .. }
                | Ask::Create { .. }
                | Ask::MarkSeen { .. }
                | Ask::Application { .. }
                | Ask::EditNote { .. }
                | Ask::MakeService { .. }
                | Ask::Adopt { .. }
                | Ask::Propose { .. }
                | Ask::Withdraw { .. }
                | Ask::Stop { .. }
                | Ask::Cancel { .. }
                | Ask::Release { .. }
                | Ask::Lift { .. }
                | Ask::TakePerson { .. }
                | Ask::HandBackPerson { .. }
                | Ask::AnswerPerson { .. }
                | Ask::Move { .. }
                | Ask::DecideProposal { .. }
                | Ask::Say { .. }
                | Ask::AnswerQuestion { .. }
                | Ask::Amend { .. }
                | Ask::ChangePolicy { .. }
                | Ask::Rename { .. }
                | Ask::MakeProject { .. }
                | Ask::RemoveProject { .. }
                | Ask::SetPool { .. }
                | Ask::DecideEscalation { .. }
                | Ask::AccountSignIn { .. }
                | Ask::AccountSignOut { .. }
                | Ask::AccountChange { .. } => unreachable!("role success retains a roster ask"),
            }
        }
        Outcome::AccountFinishing { .. }
        | Outcome::SettingsChanged { .. }
        | Outcome::MovedToNewest { .. }
        | Outcome::Compound { .. }
        | Outcome::CompoundRefused { .. }
        | Outcome::Created { .. }
        | Outcome::MarkedSeen { .. }
        | Outcome::Applied { .. }
        | Outcome::Again(_)
        | Outcome::SignedOut
        | Outcome::NoteEdited { .. }
        | Outcome::Adopted { .. }
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
        | Outcome::PolicyChanged { .. }
        | Outcome::Renamed { .. }
        | Outcome::ProjectMade { .. }
        | Outcome::ProjectRemoved { .. }
        | Outcome::PoolSet { .. }
        | Outcome::EscalationDecided { .. }
        | Outcome::AccountSignInStarted { .. }
        | Outcome::AccountSignedOut { .. }
        | Outcome::AccountKeyKept { .. }
        | Outcome::AccountProgramDeclared { .. }
        | Outcome::AccountChanged
        | Outcome::AccountMovingOut { .. }
        | Outcome::AccountMovedIn { .. }
        | Outcome::AccountRemoved { .. }
        | Outcome::Refused(_) => false,
    }
}

/// Whether the ask explicitly names this project; person-wide asks name none.
pub(super) fn names_project(ask: &Ask, project: u32) -> bool {
    match ask {
        Ask::Application { scope, .. } => match scope {
            crate::ApplicationScope::Project(named) => *named == project,
            crate::ApplicationScope::Own | crate::ApplicationScope::Deployment => false,
        },
        Ask::Lift { .. }
        | Ask::MarkSeen { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. } => false,
        Ask::Compound { project: named, .. }
        | Ask::Create { project: named, .. }
        | Ask::EditNote { project: named, .. }
        | Ask::MakeService { project: named, .. }
        | Ask::Adopt { project: named, .. }
        | Ask::Propose { project: named, .. }
        | Ask::Withdraw { project: named, .. }
        | Ask::Stop { project: named, .. }
        | Ask::Cancel { project: named, .. }
        | Ask::Release { project: named, .. }
        | Ask::TakePerson { project: named, .. }
        | Ask::HandBackPerson { project: named, .. }
        | Ask::AnswerPerson { project: named, .. }
        | Ask::Move { project: named, .. }
        | Ask::DecideProposal { project: named, .. }
        | Ask::Say { project: named, .. }
        | Ask::AnswerQuestion { project: named, .. }
        | Ask::Amend { project: named, .. }
        | Ask::SetRoles { project: named, .. }
        | Ask::ChangePolicy { project: named, .. }
        | Ask::Rename { project: named, .. }
        | Ask::RemoveProject { project: named, .. }
        | Ask::SetPool { project: named, .. }
        | Ask::DecideEscalation { project: named, .. } => *named == project,
    }
}

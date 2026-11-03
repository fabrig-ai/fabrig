//! Application core parts and request admission (domain/people.md, 5.1).
use super::{Domain, invalid_ask, project, role, valid_ask};
use crate::{Ask, Kind, Limits, Outcome, Refusal, Request, Role};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Token};

/// Judge a connector's core part as its admitted party, without allocating another key.
/// The caller completes this part before the application flight (domain/people.md, 5.1).
/// Explicit model selectors refuse `Lack::Models`: parts carry no model-name context yet.
pub fn application_part(domain: &mut Domain, env: &Env<Limits>, request: Token, ask: Ask) -> Result<Request, Outcome> {
    domain.input();
    let id = Id::from_token(request);
    let flight = domain.party_pending(id).ok_or(Outcome::Refused(Refusal::Unknown))?;
    let application_project = match &flight.ask {
        Ask::Application { scope, .. } => match scope {
            crate::ApplicationScope::Project(project) => *project,
            crate::ApplicationScope::Own | crate::ApplicationScope::Deployment => 0,
        },
        Ask::Restore { .. }
        | Ask::Compound { .. }
        | Ask::MarkSeen { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::RemoveProject { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::SetPool { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::DecideEscalation { .. }
        | Ask::DecideProposal { .. }
        | Ask::Move { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::EditNote { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => return Err(Outcome::Refused(Refusal::NotOffered)),
    };
    match &ask {
        Ask::Restore { .. } => return Err(Outcome::Refused(Refusal::NotOffered)),
        // An account request is its holder's own and a sign-out waits on the accounts'
        // component, so neither is a synchronous part of an application's request.
        Ask::Application { .. }
        | Ask::AccountSignIn { .. }
        | Ask::AccountSignOut { .. }
        | Ask::AccountChange { .. } => {
            return Err(Outcome::Refused(Refusal::NotOffered));
        }
        Ask::Compound { .. }
        | Ask::MarkSeen { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::RemoveProject { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::ChangeSettings { .. }
        | Ask::MoveToNewest { .. }
        | Ask::SetPool { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::DecideEscalation { .. }
        | Ask::DecideProposal { .. }
        | Ask::Move { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. }
        | Ask::EditNote { .. } => {}
    }
    if project(&ask) != application_project {
        return Err(Outcome::Refused(Refusal::Unknown));
    }
    if !valid_ask(&env.limits, &ask) {
        return Err(Outcome::Refused(invalid_ask(&env.limits, &ask)));
    }
    if crate::models::has_model_selector(&ask) {
        return Err(Outcome::Refused(Refusal::Authority { lacking: crate::Lack::Models, proposable: None }));
    }
    let person = flight.key.person;
    let role = role(domain, person, application_project);
    if let Some(why) = admission(domain, person, application_project, role, &ask) {
        return Err(Outcome::Refused(why));
    }
    let flight = domain.party_pending_mut(id).expect("admitted application flight");
    assert!(flight.part.is_none(), "one core part at a time");
    flight.part = Some(ask.clone());
    Ok(Request::Route { request, person, project: application_project, role, ask: Box::new(ask), models: Box::new([]) })
}

pub(super) fn admission(domain: &Domain, person: u64, _project: u32, role: Option<Role>, ask: &Ask) -> Option<Refusal> {
    match ask {
        Ask::Propose { project, action: crate::Action::Settings { .. }, .. } if *project != crate::DEPLOYMENT => {
            Some(Refusal::Unknown)
        }
        Ask::Restore { .. } | Ask::Propose { action: crate::Action::Settings { .. }, .. } => {
            for (_, holdings) in &domain.roles {
                for holding in &**holdings {
                    if holding.person == person {
                        return None;
                    }
                }
            }
            Some(Refusal::Role { held: None })
        }
        Ask::ChangeSettings { projects, .. } => {
            for &project in projects {
                if !domain.has_project(project) || domain.deletions.contains_key(&project) {
                    return Some(Refusal::Unknown);
                }
                let held = super::role(domain, person, project);
                if held != Some(Role::Owner) {
                    return Some(Refusal::Role { held });
                }
            }
            if projects.is_empty() && role != Some(Role::Owner) { Some(Refusal::Role { held: role }) } else { None }
        }
        Ask::RemoveProject { .. } | Ask::MoveToNewest { .. } => {
            if role == Some(Role::Owner) {
                None
            } else {
                Some(Refusal::Role { held: role })
            }
        }
        Ask::Application { scope, .. } => {
            if *scope == crate::ApplicationScope::Own || role.is_some() {
                None
            } else {
                Some(Refusal::Role { held: None })
            }
        }
        Ask::AccountSignIn { holder, .. } | Ask::AccountSignOut { holder, .. } | Ask::AccountChange { holder, .. } => {
            if holds(domain, person, *holder) {
                None
            } else {
                Some(Refusal::Role { held: None })
            }
        }
        Ask::Compound { .. }
        | Ask::EditNote { .. }
        | Ask::MakeService { .. }
        | Ask::Adopt { .. }
        | Ask::SetRoles { .. }
        | Ask::ChangePolicy { .. }
        | Ask::Rename { .. }
        | Ask::MakeProject { .. }
        | Ask::SetPool { .. }
        | Ask::Create { .. }
        | Ask::Propose { .. }
        | Ask::Withdraw { .. }
        | Ask::Say { .. }
        | Ask::AnswerQuestion { .. }
        | Ask::Amend { .. }
        | Ask::MarkSeen { .. }
        | Ask::DecideEscalation { .. }
        | Ask::DecideProposal { .. }
        | Ask::Move { .. }
        | Ask::TakePerson { .. }
        | Ask::HandBackPerson { .. }
        | Ask::AnswerPerson { .. }
        | Ask::Stop { .. }
        | Ask::Cancel { .. }
        | Ask::Lift { .. }
        | Ask::Release { .. } => None,
    }
}

/// Whether this party is the holder an account request names: a person for their own, a first
/// owner for the deployment's (domain/accounts.md, 3.2; domain/people.md, section 3). A service
/// holds no account and decides none of the deployment's.
pub(super) fn holds(domain: &Domain, person: u64, holder: crate::AccountHolder) -> bool {
    if domain.kind(person) != Some(Kind::Person) {
        return false;
    }
    match holder {
        crate::AccountHolder::Own => true,
        crate::AccountHolder::Deployment => {
            let Some(identity) = domain.identity(person) else { return false };
            for owner in &domain.owners {
                if owner.identity == identity.key {
                    return true;
                }
            }
            false
        }
    }
}

/// Forget the synchronous part while preserving its application key and digest.
pub fn application_part_done(domain: &mut Domain, request: Token) {
    domain.input();
    if let Some(flight) = domain.party_pending_mut(Id::from_token(request)) {
        flight.part = None;
    }
}

//! `PersonFact` admission and durability (temper/docs/design/domain/forge.md, 5.5).
use crate::{
    Application, ApplicationAnswer, Delivery, Domain, Event, Limits, PartyInput, PartyRequest, Request, StoreInput,
    StoreRequest,
};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};
use temper_engine_domain_forge as forge;

const PARTY: u64 = 1;
const SIGN_IN: u64 = 10;
const REPLY: Token = Token::new(90);

fn signed_in(role: Option<people::Role>, deleting: bool) -> (Domain, Env<Limits>) {
    let mut config = super::resume_fixture::config_with_project_room(1, 2);
    let seed = config.core.seed_projects.first().unwrap();
    let first =
        jig_domain_core::SeedProject { number: seed.number, name: seed.name.clone(), policy: seed.policy.clone() };
    let second = jig_domain_core::SeedProject { number: 2, name: "Project 2".into(), policy: first.policy.clone() };
    config.core.seed_projects = Box::new([first, second]);
    let mut limits = super::resume_fixture::limits();
    limits.authority.projects = 2;
    limits.people.projects = 3;
    let mut records = List::uncapped();
    for row in [
        people::Stored::Person {
            number: PARTY,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"member".as_slice().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::SignIn { number: SIGN_IN, person: PARTY, expires: Wall::from_nanos(100) },
        people::Stored::Roles { project: 2, holdings: Box::new([]) },
        people::Stored::Roles {
            project: 1,
            holdings: match role {
                Some(role) => Box::new([people::Holding { person: PARTY, role }]),
                None => Box::new([]),
            },
        },
    ] {
        records.push(crate::Record::Core(jig_domain_core::Record::People(row))).unwrap();
    }
    if deleting {
        records
            .push(crate::Record::Core(jig_domain_core::Record::People(people::Stored::Deletion {
                project: 1,
                step: people::DeletionStep::Tasks,
                grace: None,
            })))
            .unwrap();
    }
    super::resume_fixture::root_with_records(config, &limits, records.as_slice())
}

fn dismiss(project: u32) -> Event {
    Event::Party(PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(REPLY),
        sign_in: SIGN_IN,
        key: [1; 16],
        digest: [1; 32],
        carried: None,
        request: Application::Dismiss { what: forge::PersonFact::ToBegin { project } },
    })
}
fn settle(domain: &mut Domain, env: &Env<Limits>) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    let mut requests = List::uncapped();
    for _ in 0_u32..32 {
        crate::release(domain, env, &mut out);
        crate::release(domain, env, &mut out);
        while let Some(request) = out.pop() {
            if let Request::Store(StoreRequest::Commit { number, .. }) = request {
                crate::try_step(domain, env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
            } else if let Request::ToChild(released) = request {
                crate::try_step(domain, env, Event::Released(released)).unwrap();
            } else if let Request::Store(StoreRequest::Read { range, .. }) = request {
                assert!(domain.core.people.deletions().contains_key(&1), "only deletion reads in this fixture");
                assert_eq!(range.scope, jig_domain_core::reads::Scope::EndedTasks);
                // Leave the deletion's store walk pending while checking admission.
            } else {
                requests.push(request).unwrap();
            }
        }
        crate::try_step(domain, env, Event::Resume).unwrap();
    }
    requests.into_boxed()
}
fn refused(domain: &mut Domain, env: &Env<Limits>, project: u32) {
    crate::try_step(domain, env, dismiss(project)).unwrap();
    assert_eq!(
        settle(domain, env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::WebReply {
            to: ReplyTo::new(REPLY),
            reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None })),
        })),]
    );
    assert!(domain.forge.dismissals(PARTY).is_empty());
    assert!(domain.application.is_none());
}
#[test]
fn a_dismissal_is_of_the_project_it_names() {
    let (mut domain, env) = signed_in(Some(people::Role::Observer), false);
    assert_eq!(domain.core.people.role(PARTY, 1), Some(people::Role::Observer));
    assert_eq!(domain.core.people.role(PARTY, 2), None);
    refused(&mut domain, &env, 2);
}
#[test]
fn a_party_with_no_role_is_refused_before_the_connector() {
    let (mut domain, env) = signed_in(None, false);
    refused(&mut domain, &env, 1);
}
#[test]
fn a_project_being_deleted_refuses_a_dismissal() {
    let (mut domain, env) = signed_in(Some(people::Role::Observer), true);
    assert!(domain.core.people.deletions().contains_key(&1));
    refused(&mut domain, &env, 1);
}
#[test]
fn a_dismissal_is_answered_only_once_its_row_is_durable() {
    let (mut domain, env) = signed_in(Some(people::Role::Observer), false);
    crate::try_step(&mut domain, &env, dismiss(1)).unwrap();
    let row = forge::Stored::Dismissed { party: PARTY, what: forge::PersonFact::ToBegin { project: 1 } };
    let mut out = Queue::uncapped();
    let mut commit = None;
    for _ in 0_u32..8 {
        crate::release(&mut domain, &env, &mut out);
        while let Some(request) = out.pop() {
            let Request::Store(StoreRequest::Commit { number, writes }) = request else {
                panic!("dismissal before durable commit: {request:?}");
            };
            assert!(writes.contains(&crate::Write::Save(crate::Record::Forge { row: Box::new(row.clone()) })));
            commit = Some(number);
        }
    }
    let number = commit.expect("dismissal commit issued");
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 })).unwrap();
    assert_eq!(
        settle(&mut domain, &env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::Application {
            to: ReplyTo::new(REPLY),
            outcome: Some(ApplicationAnswer::Dismiss(forge::DismissAnswer::Dismissed)),
        })),]
    );
    assert_eq!(domain.forge.dismissals(PARTY).as_ref(), [forge::PersonFact::ToBegin { project: 1 }]);
    crate::try_step(&mut domain, &env, dismiss(1)).unwrap();
    assert_eq!(
        settle(&mut domain, &env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::Application {
            to: ReplyTo::new(REPLY),
            outcome: Some(ApplicationAnswer::Dismiss(forge::DismissAnswer::Dismissed)),
        })),]
    );
}

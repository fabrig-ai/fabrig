//! Look admission (temper/docs/design/domain/forge.md, 5.6).
use crate::{
    Application, Delivery, Domain, Event, Limits, PartyInput, PartyRequest, Request, StoreInput, StoreRequest,
};
use alloc::boxed::Box;
use jig_domain_people as people;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};

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

fn looked(scope: people::ApplicationScope, project: u32) -> Event {
    Event::Party(PartyInput::Application {
        scope,
        reply_to: ReplyTo::new(REPLY),
        sign_in: SIGN_IN,
        key: [1; 16],
        digest: [1; 32],
        carried: None,
        request: Application::Looked { project },
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
            } else {
                requests.push(request).unwrap();
            }
        }
        crate::try_step(domain, env, Event::Resume).unwrap();
    }
    requests.into_boxed()
}
fn refused(domain: &mut Domain, env: &Env<Limits>, project: u32) {
    crate::try_step(domain, env, looked(people::ApplicationScope::Project(1), project)).unwrap();
    assert_eq!(
        settle(domain, env).as_ref(),
        [Request::Party(PartyRequest::Deliver(Delivery::WebReply {
            to: ReplyTo::new(REPLY),
            reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None })),
        })),]
    );
    assert!(domain.application.is_none());
}
#[test]
fn a_look_is_of_the_project_it_names() {
    let (mut domain, env) = signed_in(Some(people::Role::Observer), false);
    refused(&mut domain, &env, 2);
    for scope in
        [people::ApplicationScope::Own, people::ApplicationScope::Deployment, people::ApplicationScope::Project(2)]
    {
        let (mut domain, env) = signed_in(Some(people::Role::Observer), false);
        crate::try_step(&mut domain, &env, looked(scope, 1)).unwrap();
        assert_eq!(
            settle(&mut domain, &env).as_ref(),
            [Request::Party(PartyRequest::Deliver(Delivery::Application {
                to: ReplyTo::new(REPLY),
                outcome: Some(crate::ApplicationAnswer::Looked(temper_engine_domain_forge::LookedAnswer::Looked)),
            }))]
        );
    }
}
#[test]
fn a_party_with_no_role_is_refused() {
    let (mut domain, env) = signed_in(None, false);
    refused(&mut domain, &env, 1);
}
#[test]
fn a_project_being_deleted_refuses_a_look() {
    let (mut domain, env) = signed_in(Some(people::Role::Observer), true);
    refused(&mut domain, &env, 1);
}

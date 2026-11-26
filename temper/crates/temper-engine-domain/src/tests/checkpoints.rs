//! Ownership and the delayed store terminal (temper/docs/design/protocol/checkpoints.md, 3).
use crate::{
    Application, ApplicationAnswer, CheckpointWhy, Checkpointed, Delivery, Domain, Event, Limits, PartyInput,
    PartyRequest, Request, StoreInput, StoreRequest,
};
use jig_domain_people as people;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};

fn signed_in(deployment: bool) -> (Domain, Env<Limits>) {
    let records = [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: b"owner".as_slice().into() },
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(100) },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
        people::Stored::Roles {
            project: people::DEPLOYMENT,
            holdings: if deployment {
                Box::new([people::Holding { person: 1, role: people::Role::Owner }])
            } else {
                Box::new([])
            },
        },
    ]
    .map(|row| crate::Record::Core(jig_domain_core::Record::People(row)));
    super::resume_fixture::root_with_records(
        super::resume_fixture::config(1),
        &super::resume_fixture::limits(),
        &records,
    )
}
fn ask() -> Event {
    Event::Party(PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in: 10,
        key: [1; 16],
        digest: [1; 32],
        carried: None,
        request: Application::Checkpoint,
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
fn pending(domain: &mut Domain, env: &Env<Limits>) -> Token {
    crate::try_step(domain, env, ask()).unwrap();
    let requests = settle(domain, env);
    let [Request::Store(StoreRequest::Checkpoint { token })] = requests.as_ref() else {
        panic!("one store ask, no early answer: {requests:?}")
    };
    *token
}
fn answered(answer: Result<Checkpointed, CheckpointWhy>) -> Request {
    Request::Party(PartyRequest::Deliver(Delivery::Application {
        to: ReplyTo::new(Token::new(90)),
        outcome: Some(ApplicationAnswer::Checkpoint(answer)),
    }))
}
#[test]
fn an_owners_checkpoint_waits_for_one_store_terminal_and_preserves_its_figures() {
    let (mut domain, env) = signed_in(true);
    let token = pending(&mut domain, &env);
    assert!(settle(&mut domain, &env).is_empty(), "no second store ask");
    let checkpoint = Checkpointed {
        name: b"00000000000000000042-00000000000000ab".as_slice().into(),
        store: 42,
        secrets: 7,
        linked: 5,
        copied: 2,
        bytes: 91,
        paused: 1_234_567,
        took: 9_876_543,
        held: 123,
        removed: 1,
        skipped: 3,
    };
    crate::try_step(
        &mut domain,
        &env,
        Event::Store(StoreInput::Checkpointed { token, checkpoint: checkpoint.clone() }),
    )
    .unwrap();
    assert_eq!(settle(&mut domain, &env).as_ref(), [answered(Ok(checkpoint.clone()))]);
    crate::try_step(
        &mut domain,
        &env,
        Event::Store(StoreInput::CheckpointRefused { token, why: CheckpointWhy::Unavailable }),
    )
    .unwrap();
    assert!(settle(&mut domain, &env).is_empty(), "a duplicate terminal is inert");
    crate::try_step(&mut domain, &env, ask()).unwrap();
    assert_eq!(
        settle(&mut domain, &env).as_ref(),
        [answered(Ok(checkpoint))],
        "a retained key answers without another store ask"
    );
}
#[test]
fn a_store_checkpoint_refusal_reaches_the_owner_unchanged() {
    let (mut domain, env) = signed_in(true);
    let token = pending(&mut domain, &env);
    let why = CheckpointWhy::Failed { path: "generations/8/store/log-1".into(), error: "Io".into() };
    crate::try_step(&mut domain, &env, Event::Store(StoreInput::CheckpointRefused { token, why: why.clone() }))
        .unwrap();
    assert_eq!(settle(&mut domain, &env).as_ref(), [answered(Err(why))]);
}
#[test]
fn a_project_owner_cannot_take_a_deployment_checkpoint() {
    let (mut domain, env) = signed_in(false);
    crate::try_step(&mut domain, &env, ask()).unwrap();
    assert_eq!(settle(&mut domain, &env).as_ref(), [answered(Err(CheckpointWhy::NotOwner))]);
}

#[test]
fn a_tasks_party_cannot_take_a_deployment_checkpoint() {
    let (mut domain, env) = signed_in(false);
    let task = super::projection_settled::make_task(
        &mut domain,
        &env,
        None,
        jig_domain_tasks::Executor::Procedure { connector: 0, code: 1 },
        jig_domain_tasks::Party::Person(1),
    );
    assert_eq!(domain.engine.core().tasks.task(task).unwrap().requester, jig_domain_tasks::Party::Person(1));
    crate::try_step(&mut domain, &env, ask()).unwrap();
    assert_eq!(settle(&mut domain, &env).as_ref(), [answered(Err(CheckpointWhy::NotOwner))]);
}

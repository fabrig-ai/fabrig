//! The accounts child owns sign-in progress and the bounded newest history of its ends.
use jig_accounts_world::{ALICE_PARTY, LIMITS, OWNER, PROVIDER, World, provider, providers};
use jig_domain_accounts::{
    Change, End, Ended, Event, Failure, FromComponent, Holder, Pending, Refusal, Request, Signing,
};
use jig_domain_link::{Form, Scheme};
use skein_lib::{Duration, Token};

// A fresh world's new source takes the first number, its account the next.
const MADE_SOURCE: u32 = 0;
const MADE_ACCOUNT: u32 = 1;
const CHANGE_REQUEST: Token = Token::new(20);

fn started() -> World {
    started_with_plan(false)
}
fn started_with_plan(plan: bool) -> World {
    let mut offered = provider(PROVIDER, true, false);
    offered.signed_in_plan = plan;
    let mut world = World::with_start(jig_accounts_world::LIMITS, providers(vec![offered]), &[]);
    world.step(Event::Want);
    world
}
fn ask(world: &mut World, by: u64, holder: Holder) -> u32 {
    let requests = world.step(Event::SignIn { request: Token::new(by), holder, by, provider: PROVIDER });
    requests
        .into_iter()
        .find_map(|request| {
            let Request::Signing(Signing::SignIn { source, .. }) = request else { return None };
            Some(source)
        })
        .expect("a sign-in started")
}
fn finish(world: &mut World, source: u32) {
    world.step(Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: b"Name".as_slice().into(),
        declined: false,
        said: jig_domain_accounts::Said::default(),
    });
}
fn grant(source: u32) -> Event {
    Event::Component(FromComponent::Granted {
        account: source,
        generation: 1,
        valid: Duration::from_secs(60),
        form: Form::Credential(Scheme::Bearer),
    })
}
fn made() -> (World, u32) {
    let mut world = started();
    let source = ask(&mut world, OWNER, Holder::Party(OWNER));
    finish(&mut world, source);
    (world, source)
}
fn end(by: u64, source: u32, end: End) -> Ended {
    Ended { request: Token::new(by), source, by, end }
}

#[test]
fn a_sign_in_under_way_is_listed_with_who_asked_it_until_it_ends() {
    let mut world = started();
    let source = ask(&mut world, OWNER, Holder::Party(OWNER));
    assert_eq!(
        world.domain.pending().as_slice(),
        &[Pending { source, provider: PROVIDER, holder: Holder::Party(OWNER), by: OWNER }]
    );
    finish(&mut world, source);
    assert!(world.domain.pending().is_empty());
}

#[test]
fn a_deployment_sign_in_is_asked_by_a_person_and_held_by_the_deployment() {
    let mut world = started();
    let source = ask(&mut world, ALICE_PARTY, Holder::Deployment);
    assert_eq!(
        world.domain.pending().as_slice(),
        &[Pending { source, provider: PROVIDER, holder: Holder::Deployment, by: ALICE_PARTY }]
    );
    finish(&mut world, source);
    let ended = world.domain.last_end(ALICE_PARTY).expect("the deployment sign-in ended");
    let End::Made { account, .. } = ended.end else { panic!("made: {ended:?}") };
    assert_eq!(ended.by, ALICE_PARTY);
    assert_eq!(world.domain.account(account).expect("made").holder(), Holder::Deployment);
    assert!(world.domain.awaiting_sign_in(ALICE_PARTY));
    assert!(!world.domain.awaiting_sign_in(OWNER));
}

#[test]
fn a_made_sign_in_awaits_its_first_grant() {
    let (mut world, source) = made();
    assert_eq!(
        world.domain.last_end(OWNER),
        Some(end(OWNER, source, End::Made { account: MADE_ACCOUNT, declined: false }))
    );
    assert!(world.domain.pending().is_empty());
    assert!(world.domain.awaiting_sign_in(OWNER));
    world.step(grant(source));
    assert!(!world.domain.awaiting_sign_in(OWNER));
    assert_eq!(world.domain.source(source).expect("made source").state, jig_domain_accounts::State::Granted);
}

#[test]
fn a_first_component_failure_ends_the_wait() {
    for failure in [Failure::Later { after: None }, Failure::SignInAgain] {
        let (mut world, source) = made();
        world.step(Event::Component(FromComponent::Failed { account: source, failure }));
        assert!(!world.domain.awaiting_sign_in(OWNER), "{failure:?}");
    }
}

#[test]
fn an_expiring_notice_is_not_a_first_answer() {
    let (mut world, source) = made();
    world.step(Event::Component(FromComponent::Expiring { account: source, generation: 1 }));
    assert!(world.domain.awaiting_sign_in(OWNER));
}

#[test]
fn a_renewal_replaces_the_wait_with_the_new_askers() {
    let mut world = started();
    let old = ask(&mut world, OWNER, Holder::Deployment);
    finish(&mut world, old);
    let renewed = ask(&mut world, ALICE_PARTY, Holder::Deployment);
    finish(&mut world, renewed);
    assert_eq!(
        world.domain.last_end(ALICE_PARTY),
        Some(end(ALICE_PARTY, renewed, End::Renewed { account: MADE_ACCOUNT, declined: false }))
    );
    assert!(!world.domain.awaiting_sign_in(OWNER));
    assert!(world.domain.awaiting_sign_in(ALICE_PARTY));
    world.step(grant(old));
    assert!(world.domain.awaiting_sign_in(ALICE_PARTY), "the replaced source's late answer is not the renewal's");
    world.step(grant(renewed));
    assert!(!world.domain.awaiting_sign_in(ALICE_PARTY));
}

#[test]
fn a_source_going_ends_its_wait_without_an_answer() {
    let holder = Holder::Party(OWNER);
    for going in [
        Event::SignOut { request: CHANGE_REQUEST, holder, source: MADE_SOURCE },
        Event::Change { request: CHANGE_REQUEST, holder, change: Change::MoveOut { account: MADE_ACCOUNT } },
        Event::Change { request: CHANGE_REQUEST, holder, change: Change::Remove { account: MADE_ACCOUNT } },
        Event::Close,
    ] {
        let (mut world, _) = made();
        let label = format!("{going:?}");
        world.step(going);
        assert!(!world.domain.awaiting_sign_in(OWNER), "{label}");
    }
}

#[test]
fn an_end_names_who_asked_it() {
    let (mut world, made) = made();
    assert_eq!(
        world.domain.last_end(OWNER),
        Some(end(OWNER, made, End::Made { account: MADE_ACCOUNT, declined: false }))
    );
    let renewed = ask(&mut world, OWNER, Holder::Party(OWNER));
    finish(&mut world, renewed);
    let renewal = end(OWNER, renewed, End::Renewed { account: MADE_ACCOUNT, declined: false });
    assert_eq!(world.domain.last_end(OWNER), Some(renewal));
    let refused = ask(&mut world, ALICE_PARTY, Holder::Party(ALICE_PARTY));
    finish(&mut world, refused);
    assert_eq!(
        world.domain.last_end(ALICE_PARTY),
        Some(end(ALICE_PARTY, refused, End::Refused { refusal: Refusal::Held { provider: PROVIDER } }))
    );
    assert_eq!(world.domain.last_end(OWNER), Some(renewal));
    assert!(!world.domain.awaiting_sign_in(ALICE_PARTY), "the refused sign-in brought no source");
}

#[test]
fn a_full_history_of_ends_drops_its_oldest_and_loses_no_fact() {
    let mut world = started();
    let lost = world.domain.facts_lost();
    // One distinct asker per retained end, then one past the history bound.
    for by in 0..=u64::from(LIMITS.facts) {
        let source = ask(&mut world, by, Holder::Deployment);
        world.step(Event::SignInFailed { source, failure: Failure::Cancelled });
    }
    assert!(!world.domain.awaiting_sign_in(u64::from(LIMITS.facts)), "a failed sign-in brought no source");
    assert_eq!(world.domain.last_end(0), None);
    assert!(world.domain.last_end(u64::from(LIMITS.facts)).is_some());
    assert_eq!(world.ended().len(), skein_lib::length::usize_of(LIMITS.facts));
    assert_eq!(world.domain.facts_lost(), lost);
}

#[test]
fn a_deployment_plan_sign_in_is_refused_without_taking_a_source_number() {
    let mut world = started_with_plan(true);
    assert_eq!(
        world.step(Event::SignIn {
            request: Token::new(OWNER),
            holder: Holder::Deployment,
            by: OWNER,
            provider: PROVIDER
        }),
        vec![Request::Answer {
            request: Token::new(OWNER),
            answer: jig_domain_accounts::Answer::Refused { refusal: Refusal::Provider { provider: PROVIDER } }
        }]
    );
    assert!(world.domain.pending().is_empty());
    // The person's sign-in to that same provider still takes the first source number.
    assert_eq!(ask(&mut world, OWNER, Holder::Party(OWNER)), 0);
}

#[test]
fn a_person_can_start_a_plan_sign_in() {
    let mut world = started_with_plan(true);
    let source = ask(&mut world, OWNER, Holder::Party(OWNER));
    assert_eq!(
        world.domain.pending().as_slice(),
        &[Pending { source, provider: PROVIDER, holder: Holder::Party(OWNER), by: OWNER }]
    );
}

//! Owner asks and deployment projections through the core world's store and peers.
use crate::settings_support::{self as support, OWNER, SEED, ask, sign_in};
use jig_core_world::world::World;
use jig_domain_core::{self as core, documents, reads};
use jig_domain_fleet as fleet;
use jig_domain_link::{Hello, PauseMode, PauseState};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

const CHANNEL: Token = Token::new(7);
fn linked(world: &mut World) {
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: CHANNEL,
        hello: Hello {
            jobs: Box::new([]),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}
fn pause(world: &mut World, signed: u64) -> PauseState {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        read: reads::Read::Deployment,
    }));
    let (_, root::Delivery::Core(core::Held::Read { answer: reads::Answer::Document(document), .. })) =
        world.client_peer.as_mut().expect("client").delivered.pop().expect("deployment reply")
    else {
        panic!("deployment reply")
    };
    let documents::Document::Deployment(document) = *document else { panic!("deployment") };
    document.worker_settings.get(0).expect("configured worker").pause
}

#[test]
fn each_owner_pause_ask_is_carried_and_its_document_follows_the_worker() {
    let mut world = support::start(64);
    let signed = sign_in(&mut world);
    assert_eq!(pause(&mut world, signed), PauseState::Taking);
    linked(&mut world);
    for (key, mode) in [(1, PauseMode::Pause), (2, PauseMode::Drain), (3, PauseMode::Drain)] {
        assert_eq!(
            ask(&mut world, signed, key, people::Ask::PauseWorker { worker: 1, mode }),
            people::Reply::Outcome(people::Outcome::WorkerPauseAsked { worker: 1 })
        );
        assert_eq!(world.worker_pauses.last(), Some(&(CHANNEL, mode)));
        assert_eq!(pause(&mut world, signed), PauseState::Paused);
    }
    assert_eq!(world.worker_pauses.len(), 3, "a repeated ask is carried again");
    for state in [PauseState::Draining, PauseState::Drained, PauseState::Paused] {
        world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Paused { channel: CHANNEL, state })));
        assert_eq!(pause(&mut world, signed), state);
    }
    assert_eq!(
        ask(&mut world, signed, 4, people::Ask::PauseWorker { worker: 1, mode: PauseMode::Resume }),
        people::Reply::Outcome(people::Outcome::WorkerPauseAsked { worker: 1 })
    );
    assert_eq!(world.worker_pauses.last(), Some(&(CHANNEL, PauseMode::Resume)));
    assert_eq!(pause(&mut world, signed), PauseState::Paused);
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Paused {
        channel: CHANNEL,
        state: PauseState::Taking,
    })));
    assert_eq!(pause(&mut world, signed), PauseState::Taking);
}

#[test]
fn unlinked_workers_and_nonowners_are_refused_before_a_pause_is_carried() {
    let mut world = support::start(64);
    let signed = sign_in(&mut world);
    for (key, worker) in [(1, 0), (2, 1), (3, 2)] {
        assert_eq!(
            ask(&mut world, signed, key, people::Ask::PauseWorker { worker, mode: PauseMode::Pause }),
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unlinked { worker }))
        );
    }
    linked(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Member }]),
    })));
    assert_eq!(
        ask(&mut world, signed, 4, people::Ask::PauseWorker { worker: 1, mode: PauseMode::Drain }),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: Some(people::Role::Member) }))
    );
    assert_eq!(world.worker_pauses.len(), 0);
}

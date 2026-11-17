//! Project read through the real core, store and client wire.
use jig_client::{Asker, Event, Key, contract as c};
use jig_client_world::ClientId;
use skein_lib::{Duration, Time, Wall};

// The deleted read crosses the real core, store and client wire, not scripted truth.
#[test]
fn a_deleted_projects_read_reaches_the_client_as_gone() {
    use jig_client_world::joint::{engine::Driver, session::Session};
    use jig_core_kit::peers::Peers;
    use jig_domain_people::{Identity, Role};
    use skein_lib::List;
    const DELETED: u32 = 2;
    let (configuration, limits) = jig_core_kit::fixtures::two_projects(8457);
    let identity = Identity {
        key: configuration.core.owners[0].identity.clone(),
        login: b"owner".as_slice().into(),
        name: b"Owner".as_slice().into(),
    };
    let now = Time::ZERO;
    let wall = Wall::EPOCH;
    let driver = Driver::new(configuration, limits, Peers::new(Vec::new(), Vec::new()), now, wall);
    let receiving = jig_client_world::test::limits();
    let mut session = Session::new(
        driver,
        receiving,
        jig_protocol_transcript::Limits {
            smith: smith_transcript::v7::CEILINGS,
            bytes: 1_048_576,
            turn: c::TurnLimits { messages: 8, blocks: 8, text: 128, outcome: 128 },
        },
    );
    session.bind(ClientId(0), 1, identity, now, wall);
    session.poll(now, wall).expect("owner signed in");
    assert_eq!(session.driver.world.domain.core().people.role(1, DELETED), Some(Role::Owner));
    let mut client = jig_client::Client::unlinked(&receiving.client, 8457).expect("public client");
    exchange(&mut session, &mut client, Event::StartUnlinked { kept: List::with_capacity(0) });
    exchange(&mut session, &mut client, Event::Up { up: jig_client::Up::SignedIn });
    let key = Key { asker: Asker::JIG, number: 1 };
    let expected = c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectRemoved { project: DELETED }));
    {
        let told = exchange(
            &mut session,
            &mut client,
            Event::Send {
                key,
                name: None,
                request: jig_client::Ask::Jig(Box::new(c::Ask::RemoveProject { project: DELETED, grace: None })),
            },
        );
        assert!(told.iter().any(|told| matches!(told,
            jig_client::Told::Answered { key: answered, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) }
            if *answered == key && *answer == expected)));
        session.driver.world.advance(Duration::from_millis(1));
        assert!(
            session.driver.world.domain.core().people.deletions().is_empty(),
            "deletion ended: {:?}",
            session.driver.world.domain.core().people.deletions()
        );
    }
    let handle = jig_client::Handle { asker: Asker::JIG, number: 2 };
    let told = exchange(
        &mut session,
        &mut client,
        Event::Read { handle, read: jig_client::Readable::Jig(c::Read::Project { project: DELETED }) },
    );
    assert!(told.iter().any(|told| matches!(told,
        jig_client::Told::Page { handle: delivered, page: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Gone)) }
        if *delivered == handle)), "gone on the public client: {told:?}");
}

fn exchange(
    session: &mut jig_client_world::joint::session::Session,
    client: &mut jig_client::Client,
    event: Event,
) -> Vec<jig_client::Told> {
    use jig_client::UnlinkedRequest;
    use skein_lib::Queue;
    let (now, wall) = session.driver.world.clocks();
    let mut out = Queue::uncapped();
    client.step_unlinked(now, wall, event, &mut out);
    let mut told = Vec::new();
    for _ in 0..16 {
        // A synchronous public exchange settles within sixteen passes.
        while let Some(request) = out.pop() {
            match request {
                UnlinkedRequest::Told { told: value } => told.push(value),
                UnlinkedRequest::Down { down: jig_client::Down::Save { .. } | jig_client::Down::Erase { .. } } => {}
                request => session
                    .send(ClientId(0), now, wall, jig_client_test_domain::Request::Unlinked { request })
                    .expect("public request"),
            }
        }
        let callbacks = session.poll(now, wall).expect("real engine replies");
        if callbacks.is_empty() {
            return told;
        }
        for (owner, callback) in callbacks {
            session.received(owner, now, wall, &callback);
            let jig_client_test_domain::Event::Client { event } = callback else { panic!("client callback") };
            client.step_unlinked(now, wall, event, &mut out);
        }
    }
    panic!("public exchange did not settle");
}

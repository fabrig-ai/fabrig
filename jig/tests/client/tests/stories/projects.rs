//! Project requests through the application's public client, on the scripted engine.
use jig_client::{Asker, Event, Key, contract as c};
use jig_client_scripted_engine::Change;
use jig_client_test_domain::store::Record;
use jig_client_world::{ClientId, FrontFault, PARTY, PROJECT, Scripted, Test, Until, World, stories};
use skein_lib::{Duration, Time, Wall};
type ProjectWorld = World<Test, Scripted>;
const POLICY: u32 = 1;

fn text(value: &[u8]) -> c::Text {
    c::Text::new(value.into(), 64).expect("project story fixture or delivered fact")
}
fn ready(owner: bool) -> (ProjectWorld, ClientId) {
    let mut world = stories::world(51);
    let mut frame = stories::frame(PARTY);
    frame.deployment_role = if owner { Some(c::HeldRole::Owner) } else { None };
    frame.projects.get_mut(0).expect("project story fixture or delivered fact").role = c::HeldRole::Owner;
    world
        .front()
        .engine
        .act(Time::ZERO, Wall::EPOCH, Change::Me(frame))
        .expect("project story fixture or delivered fact");
    let client = world.client(PARTY, stories::inbox());
    settle(&mut world);
    (world, client)
}
fn settle(world: &mut ProjectWorld) {
    world.run(Until::Time(world.now().saturating_add(Duration::from_millis(80))));
}
fn send(world: &mut ProjectWorld, client: ClientId, number: u64, ask: c::Ask) -> Key {
    let key = Key { asker: Asker::JIG, number };
    world.event(
        client,
        jig_client_test_domain::Event::Client {
            event: Event::Send { name: None, key, request: jig_client::Ask::Jig(Box::new(ask)) },
        },
    );
    key
}
fn make(name: &[u8]) -> c::Ask {
    c::Ask::MakeProject { name: text(name), policy: POLICY }
}
fn answer(world: &mut ProjectWorld) -> c::Answer {
    world
        .front()
        .engine
        .commits()
        .iter()
        .rev()
        .find_map(|commit| commit.answer.clone())
        .expect("project story fixture or delivered fact")
}
fn frame(world: &ProjectWorld, client: ClientId) -> &c::Me {
    let root = world.root(client);
    root.jig().frame(root.client()).document.expect("project story fixture or delivered fact")
}
fn project_made(world: &mut ProjectWorld) -> u32 {
    let c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectMade { project })) = answer(world) else {
        panic!("make answered ProjectMade");
    };
    project
}

#[test]
fn a_deployment_owner_makes_a_project_and_finds_it_in_me() {
    let (mut world, client) = ready(true);
    let key = send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    let project = project_made(&mut world);
    assert!(project > PROJECT);
    let made = frame(&world, client)
        .projects
        .iter()
        .find(|p| p.project == project)
        .expect("project story fixture or delivered fact");
    assert_eq!(made.name.bytes(), b"Tally");
    assert_eq!(made.role, c::HeldRole::Owner);
    assert!(world.root(client).client().pending(key).is_none(), "the client received its answer");
    assert!(!world.front().engine.truth().frames[&8].projects.iter().any(|p| p.project == project));
}

#[test]
fn an_owner_renames_a_project_and_me_says_the_new_name() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, c::Ask::Rename { project: PROJECT, name: text(b"Tally") });
    settle(&mut world);
    assert_eq!(
        answer(&mut world),
        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Renamed { project: PROJECT }))
    );
    assert_eq!(
        frame(&world, client).projects.get(0).expect("project story fixture or delivered fact").name.bytes(),
        b"Tally"
    );
    assert_eq!(
        world.front().engine.truth().frames[&8]
            .projects
            .get(0)
            .expect("project story fixture or delivered fact")
            .name
            .bytes(),
        b"Tally"
    );
}

#[test]
fn a_second_make_of_one_name_is_refused_as_name_taken() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    let project = project_made(&mut world);
    send(&mut world, client, 92, make(b"tally"));
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Refused { refusal: c::Refusal::NameTaken, at: None });
    assert_eq!(frame(&world, client).projects.iter().filter(|p| p.project == project).count(), 1);
}

#[test]
fn a_make_by_a_party_who_owns_no_deployment_is_refused_by_role() {
    let (mut world, client) = ready(false);
    send(&mut world, client, 91, make(b"Tally"));
    settle(&mut world);
    assert_eq!(answer(&mut world), c::Answer::Refused { refusal: c::Refusal::Role { held: None }, at: None });
    assert_eq!(frame(&world, client).projects.len(), 1);
}

#[test]
fn a_name_of_digits_alone_is_refused_naming_its_rule() {
    let (mut world, client) = ready(true);
    send(&mut world, client, 91, make(b"123"));
    settle(&mut world);
    assert_eq!(
        answer(&mut world),
        c::Answer::Refused {
            refusal: c::Refusal::Malformed { at: 0, why: c::Malformed::Name(c::NameRule::Digits) },
            at: None
        }
    );
}

#[test]
fn a_make_retried_under_its_key_after_a_reload_is_answered_once() {
    let (mut world, client) = ready(true);
    world.fault(FrontFault::NoAnswer);
    let key = send(&mut world, client, 91, make(b"Tally"));
    assert!(world.root(client).client().pending(key).is_some());
    world.reload(client);
    settle(&mut world);
    let project = project_made(&mut world);
    assert_eq!(world.front().engine.commits().iter().filter(|commit| commit.key.is_some()).count(), 1);
    assert_eq!(frame(&world, client).projects.iter().filter(|p| p.project == project).count(), 1);
    assert!(world.root(client).client().pending(key).is_none());
}

#[test]
fn an_owner_removes_a_project_and_reads_its_answer_once() {
    let (mut world, client) = ready(true);
    let key = send(&mut world, client, 91, c::Ask::RemoveProject { project: PROJECT, grace: None });
    settle(&mut world);
    assert_eq!(
        answer(&mut world),
        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectRemoved { project: PROJECT }))
    );
    assert!(world.root(client).client().pending(key).is_none());
    assert!(!frame(&world, client).projects.iter().any(|p| p.project == PROJECT));
    settle(&mut world);
    assert_eq!(world.front().engine.commits().iter().filter(|commit| commit.key.is_some()).count(), 1);
}

#[test]
fn a_remove_project_kept_across_a_restart_is_answered_once() {
    let (mut world, client) = ready(true);
    world.fault(FrontFault::NoAnswer);
    let key = send(&mut world, client, 91, c::Ask::RemoveProject { project: PROJECT, grace: None });
    assert!(world.root(client).client().pending(key).is_some());
    assert!(
        world
            .kept(client)
            .iter()
            .any(|record| { matches!(record, Record::Unlinked(record) if record.request.key == key) })
    );
    world.reload(client);
    settle(&mut world);
    assert_eq!(
        answer(&mut world),
        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::ProjectRemoved { project: PROJECT }))
    );
    assert!(world.root(client).client().pending(key).is_none());
    assert!(
        !world
            .kept(client)
            .iter()
            .any(|record| { matches!(record, Record::Unlinked(record) if record.request.key == key) })
    );
    assert!(!frame(&world, client).projects.iter().any(|p| p.project == PROJECT));
    assert_eq!(world.front().engine.commits().iter().filter(|commit| commit.key.is_some()).count(), 1);
}

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

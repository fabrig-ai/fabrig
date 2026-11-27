//! A refused close must refresh adoption once the read table has room
//! (`temper/docs/design/testing/engine-checks.md`, 5.2).
use crate::forge_routes::assigned_chat;
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_forge::api;
use skein_forge_client as client;
use skein_lib::Token;
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{
    forge_routes::{HoldWrite, Until, World},
    protocol,
};
use temper_engine_forge_world::REPO;

const REMOTE: &[u8] = b"org/repo";
const HEAD: &[u8] = b"work";
const STEPS: u32 = 200; // The forge-route world's durable round-trip allowance.

#[test]
#[ignore = "failure worlds: a close refused with no room for a capability read leaves the repository unavailable, repaired as a red"]
fn a_failed_capability_refresh_reads_again() {
    let world = World::configured(true, true);
    let mut limits = world.env.limits.forge.client;
    limits.pending = 2;
    let mut world = world.with_forge_client(limits);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 1, Box::new([])));
    let ask = world.chat(1, b"Close the pull");
    let task = assigned_chat(&mut world, 93, ask).task;
    world.until(Until::Ready);
    let kinds = |world: &World| world.root.forge().repository(REPO).expect("adopted repository").kinds;
    let before = kinds(&world);
    assert!(before.read && before.open && before.branch);

    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: HEAD.into(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    );
    fake::advance(&mut world.fake, &world.fake_env, REMOTE, HEAD, b"file", b"change", 1).expect("head");
    let raw::Answer::Created(number) = world.external(
        1,
        raw::Op::Write(raw::Write::OpenPull {
            opens_empty: false,
            title: b"A pull".as_slice().into(),
            body: Box::new([]),
            head: HEAD.into(),
            base: b"main".as_slice().into(),
        }),
    ) else {
        panic!("pull opened")
    };
    fake::refuse_close(&mut world.fake, REMOTE, number, true);
    world.hold_write_answers = Some(HoldWrite::All);
    world.send(engine::Event::Forge(Box::new(forge::Event::Enqueue {
        entry: client::Entry {
            number: 1000,
            task,
            repository: REPO,
            effect: client::Effect { write: api::Write::Close { number }, condition: client::Condition::None },
            start: None,
            attempt: None,
            failures: 0,
            queued: false,
        },
    })));
    for _ in 0..STEPS {
        world.tick();
        if !world.held_write_answers.is_empty() {
            break;
        }
    }
    assert!(matches!(world.held_write_answers.as_slice(), [engine::Event::Forge(event)] if matches!(event.as_ref(),
        forge::Event::Client(client::Event::Answered { result: Err(api::Error::Protected), .. }))));

    // The refused write still takes its slot as its terminal is handled,
    // and a fresh read takes the other. Do not withhold the refresh itself.
    world.busy_reads = true;
    world.send(engine::Event::Forge(Box::new(forge::Event::Client(client::Event::Read {
        owner: Token::new(1001),
        repository: REPO,
        read: api::Read::Branch { branch: HEAD.into() },
    }))));
    for _ in 0..STEPS {
        world.tick();
        if world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty()) {
            break;
        }
    }
    let client = world.root.forge().inspection_client();
    assert_eq!(client.inspection_calls().len(), 2, "the close and read occupy the two admissions");
    world.forge_reads = Some(Vec::new());
    world.release_write_answers();
    world.tick();
    fake::refuse_close(&mut world.fake, REMOTE, number, false);
    world.busy_reads = false;
    for request in world.held_reads.take().expect("an admitted held read") {
        world.out.push(request);
    }
    world.collect();
    assert!(matches!(world.external(1, raw::Op::Read(raw::Read::Permission { user: 1 })),
        raw::Answer::Permission(permission) if permission.permission == raw::Permission::Admin));
    world.advance_ticks(STEPS);
    assert_eq!(
        kinds(&world),
        before,
        "capability refresh must read again once there is room; reads: {:?}",
        world.forge_reads
    );
}

//! Request reports interleaved with operations and streamed answers (browser.md, 9).
use crate::{referee::Referee, world::World};
use skein_browser::boundary::{Event, Request};
use skein_lib::Token;

pub fn run(seed: u64) {
    let mut world = World::new();
    world.start();
    world.requests_on_operations(seed);
    world.open_with_requests();
    let mut referee = Referee::new();
    let mut sent = 0;
    let mut answered = 0;
    let mut snapshots = 0;
    for step in 0..6 {
        world.ask(Request::Snapshot { page: Token::new(3), op: Token::new(10 + step) });
        if seed.wrapping_add(step) & 2 == 0 {
            world.release_delayed();
        }
        observe(&mut world, &mut referee, &mut sent, &mut answered, &mut snapshots);
    }
    world.ask(Request::Close { entity: Token::new(3) });
    // Late first chunks, later chunks and ends meet an already closed page.
    world.release_delayed();
    observe(&mut world, &mut referee, &mut sent, &mut answered, &mut snapshots);
    assert_eq!(sent, 7, "navigation and six requests, seed {seed}");
    assert!(answered > 0 && answered <= sent, "first answers observed, seed {seed}");
    assert_eq!(snapshots, 6, "operations settle among request reports, seed {seed}");
    world.ask(Request::Close { entity: Token::new(1) });
    observe(&mut world, &mut referee, &mut sent, &mut answered, &mut snapshots);
    world.check_commands_settled().expect("commands settle");
}

fn observe(world: &mut World, referee: &mut Referee, sent: &mut u32, answered: &mut u32, snapshots: &mut u32) {
    while let Some(event) = world.above.pop() {
        referee.event(&event, 1).expect("request contract");
        match event {
            Event::Sent { .. } => *sent += 1,
            Event::Answered { .. } => *answered += 1,
            Event::Snapshot { .. } => *snapshots += 1,
            Event::Opened { .. } | Event::Closed { .. } => {}
            event => panic!("unexpected scenario event: {event:?}"),
        }
    }
}

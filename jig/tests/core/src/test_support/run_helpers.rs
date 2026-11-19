//! Worker greetings and transcript-tail reads shared by memory and owner-set stories (jig/docs/design/domain/testing.md, 9).
use crate::world::World;
use jig_domain_core as core;
use jig_domain_link::{Hello, Hosting};
use jig_test_domain as root;
use skein_lib::{Duration, Token};

pub fn hello(world: &mut World) {
    hello_hosting(world, Box::new([]));
}

pub fn hello_hosting(world: &mut World, hosting: Box<[Hosting]>) {
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: Hello {
            jobs: Box::default(),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 1,
            workstreams: Box::new([]),
            hosting,
        },
    })));
}

#[must_use]
pub fn tail(world: &World) -> Option<&[u8]> {
    world.assigned.as_ref().expect("chat assignment").brief.iter().find_map(|section| {
        if let jig_domain_brief::GatherPlaced::Core { kind: jig_domain_brief::Core::TranscriptTail, text, .. } = section
        {
            Some(text.as_ref())
        } else {
            None
        }
    })
}

pub const SEED: u64 = 118;

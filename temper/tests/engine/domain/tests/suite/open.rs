//! Open problems (temper/docs/design/draft/failure-worlds.md, 6–7;
//! temper/docs/design/testing/engine-checks.md, 5.1).
use jig_domain_host as host;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_fake_forge_domain::{self as fake, api as raw};
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_world::forge_routes::{StoryLimits, World};
use temper_engine_forge_world::translate;

use crate::forge_routes::landing::{landing_rows, task_row};
use crate::forge_routes::light_rules::change_row;

mod p113;
mod p120;
mod p53;
mod p58;
mod p63;
mod p85;
mod s7;

fn person(world: &mut World, key: u8, ask: people::Ask) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [key; 16],
        ask,
    }));
}

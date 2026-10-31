//! Shared construction and terminal inspection for focused and fuzzy run stories.

use smith_domain_run::Answer;
use smith_run_world::{Settings, World};

pub(super) fn settled(settings: &Settings) -> World {
    let mut world = World::new(*settings);
    world.run(1_000_000);
    world
}

pub(super) fn answers(world: &World) -> Vec<&Answer> {
    world.answers().map(|answer| answer.expect("every start is answered")).collect()
}

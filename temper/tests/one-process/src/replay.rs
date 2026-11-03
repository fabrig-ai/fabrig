//! The kept visit under the forge referee (temper/docs/design/playground.md, 9;
//! skein/docs/design/playground.md, 6.2).

use crate::{referee::forge::Forge, world::Story};
use skein_lib::Time;
use skein_sim_world::Turns;
use temper_playground::{programs, programs::Process, stories, story};

/// The one-process world's forge referee over the playground's composition, observing every turn.
pub struct Refereed {
    forge: Forge,
}

impl Turns<Process<()>> for Refereed {
    fn turned(&mut self, _now: Time, processes: &[Process<()>]) {
        self.forge.observe(processes);
    }
}

/// The log's digests and the forge referee's verdict at its end, without teardown.
#[derive(Debug)]
pub enum Replayed {
    Complete,
    Incomplete { why: String },
    Differs(skein_playground::replay::Differs),
}

/// Replays the log's text over the story at its seed and start, with the visitor's reach and
/// factory parameters installed; the forge referee judges its end. The log's digests check the
/// visitor's answers; the terminal person's reply count and kept links are outside this replay.
#[must_use]
pub fn replay(story: &stories::Story, log: &[u8]) -> Replayed {
    let read = match skein_ui_web::log::read(log) {
        Ok(read) => read,
        Err(why) => return Replayed::Incomplete { why: format!("session log refused: {why:?}") },
    };
    let start = story.start(read.seed, read.start);
    let mut turns = Refereed { forge: Forge::new(start.fakes.forge.clone(), Story::PlaygroundA) };
    let world = temper_playground::world(start);
    let parameters = story::parameters(&world);
    match programs::with(parameters, || {
        skein_playground::replay::replay(world, story::reach(), &read, &mut turns)
    }) {
        Err(differs) => Replayed::Differs(differs),
        Ok(_world) => {
            if turns.forge.forge_complete() {
                Replayed::Complete
            } else {
                Replayed::Incomplete {
                    why: "main must hold the scripted CSV edit at an observed pushed head with green CI".into(),
                }
            }
        }
    }
}

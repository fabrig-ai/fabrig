//! The kept visit under the forge referee (temper/docs/design/playground.md, 9;
//! skein/docs/design/playground.md, 6.2).

use crate::{referee::forge::Forge, world::Story};
use skein_lib::Time;
use skein_sim_world::Turns;
use temper_playground::{programs, programs::Process, stories, story};

/// The one-process world's forge referee over the playground's composition, observing every turn.
pub struct Refereed {
    forge: Forge,
    finding: Option<String>,
}

impl Turns<Process<()>> for Refereed {
    fn turned(&mut self, _now: Time, processes: &[Process<()>]) {
        if self.finding.is_none() {
            self.finding = self.forge.observe(processes).err();
        }
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
#[expect(clippy::result_large_err, reason = "the replay wrapper preserves the complete mismatch diagnostic")]
pub fn replay(story: &stories::Story, log: &[u8]) -> Replayed {
    let read = match skein_ui_web::log::read(log) {
        Ok(read) => read,
        Err(why) => return Replayed::Incomplete { why: format!("session log refused: {why:?}") },
    };
    let start = story.start(read.seed, read.start);
    let mut turns = Refereed { finding: None, forge: Forge::new(start.fakes.forge.clone(), Story::PlaygroundA) };
    let world = temper_playground::world(start);
    let parameters = story::parameters(&world);
    match programs::with(parameters, || skein_playground::replay::replay(world, story::reach(), &read, &mut turns)) {
        Err(differs) => Replayed::Differs(differs),
        Ok(world) => {
            eprintln!("refereed replay: world {} ns, {} world steps", world.now().as_nanos(), world.steps());
            if let Some(why) = turns.finding {
                Replayed::Incomplete { why }
            } else if turns.forge.forge_complete() {
                Replayed::Complete
            } else {
                Replayed::Incomplete {
                    why: "main must hold the scripted CSV edit at an observed pushed head or its merge commit, with green CI at main".into(),
                }
            }
        }
    }
}

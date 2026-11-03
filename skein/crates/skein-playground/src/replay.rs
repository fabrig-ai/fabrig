//! A session log replayed natively (skein/docs/design/playground.md, 6.2).

use crate::slices::Reach;
use skein_shell::Host;
use skein_sim_world::{Machine, Meter, Turns, World};
use skein_ui_web::{bridge, bridge::Stamp, fetch, log};

/// Where a replay first differed from its log, naming the build the log came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Differs {
    /// The log's line, from 1: the input's, or the last line's.
    pub line: u32,
    pub build: Option<Box<[u8]>>,
    pub why: Why,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// The world reached the stamp's step at another instant, or the instant at another step.
    Moment { expected: Stamp, found: Stamp },
    /// What the world said differs from the log's digest.
    Said,
}

/// Replays `log` over `world`, built by the application from the log's seed and start: each
/// input enters at its stamp and each digest is checked; it runs to the log's end and hands
/// the world back unsettled, with no teardown of its own to check.
pub fn replay<P: Host, M: Machine, H: Meter, T: Turns<P>>(
    mut world: World<P, M, H>,
    reach: Reach<P>,
    log: &log::Read,
    turns: &mut T,
) -> Result<World<P, M, H>, Differs> {
    let mut fold = Fold { saying: log::Saying::new(), reports: Vec::new(), stopped: false };
    let mut line = 2_u32;
    for entry in &log.entries {
        fold.until(&mut world, reach, entry.stamp, entry.said, turns).map_err(|why| Differs {
            line,
            build: log.build.clone(),
            why,
        })?;
        (reach.enter)(world.process(reach.at), entry.input.clone());
        fold.saying = log::Saying::new();
        line = line.checked_add(1).expect("the log's line count fits u32");
    }
    match log.end {
        log::End::At { step, instant, said } => {
            fold.until(&mut world, reach, Stamp { step, instant }, said, turns).map_err(|why| Differs {
                line,
                build: log.build.clone(),
                why,
            })?;
        }
        log::End::Cut => {}
    }
    Ok(world)
}

struct Fold {
    saying: log::Saying,
    reports: Vec<fetch::Report>,
    stopped: bool,
}

impl Fold {
    fn until<P: Host, M: Machine, H: Meter, T: Turns<P>>(
        &mut self,
        world: &mut World<P, M, H>,
        reach: Reach<P>,
        expected: Stamp,
        said: log::Said,
        turns: &mut T,
    ) -> Result<(), Why> {
        loop {
            let found = Stamp { step: world.steps(), instant: world.now() };
            if found == expected {
                return if self.saying.said() == said { Ok(()) } else { Err(Why::Said) };
            }
            if found.step > expected.step || found.instant > expected.instant || world.settled() {
                return Err(Why::Moment { expected, found });
            }
            if world.busy() {
                if found.step == expected.step {
                    return Err(Why::Moment { expected, found });
                }
                world.step_with(turns);
                self.take(world, reach);
            } else {
                let next = world.next_due().map_or(expected.instant, |due| due.min(expected.instant));
                if next == found.instant {
                    return Err(Why::Moment { expected, found });
                }
                // Like the slices: move the idle clock alone, then step only while busy.
                world.advance(next);
            }
        }
    }

    fn take<P: Host, M: Machine, H: Meter>(&mut self, world: &mut World<P, M, H>, reach: Reach<P>) {
        (reach.take)(world.process(reach.at), &mut self.reports);
        for report in self.reports.drain(..) {
            self.saying.report(&report);
        }
        for line in world.printed() {
            let stream = match line.stream {
                skein_sim_world::Stream::Output => bridge::Stream::Output,
                skein_sim_world::Stream::Error => bridge::Stream::Error,
            };
            self.saying.printed(&bridge::Printed { process: line.process, stream, line: line.line });
        }
        if world.settled() && !self.stopped {
            self.saying.stopped();
            self.stopped = true;
        }
    }
}

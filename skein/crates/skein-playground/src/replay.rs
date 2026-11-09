//! A session log replayed natively (skein/docs/design/playground.md, 6.2).

use crate::slices::Reach;
use alloc::collections::VecDeque;
use skein_shell::Host;
use skein_sim_world::{Machine, Meter, Turns, World};
use skein_ui_web::{bridge, bridge::Stamp, fetch, log};

/// Where a replay first differed from its log, and what differed (skein/docs/design/playground.md, 6.2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Differs {
    /// The log's line, from 1: the input's, or the last line's.
    pub line: u32,
    pub build: Option<Box<[u8]>>,
    /// The line's input in the log's text (`log::input_text`); none on the last line.
    pub input: Option<String>,
    pub why: Why,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// The world reached the stamp's step at another instant, or the instant at another step.
    Moment { expected: Stamp, found: Stamp },
    /// What the world said differs from the line's digest: the log's, the replay's, and
    /// the replay's last lines before it, oldest first.
    Said { expected: log::Said, found: log::Said, last: Vec<String> },
}

/// The most lines a `Why::Said` keeps.
pub const LAST_LINES: usize = 8;
/// The most bytes of each kept line, before a cut gains `...`.
pub const LAST_LINE_BYTES: usize = 200;

/// Replays `log` over `world`, built by the application from the log's seed and start: each
/// input enters at its stamp and each digest is checked; it runs to the log's end and hands
/// the world back unsettled, with no teardown of its own to check.
#[expect(clippy::result_large_err, reason = "the replay contract carries the input, both digests and last lines")]
pub fn replay<P: Host, M: Machine, H: Meter, T: Turns<P>>(
    mut world: World<P, M, H>,
    reach: Reach<P>,
    log: &log::Read,
    turns: &mut T,
) -> Result<World<P, M, H>, Differs> {
    let mut fold = Fold { saying: log::Saying::new(), last: VecDeque::new(), reports: Vec::new(), stopped: false };
    let mut line = 2_u32;
    for entry in &log.entries {
        fold.until(&mut world, reach, entry.stamp, entry.said, turns).map_err(|why| Differs {
            line,
            build: log.build.clone(),
            input: Some(log::input_text(&entry.input)),
            why,
        })?;
        (reach.enter)(world.process(reach.at), entry.input.clone());
        keep_line(
            &mut fold.last,
            format!(
                "input step={} instant={} {}",
                entry.stamp.step,
                entry.stamp.instant.as_nanos(),
                log::input_text(&entry.input)
            ),
        );
        fold.saying = log::Saying::new();
        line = line.checked_add(1).expect("the log's line count fits u32");
    }
    match log.end {
        log::End::At { step, instant, said } => {
            fold.until(&mut world, reach, Stamp { step, instant }, said, turns).map_err(|why| Differs {
                line,
                build: log.build.clone(),
                input: None,
                why,
            })?;
        }
        log::End::Cut => {}
    }
    Ok(world)
}

struct Fold {
    saying: log::Saying,
    last: VecDeque<String>,
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
                return if self.saying.said() == said {
                    Ok(())
                } else {
                    Err(Why::Said {
                        expected: said,
                        found: self.saying.said(),
                        last: self.last.iter().cloned().collect(),
                    })
                };
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
            keep_line(&mut self.last, log::report_line(&report).trim_end_matches('\n').into());
        }
        for line in world.printed() {
            let stream = match line.stream {
                skein_sim_world::Stream::Output => bridge::Stream::Output,
                skein_sim_world::Stream::Error => bridge::Stream::Error,
            };
            let printed = bridge::Printed { process: line.process, stream, line: line.line };
            self.saying.printed(&printed);
            keep_line(&mut self.last, log::printed_line(&printed).trim_end_matches('\n').into());
        }
        if world.settled() && !self.stopped {
            self.saying.stopped();
            keep_line(&mut self.last, "stopped".into());
            self.stopped = true;
        }
    }
}

fn keep_line(last: &mut VecDeque<String>, mut line: String) {
    if line.len() > LAST_LINE_BYTES {
        line.truncate(LAST_LINE_BYTES);
        line.push_str("...");
    }
    if last.len() == LAST_LINES {
        last.pop_front();
    }
    last.push_back(line);
}

#[cfg(test)]
mod tests {
    use super::{LAST_LINE_BYTES, LAST_LINES, keep_line};
    use alloc::collections::VecDeque;

    #[test]
    fn last_lines_are_cut_and_drop_the_oldest() {
        let mut last = VecDeque::new();
        let long = "abc".repeat(100);
        keep_line(&mut last, long.clone());
        assert_eq!(last.back(), Some(&format!("{}...", &long[..LAST_LINE_BYTES])));
        let exact = "x".repeat(LAST_LINE_BYTES);
        keep_line(&mut last, exact.clone());
        assert_eq!(last.back(), Some(&exact));
        for number in 2..=LAST_LINES {
            keep_line(&mut last, number.to_string());
        }
        let mut expected = VecDeque::from([exact]);
        expected.extend((2..=LAST_LINES).map(|number| number.to_string()));
        assert_eq!(last, expected, "a ninth line drops the first, keeping oldest first");
    }
}

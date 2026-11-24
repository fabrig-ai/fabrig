//! The rig's close window (skein/docs/design/testing/browser.md, 9).

use super::{PROCESS, Rig, serve};
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Time};
use skein_shell::Wait;
use skein_world::Deadline;

pub(super) const CLOSE_WINDOW: Duration = Duration::from_secs(4);

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Progress {
    Drained,
    Drive,
    Cutoff,
}

pub(super) fn progress(now: Time, until: Time, drained: bool) -> Progress {
    if drained {
        Progress::Drained
    } else if now >= until {
        Progress::Cutoff
    } else {
        Progress::Drive
    }
}

impl Rig {
    pub(super) fn close_cutoff(&mut self) -> Time {
        *self.close_until.get_or_insert_with(|| Deadline::arm("Chromium cleanup", CLOSE_WINDOW).until())
    }

    // browser.md, 9: abort the child group and pipes before waiting for their terminal events.
    pub(super) fn end(&mut self) -> Result<(), String> {
        let until = self.close_cutoff();
        self.io_requests.retain(|request| matches!(request, io::Request::Spawn { .. }));
        while self.browser_down.pop().is_some() {}
        self.pages.stop();
        for entity in self.child.into_iter().chain(self.pipes.into_iter().flatten()) {
            self.io_requests.push_back(io::Request::Abort { entity });
        }
        self.io_down();
        self.kernel.submit(&mut self.submits, Wait::No);
        let mut failure = None;
        loop {
            self.reap(None);
            let now = self.clock.now();
            self.io_env.now = now.now;
            self.io_env.wall = now.wall;
            self.pages.update_time(now.now, now.wall);
            io::stage::up(
                &mut self.io,
                &self.io_env,
                &mut self.complete,
                &mut self.io_events,
                &mut self.submits,
                now.now,
            );
            while let Some(event) = self.io_events.pop() {
                if let Some(report) = self.cleanup_event(event) {
                    failure.get_or_insert(report);
                }
            }
            self.io_down();
            match progress(self.clock.now().now, until, self.drained()) {
                Progress::Drained => return failure.map_or(Ok(()), Err),
                Progress::Drive => self.enter(until, false),
                Progress::Cutoff => {
                    let outstanding = self.outstanding();
                    // Submit any aborts made by a late spawn, without waiting past the window.
                    self.kernel.submit(&mut self.submits, Wait::No);
                    return Err(match failure {
                        Some(report) => format!("{report}; four-second cutoff: {outstanding}"),
                        None => format!("four-second cutoff: {outstanding}"),
                    });
                }
            }
        }
    }

    fn cleanup_event(&mut self, event: io::Event) -> Option<String> {
        match event {
            io::Event::Spawned { owner: PROCESS, child, pipes } => {
                self.spawned(child, &pipes);
                for entity in core::iter::once(child).chain(pipes.iter().copied()) {
                    self.io_requests.push_back(io::Request::Abort { entity });
                }
            }
            io::Event::Exited { owner: PROCESS, exit } => {
                self.exited = true;
                // This path aborts the group with SIGKILL. Other bad exits are failures.
                if !matches!(exit, kernel::Exit::Code(0) | kernel::Exit::Signal(9)) {
                    return Some(format!("Chromium exited with {exit:?}"));
                }
            }
            io::Event::Failed { owner, error } => {
                if owner == PROCESS {
                    self.submit_root_close();
                }
                return Some(format!("io failed for {owner:?}: {error:?}"));
            }
            io::Event::Listening { owner: serve::OWNER, .. } | io::Event::Accepted { owner: serve::OWNER, .. } => {
                self.pages.event(event);
            }
            io::Event::Stream { owner, .. } | io::Event::Closed { owner } if self.pages.owns(owner) => {
                self.pages.event(event);
            }
            io::Event::Closed { owner } => self.note_closed(owner),
            _ => {}
        }
        None
    }

    pub(super) fn io_down(&mut self) {
        while self.io.takes() {
            let Some(request) = self.io_requests.pop_front().or_else(|| self.pages.take()) else { break };
            io::down(&mut self.io, &self.io_env, request, &mut self.submits);
        }
        self.io.reclaim();
    }

    pub(super) fn enter(&mut self, until: Time, browser_pending: bool) {
        let pending = browser_pending
            || self.io.is_ready()
            || !self.complete.is_empty()
            || !self.io_events.is_empty()
            || !self.io_requests.is_empty()
            || self.pages.pending();
        let wait =
            if pending { Wait::No } else { Wait::Until(self.io.next_deadline().map_or(until, |at| at.min(until))) };
        self.kernel.submit(&mut self.submits, wait);
    }

    pub(super) fn drained(&self) -> bool {
        self.root_closed
            && self.child_closed
            && self.pages.is_closed()
            && self.io.is_empty()
            && self.kernel.in_flight() == 0
            && self.submits.is_empty()
            && self.complete.is_empty()
            && self.io_requests.is_empty()
    }

    pub(super) fn outstanding(&self) -> String {
        let pipes = self
            .pipes
            .as_ref()
            .into_iter()
            .flatten()
            .zip(self.pipes_closed)
            .filter_map(|(pipe, closed)| (!closed).then_some(pipe));
        let pipes: Vec<_> = pipes.collect();
        format!(
            "stage {:?}; root closed: {}; child {:?} (exited: {}, closed: {}); unclosed pipes: {pipes:?}; origin closed: {}; records in flight: {}, queued: {} submissions, {} completions, {} requests",
            self.stage,
            self.root_closed,
            self.child,
            self.exited,
            self.child_closed,
            self.pages.is_closed(),
            self.kernel.in_flight(),
            self.submits.len(),
            self.complete.len(),
            self.io_requests.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{CLOSE_WINDOW, Progress, progress};
    use skein_lib::{Duration, Time};

    #[test]
    fn cleanup_must_drain_before_it_can_release_a_verdict() {
        let start = Time::ZERO.saturating_add(Duration::from_secs(1));
        let until = start.saturating_add(CLOSE_WINDOW);
        let just_before = start.saturating_add(Duration::from_nanos(3_999_999_999));
        let four_seconds = start.saturating_add(Duration::from_secs(4));
        assert_eq!(progress(just_before, until, false), Progress::Drive);
        assert_eq!(progress(just_before, until, true), Progress::Drained);
        assert_eq!(progress(four_seconds, until, false), Progress::Cutoff);
        assert_eq!(progress(four_seconds, until, true), Progress::Drained);
    }
}

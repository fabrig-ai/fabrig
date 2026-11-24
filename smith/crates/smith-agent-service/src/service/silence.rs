//! One no-progress deadline for the framed agent's waits on its host.
//! The inline effects have no host link to supervise. Contract:
//! smith/docs/design/domain/host.md, section 4.

use skein_lib::{Duration, Time};
use smith_agent_profile::Link;

use super::{Failure, Frame, Stages, orphan};

#[derive(Debug)]
pub(super) struct Silence {
    interval: Option<Duration>,
    deadline: Option<Time>,
}

impl Silence {
    pub(super) const fn new(link: Link) -> Self {
        let interval = match link {
            Link::Policy { .. } => None,
            Link::Derived { no_progress, .. } => Some(no_progress),
        };
        Self { interval, deadline: None }
    }

    fn observe(&mut self, now: Time, waiting: bool, progress: bool) {
        self.deadline = match self.interval {
            Some(interval) if waiting => {
                if progress || self.deadline.is_none() {
                    Some(now.saturating_add(interval))
                } else {
                    self.deadline
                }
            }
            Some(_) | None => None,
        };
    }
}

pub(super) fn deadline(frame: Option<&Frame>) -> Option<Time> {
    frame?.silence.deadline
}

pub(super) fn pipe_output(service: &Stages, frame: &Frame) -> u32 {
    match service.output {
        Some(output) => frame.io.pending_pipe_output(output),
        None => 0,
    }
}

pub(super) fn refresh(service: &Stages, frame: &mut Frame, wrote_frame: bool) {
    let channel_progress = frame.channel.take_progress();
    // Drained hands the frame to IO, where the last frame can still be
    // blocked in a kernel write with no further channel reservation pending.
    let waiting = !service.host.gone()
        && (frame.channel.waits_for_host() || pipe_output(service, frame) > 0 || frame.domain.waits_for_host_credit());
    frame.silence.observe(service.io_env.now, waiting, channel_progress || wrote_frame);
}

pub(super) fn judge(service: &mut Stages, frame: &mut Frame, wrote_frame: bool) {
    // Completions and records at the deadline count before we judge silence.
    refresh(service, frame, wrote_frame);
    if let Some(at) = frame.silence.deadline
        && at <= service.io_env.now
    {
        orphan::host_lost(service, Failure::HostSilent);
        if service.channel_events.room() > 0 {
            orphan::abandon(service, frame);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steady_progress_restarts_the_clock_until_the_wait_ends() {
        let gap = Duration::from_millis(6);
        let interval = Duration::from_millis(10);
        let mut clock = Silence::new(Link::Derived {
            no_progress: interval,
            cancel_grace: Duration::ZERO,
            orphan_exit: Duration::ZERO,
        });
        let mut now = Time::ZERO;
        clock.observe(now, true, false);
        assert_eq!(clock.deadline, Some(now.saturating_add(interval)));
        for _ in 0_u8..4_u8 {
            now = now.saturating_add(gap);
            assert!(Some(now) < clock.deadline, "each frame is taken before the deadline");
            clock.observe(now, true, true);
            assert_eq!(clock.deadline, Some(now.saturating_add(interval)));
        }
        let deadline = clock.deadline;
        now = now.saturating_add(gap);
        clock.observe(now, true, false);
        assert_eq!(clock.deadline, deadline, "more queued work is not progress");
        clock.observe(now, false, false);
        assert_eq!(clock.deadline, None, "the host no longer owes anything");
        now = now.saturating_add(interval);
        clock.observe(now, true, false);
        assert_eq!(clock.deadline, Some(now.saturating_add(interval)), "a new wait gets a full interval");
    }
}

//! Progress, cancellation and tree-signal deadlines (domain/host.md, section 4).
//! Watching owns the progress and wall clocks; shutdown states own only their
//! grace and any untold cause. `domain::follow` derives alarms and demands after
//! each entrance applies these transitions.
use crate::Fault;
use core::mem;
use skein_lib::{Duration, Time};

#[derive(Clone, Copy, Debug)]
pub(super) struct Watch {
    pub(super) progress: Time,
    pub(super) wall: Time,
    pub(super) paused: bool,
}

#[derive(Debug)]
pub(super) enum State {
    Spawning,
    Watching(Watch),
    Cancelled { until: Time, owed: Option<Fault> },
    Draining { until: Time, owed: Option<Fault> },
    Exiting { until: Time },
    Terminating { until: Time },
    Killing,
    Closed,
}

impl State {
    pub(super) fn watch(&self) -> Option<Watch> {
        match self {
            Self::Watching(watch) => Some(*watch),
            Self::Spawning
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed => None,
        }
    }

    pub(super) fn spawning(&self) -> bool {
        match self {
            Self::Spawning => true,
            Self::Watching(_)
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed => false,
        }
    }

    pub(super) fn closed(&self) -> bool {
        match self {
            Self::Closed => true,
            Self::Spawning
            | Self::Watching(_)
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing => false,
        }
    }

    pub(super) fn forced(&self) -> bool {
        match self {
            Self::Terminating { .. } | Self::Killing => true,
            Self::Spawning
            | Self::Watching(_)
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Closed => false,
        }
    }

    pub(super) fn fault(&self) -> Option<Fault> {
        match self {
            Self::Cancelled { owed, .. } | Self::Draining { owed, .. } => *owed,
            Self::Spawning
            | Self::Watching(_)
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed => None,
        }
    }

    pub(super) fn deadline(&self) -> Option<Time> {
        match self {
            Self::Cancelled { until, .. }
            | Self::Draining { until, .. }
            | Self::Exiting { until }
            | Self::Terminating { until } => Some(*until),
            Self::Spawning | Self::Watching(_) | Self::Killing | Self::Closed => None,
        }
    }

    pub(super) fn start(&mut self, now: Time, wall_time: Duration) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Spawning => {
                Self::Watching(Watch { progress: now, wall: now.saturating_add(wall_time), paused: false })
            }
            Self::Watching(_)
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed => unreachable!("one process spawn terminal"),
        };
    }

    pub(super) fn progress(&mut self, now: Time) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Watching(mut watch) => {
                watch.progress = now;
                Self::Watching(watch)
            }
            other @ (Self::Spawning
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed) => other,
        };
    }

    pub(super) fn pause(&mut self, paused: bool, now: Time) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Watching(mut watch) => {
                if watch.paused && !paused {
                    watch.progress = now;
                }
                watch.paused = paused;
                Self::Watching(watch)
            }
            other @ (Self::Spawning
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed) => other,
        };
    }

    pub(super) fn clear_fault(&mut self) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Cancelled { until, .. } => Self::Cancelled { until, owed: None },
            Self::Draining { until, .. } => Self::Draining { until, owed: None },
            other @ (Self::Spawning
            | Self::Watching(_)
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed) => other,
        };
    }

    pub(super) fn take_fault(&mut self) -> Option<Fault> {
        let fault = self.fault();
        self.clear_fault();
        fault
    }

    pub(super) fn cancel(&mut self, now: Time, grace: Duration, owed: Option<Fault>) -> bool {
        let cancel = self.watch().is_some();
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Watching(_) => Self::Cancelled { until: now.saturating_add(grace), owed },
            other @ (Self::Spawning
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed) => other,
        };
        cancel
    }

    pub(super) fn drain(&mut self, now: Time, grace: Duration) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Watching(_) => Self::Draining { until: now.saturating_add(grace), owed: Some(Fault::Exited) },
            Self::Cancelled { until, owed } => Self::Draining { until, owed },
            other @ (Self::Spawning
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed) => other,
        };
    }

    pub(super) fn answer(&mut self, now: Time, grace: Duration) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Watching(_) | Self::Cancelled { .. } | Self::Draining { .. } => {
                Self::Exiting { until: now.saturating_add(grace) }
            }
            other @ (Self::Exiting { .. } | Self::Terminating { .. } | Self::Killing) => other,
            Self::Spawning | Self::Closed => unreachable!("answer after process start"),
        };
    }

    pub(super) fn hangup(&mut self) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Cancelled { until, .. } | Self::Draining { until, .. } => Self::Exiting { until },
            Self::Spawning
            | Self::Watching(_)
            | Self::Exiting { .. }
            | Self::Terminating { .. }
            | Self::Killing
            | Self::Closed => unreachable!("untold shutdown retains its original deadline"),
        };
    }

    pub(super) fn terminate(&mut self, now: Time, grace: Duration) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Watching(_) | Self::Cancelled { .. } | Self::Draining { .. } | Self::Exiting { .. } => {
                Self::Terminating { until: now.saturating_add(grace) }
            }
            Self::Spawning | Self::Terminating { .. } | Self::Killing | Self::Closed => {
                unreachable!("one polite tree signal")
            }
        };
    }

    pub(super) fn kill(&mut self) {
        let state = mem::replace(self, Self::Closed);
        *self = match state {
            Self::Terminating { .. } => Self::Killing,
            Self::Spawning
            | Self::Watching(_)
            | Self::Cancelled { .. }
            | Self::Draining { .. }
            | Self::Exiting { .. }
            | Self::Killing
            | Self::Closed => unreachable!("kill follows terminate grace"),
        };
    }
}

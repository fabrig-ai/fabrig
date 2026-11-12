//! Child timers and deployment headers (jig/docs/design/domain/engine/README.md, 4.1 and jig/docs/design/domain/engine/restart.md).
use crate::{Core, Record, Timer};
use skein_lib::{Duration, Time, Wall};

impl Core {
    /// Earliest child alarm at the supplied clocks. Paused stopping timers wait.
    #[must_use]
    pub fn next_deadline(&self, now: Time, wall: Wall) -> Option<Time> {
        self.deadline(now, wall, true)
    }

    /// First due child in people, tasks, fleet, brief and account order.
    #[must_use]
    pub fn due(&self, now: Time, wall: Wall) -> Option<Timer> {
        self.due_timer(now, wall, true)
    }

    /// Earliest alarm that advances admitted work while new inputs are held.
    #[must_use]
    pub fn next_finishing_deadline(&self, now: Time, wall: Wall) -> Option<Time> {
        self.deadline(now, wall, false)
    }

    /// First due timer that advances admitted work while new inputs are held.
    #[must_use]
    pub fn finishing_due(&self, now: Time, wall: Wall) -> Option<Timer> {
        self.due_timer(now, wall, false)
    }

    fn deadlines(&self, now: Time, wall: Wall) -> [(Timer, Option<Time>); 5] {
        let saved = match self.people.next_wall_expiry() {
            Some(expiry) if self.people.ready() => {
                Some(now.saturating_add(Duration::from_nanos(expiry.as_nanos().saturating_sub(wall.as_nanos()))))
            }
            Some(_) | None => None,
        };
        [
            (Timer::People, earliest(self.people.next_deadline(), saved)),
            (Timer::Tasks, self.tasks.next_deadline()),
            (Timer::Fleet, self.fleet.next_deadline()),
            (Timer::Brief, self.brief.next_deadline()),
            (Timer::Account, self.accounts.next_deadline()),
        ]
    }

    fn deadline(&self, now: Time, wall: Wall, new_work: bool) -> Option<Time> {
        let mut next = None;
        for (timer, deadline) in self.deadlines(now, wall) {
            if self.timer_runs(timer, new_work) {
                next = earliest(next, deadline);
            }
        }
        next
    }

    fn due_timer(&self, now: Time, wall: Wall, new_work: bool) -> Option<Timer> {
        for (timer, deadline) in self.deadlines(now, wall) {
            if self.timer_runs(timer, new_work)
                && let Some(deadline) = deadline
                && deadline <= now
            {
                return Some(timer);
            }
        }
        None
    }

    fn timer_runs(&self, timer: Timer, new_work: bool) -> bool {
        !(self.stopping && timer.waits_while_stopping())
            && (self.start_opened() || !timer.new_work())
            && (new_work || !timer.new_work())
    }

    /// The deployment row this decision writes, if any (domain/engine/README.md, 5.4).
    #[must_use]
    pub fn decision_header(&mut self, wrote: bool) -> Option<Record> {
        if wrote || self.counters.dirty() { Some(self.counters.commit_record()) } else { None }
    }
}

fn earliest(left: Option<Time>, right: Option<Time>) -> Option<Time> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (left, None) => left,
        (None, right) => right,
    }
}

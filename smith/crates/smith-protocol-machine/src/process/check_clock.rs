//! A check's held time and fixed-space stderr line matcher (domain/run.md, 8.1).
//! Only a whole newline-ended marker changes the clock. No output is retained
//! here; the process keeps its ordinary bounded tail independently.

use skein_lib::{Duration, Time};

const WAITING: &[u8] = b"smith-check: waiting\n";
const ADMITTED: &[u8] = b"smith-check: admitted\n";

#[derive(Debug)]
pub(crate) struct CheckClock {
    left: Duration,
    cap: Time,
    running: Option<Time>,
    position: usize,
    waiting_line: bool,
    admitted_line: bool,
}

impl CheckClock {
    pub(super) fn new(now: Time, bound: Duration, cap: Time) -> Self {
        Self { left: bound, cap, running: Some(now), position: 0, waiting_line: true, admitted_line: true }
    }

    pub(super) fn byte(&mut self, now: Time, byte: u8) -> Option<Time> {
        self.waiting_line &= WAITING.get(self.position) == Some(&byte);
        self.admitted_line &= ADMITTED.get(self.position) == Some(&byte);
        self.position = self.position.saturating_add(1);
        if byte != b'\n' {
            return None;
        }
        let deadline = if self.waiting_line {
            match self.running.take() {
                Some(since) => {
                    self.left = Duration::from_nanos(
                        self.left.as_nanos().saturating_sub(now.saturating_since(since).as_nanos()),
                    );
                    Some(self.cap)
                }
                None => None,
            }
        } else if self.admitted_line && self.running.is_none() {
            self.running = Some(now);
            Some(now.saturating_add(self.left).min(self.cap))
        } else {
            None
        };
        self.position = 0;
        self.waiting_line = true;
        self.admitted_line = true;
        deadline
    }
}

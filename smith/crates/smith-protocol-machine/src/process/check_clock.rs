//! A check's held time and fixed-space stderr line matcher (domain/run-delivery.md, 1.1).
//! Only whole newline-ended markers change the clock or record that the check
//! didn't judge. The process keeps its bounded output tail independently.

use skein_lib::{Duration, Time};

const WAITING: &[u8] = b"smith-check: waiting\n";
const ADMITTED: &[u8] = b"smith-check: admitted\n";
const NOT_JUDGED: &[u8] = b"smith-check: not judged\n";

#[derive(Debug)]
enum Marker {
    Waiting,
    Admitted,
    NotJudged,
}

#[derive(Debug)]
struct Line {
    position: usize,
    waiting: bool,
    admitted: bool,
    not_judged: bool,
}

impl Line {
    const fn new() -> Self {
        Self { position: 0, waiting: true, admitted: true, not_judged: true }
    }

    fn byte(&mut self, byte: u8) -> Option<Marker> {
        self.waiting &= WAITING.get(self.position) == Some(&byte);
        self.admitted &= ADMITTED.get(self.position) == Some(&byte);
        self.not_judged &= NOT_JUDGED.get(self.position) == Some(&byte);
        if byte != b'\n' {
            self.position = self.position.saturating_add(1);
            return None;
        }
        let marker = if self.waiting {
            Some(Marker::Waiting)
        } else if self.admitted {
            Some(Marker::Admitted)
        } else if self.not_judged {
            Some(Marker::NotJudged)
        } else {
            None
        };
        *self = Self::new();
        marker
    }
}

#[derive(Debug)]
pub(crate) struct CheckClock {
    left: Duration,
    cap: Time,
    running: Option<Time>,
    line: Line,
    pub(super) said_not_judged: bool,
}

impl CheckClock {
    pub(super) fn new(now: Time, bound: Duration, cap: Time) -> Self {
        Self { left: bound, cap, running: Some(now), line: Line::new(), said_not_judged: false }
    }

    pub(super) fn byte(&mut self, now: Time, byte: u8, clock_live: bool) -> Option<Time> {
        match self.line.byte(byte)? {
            Marker::Waiting if clock_live => match self.running.take() {
                Some(since) => {
                    self.left = Duration::from_nanos(
                        self.left.as_nanos().saturating_sub(now.saturating_since(since).as_nanos()),
                    );
                    Some(self.cap)
                }
                None => None,
            },
            Marker::Admitted if clock_live && self.running.is_none() => {
                self.running = Some(now);
                Some(now.saturating_add(self.left).min(self.cap))
            }
            Marker::NotJudged => {
                self.said_not_judged = true;
                None
            }
            Marker::Waiting | Marker::Admitted => None,
        }
    }
}

//! A long wait's report bound and doubling clock (protocol/README.md, 6.3;
//! protocol/configuration/derived.md, 5.6). Reports do not end the wait.
use alloc::boxed::Box;
use skein_lib::{Decimal, Duration, Time, bytes};

pub const WAIT_REPORT: Duration = Duration::from_secs(30);

#[derive(Hash, Debug)]
pub struct Report {
    since: Time,
    next: Duration,
}
impl Report {
    #[must_use]
    pub const fn new(since: Time) -> Self {
        Self { since, next: WAIT_REPORT }
    }

    #[must_use]
    pub fn since(&self) -> Time {
        self.since
    }

    /// Whether this wait has emitted its first report.
    #[must_use]
    pub fn reported(&self) -> bool {
        self.next != WAIT_REPORT
    }

    #[must_use]
    pub fn deadline(&self) -> Time {
        self.since.saturating_add(self.next)
    }

    /// Report once when due, skipping any doublings the pass has missed.
    pub fn due(&mut self, now: Time) -> Option<Duration> {
        if now < self.deadline() {
            return None;
        }
        let age = now.saturating_since(self.since);
        // A u64 span has at most 64 doublings before it saturates.
        for _ in 0..u64::BITS {
            let next = self.next.saturating_mul(2);
            if next == self.next {
                break;
            }
            self.next = next;
            if self.next > age {
                break;
            }
        }
        Some(age)
    }
}

#[must_use]
pub fn age_words(age: Duration) -> Box<[u8]> {
    let seconds = age.as_nanos().div_euclid(Duration::from_secs(1).as_nanos());
    if seconds < 120 {
        bytes::concat(&[Decimal::of(seconds).as_bytes(), b" s"])
    } else {
        bytes::concat(&[Decimal::of(seconds.div_euclid(60)).as_bytes(), b" min"])
    }
}

/// One kind's count and oldest names (protocol/README.md, 6.3).
/// Callers supply at most three names in oldest-first order.
#[must_use]
pub fn wait_kind(count: u64, singular: &[u8], plural: &[u8], names: &[Box<[u8]>]) -> Box<[u8]> {
    if count == 0 {
        return Box::default();
    }
    let kind = if count == 1 { singular } else { plural };
    let mut words = bytes::concat(&[Decimal::of(count).as_bytes(), b" ", kind]);
    if !names.is_empty() {
        words = bytes::concat(&[&words, b" ("]);
        for (at, name) in names.iter().take(3).enumerate() {
            let separator: &[u8] = if at == 0 { b"" } else { b", " };
            words = bytes::concat(&[&words, separator, name]);
        }
        words = bytes::concat(&[&words, b")"]);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_at_the_bound_then_doublings_skipping_missed_reports() {
        let t0 = Time::from_nanos(123_456_789);
        let mut report = Report::new(t0);
        assert_eq!(WAIT_REPORT, Duration::from_secs(30));
        assert_eq!(report.since(), t0);
        assert_eq!(report.deadline(), t0.saturating_add(Duration::from_secs(30)));
        for (seconds, expected) in [(29, None), (30, Some(30)), (45, None), (60, Some(60)), (300, Some(300))] {
            let expected = match expected {
                Some(seconds) => Some(Duration::from_secs(seconds)),
                None => None,
            };
            assert_eq!(report.due(t0.saturating_add(Duration::from_secs(seconds))), expected);
        }
        assert_eq!(report.deadline(), t0.saturating_add(Duration::from_secs(480)));
        assert_eq!(report.due(t0.saturating_add(Duration::from_secs(300))), None);
    }

    #[test]
    fn ages_use_whole_seconds_below_two_minutes_else_whole_minutes() {
        for (seconds, words) in [(30, "30 s"), (119, "119 s"), (120, "2 min"), (1079, "17 min")] {
            let age = Duration::from_secs(seconds);
            assert_eq!(age_words(age).as_ref(), words.as_bytes());
            assert_eq!(age_words(age.saturating_add(Duration::from_nanos(999_999_999))).as_ref(), words.as_bytes());
        }
    }
}

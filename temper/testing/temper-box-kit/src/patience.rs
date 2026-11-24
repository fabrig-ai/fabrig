//! Shared helpers for real-clock waits (skein/docs/foundation/testing-strategy.md, 2.10).
use skein_lib::{Duration, Time};
use skein_world::{Deadline, Expired, Figures, end_to_end::Binary};

/// An inner wait defers starvation to the world that owns its hosts and ring.
#[derive(Debug)]
pub struct Patience {
    deadline: Option<Deadline>,
    watched: bool,
}

enum Verdict {
    Starved,
    Failed(Figures),
}

impl Patience {
    #[must_use]
    pub fn arm(what: &'static str, within: Duration) -> Self {
        Self::from_deadline(Deadline::arm(what, within))
    }

    pub(crate) fn from_deadline(deadline: Deadline) -> Self {
        Self { deadline: Some(deadline), watched: false }
    }

    #[must_use]
    pub fn watching(what: &'static str, within: Duration, binary: &Binary) -> Self {
        let mut patience = Self::arm(what, within);
        patience.watch(binary);
        patience
    }

    pub fn watch(&mut self, binary: &Binary) {
        if !self.watched
            && let Some(deadline) = &mut self.deadline
        {
            deadline.watch(binary);
            self.watched = true;
        }
    }

    pub fn watch_process(&mut self, pid: u32) {
        if !self.watched
            && let Some(deadline) = &mut self.deadline
        {
            deadline.watch_process(pid);
            self.watched = true;
        }
    }

    pub fn defer(&mut self) {
        self.deadline = None;
    }

    pub fn until(&self) -> Option<Time> {
        self.deadline.as_ref().map(Deadline::until)
    }

    #[must_use]
    pub fn deferred(&self) -> bool {
        self.deadline.is_none()
    }

    pub fn socket_timeout(&self) -> Option<std::time::Duration> {
        self.socket_timeout_with(Deadline::remaining)
    }

    fn socket_timeout_with(&self, remaining: impl FnOnce(&Deadline) -> Duration) -> Option<std::time::Duration> {
        self.deadline.as_ref().map(|deadline| round_socket_timeout(remaining(deadline)))
    }

    /// A probe can run in a referee; its caller's world owns the final verdict.
    /// A starved operation retries with its deadline disarmed.
    pub fn socket_retry(&mut self, error: std::io::Error) -> std::io::Result<()> {
        if error.kind() == std::io::ErrorKind::Interrupted {
            return Ok(());
        }
        if matches!(error.kind(), std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock)
            && self.deadline.as_ref().is_some_and(Deadline::passed)
        {
            if let Some(figures) = self.failed() {
                return Err(std::io::Error::new(error.kind(), format!("{error}; {figures}")));
            }
            return Ok(());
        }
        Err(error)
    }

    /// Call after checking answers and ordinary failures, only while unanswered.
    pub fn failed(&mut self) -> Option<Figures> {
        let deadline = self.deadline.as_ref()?;
        if !deadline.passed() {
            return None;
        }
        let verdict = match deadline.expired() {
            Expired::Starved(_) => Verdict::Starved,
            Expired::Failed(figures) => Verdict::Failed(figures),
        };
        self.handle(verdict)
    }

    fn handle(&mut self, verdict: Verdict) -> Option<Figures> {
        match verdict {
            Verdict::Starved => {
                self.defer();
                None
            }
            Verdict::Failed(figures) => Some(figures),
        }
    }
}

/// Socket APIs reject zero and round their waits to the kernel's clock granularity.
#[must_use]
pub fn socket_timeout(deadline: &Deadline) -> std::time::Duration {
    round_socket_timeout(deadline.remaining())
}

fn round_socket_timeout(remaining: Duration) -> std::time::Duration {
    std::time::Duration::from_nanos(remaining.as_nanos()).max(std::time::Duration::from_millis(1))
}

pub mod tests {
    use super::{Patience, Verdict};
    use skein_lib::{Duration, Time};
    use skein_world::{Figures, testing::deadline::at};

    fn patience(what: &'static str, within: Duration) -> Patience {
        Patience::from_deadline(at(what, within, Time::ZERO))
    }

    pub fn starvation_disarms_the_inner_wait_without_passing_the_world() {
        let mut patience = patience("the helper's answer", Duration::ZERO);
        assert!(patience.until().is_some());
        assert!(patience.handle(Verdict::Starved).is_none());
        assert!(patience.until().is_none());
        assert!(patience.socket_timeout_with(|_| panic!("disarmed deadline has no remaining time")).is_none());
        assert!(patience.failed().is_none());
    }

    pub fn a_failed_wait_keeps_its_figures_and_deadline() {
        let mut patience = patience("the helper's answer", Duration::ZERO);
        let until = patience.until();
        let figures = Figures {
            what: "the helper's answer",
            patience: Duration::from_secs(1),
            elapsed: Duration::from_secs(2),
            thread_waited: None,
            thread_cpu: None,
            tree_waited: None,
        };
        let failed = patience.handle(Verdict::Failed(figures)).expect("failure remains a failure");
        assert_eq!(failed.elapsed, Duration::from_secs(2));
        assert_eq!(failed.what, "the helper's answer");
        assert!(patience.until().is_some());
        assert_eq!(patience.until(), until);
    }

    pub fn socket_waits_round_up_and_refusals_never_classify_as_starvation() {
        let mut patience = patience("the socket's answer", Duration::ZERO);
        assert_eq!(
            patience.socket_timeout_with(|deadline| deadline.until().saturating_since(Time::ZERO)),
            Some(std::time::Duration::from_millis(1)),
        );
        let refusal = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "fixture refusal");
        let error = patience.socket_retry(refusal).expect_err("a refusal fails");
        assert_eq!(error.kind(), std::io::ErrorKind::ConnectionRefused);
        assert_eq!(error.to_string(), "fixture refusal");
    }

    pub fn an_interrupted_socket_retries_with_the_same_deadline() {
        let mut patience = patience("the socket's answer", Duration::from_secs(2));
        let until = patience.until();
        patience.socket_retry(std::io::ErrorKind::Interrupted.into()).expect("retry the interrupted operation");
        assert_eq!(patience.until(), until);
        assert!(!patience.deferred());
    }
}

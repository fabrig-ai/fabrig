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
        Self { deadline: Some(Deadline::arm(what, within)), watched: false }
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
        self.deadline.as_ref().map(socket_timeout)
    }

    /// A probe can run in a referee; its caller's world owns the final verdict.
    /// A starved operation retries with its deadline disarmed.
    pub fn socket_retry(&mut self, error: std::io::Error) -> std::io::Result<()> {
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
    std::time::Duration::from_nanos(deadline.remaining().as_nanos()).max(std::time::Duration::from_millis(1))
}

#[cfg(test)]
mod tests {
    use super::{Patience, Verdict, socket_timeout};
    use skein_lib::Duration;
    use skein_world::{Deadline, Figures};

    #[test]
    fn starvation_disarms_the_inner_wait_without_passing_the_world() {
        let mut patience = Patience::arm("the helper's answer", Duration::ZERO);
        assert!(patience.until().is_some());
        assert!(patience.handle(Verdict::Starved).is_none());
        assert!(patience.until().is_none());
        assert!(patience.socket_timeout().is_none());
        assert!(patience.failed().is_none());
    }

    #[test]
    fn a_failed_wait_keeps_its_figures_and_deadline() {
        let mut patience = Patience::arm("the helper's answer", Duration::ZERO);
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
    }

    #[test]
    fn socket_waits_round_up_and_refusals_never_classify_as_starvation() {
        let deadline = Deadline::arm("the socket's answer", Duration::ZERO);
        assert_eq!(socket_timeout(&deadline), std::time::Duration::from_millis(1));
        let mut patience = Patience::arm("the socket's answer", Duration::ZERO);
        let error = patience.socket_retry(std::io::ErrorKind::ConnectionRefused.into()).expect_err("a refusal fails");
        assert_eq!(error.kind(), std::io::ErrorKind::ConnectionRefused);
    }
}

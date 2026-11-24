//! The real-browser referee's wait verdicts (skein/docs/design/testing/browser.md, 9).

use super::{FIND, PAGE, Rig, Stage};
use skein_browser::boundary::{Event, Refusal, Request};
use skein_io::kernel::Exit;
use skein_lib::{Duration, Time, Token};
use skein_world::{Deadline, Expired, Figures};

pub(super) enum End<'a> {
    Browser(&'a Event),
    Tabs { now: Time, until: Time, last: Option<&'a str> },
    Exit(Exit),
    Unexpected(&'a Event),
    Unanswered(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Finding {
    Observe,
    Fail(String),
    Unanswered { op: Option<Token>, report: String },
}

impl End<'_> {
    pub(super) fn finding(self) -> Finding {
        match self {
            Self::Browser(Event::Missed { op, seen, more }) => Finding::Unanswered {
                op: Some(*op),
                report: format!("expectation {op:?} missed; saw {seen:?}, {more} more"),
            },
            Self::Browser(Event::Refused { op, why: Refusal::Timeout, .. }) => {
                Finding::Unanswered { op: Some(*op), report: format!("browser refused operation {op:?}: Timeout") }
            }
            Self::Tabs { now, until, last } if now >= until => {
                Finding::Unanswered { op: None, report: format!("tabs did not settle; {}", last.unwrap_or("no reply")) }
            }
            Self::Exit(exit) if exit != Exit::Code(0) => Finding::Fail(format!("Chromium exited with {exit:?}")),
            Self::Unexpected(Event::Refused { op, why, .. }) => {
                Finding::Fail(format!("browser refused operation {op:?}: {why:?}"))
            }
            Self::Unexpected(event) => Finding::Fail(format!("unexpected browser event: {event:?}")),
            Self::Unanswered(report) => Finding::Unanswered { op: None, report },
            Self::Browser(_) | Self::Tabs { .. } | Self::Exit(_) => Finding::Observe,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Decision {
    Observe,
    Fail(String),
    EndAndFail(String),
    EndAndPass,
}

impl Finding {
    /// Only call with a deadline verdict for an unanswered wait. `None` is measured starvation.
    pub(super) fn decide(self, failed: Option<&Figures>) -> Decision {
        match self {
            Self::Observe => Decision::Observe,
            Self::Fail(report) => Decision::Fail(report),
            Self::Unanswered { report, .. } => match failed {
                Some(figures) => Decision::EndAndFail(format!("{report}; {figures}")),
                None => Decision::EndAndPass,
            },
        }
    }
}

pub(super) struct OperationWait {
    deadline: Deadline,
    subject: Option<String>,
}

impl Rig {
    pub(super) fn arm(&self, what: &'static str, within: Duration) -> Deadline {
        let mut deadline = Deadline::arm(what, within);
        if let Some(pid) = self.pid {
            deadline.watch_process(pid);
        }
        deadline
    }

    pub(super) fn arm_operation(&mut self, request: &Request) {
        let answer = self.browser_env.limits.answer;
        let (op, what, bound, subject) = match request {
            Request::Await { op, query, expect, within, .. } => (
                *op,
                "a Chromium expectation",
                *within,
                Some(format!(
                    "{expect:?} {} {:?}",
                    String::from_utf8_lossy(&query.role),
                    String::from_utf8_lossy(&query.name)
                )),
            ),
            Request::Find { op, .. } => (*op, "Chromium Find", answer, None),
            Request::Go { op, .. } => (*op, "Chromium Go", answer, None),
            Request::Press { op, .. } => (*op, "Chromium Press", answer, None),
            Request::Type { op, .. } => (*op, "Chromium Type", answer, None),
            Request::Compose { op, .. } => (*op, "Chromium Compose", answer, None),
            Request::Keys { op, .. } => (*op, "Chromium Keys", answer, None),
            Request::Key { op, .. } => (*op, "Chromium Key", answer, None),
            Request::Viewport { op, .. } => (*op, "Chromium Viewport", answer, None),
            Request::Address { op, .. } => (*op, "Chromium Address", answer, None),
            Request::Tabs { op, .. } if self.tab_wait.is_none() => (*op, "Chromium Tabs", answer, None),
            Request::Attribute { op, .. } => (*op, "Chromium Attribute", answer, None),
            Request::Snapshot { op, .. } => (*op, "Chromium Snapshot", answer, None),
            Request::Screenshot { op, .. } => (*op, "Chromium Screenshot", answer, None),
            Request::Person { .. } | Request::Page { .. } | Request::Close { .. } | Request::Tabs { .. } => return,
        };
        let deadline = self.arm(what, bound);
        let previous = self.operation_waits.insert(op, OperationWait { deadline, subject });
        assert!(previous.is_none(), "overlapping browser operations for {op:?}");
    }

    pub(super) fn observe_browser(&mut self, event: Event) {
        let op = match &event {
            Event::Found { op, .. }
            | Event::Met { op, .. }
            | Event::Missed { op, .. }
            | Event::Done { op }
            | Event::Refused { op, .. }
            | Event::Address { op, .. }
            | Event::Tabs { op, .. }
            | Event::Attribute { op, .. }
            | Event::Snapshot { op, .. }
            | Event::Screenshot { op, .. } => Some(*op),
            _ => None,
        };
        let operation = op.and_then(|op| self.operation_waits.remove(&op));
        if let Finding::Unanswered { op, mut report } = End::Browser(&event).finding() {
            let missed = matches!(event, Event::Missed { .. });
            if let Some(OperationWait { deadline, subject }) = operation {
                if missed && subject.is_none() {
                    self.fail_unexpected(&event);
                }
                if let Some(subject) = subject {
                    report = format!("{report}; awaited {subject}");
                }
                self.expired_operations.push_back((deadline, Finding::Unanswered { op, report }));
                return;
            }
            if !missed
                && op == Some(FIND)
                && let Some(wait) = self.tab_wait.take()
            {
                let report = format!("{report}; tabs did not settle; {}", wait.last.as_deref().unwrap_or("no reply"));
                self.expired_operations.push_back((wait.deadline, Finding::Unanswered { op, report }));
                return;
            }
            self.fail_unexpected(&event);
        }
        self.browser_event(event);
    }

    // Give replies and exits already delivered to the rig their normal checks before a verdict.
    pub(super) fn check_browser_waits(&mut self) -> Result<(), String> {
        if self.answers_pending() {
            return Ok(());
        }
        if self.expired_operations.front().is_some_and(|(deadline, _)| deadline.passed()) {
            let (deadline, finding) = self.expired_operations.pop_front().expect("the passed operation");
            return Err(self.expire(finding, deadline.expired()));
        }
        let now = self.clock.now().now;
        if let Some(wait) = &mut self.tab_wait {
            let finding = End::Tabs { now, until: wait.deadline.until(), last: wait.last.as_deref() }.finding();
            if finding != Finding::Observe && wait.deadline.passed() {
                let expired = wait.deadline.expired();
                return Err(self.expire(finding, expired));
            }
            if wait.poll.is_some_and(|poll| poll <= now) {
                wait.poll = None;
                self.ask(Request::Tabs { page: PAGE, op: FIND });
            }
        }
        Ok(())
    }

    pub(super) fn answers_pending(&self) -> bool {
        !self.events.is_empty() || !self.complete.is_empty()
    }

    pub(super) fn tab_wake(&self) -> Option<Time> {
        self.tab_wait.as_ref().map(|wait| wait.poll.unwrap_or(wait.deadline.until()).min(wait.deadline.until()))
    }

    pub(super) fn expire(&mut self, finding: Finding, expired: Expired) -> String {
        let stderr = String::from_utf8_lossy(&self.stderr).into_owned();
        finish_expiry(finding, expired, self.stage, &stderr, || self.end())
    }

    pub(super) fn fail_unexpected(&self, event: &Event) -> ! {
        let Decision::Fail(report) = End::Unexpected(event).finding().decide(None) else {
            unreachable!("an unexpected event is a failure")
        };
        panic!("{report}; stderr: {}", String::from_utf8_lossy(&self.stderr));
    }
}

fn finish_expiry(
    finding: Finding,
    expired: Expired,
    stage: Stage,
    stderr: &str,
    end: impl FnOnce() -> Result<(), String>,
) -> String {
    let figures = match &expired {
        Expired::Failed(figures) => Some(figures),
        Expired::Starved(_) => None,
    };
    let decision = finding.decide(figures);
    if let Err(cleanup) = end() {
        let original = match &decision {
            Decision::EndAndFail(report) => report.clone(),
            _ => match &expired {
                Expired::Starved(starved) => starved.figures().to_string(),
                Expired::Failed(figures) => figures.to_string(),
            },
        };
        panic!("browser cleanup failed at {stage:?}: {cleanup}; preceding wait: {original}; stderr: {stderr}");
    }
    match (decision, expired) {
        (Decision::EndAndPass, Expired::Starved(starved)) => starved.pass(),
        (Decision::EndAndFail(report), Expired::Failed(_)) => format!("at {stage:?}: {report}; stderr: {stderr}"),
        _ => unreachable!("only unanswered waits are classified"),
    }
}

#[cfg(test)]
mod tests;

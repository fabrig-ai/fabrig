//! Job observations in the shared world (jig/docs/design/domain/hosts/hub.md, 9).
use super::{Event, Hosted, Launch, LinkUp, Request, Token, World, host};

impl World {
    pub(super) fn job_assigned(&mut self, run: Token, attempt: Token, request: &Request) {
        self.job_keys.insert((run, attempt));
        match request {
            Request::Prepare { .. } => self.assigned(run, attempt, request),
            Request::Job(host::ToJob::Run { owner, .. }) => {
                let entry = Hosted {
                    agent: None,
                    delivered: None,
                    prepared: Some(true),
                    launch: Launch::Asked,
                    word: None,
                    stopped_first: false,
                };
                assert!(self.hosted.insert(*owner, entry).is_none());
                assert!(self.admitted.insert((run, attempt), *owner).is_none());
            }
            Request::Up(LinkUp::JobAnswer { run: answered, attempt: of, answer }) => {
                assert_eq!((*answered, *of), (run, attempt));
                assert!(matches!(answer, host::JobEnding::Refused(_)));
            }
            other @ (Request::Agent(_)
            | Request::Asked { .. }
            | Request::Job(host::ToJob::Stop { .. })
            | Request::DeliverJob { .. }
            | Request::ReadList { .. }
            | Request::CancelRead { .. }
            | Request::Source { .. }
            | Request::SourceFault
            | Request::Up(_)
            | Request::Hello { .. }
            | Request::DeliverUnchecked { .. }
            | Request::Deliver { .. }
            | Request::CancelRelay { .. }
            | Request::Abort { .. }
            | Request::Save { .. }
            | Request::Release { .. }) => panic!("a job is refused, prepared, or run: {other:?}"),
        }
    }

    pub(super) fn job_answered(&mut self, run: Token, attempt: Token, answer: &host::JobEnding) {
        assert!(self.job_keys.contains(&(run, attempt)), "only an assigned job answers");
        if let Some(owner) = self.admitted.get(&(run, attempt)) {
            assert!(self.parent.settled(*owner), "a job answers after Gone, delivery and release");
        }
        if let Some(previous) = self.job_answers.insert((run, attempt), answer.clone()) {
            assert_eq!(&previous, answer, "a kept job answer replays unchanged");
        }
    }

    pub(super) fn job_opened(&self) -> Event {
        let Event::Opened { stop_bound, workstreams, bounds, jobs: _ } = crate::fixtures::opened() else {
            unreachable!("opened fixture");
        };
        Event::Opened { stop_bound, workstreams, bounds, jobs: self.job_kinds.clone() }
    }
}

#[cfg(test)]
mod tests;

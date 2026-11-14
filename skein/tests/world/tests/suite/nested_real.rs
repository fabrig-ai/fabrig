//! A scripted step waits for its inner real run (skein/docs/design/testing/README.md, 5).

use skein_io::kernel::{Complete, Submit};
use skein_lib::{Duration, Queue, Time, Wall};
use skein_shell::Clock;
use skein_world::{Host, Referee, real};
use skein_world_tests::hosted::Script;

struct Judge(Option<&'static str>);

impl<P> Referee<P> for Judge {
    fn watch(&mut self, _now: Time, _procs: &[P]) -> Result<(), String> {
        self.0.map_or(Ok(()), |why| Err(why.into()))
    }

    fn passed(&self) -> bool {
        self.0.is_none()
    }
}

#[derive(Debug)]
struct Nested {
    finding: Option<&'static str>,
    answer: Option<Result<(), String>>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}

impl Host for Nested {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        if self.answer.is_none() {
            self.answer = Some(
                real::try_run(vec![Script::new(&[])], Judge(self.finding), &Clock::new(), Duration::from_secs(2)).map(
                    |outcome| {
                        assert!(outcome.killed.is_empty());
                        assert!(outcome.procs[0].is_empty());
                    },
                ),
            );
        }
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, _now: Time) -> bool {
        self.answer.is_none()
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }

    fn is_empty(&self) -> bool {
        self.answer.is_some()
    }

    fn operations(&self) -> u32 {
        1
    }
}

#[test]
fn a_scripted_host_runs_an_inner_world_to_its_end_and_keeps_its_finding_local() {
    for finding in [None, Some("the inner run's finding")] {
        let host = Nested {
            finding,
            answer: None,
            completions: Queue::with_capacity(1),
            submissions: Queue::with_capacity(1),
        };
        let outer = real::try_run(vec![host], Judge(None), &Clock::new(), Duration::from_secs(2))
            .expect("the outer run stays green even when the inner run returns a finding");
        assert!(outer.killed.is_empty());
        assert_eq!(outer.procs[0].answer, Some(finding.map_or(Ok(()), |why| Err(why.into()))));
    }
}

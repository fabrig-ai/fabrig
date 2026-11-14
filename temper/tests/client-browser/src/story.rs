//! Naming the stories a failed step stands in (temper/docs/design/client/web/testing.md, 3).
use skein_fake_person_browser::{Step, harness::Outcome};

/// The stories begun in one person's script.
pub struct Stories {
    person: u32,
    begun: Vec<(u32, &'static str)>,
}

impl Stories {
    #[must_use]
    pub fn of_person(person: u32) -> Self {
        Self { person, begun: Vec::new() }
    }

    /// The story `name` begins at the next step of `steps`, the script as built so far.
    pub fn begin(&mut self, name: &'static str, steps: &[Step]) {
        self.begun.push((u32::try_from(steps.len()).expect("script length fits a step number"), name));
    }

    /// The story step `step` stands in: the last begun at or before it.
    #[must_use]
    pub fn story(&self, step: u32) -> Option<&'static str> {
        self.begun.iter().rev().find(|(begun, _)| *begun <= step).map(|(_, name)| *name)
    }

    /// Panic unless the run is done, with the failed step's story first in the message.
    #[track_caller]
    pub fn assert_done(&self, outcome: Outcome) {
        if let Err(finding) = self.check_done(outcome) {
            panic!("{finding}");
        }
    }

    /// Returns a failed run's report, attributed to its story where one had begun.
    pub fn check_done(&self, outcome: Outcome) -> Result<(), String> {
        match outcome {
            Outcome::Done => Ok(()),
            Outcome::Failed { person, step, report } => {
                if let Some(story) = (person == self.person).then(|| self.story(step)).flatten() {
                    return Err(format!("{story}: person {person}, step {step}: {report}"));
                }
                Err(format!("person {person}, step {step}: {report}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stories() -> Stories {
        let mut stories = Stories::of_person(1);
        let mut steps = vec![Step::Back];
        stories.begin("first", &steps);
        steps.extend([Step::Back, Step::Back]);
        stories.begin("second", &steps);
        stories
    }

    #[test]
    fn a_step_stands_in_the_last_story_begun_at_or_before_it() {
        let stories = stories();
        assert_eq!(stories.story(0), None);
        assert_eq!(stories.story(1), Some("first"));
        assert_eq!(stories.story(2), Some("first"));
        assert_eq!(stories.story(3), Some("second"));
        assert_eq!(stories.story(9), Some("second"));
    }

    #[test]
    fn a_failure_is_named_by_its_story() {
        assert_eq!(
            stories().check_done(Outcome::Failed { person: 1, step: 3, report: "the report".into() }),
            Err("second: person 1, step 3: the report".into())
        );
    }

    #[test]
    fn another_persons_failure_has_no_story() {
        assert_eq!(
            stories().check_done(Outcome::Failed { person: 0, step: 3, report: "the report".into() }),
            Err("person 0, step 3: the report".into())
        );
    }

    #[test]
    fn unattributed_failures_report_only_the_person_and_step() {
        for (person, step) in [(0, 3), (1, 0)] {
            let failure = stories()
                .check_done(Outcome::Failed { person, step, report: "the report".into() })
                .expect_err("a failed run returns its finding");
            assert_eq!(failure, format!("person {person}, step {step}: the report"));
        }
    }

    #[test]
    fn a_done_run_passes() {
        stories().assert_done(Outcome::Done);
    }
}

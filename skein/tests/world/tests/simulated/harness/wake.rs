//! The browser harness waits on a script's own clock (skein/docs/design/ui/web/testing.md, 2).
use super::{run_story, setup};
use skein_fake_person_browser::{
    BrowserFace, Outcome, Play, Script, Step,
    harness::{Scripted, Shown},
    person::{Find, Next, Step as PersonStep},
};
use skein_lib::{Duration, Time};

struct Turns {
    script: Script,
    checks: Vec<Time>,
}

impl Play for Turns {
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        self.script.next(face, now)
    }
    fn next_wake(&self, face: &BrowserFace<'_>, now: Time, runnable: bool) -> Option<Time> {
        self.script.next_wake(face, now, runnable)
    }
    fn check_deadline(&mut self, face: &mut BrowserFace<'_>, now: Time) {
        self.checks.push(now);
        self.script.check_deadline(face, now);
    }
    fn at(&self) -> u32 {
        self.script.at()
    }
}

#[test]
fn a_waiting_find_only_wakes_at_its_poll_times() {
    const TEXT: &[u8] = b"Ready";
    const WITHIN: Duration = Duration::from_secs(1);
    // Four of the kit's 50 ms poll intervals (testing/browser-operations.md, 1).
    const VISIBLE: Time = Time::from_nanos(200_000_000);
    const WAKES: [Time; 4] =
        [Time::from_nanos(50_000_000), Time::from_nanos(100_000_000), Time::from_nanos(150_000_000), VISIBLE];
    // Each look can take a turn to ask and another to take the reply, then one to finish.
    const WAIT_TURNS: usize = 2 * WAKES.len() + 1;
    let browser = Scripted::new()
        .page(b"/", Shown { nodes: vec![(b"StaticText".as_slice().into(), TEXT.into())], elements: vec![] })
        .find_after(TEXT, VISIBLE);
    let mut person = Turns {
        script: Script::new(Box::new([Step::Person(PersonStep::See { find: Find::text(TEXT), within: WITHIN })])),
        checks: vec![],
    };
    let run = run_story(&mut setup(vec![]), &mut [&mut person], browser);
    assert_eq!(run.outcome, Outcome::Done);
    let mut waited: Vec<_> = person.checks.into_iter().filter(|now| *now > Time::ZERO && *now <= VISIBLE).collect();
    assert!(waited.len() <= WAIT_TURNS, "unexpected turns while waiting: {waited:?}");
    waited.dedup();
    assert_eq!(waited, WAKES);
}

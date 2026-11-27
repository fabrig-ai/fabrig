use super::*;
use skein_forge::api::{self, Answer, Check, Ci, Op, Read, RunState};
use temper_engine_domain_forge_change as change;

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one gather trace covers execution eligibility, checks reuse and its next poll"
)]
fn checking_reads_execution_once_and_caps_its_next_poll() {
    for (required, ci, state, expected) in [
        (false, Ci::Pending, change::State::Checking { head: fixtures::HEAD }, Some(change::Execution::Waiting)),
        (true, Ci::Pending, change::State::Checking { head: fixtures::HEAD }, Some(change::Execution::Waiting)),
        (false, Ci::None, change::State::Checking { head: fixtures::HEAD }, Some(change::Execution::Waiting)),
        (false, Ci::Passed, change::State::Checking { head: fixtures::HEAD }, None),
        (false, Ci::Failed, change::State::Checking { head: fixtures::HEAD }, None),
        (false, Ci::Pending, change::State::Gating { head: fixtures::HEAD, asked: Box::new([]) }, None),
        (false, Ci::Pending, change::State::Checking { head: [9; 32] }, None),
        (false, Ci::Pending, change::State::Opening { requested: true }, None),
    ] {
        let mut d = fixtures::connected_domain();
        let mut repository = adopted();
        repository.protection = ci_protection(required);
        outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::CatchUp);

        let mut row = fixtures::opening_row();
        row.pending = None;
        row.effect = change::EffectResult::None;
        row.pull = Some(1);
        row.change.state = state.clone();
        outputs(&mut d, Event::Change { row });
        fixtures::step_change(&mut d);
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::HEAD)));
        fixtures::answer_next(&mut d, Ok(Answer::Commit(fixtures::BASE)));
        let mut pull = fixtures::open_pull();
        pull.ci = ci;
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Pull { number: 1, ci: !required }));
        fixtures::answer(&mut d, call, Ok(Answer::Pull(pull)));
        let link = api::JobAttempt { head: fixtures::HEAD, run: 7, job: 41, attempt: 2 };
        let mut head_checks = 0_u32;
        let mut run_reads = 0;
        let mut done = skein_lib::List::uncapped();
        for _ in 0_u32..6 {
            let (call, op) = fixtures::next_call(&mut d);
            let answer = match op {
                Op::Read(Read::Checks { commit }) if commit == fixtures::HEAD => {
                    head_checks += 1;
                    let Answer::Checks(mut rows) =
                        fixtures::checks(if ci == Ci::Passed { Check::Passed } else { Check::Pending })
                    else {
                        panic!("checks")
                    };
                    rows[0].job = Some(link);
                    Answer::Checks(rows)
                }
                Op::Read(Read::Checks { commit }) => {
                    assert_eq!(commit, fixtures::BASE);
                    fixtures::checks(Check::Passed)
                }
                Op::Read(Read::Run { run }) => {
                    assert!(expected.is_some(), "no execution read at {state:?} with {ci:?}");
                    assert_eq!(run, 7);
                    run_reads += 1;
                    let mut run = fixtures::failed_run(link, None);
                    run.jobs[0].state = RunState::Waiting;
                    Answer::Run(run)
                }
                Op::Read(Read::Commits { .. }) => Answer::Commits { commits: Box::new([]), more: false },
                Op::Read(Read::Reviews { .. }) => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
                other @ (Op::Read(_) | Op::Write(_)) => panic!("unexpected gather read: {other:?}"),
            };
            let mut decided = false;
            for request in fixtures::answer(&mut d, call, Ok(answer)) {
                decided |= matches!(request, Request::ChangeDecision { .. });
                done.push(request).unwrap();
            }
            if decided {
                break;
            }
        }
        assert_eq!(run_reads, u32::from(expected.is_some()));
        assert!(head_checks <= 1, "reuse this step's checks");
        let row = d.change(fixtures::CHANGE).unwrap();
        assert_eq!(
            match row.change.clock {
                Some(clock) => Some(clock.seen),
                None => None,
            },
            expected,
            "{state:?} / {ci:?}"
        );
        if expected.is_some() {
            let clock = row.change.clock.unwrap();
            assert_eq!((clock.run, clock.attempt), (7, 2));
        }
        if expected.is_some() || matches!(state, change::State::Opening { .. }) {
            let delay = if expected.is_some() { env().limits.client.poll_max } else { Duration::ZERO };
            let deadline = Wall::from_nanos(env().wall.as_nanos() + delay.as_nanos());
            assert!(
                done.as_slice().contains(&Request::ChangeDecision {
                    task: fixtures::CHANGE,
                    decision: change::Decision::Wait { until: Some(deadline) },
                    entry: None,
                    evidence: None,
                    close_at: None,
                    close_refusal: None,
                }),
                "poll cap: {done:?}"
            );
        }
    }
}

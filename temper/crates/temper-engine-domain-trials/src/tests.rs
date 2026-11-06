//! The child's transitions (temper/docs/design/domain/trial.md, 3.2).
use super::*;
use skein_lib::Time;

const ALICE: u64 = 11;
const BOB: u64 = 12;
const ACCOUNT: u32 = 21;
const OTHER: u32 = 22;
const THIRD: u32 = 23;
const TASK: u64 = 31;
const OTHER_TASK: u64 = 32;
const THIRD_TASK: u64 = 33;
// Three rows let a deadline or erase act on several rows and leave another.
const LIMITS: Limits = Limits { rows: 3 };
const SECOND: u64 = Duration::from_secs(1).as_nanos();
const MINUTE: u64 = WAIT.as_nanos();

fn trials() -> Trials { Trials::new(&LIMITS) }
fn row(account: u32, person: u64, task: Option<u64>, began: u64, state: State) -> Row {
    Row { account, person, task, began: Wall::from_nanos(began), took: None, step: Step::Starts, state }
}
fn running(account: u32, task: Option<u64>, began: u64) -> Row {
    row(account, ALICE, task, began, State::Running)
}
fn seed(rows: impl IntoIterator<Item = Row>) -> Trials {
    let mut trials = trials();
    for row in rows { trials.rows.push(row).expect("fixture fits"); }
    trials
}
fn decide(trials: &mut Trials, wall: u64, event: Event) -> Queue<Request> {
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(wall), limits: LIMITS };
    let mut out = Queue::uncapped();
    step(trials, &env, event, &mut out);
    out
}
fn first(trials: &Trials) -> &Row { trials.rows.get(0).expect("one row") }
fn requests(out: &mut Queue<Request>, expected: &[Request]) {
    for request in expected { assert_eq!(out.pop(), Some(*request)); }
    assert!(out.is_empty());
}
fn try_it(account: u32, again: bool) -> Event { Event::Try { account, person: ALICE, again } }
fn made_and_changed(out: &mut Queue<Request>, account: u32) {
    requests(out, &[Request::Make { account, person: ALICE }, Request::Changed { person: ALICE }]);
}

#[test]
fn a_first_try_makes_a_task() {
    let mut t = trials();
    let mut out = decide(&mut t, SECOND, try_it(ACCOUNT, false));
    assert_eq!(t.rows.as_slice(), &[running(ACCOUNT, None, SECOND)]);
    made_and_changed(&mut out, ACCOUNT);
}

#[test]
fn again_on_an_ended_row_replaces_it() {
    for state in [State::Answered { seconds: 2 }, State::Failed { why: Why::Refused }] {
        let mut t = seed([row(ACCOUNT, ALICE, Some(TASK), 0, state)]);
        let mut out = decide(&mut t, SECOND, try_it(ACCOUNT, true));
        assert_eq!(t.rows.as_slice(), &[running(ACCOUNT, None, SECOND)]);
        made_and_changed(&mut out, ACCOUNT);
    }
}

#[test]
fn a_full_table_evicts_the_oldest_ended_row() {
    let mut t = seed([
        row(ACCOUNT, BOB, Some(TASK), SECOND, State::Answered { seconds: 1 }),
        running(OTHER, Some(OTHER_TASK), 0),
        row(THIRD, ALICE, None, 0, State::Failed { why: Why::Refused }),
    ]);
    let mut out = decide(&mut t, SECOND, try_it(THIRD + 1, false));
    assert!(t.rows.as_slice().iter().all(|row| row.account != THIRD));
    made_and_changed(&mut out, THIRD + 1);
}

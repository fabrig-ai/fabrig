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

fn trials() -> Trials {
    Trials::new(&LIMITS)
}
fn row(account: u32, person: u64, task: Option<u64>, began: u64, state: State) -> Row {
    Row { account, person, task, began: Wall::from_nanos(began), took: None, step: Step::Starts, state }
}
fn running(account: u32, task: Option<u64>, began: u64) -> Row {
    row(account, ALICE, task, began, State::Running)
}
fn seed(rows: impl IntoIterator<Item = Row>) -> Trials {
    let mut trials = trials();
    for row in rows {
        trials.rows.push(row).expect("fixture fits");
    }
    trials
}
fn decide(trials: &mut Trials, wall: u64, event: Event) -> Queue<Request> {
    let env = Env { now: Time::ZERO, wall: Wall::from_nanos(wall), limits: LIMITS };
    let mut out = Queue::uncapped();
    step(trials, &env, event, &mut out);
    out
}
fn first(trials: &Trials) -> &Row {
    trials.rows.get(0).expect("one row")
}
fn requests(out: &mut Queue<Request>, expected: &[Request]) {
    for request in expected {
        assert_eq!(out.pop(), Some(*request));
    }
    assert!(out.is_empty());
}
fn try_it(account: u32, again: bool) -> Event {
    Event::Try { account, person: ALICE, again }
}
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
    assert_eq!(
        t.rows.as_slice(),
        &[
            row(ACCOUNT, BOB, Some(TASK), SECOND, State::Answered { seconds: 1 }),
            running(OTHER, Some(OTHER_TASK), 0),
            running(THIRD + 1, None, SECOND),
        ]
    );
    made_and_changed(&mut out, THIRD + 1);
}

#[test]
fn a_second_try_of_an_account_asks_nothing() {
    for state in [State::Running, State::Answered { seconds: 1 }, State::Failed { why: Why::Refused }] {
        let mut t = seed([row(ACCOUNT, ALICE, Some(TASK), 0, state)]);
        assert!(decide(&mut t, SECOND, try_it(ACCOUNT, false)).is_empty());
        assert_eq!(first(&t), &row(ACCOUNT, ALICE, Some(TASK), 0, state));
    }
}

#[test]
fn again_while_running_asks_nothing() {
    let mut t = seed([running(ACCOUNT, Some(TASK), 0)]);
    assert!(decide(&mut t, SECOND, try_it(ACCOUNT, true)).is_empty());
    assert_eq!(first(&t), &running(ACCOUNT, Some(TASK), 0));
}

#[test]
fn made_sets_the_task() {
    let mut t = seed([running(ACCOUNT, None, 0)]);
    let mut out = decide(&mut t, 0, Event::Made { account: ACCOUNT, task: TASK });
    assert_eq!(first(&t).task, Some(TASK));
    requests(&mut out, &[Request::Changed { person: ALICE }]);
    assert!(decide(&mut t, 0, Event::Made { account: ACCOUNT, task: TASK }).is_empty());
}

#[test]
fn refused_fails_it() {
    let mut t = seed([running(ACCOUNT, None, 0)]);
    let mut out = decide(&mut t, 0, Event::Refused { account: ACCOUNT });
    assert_eq!(first(&t).state, State::Failed { why: Why::Refused });
    requests(&mut out, &[Request::Changed { person: ALICE }]);
}

#[test]
fn observed_moves_only_forward() {
    let mut t = seed([running(ACCOUNT, Some(TASK), 0)]);
    let mut out = decide(&mut t, 0, Event::Observed { task: TASK, past_starting: true, took: Some(OTHER) });
    assert_eq!(first(&t).step, Step::Answers);
    assert_eq!(first(&t).took, Some(OTHER));
    requests(&mut out, &[Request::Changed { person: ALICE }]);
    for took in [None, Some(THIRD), Some(OTHER)] {
        assert!(decide(&mut t, 0, Event::Observed { task: TASK, past_starting: false, took }).is_empty());
        assert_eq!(first(&t).step, Step::Answers);
        assert_eq!(first(&t).took, Some(OTHER));
    }
}

fn answered(event: Event) {
    let mut t = seed([running(ACCOUNT, Some(TASK), 0)]);
    let mut out = decide(&mut t, SECOND, event);
    assert_eq!(first(&t).state, State::Answered { seconds: 1 });
    requests(&mut out, &[Request::Cancel { task: TASK }, Request::Changed { person: ALICE }]);
    assert!(decide(&mut t, MINUTE, event).is_empty());
}

#[test]
fn a_turn_answers_and_cancels() {
    answered(Event::Turn { task: TASK });
}
#[test]
fn finished_answers() {
    answered(Event::Finished { task: TASK });
}

#[test]
fn seconds_round_up_and_are_at_least_one() {
    // Zero, fractional and exact seconds; clock reversal; saturation of the wire's u32.
    for (began, wall, seconds) in [
        (SECOND, SECOND, 1),
        (SECOND, SECOND + 1, 1),
        (0, SECOND, 1),
        (SECOND, 2 * SECOND + 1, 2),
        (0, 2 * SECOND, 2),
        (SECOND, 0, 1),
        (0, u64::MAX, u32::MAX),
    ] {
        let mut t = seed([running(ACCOUNT, Some(TASK), began)]);
        drop(decide(&mut t, wall, Event::Turn { task: TASK }));
        assert_eq!(first(&t).state, State::Answered { seconds });
    }
}

#[test]
fn a_failed_run_fails_it_and_cancels() {
    for class in
        [Class::Transient, Class::Permanent, Class::Run, Class::Agent, Class::Lost, Class::Invalid, Class::Refused, Class::Infrastructure]
    {
        let mut t = seed([running(ACCOUNT, Some(TASK), 0)]);
        let event = Event::RunFailed { task: TASK, class };
        let mut out = decide(&mut t, 0, event);
        assert_eq!(first(&t).state, State::Failed { why: Why::Ended { class } });
        requests(&mut out, &[Request::Cancel { task: TASK }, Request::Changed { person: ALICE }]);
        assert!(decide(&mut t, 0, event).is_empty());
    }
}

#[test]
fn a_minute_fails_it_and_cancels() {
    for step in [Step::Starts, Step::Answers] {
        let mut t =
            seed([running(ACCOUNT, Some(TASK), 0), running(OTHER, None, 0), running(THIRD, Some(THIRD_TASK), SECOND)]);
        t.rows.get_mut(0).expect("fixture row").step = step;
        assert!(decide(&mut t, MINUTE - 1, Event::Due).is_empty());
        let mut out = decide(&mut t, MINUTE, Event::Due);
        assert_eq!(first(&t).state, State::Failed { why: Why::Waited });
        assert_eq!(t.rows.get(1).expect("fixture row").state, State::Failed { why: Why::Waited });
        assert_eq!(t.rows.get(2).expect("fixture row").state, State::Running);
        requests(
            &mut out,
            &[Request::Cancel { task: TASK }, Request::Changed { person: ALICE }, Request::Changed { person: ALICE }],
        );
        let mut out = decide(&mut t, MINUTE + SECOND + 1, Event::Due);
        requests(&mut out, &[Request::Cancel { task: THIRD_TASK }, Request::Changed { person: ALICE }]);
        assert!(decide(&mut t, u64::MAX, Event::Due).is_empty());
    }
}

#[test]
fn next_due_is_the_earliest_runner() {
    let mut t = seed([
        running(ACCOUNT, Some(TASK), SECOND),
        row(OTHER, ALICE, Some(OTHER_TASK), 0, State::Answered { seconds: 1 }),
        running(THIRD, None, 0),
    ]);
    assert_eq!(t.next_due(), Some(Wall::from_nanos(MINUTE)));
    drop(decide(&mut t, 0, Event::Refused { account: THIRD }));
    assert_eq!(t.next_due(), Some(Wall::from_nanos(MINUTE + SECOND)));
    drop(decide(&mut t, 0, Event::Turn { task: TASK }));
    assert_eq!(t.next_due(), None);
    assert_eq!(trials().next_due(), None);
    let t = seed([running(ACCOUNT, None, u64::MAX)]);
    assert_eq!(t.next_due(), Some(Wall::from_nanos(u64::MAX)));
}

#[test]
fn a_removed_account_cancels_what_it_started_and_what_took_it() {
    for state in [State::Running, State::Answered { seconds: 1 }, State::Failed { why: Why::Refused }] {
        let mut started = row(ACCOUNT, ALICE, Some(TASK), 0, state);
        started.took = Some(ACCOUNT);
        let mut took = running(OTHER, Some(OTHER_TASK), 0);
        took.took = Some(ACCOUNT);
        took.person = BOB;
        let mut t = seed([started, took, running(THIRD, Some(THIRD_TASK), 0)]);
        let mut out = decide(&mut t, 0, Event::AccountGone { account: ACCOUNT });
        assert_eq!(t.rows.as_slice(), &[running(THIRD, Some(THIRD_TASK), 0)]);
        let mut expected = Queue::uncapped();
        if state == State::Running {
            expected.push(Request::Cancel { task: TASK });
        }
        expected.push(Request::Changed { person: ALICE });
        expected.push(Request::Cancel { task: OTHER_TASK });
        expected.push(Request::Changed { person: BOB });
        while let Some(request) = expected.pop() {
            assert_eq!(out.pop(), Some(request));
        }
        assert!(out.is_empty());
        assert!(decide(&mut t, 0, Event::AccountGone { account: ACCOUNT }).is_empty());
    }
    let mut t = seed([running(ACCOUNT, None, 0)]);
    requests(&mut decide(&mut t, 0, Event::AccountGone { account: ACCOUNT }), &[Request::Changed { person: ALICE }]);
    assert!(t.rows.is_empty());
}

fn accounts(trials: &Trials, person: u64) -> List<u32> {
    let mut accounts = List::with_capacity(LIMITS.rows);
    for row in trials.rows(person) {
        accounts.push(row.account).expect("fixture fits");
    }
    accounts
}

#[test]
fn each_person_reads_their_own_rows() {
    let t = seed([
        running(ACCOUNT, Some(TASK), 0),
        row(OTHER, BOB, None, 0, State::Running),
        running(THIRD, Some(THIRD_TASK), 0),
    ]);
    assert_eq!(accounts(&t, ALICE).as_slice(), &[ACCOUNT, THIRD]);
    assert_eq!(accounts(&t, BOB).as_slice(), &[OTHER]);
    assert_eq!(t.rows(0).count(), 0);
}

#[test]
fn every_change_asks_changed_once() {
    // Observing both progress and the taken account changes one row once.
    let mut t = seed([running(ACCOUNT, Some(TASK), 0)]);
    requests(
        &mut decide(&mut t, 0, Event::Observed { task: TASK, past_starting: true, took: Some(OTHER) }),
        &[Request::Changed { person: ALICE }],
    );
}

#[test]
fn only_running_tasks_are_observed_but_ended_tasks_are_held() {
    let t = seed([
        running(ACCOUNT, Some(TASK), 0),
        row(OTHER, ALICE, Some(OTHER_TASK), 0, State::Answered { seconds: 1 }),
        running(THIRD, None, 0),
    ]);
    let mut tasks = t.running();
    assert_eq!(tasks.next(), Some(TASK));
    assert_eq!(tasks.next(), None);
    assert!(t.holds(TASK));
    assert!(t.holds(OTHER_TASK));
    assert!(!t.holds(THIRD_TASK));
}

#[test]
fn an_event_for_no_row_is_nothing() {
    let unknown = THIRD_TASK;
    for event in [
        Event::Made { account: OTHER, task: unknown },
        Event::Refused { account: OTHER },
        Event::Observed { task: unknown, past_starting: true, took: Some(OTHER) },
        Event::Turn { task: unknown },
        Event::Finished { task: unknown },
        Event::RunFailed { task: unknown, class: Class::Lost },
        Event::AccountGone { account: OTHER },
    ] {
        let mut t = seed([running(ACCOUNT, Some(TASK), 0)]);
        assert!(decide(&mut t, 0, event).is_empty());
        assert_eq!(first(&t), &running(ACCOUNT, Some(TASK), 0));
    }
}

#[test]
fn a_full_table_of_runners_drops_the_try() {
    for rows in [0, LIMITS.rows] {
        let mut t = Trials::new(&Limits { rows });
        for account in 0..rows {
            t.rows.push(running(account, None, 0)).expect("fixture fits");
        }
        assert!(decide(&mut t, SECOND, try_it(ACCOUNT, false)).is_empty());
        assert_eq!(t.rows.len(), rows);
        for row in &t.rows {
            assert_eq!(row.began, Wall::from_nanos(0));
        }
    }
}

#[test]
fn ended_rows_ignore_late_events() {
    for state in [State::Answered { seconds: 1 }, State::Failed { why: Why::Waited }] {
        for event in [
            Event::Made { account: ACCOUNT, task: OTHER_TASK },
            Event::Refused { account: ACCOUNT },
            Event::Observed { task: TASK, past_starting: true, took: Some(OTHER) },
            Event::Turn { task: TASK },
            Event::Finished { task: TASK },
            Event::RunFailed { task: TASK, class: Class::Run },
            Event::Due,
        ] {
            let mut t = seed([row(ACCOUNT, ALICE, Some(TASK), 0, state)]);
            assert!(decide(&mut t, MINUTE, event).is_empty());
            assert_eq!(first(&t), &row(ACCOUNT, ALICE, Some(TASK), 0, state));
        }
    }
}

#[test]
fn eviction_notifies_the_displaced_person() {
    let mut t = seed([
        row(ACCOUNT, BOB, Some(TASK), 0, State::Answered { seconds: 1 }),
        running(OTHER, Some(OTHER_TASK), 0),
        running(THIRD, Some(THIRD_TASK), 0),
    ]);
    requests(
        &mut decide(&mut t, SECOND, try_it(THIRD + 1, false)),
        &[
            Request::Changed { person: BOB },
            Request::Make { account: THIRD + 1, person: ALICE },
            Request::Changed { person: ALICE },
        ],
    );
    assert!(!t.holds(TASK));
}

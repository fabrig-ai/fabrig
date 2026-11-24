//! The referee alone: committed rows and hand-fed clocks, never an engine.
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::Time;
use temper_engine_domain::Record;
use temper_engine_domain_world::{
    ends::{Bound, Ends, Expect},
    store::Store,
};

const BOUND: Bound = Bound { minutes: 10, steps: 4 };

fn at(seconds: u64) -> Time {
    Time::from_nanos(seconds * 1_000_000_000)
}

fn row(phase: tasks::Phase) -> Record {
    let mut row = jig_protocol_store_kit::task_record();
    row.number = 7;
    row.phase = phase;
    let row = Box::new(row);
    Record::Core(core::Record::Tasks(if matches!(row.phase, tasks::Phase::Ended(_)) {
        tasks::Stored::Ended(row)
    } else {
        tasks::Stored::Live(row)
    }))
}

fn stored(phase: tasks::Phase) -> Store {
    let mut store = Store::new();
    let row = row(phase);
    store.rows.insert(row.key(), row);
    store
}

fn cancelled() -> tasks::Ending {
    tasks::Ending::Cancelled {
        by: tasks::Party::Person(1),
        at: skein_lib::Wall::EPOCH,
        reason: Box::new([]),
        result: None,
    }
}

fn held(words: &[u8]) -> tasks::Phase {
    tasks::Phase::Held {
        was: tasks::Was::Waiting,
        why: tasks::Hold::EffectFailed,
        source: Some(Box::new(tasks::HoldSource::Account { connector: 1, entry: Some(9), words: words.into() })),
    }
}

fn asking_producer(store: &mut Store, requester: u64) {
    let mut producer = jig_protocol_store_kit::task_record();
    producer.number = 8;
    producer.requester = tasks::Party::Task(requester);
    producer.phase = tasks::Phase::Active(tasks::Active::Idle);
    producer.questions = Box::new([tasks::QuestionCredit {
        number: 1,
        answerer: tasks::Party::Person(1),
        asked: tasks::Party::Person(1),
        line: 1,
    }]);
    let row = Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(producer))));
    store.rows.insert(row.key(), row);
}

#[test]
fn an_idle_goal_waits_on_its_unanswered_producer() {
    let mut store = stored(tasks::Phase::Active(tasks::Active::Idle));
    asking_producer(&mut store, 7);
    let mut ends = Ends::new(0, BOUND);
    ends.expect(7, Expect::Waits, BOUND).expect("wait expectation");
    ends.close(at(1), &store).expect("the unanswered descendant is a wait");
}

#[test]
fn an_idle_goal_without_a_waiting_descendant_does_not_wait() {
    let mut store = stored(tasks::Phase::Active(tasks::Active::Idle));
    // An unrelated task's unanswered question must not justify this goal's idle row.
    asking_producer(&mut store, 9);
    let mut ends = Ends::new(0, BOUND);
    ends.expect(7, Expect::Waits, BOUND).expect("wait expectation");
    let finding = ends.close(at(1), &store).expect_err("idle with no descendant wait");
    assert!(finding.contains("task 7: expectation still unmet at close"), "{finding}");
}

#[test]
fn an_asking_descendant_does_not_meet_end_expectations() {
    let mut store = stored(tasks::Phase::Active(tasks::Active::Idle));
    asking_producer(&mut store, 7);
    for expect in [Expect::Ends, Expect::Untouched] {
        let mut ends = Ends::new(0, BOUND);
        ends.expect(7, expect, BOUND).expect("end expectation");
        let finding = ends.close(at(1), &store).expect_err("waiting is not ended");
        assert!(finding.contains("task 7: expectation still unmet at close"), "{finding}");
    }
}

#[test]
fn ended_rows_meet_the_expectation() {
    for ending in [
        tasks::Ending::Done(tasks::TaskResult::Report { words: Box::new([]) }),
        tasks::Ending::Failed { reason: Box::new([]) },
        cancelled(),
    ] {
        let mut ends = Ends::new(0, BOUND);
        ends.expect(7, Expect::Ends, BOUND).expect("nonzero");
        let store = stored(tasks::Phase::Ended(ending));
        ends.step(at(1), &store).expect("ended");
        ends.close(at(700), &store).expect("already met");
    }
}

#[test]
fn unmet_bounds_name_the_task_and_both_limits() {
    for by_time in [true, false] {
        let mut ends = Ends::new(0, BOUND);
        ends.expect(7, Expect::Ends, BOUND).expect("nonzero");
        let store = Store::new();
        let finding = if by_time {
            ends.step(at(600), &store).expect_err("minutes")
        } else {
            for _ in 0..3 {
                ends.step(at(0), &store).expect("within steps");
            }
            ends.step(at(0), &store).expect_err("steps")
        };
        assert!(finding.starts_with("seed 0: at ") && finding.contains(": engine task 7:"), "{finding}");
        assert!(finding.contains("10 minutes and 4 steps"), "{finding}");
        assert_eq!(ends.close(at(601), &store), Err(finding), "first finding retained");
    }
}

#[test]
fn held_words_must_name_the_fault() {
    let store = stored(held(b"open pull request: permission refused"));
    let mut ends = Ends::new(0, BOUND);
    ends.expect(7, Expect::Held("permission"), BOUND).expect("nonzero");
    ends.step(at(1), &store).expect("named fault");
    ends.close(at(2), &store).expect("met");
    let mut wrong = Ends::new(0, BOUND);
    wrong.expect(7, Expect::Held("timeout"), BOUND).expect("nonzero");
    let finding = wrong.step(at(1), &store).expect_err("wrong words");
    assert!(finding.contains("engine task 7: held without expected words \"timeout\""), "{finding}");
}

#[test]
fn untouched_counts_from_naming_not_faults_stop() {
    let store = Store::new();
    let mut ends = Ends::new(0, BOUND);
    ends.step(at(60), &store).expect("clock");
    ends.expect(7, Expect::Untouched, BOUND).expect("nonzero");
    ends.faults_stop(at(650));
    ends.step(at(659), &store).expect("599 seconds since naming");
    let finding = ends.step(at(660), &store).expect_err("ten minutes since naming");
    assert!(finding.contains("engine task 7:"), "{finding}");
}

#[test]
fn ends_counts_from_the_later_opening_or_faults_stop() {
    for opening_last in [true, false] {
        let mut ends = Ends::new(0, BOUND);
        let store = Store::new();
        ends.expect(7, Expect::Ends, BOUND).expect("nonzero");
        ends.restarted(at(0));
        ends.opened(at(if opening_last { 20 } else { 10 }));
        ends.faults_stop(at(if opening_last { 10 } else { 20 }));
        ends.step(at(619), &store).expect("599 seconds since later anchor");
        assert!(ends.step(at(620), &store).expect_err("later anchor bound").contains("task 7"));
    }
}

#[test]
fn either_zero_bound_is_refused() {
    for bound in [Bound { minutes: 0, steps: 1 }, Bound { minutes: 1, steps: 0 }] {
        let mut ends = Ends::new(0, BOUND);
        assert!(ends.expect(7, Expect::Ends, bound).expect_err("zero").contains("nonzero"));
    }
}

#[test]
fn opening_is_bounded_in_minutes_and_steps() {
    for by_time in [true, false] {
        let store = Store::new();
        let mut ends = Ends::new(0, BOUND);
        ends.restarted(at(0));
        let finding = if by_time {
            ends.step(at(600), &store).expect_err("minutes")
        } else {
            for _ in 0..3 {
                ends.step(at(0), &store).expect("opening within bound");
            }
            ends.step(at(0), &store).expect_err("steps")
        };
        assert!(finding.contains("engine opening:") && finding.contains("10 minutes and 4 steps"), "{finding}");
    }
}

#[test]
fn a_late_opening_does_not_erase_its_finding() {
    let mut ends = Ends::new(0, BOUND);
    ends.restarted(at(0));
    ends.opened(at(601));
    assert!(ends.step(at(601), &Store::new()).expect_err("late opening").contains("engine opening:"));
}

#[test]
fn close_finds_unmet_expectations_and_unopened_starts() {
    let store = Store::new();
    let mut ends = Ends::new(0, BOUND);
    ends.expect(7, Expect::Ends, BOUND).expect("nonzero");
    assert!(ends.close(at(1), &store).expect_err("unmet").contains("engine task 7: expectation still unmet at close"));
    let mut opening = Ends::new(0, BOUND);
    opening.restarted(at(0));
    assert!(opening.close(at(1), &store).expect_err("unopened").contains("engine opening:"));
    opening = Ends::new(0, BOUND);
    opening.restarted(at(0));
    opening.no_opening(at(1));
    opening.close(at(2), &store).expect("deliberately interrupted");
}

#[test]
fn engine_drivers_open_within_the_shared_bound() {
    use temper_engine_domain_world::{
        direct::Driver, escalation, escalation_referee::Story, forge_routes, roles, walking,
    };
    let mut direct = Driver::new(Store::new());
    direct.settle();
    direct.reopen(walking::config(91));
    direct.settle();
    direct.close();
    let mut walking = walking::World::new(walking::Settings {
        restart: true,
        commit_delay: 2,
        page_delay: 2,
        ..walking::Settings::calm(62)
    });
    walking.close();
    let mut escalation = escalation::World::new(escalation::Settings {
        cut: escalation::Cut::Held,
        commit_delay: 2,
        page_delay: 2,
        ..escalation::Settings::calm(9208, Story::RaceRelease)
    });
    escalation.close();
    let mut roles = roles::World::new(roles::Settings {
        commit_delay: 2,
        page_delay: 1,
        expired: true,
        ..roles::Settings::calm(9306, roles::Base::Requester)
    });
    roles.close();
    let mut forge = forge_routes::World::new();
    forge.until(forge_routes::Until::Ready);
    forge.restart_same();
    forge.until(forge_routes::Until::Ready);
    forge.close();
}

#[test]
fn waits_accepts_waiting_or_held_but_not_ended() {
    for phase in [tasks::Phase::Waiting, held(b"party stopped")] {
        let mut ends = Ends::new(0, BOUND);
        ends.expect(7, Expect::Waits, BOUND).expect("nonzero");
        ends.close(at(1), &stored(phase)).expect("named waiting");
    }
    let mut ends = Ends::new(0, BOUND);
    ends.expect(7, Expect::Waits, BOUND).expect("nonzero");
    assert!(ends.close(at(1), &stored(tasks::Phase::Ended(cancelled()))).is_err());
}

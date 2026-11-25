//! Closer reclamation (temper/docs/design/domain/forge/branches.md, 2).
use super::*;

pub(super) fn closer(task: u64, goal: u64, done: bool) -> CloserRow {
    CloserRow {
        task,
        goal,
        closer: temper_engine_domain_forge_closer::Closer::new(),
        making: None,
        close_changes: Box::new([]),
        superseded: Box::new([]),
        landing_changes: Box::new([]),
        done,
        relay: temper_engine_domain_forge_closer::relay::Relay::new(),
    }
}

#[test]
fn a_forgotten_goal_reclaims_its_completed_closer_rows() {
    let mut d = domain();
    let live = closer(1, 7, false);
    let other = closer(2, 8, true);
    outputs(&mut d, Event::Closer { row: live.clone() });
    outputs(&mut d, Event::Closer { row: other.clone() });
    for task in 100..100 + u64::from(LIMITS.tasks) * 2 {
        outputs(&mut d, Event::Closer { row: closer(task, 7, true) });
        let erased = outputs(&mut d, Event::ForgetProjection { goal: 7 });
        assert!(d.closer(task).is_none(), "completed closer {task} retained");
        assert!(erased.contains(&Request::Erase { key: Key::Closer(task) }));
        assert_eq!(d.closer(1), Some(&live));
        assert_eq!(d.closer(2), Some(&other));
    }
}

const CLOSER: u64 = 3;
const GOAL: u64 = 7;
const STEP: u64 = 1;
const PULL: u64 = 11;

fn check(step: u64) -> Event {
    Event::CheckCloser {
        task: CLOSER,
        step,
        repository: REPO,
        branch: b"g".as_slice().into(),
        base: b"main".as_slice().into(),
    }
}

fn checking(d: &mut Domain, step: u64) {
    outputs(d, check(step));
    answer_tips(d);
}

fn answer_tips(d: &mut Domain) {
    fixtures::answer_next(d, Ok(skein_forge::api::Answer::Commit(fixtures::HEAD)));
    fixtures::answer_next(d, Ok(skein_forge::api::Answer::Commit(fixtures::BASE)));
}

fn checking_domain() -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);

    outputs(&mut d, Event::Closer { row: closer(CLOSER, GOAL, false) });
    d
}

#[test]
fn closer_checks_the_reverse_ancestry_first_page() {
    use skein_forge::api::{Op, Read};
    use temper_engine_domain_forge_change::Status;
    for (result, status) in fixtures::ancestry_cases() {
        let mut d = checking_domain();
        checking(&mut d, 1);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(op, Op::Read(Read::Commits { before: fixtures::BASE, after: fixtures::HEAD, page: 1 }));
        fixtures::answer_once(&mut d, call, result);
        let contained = match status {
            Status::Passed => Some(true),
            Status::Failed => Some(false),
            Status::Unknown | Status::Pending => None,
        };
        assert_eq!(d.closer_checked(3, 1).expect("checked").ok(), contained);
    }
}

#[test]
fn unknown_closer_checks_double_their_wait_to_poll_max_and_known_resets() {
    use skein_forge::api::{Answer, CommitMessage, Error};
    for unknown in [Err(Error::Unavailable), Ok(Answer::Commits { commits: Box::new([]), more: true })] {
        for contained in [true, false] {
            let mut d = checking_domain();
            for (step, seconds) in [1, 2, 4, 8, 16, 30, 30].into_iter().enumerate() {
                d.offered_change(3);
                checking(&mut d, u64::try_from(step).unwrap());
                let (call, _) = fixtures::next_call(&mut d);
                fixtures::answer_once(&mut d, call, unknown.clone());
                assert_eq!(d.closer_checked(3, u64::try_from(step).unwrap()).expect("checked").ok(), None);
                assert_eq!(
                    d.next_wall_deadline(),
                    Some(Wall::from_nanos(env().wall.as_nanos() + seconds * 1_000_000_000))
                );
            }
            d.offered_change(3);
            checking(&mut d, 8);
            let commits: Box<[_]> = if contained {
                Box::new([])
            } else {
                Box::new([CommitMessage { id: fixtures::HEAD, parents: Box::new([]), message: Box::new([]) }])
            };
            fixtures::answer_next(&mut d, Ok(Answer::Commits { commits, more: false }));
            assert_eq!(d.closer_checked(3, 8), Some(Ok(contained)));
            assert_eq!(d.next_wall_deadline(), None);
            checking(&mut d, 9);
            let (call, _) = fixtures::next_call(&mut d);
            fixtures::answer_once(&mut d, call, unknown.clone());
            assert_eq!(d.next_wall_deadline(), Some(Wall::from_nanos(env().wall.as_nanos() + 1_000_000_000)));
        }
    }
}

#[test]
fn a_finished_closer_discards_its_containment_check() {
    let mut d = domain();
    outputs(&mut d, Event::Closer { row: closer(CLOSER, GOAL, false) });
    outputs(
        &mut d,
        Event::CheckCloser {
            task: 3,
            step: 1,
            repository: REPO,
            branch: b"g".as_slice().into(),
            base: b"main".as_slice().into(),
        },
    );
    assert!(!d.closing.reads.is_empty() || d.closer_checked(3, 1).is_some());
    outputs(&mut d, Event::Closer { row: closer(3, 7, true) });
    assert!(d.closing.reads.is_empty());
    assert_eq!(d.closer_checked(3, 1), None);
}

use jig_domain_core_vocabulary::waits::{AtBound, ReportedBound, WaitKind, WaitObject};
use temper_client_protocol_wire::waits::{CLOSER_LANDING, CLOSER_REPORTING};

fn expect_step(requests: &[Request], code: u32, since: Wall, bound: ReportedBound) {
    let [wait] = wait_report(requests, CLOSER).expect("closer wait report") else { panic!("one closer wait") };
    assert_eq!(wait.kind, WaitKind::Step);
    let WaitObject::Procedure { step, .. } = wait.object else { panic!("procedure wait object") };
    assert_eq!((step, wait.since, wait.bound), (code, since, bound));
}

#[test]
fn unknown_containment_holds_at_the_original_step_figure() {
    use skein_forge::api::{Answer, Error};
    let clock = env();
    let bound = advanced_wall(clock.wall, clock.limits.waits.step);
    for (result, why) in [
        (Ok(Answer::Commits { commits: Box::new([]), more: true }), "not there yet"),
        (Err(Error::Unavailable), "forge unavailable"),
    ] {
        let mut d = checking_domain();
        let out = outputs(&mut d, check(STEP));
        expect_step(&out, CLOSER_LANDING, clock.wall, ReportedBound::Figure(AtBound::Hold));
        answer_tips(&mut d);
        let (call, _) = fixtures::next_call(&mut d);
        let retry = fixtures::answer_once(&mut d, call, result);
        assert_eq!(wait_report(&retry, CLOSER), None, "a retry must not replace the wait");
        fire_at(&mut d, &clock, Wall::from_nanos(bound.as_nanos().saturating_sub(1)));
        assert_eq!(d.closer_stall_words(CLOSER), None);
        let out = fire_at(&mut d, &clock, bound);
        assert!(out.contains(&Request::CloserChecked { task: CLOSER }));
        let words = format!(
            "no progress at landing for 30 minutes: waiting for whether main at 040404040404 holds the goal's tip 030303030303; the last answer: {why}"
        );
        assert_eq!(d.closer_stall_words(CLOSER), Some(words.as_bytes()));
    }
}

#[test]
fn an_identical_closer_row_does_not_renew_its_clock() {
    let mut d = checking_domain();
    let clock = env();
    outputs(&mut d, check(STEP));
    let row = d.closer(CLOSER).expect("live closer").clone();
    let later = at_wall(&clock, advanced_wall(clock.wall, Duration::from_secs(1)));
    let mut out = Queue::uncapped();
    step(&mut d, &later, Event::Closer { row }, &mut out);
    assert_eq!(d.next_wall_deadline(), Some(advanced_wall(clock.wall, clock.limits.waits.step)));
}

#[test]
fn a_closer_row_advance_renews_its_clock() {
    let mut d = checking_domain();
    let clock = env();
    outputs(&mut d, check(STEP));
    let mut row = d.closer(CLOSER).expect("live closer").clone();
    row.closer.closed = Some(Box::new([]));
    let later = at_wall(&clock, advanced_wall(clock.wall, Duration::from_secs(1)));
    let mut out = Queue::uncapped();
    step(&mut d, &later, Event::Closer { row }, &mut out);
    assert_eq!(d.next_wall_deadline(), Some(advanced_wall(later.wall, clock.limits.waits.step)));
}

#[test]
fn a_closer_starts_a_fresh_wait_after_release() {
    let mut d = checking_domain();
    let clock = env();
    outputs(&mut d, check(STEP));
    let released = advanced_wall(clock.wall, clock.limits.waits.step);
    fire_at(&mut d, &clock, released);
    d.release_closer_wait(CLOSER, released);
    let mut out = Queue::uncapped();
    step(&mut d, &at_wall(&clock, released), check(STEP + 1), &mut out);
    assert_eq!(d.next_wall_deadline(), Some(advanced_wall(released, clock.limits.waits.step)));
}

#[test]
fn a_restored_closer_counts_from_the_engines_start() {
    let mut d = domain();
    let clock = env();
    let started = Wall::from_nanos(clock.wall.as_nanos().saturating_sub(1));
    let mut out = Queue::uncapped();
    d.restore_begin(started);
    for record in [Stored::Repository(adopted()), Stored::Closer(closer(CLOSER, GOAL, false))] {
        step(&mut d, &clock, Event::Restore { record }, &mut out);
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    d.offered_change(CLOSER);
    let out = outputs(&mut d, check(STEP));
    expect_step(&out, CLOSER_LANDING, started, ReportedBound::Figure(AtBound::Hold));
}

#[test]
fn a_closer_reports_and_bounds_its_report_reads() {
    use temper_engine_domain_forge_closer::{PullSelection, SelectionProgress};
    let job = skein_forge::api::JobAttempt { head: fixtures::HEAD, run: 1, job: 1, attempt: 1 };
    for (progress, what) in [
        (SelectionProgress::Pull, "pull request #11"),
        (SelectionProgress::Checks, "checks"),
        (SelectionProgress::Jobs { attempts: Box::new([job]), next: 0 }, "a job log"),
    ] {
        let mut d = checking_domain();
        let clock = env();
        let mut row = d.closer(CLOSER).expect("live closer").clone();
        row.closer.selections = Box::new([PullSelection { pull: PULL, head: Some(fixtures::HEAD), progress }]);
        outputs(&mut d, Event::Closer { row });
        let out = outputs(&mut d, Event::ReadCloserSelection { task: CLOSER, repository: REPO });
        expect_step(&out, CLOSER_REPORTING, clock.wall, ReportedBound::Figure(AtBound::Hold));
        fire_at(&mut d, &clock, advanced_wall(clock.wall, clock.limits.waits.step));
        let words = format!("no progress at reporting for 30 minutes: waiting for {what}; the last answer: no answer");
        assert_eq!(d.closer_stall_words(CLOSER), Some(words.as_bytes()));
    }
}

#[test]
fn a_closer_reports_no_wait_for_its_live_delegate() {
    use temper_engine_domain_forge_closer::{Running, Step};
    let mut d = checking_domain();
    outputs(&mut d, check(STEP));
    let mut row = d.closer(CLOSER).expect("live closer").clone();
    row.closer.running = Some(Running { step: Step::Land, change: fixtures::CHANGE });
    let out = outputs(&mut d, Event::Closer { row });
    assert_eq!(wait_report(&out, CLOSER), Some([].as_slice()));
}

#[test]
fn the_operator_hold_has_no_automatic_closer_bound() {
    use jig_domain_core_vocabulary::waits::Unbounded;
    let mut d = checking_domain();
    let out = outputs(&mut d, Event::ParkCloser { task: CLOSER, wait: CloserPark::OperatorHold });
    expect_step(&out, CLOSER_LANDING, env().wall, ReportedBound::None(Unbounded::OperatorHold));
    assert_eq!(d.next_wall_deadline(), None);
}

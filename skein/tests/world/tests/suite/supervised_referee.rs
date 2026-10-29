//! Supplied-state transitions of the supervised judge; no binary starts
//! (skein/docs/foundation/testing-strategy.md, 2.1).

use super::{Cleanup, Duration, Exit, Referee, Signal, SupervisedJudge, SupervisedSnapshot, SupervisedTrigger, Time};
use skein_world::end_to_end::TreeStatus;

// A supplied descendant identity, never a process to signal.
const DESCENDANT: u32 = 41;
const TERM_AT: Time = Time::from_nanos(10_000_000);
const GRACE: Duration = Duration::from_millis(20);
const BEFORE: Time = Time::from_nanos(29_999_999);
const BOUNDARY: Time = Time::from_nanos(30_000_000);
const AFTER: Time = Time::from_nanos(30_000_001);
const TRIGGERS: [(SupervisedTrigger, &[u8], bool); 2] =
    [(SupervisedTrigger::StderrOverflow, b"", true), (SupervisedTrigger::Ready, b"descendant:41\n", false)];
const MISSING_GRACE: &str =
    "waiting for a live retained leader exit during TERM grace: the grace window closed before it was observed";

fn live_tree() -> TreeStatus {
    TreeStatus {
        live_pids: vec![DESCENDANT],
        live_at_leader_exit: None,
        interrupted_pids: Vec::new(),
        terminated_pids: vec![DESCENDANT],
        killed_pids: Vec::new(),
        stopped_pids: Vec::new(),
        continued_pids: Vec::new(),
        forced_cleanup: false,
    }
}

fn snapshot<'a>(tree: &'a TreeStatus, stderr: &'a [u8], stderr_overflowed: bool) -> SupervisedSnapshot<'a> {
    SupervisedSnapshot { stderr, stderr_overflowed, exit: None, settled: false, tree }
}

fn triggered(trigger: SupervisedTrigger, state: &SupervisedSnapshot<'_>, cleanup: Cleanup) -> SupervisedJudge {
    let mut judge = SupervisedJudge::new(trigger, GRACE, cleanup);
    judge.act_snapshot(TERM_AT, state);
    judge
}

#[test]
fn escalation_waits_until_the_full_grace_has_passed() {
    let tree = live_tree();
    for (trigger, stderr, overflowed) in TRIGGERS {
        let state = snapshot(&tree, stderr, overflowed);
        let mut judge = triggered(trigger, &state, Cleanup::Kill);
        assert_eq!(judge.act_snapshot(BEFORE, &state), None, "{trigger:?}");
        assert_eq!(judge.kill_at, None);
    }
}

#[test]
fn escalation_at_or_after_grace_requests_kill_once() {
    let tree = live_tree();
    for (trigger, stderr, overflowed) in TRIGGERS {
        for now in [BOUNDARY, AFTER] {
            let state = snapshot(&tree, stderr, overflowed);
            let mut judge = triggered(trigger, &state, Cleanup::Kill);
            assert_eq!(judge.act_snapshot(now, &state), Some(Signal::Kill), "{trigger:?} at {now:?}");
            assert_eq!(judge.kill_at, Some(now));
            assert_eq!(judge.act_snapshot(now, &state), None);
        }
    }
}

#[test]
fn missing_grace_observation_fails_immediately_and_stays_failed() {
    let tree = live_tree();
    for (trigger, stderr, overflowed) in TRIGGERS {
        for now in [BOUNDARY, AFTER] {
            let state = snapshot(&tree, stderr, overflowed);
            let mut judge = triggered(trigger, &state, Cleanup::Kill);
            judge.act_snapshot(now, &state);
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                judge.observe_snapshot(now, &state);
            }))
            .expect_err("the missing grace fails on this observation");
            assert_eq!(
                failure.downcast_ref::<String>().expect("referee diagnostic"),
                &format!("supervised referee failed: {MISSING_GRACE}"),
                "{trigger:?} at {now:?}"
            );
            // Even a later-supplied grace observation cannot undo the verdict.
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                judge.observe_snapshot(BEFORE, &state);
            }))
            .expect_err("the failed verdict remains permanent");
            assert_eq!(judge.overdue(now).as_deref(), Some(MISSING_GRACE));
            assert!(!judge.passed());
        }
    }
}

#[test]
fn retained_exit_observation_records_grace_before_another_act() {
    let tree = live_tree();
    for (trigger, stderr, overflowed) in TRIGGERS {
        for now in [TERM_AT, BEFORE] {
            let mut state = snapshot(&tree, stderr, overflowed);
            let mut judge = triggered(trigger, &state, Cleanup::Terminate);
            state.exit = Some(Exit::Signal(15));
            judge.observe_snapshot(now, &state);
            assert!(judge.saw_retained_exit, "{trigger:?}");
            assert!(judge.saw_grace, "{trigger:?} at {now:?}");
            assert_eq!(judge.kill_at, None);
            assert_eq!(judge.next_deadline(), Some(BOUNDARY));
        }
    }
}

#[test]
fn retained_exit_at_or_after_grace_does_not_record_grace() {
    let tree = live_tree();
    for (trigger, stderr, overflowed) in TRIGGERS {
        for now in [BOUNDARY, AFTER] {
            let mut state = snapshot(&tree, stderr, overflowed);
            let mut judge = triggered(trigger, &state, Cleanup::Terminate);
            state.exit = Some(Exit::Signal(15));
            judge.observe_snapshot(now, &state);
            assert!(judge.saw_retained_exit, "{trigger:?}");
            assert!(!judge.saw_grace, "{trigger:?} at {now:?}");
        }
    }
}

#[test]
fn delayed_observation_uses_retained_exit_after_kill_and_settlement() {
    for (trigger, stderr, overflowed) in TRIGGERS {
        let mut tree = live_tree();
        let mut state = snapshot(&tree, stderr, overflowed);
        let mut judge = triggered(trigger, &state, Cleanup::Kill);
        judge.observe_snapshot(BEFORE, &state);
        judge.act_snapshot(BOUNDARY, &state);
        // Only the historical membership survives the settled current poll.
        tree.live_pids.clear();
        tree.live_at_leader_exit = Some(vec![DESCENDANT]);
        tree.killed_pids = vec![DESCENDANT];
        tree.forced_cleanup = true;
        state = snapshot(&tree, stderr, overflowed);
        state.exit = Some(Exit::Signal(15));
        state.settled = true;
        judge.observe_snapshot(AFTER, &state);
        assert!(judge.saw_retained_exit, "{trigger:?}");
        assert!(judge.passed(), "{trigger:?}");
        assert_eq!(judge.overdue(AFTER), None);
        assert_eq!(judge.next_deadline(), None);
    }
}

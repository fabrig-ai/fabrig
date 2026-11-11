//! Procedure cells, queue order and landing-rule agreement.
use crate::{
    Change, Ci, Decision, Delegate, Early, Effect, EffectResult, Facts, Freshness, Gate, GateFailure, GateKind,
    GateReport, Heard, Hold, Limits, Pending, PendingKind, Pull, QUESTIONS_OPEN, Questions, Ready, Repair, Route,
    StartedFrom, State, Status, first, released, step,
};
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

const HEAD: [u8; 32] = [1; 32];
const BASE: [u8; 32] = [2; 32];
const NEW: [u8; 32] = [3; 32];
const MERGE: [u8; 32] = [4; 32];
const LIMITS: Limits = Limits {
    reruns: 1,
    repairs: 2,
    resolutions: 2,
    updates: 3,
    questions: 4,
    tasks: 6,
    stall: Duration::from_secs(30),
    gates: 4,
    clean_heads: 4,
};
fn at(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.saturating_mul(1_000_000_000))
}
fn change(state: State) -> Change {
    Change {
        early: Early::None,
        way: crate::Way::Queue,
        landing: None,
        task: 7,
        state,
        gates: Box::new([]),
        clean: Box::new([]),
        gate_failures: Box::new([]),
        released: None,
        repairs: 0,
        resolutions: 0,
        updates: 0,
        since: at(0),
        ready_since: None,
        owns_turn: false,
        last_head: Some(HEAD),
    }
}
fn facts() -> Facts {
    Facts {
        fault: crate::Fault::Change,
        pending: Box::new([]),
        ask: None,
        branch: Some(HEAD),
        titled: true,
        expected_base: BASE,
        pull: Pull::Open { head: HEAD, base: BASE },
        ci: Ci { head: HEAD, status: Status::Passed },
        base_ci: Ci { head: BASE, status: Status::Passed },
        base_tip: BASE,
        contains_base: Status::Passed,
        contains_head: Status::Unknown,
        merge_job: false,
        delegate: None,
        catch_up: None,
        close: false,
        contains_catch_up: Status::Pending,
        updates: true,
        mergeable: Status::Passed,
        gates: Box::new([]),
        live_gate: None,
        writer_taken: false,
        requester_branch: false,
        drift: None,
        first: false,
        may_merge: true,
        queue_repair_active: false,
        kept: false,
        asking: false,
        settling: None,
        verification: crate::Verification::Unfollowed,
        verification_bound: Duration::from_secs(7200),
    }
}
fn heard() -> Heard {
    Heard {
        review_request: None,
        start_conflict: None,
        lacking: None,
        spend: None,
        delegate: Status::Pending,
        delegate_running: false,
        delegate_live: false,
        effect: EffectResult::None,
        released: false,
        cancelled: false,
    }
}
fn decide(change: &Change, facts: &Facts, heard: &Heard) -> crate::Stepped {
    step(change, facts, heard, at(1), &LIMITS)
}

#[test]
fn an_agent_with_an_open_question_never_stalls() {
    let mut f = facts();
    f.branch = None;
    f.pull = Pull::Missing;
    f.asking = true;
    for (state, early) in [
        (State::Producing { requested: true }, Early::None),
        (State::Producing { requested: true }, Early::Started(Box::new([StartedFrom { task: 11, head: HEAD }]))),
        (State::Producing { requested: true }, Early::Spent(Box::new([StartedFrom { task: 11, head: HEAD }]))),
        (State::Repairing { requested: true, why: Repair::Ci }, Early::None),
        (State::Resolving { requested: true, base: BASE }, Early::None),
    ] {
        let mut c = change(state);
        c.early = early;
        let next = step(&c, &f, &heard(), at(3600), &LIMITS);
        assert_eq!(next.decision, Decision::Wait { until: None });
        assert_eq!(next.change.since, at(3600));
        assert_eq!(next.change.state, c.state);
    }
}

#[test]
fn a_live_delegate_waits_another_stall_without_polling_faster() {
    let mut f = facts();
    f.branch = None;
    f.pull = Pull::Missing;
    let h = Heard { delegate_live: true, ..heard() };
    for (state, early) in [
        (State::Producing { requested: true }, Early::None),
        (State::Producing { requested: true }, Early::Started(Box::new([StartedFrom { task: 11, head: HEAD }]))),
        (State::Producing { requested: true }, Early::Spent(Box::new([StartedFrom { task: 11, head: HEAD }]))),
        (State::Repairing { requested: true, why: Repair::Ci }, Early::None),
        (State::Resolving { requested: true, base: BASE }, Early::None),
    ] {
        let mut c = change(state);
        c.early = early;
        let next = step(&c, &f, &h, at(3600), &LIMITS);
        assert_eq!(next.decision, Decision::Wait { until: Some(at(3630)) });
        assert_eq!(next.change.since, at(3600));
        assert_eq!(next.change.state, c.state);
    }
}

#[test]
fn a_live_task_does_not_suspend_ci_or_gate_stalls() {
    let mut f = facts();
    f.ci.status = Status::Pending;
    let h = Heard { delegate_live: true, ..heard() };
    assert_eq!(
        step(&change(State::Checking { head: HEAD }), &f, &h, at(3600), &LIMITS).decision,
        Decision::Hold(Hold::Stalled)
    );
    f.ci.status = Status::Passed;
    let mut c = change(State::Gating { head: HEAD, asked: Box::new([1]) });
    c.gates = Box::new([Gate {
        number: 1,
        kind: GateKind::Agent,
        blocking: true,
        freshness: Freshness::Clean,
        eager: false,
    }]);
    f.live_gate = Some(HEAD);
    assert_eq!(step(&c, &f, &h, at(3600), &LIMITS).decision, Decision::Hold(Hold::Stalled));
}

#[test]
fn the_stall_counts_from_the_last_question() {
    let mut f = facts();
    f.branch = None;
    f.pull = Pull::Missing;
    f.asking = true;
    let c = change(State::Producing { requested: true });
    let asking = step(&c, &f, &heard(), at(3600), &LIMITS);
    f.asking = false;
    let waiting = step(&asking.change, &f, &heard(), at(3629), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: Some(at(3630)) });
    assert_eq!(waiting.change.since, at(3600));
    let held = step(&waiting.change, &f, &heard(), at(3631), &LIMITS);
    assert_eq!(held.decision, Decision::Hold(Hold::Stalled));
    assert_eq!(held.change.state, State::Held { was: Box::new(c.state), why: Hold::Stalled });
}

#[test]
fn an_open_question_does_not_suspend_a_wait_on_ci() {
    let mut f = facts();
    f.asking = true;
    f.ci.status = Status::Pending;
    let next = step(&change(State::Checking { head: HEAD }), &f, &heard(), at(3600), &LIMITS);
    assert_eq!(next.decision, Decision::Hold(Hold::Stalled));
}

#[test]
fn a_kept_queued_or_first_change_leaves_the_queue_without_losing_its_ready_time() {
    let mut f = facts();
    f.kept = true;
    f.first = true;
    for state in [State::Queued { head: HEAD, ready_since: at(0) }, State::First { head: HEAD, ready_since: at(0) }] {
        let mut c = change(state);
        c.ready_since = Some(at(0));
        c.owns_turn = true;
        let next = decide(&c, &f, &heard());
        assert_eq!(next.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
        assert_eq!(next.change.ready_since, Some(at(0)));
        assert!(!next.change.owns_turn);
        assert_eq!(next.decision, Decision::None);
    }
}

#[test]
fn a_kept_gating_change_waits_without_a_stall_deadline_or_a_ready_time() {
    let mut f = facts();
    f.kept = true;
    let c = change(State::Gating { head: HEAD, asked: Box::new([]) });
    let next = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(next.decision, Decision::Wait { until: None });
    assert_eq!(next.change.state, c.state);
    assert_eq!(next.change.ready_since, None);
}

#[test]
fn an_unkept_change_is_ready_again_at_its_old_time_with_its_gate_verdict() {
    let mut c = change(State::First { head: HEAD, ready_since: at(0) });
    c.ready_since = Some(at(0));
    c.owns_turn = true;
    c.gates = Box::new([Gate {
        number: 9,
        kind: GateKind::Agent,
        blocking: true,
        freshness: Freshness::Exact,
        eager: false,
    }]);
    let mut f = facts();
    f.kept = true;
    f.gates = Box::new([GateReport { number: 9, head: HEAD, status: Status::Passed }]);
    let left = decide(&c, &f, &heard());
    let waiting = step(&left.change, &f, &heard(), at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    f.kept = false;
    let ready = step(&waiting.change, &f, &heard(), at(61), &LIMITS);
    assert_eq!(ready.decision, Decision::Ready);
    assert_eq!(ready.change.state, State::Queued { head: HEAD, ready_since: at(0) });
    assert_eq!(ready.change.ready_since, Some(at(0)));
    assert!(!ready.change.owns_turn);
}
#[test]
fn a_change_is_produced_opened_checked_queued_and_landed() {
    let mut f = facts();
    f.branch = None;
    f.pull = Pull::Missing;
    let one = decide(&change(State::Producing { requested: false }), &f, &heard());
    assert_eq!(one.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(one.change.state, State::Producing { requested: true });
    assert_eq!(decide(&one.change, &f, &heard()).decision, Decision::Wait { until: Some(at(31)) });
    f.branch = Some(HEAD);
    let two = decide(&one.change, &f, &heard());
    assert_eq!(two.change.state, State::Opening { requested: false });
    let three = decide(&two.change, &f, &heard());
    assert_eq!(three.decision, Decision::Effect(Effect::Open));
    assert_eq!(decide(&three.change, &f, &heard()).decision, Decision::Wait { until: Some(at(31)) });
    f.pull = Pull::Open { head: HEAD, base: BASE };
    let four = decide(&three.change, &f, &heard());
    assert_eq!(four.change.state, State::Checking { head: HEAD });
    let five = decide(&four.change, &f, &heard());
    assert_eq!(five.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
    let six = decide(&five.change, &f, &heard());
    assert_eq!(six.decision, Decision::Ready);
    assert_eq!(six.change.state, State::Queued { head: HEAD, ready_since: at(1) });
    assert_eq!(decide(&six.change, &f, &heard()).decision, Decision::Wait { until: None });
    f.first = true;
    let seven = decide(&six.change, &f, &heard());
    assert_eq!(seven.change.state, State::First { head: HEAD, ready_since: at(1) });
    let eight = decide(&seven.change, &f, &heard());
    assert_eq!(eight.decision, Decision::Effect(Effect::Merge { head: HEAD, base: BASE }));
    assert_eq!(decide(&eight.change, &f, &heard()).decision, Decision::Wait { until: Some(at(31)) });
    f.pull = Pull::Merged { commit: MERGE };
    let nine = decide(&eight.change, &f, &heard());
    assert_eq!(nine.change.state, State::Landed { merge: MERGE, since: at(1) });
    assert_eq!(nine.decision, Decision::Finish { merge: MERGE });
}
fn failed_producer() -> (Change, Facts, Heard) {
    let mut f = facts();
    f.branch = None;
    f.pull = Pull::Missing;
    f.titled = false;
    let mut c = change(State::Producing { requested: false });
    c.last_head = None;
    let first = decide(&c, &f, &heard());
    assert_eq!(first.decision, Decision::Delegate(Delegate::Produce));
    let mut h = heard();
    h.delegate = Status::Failed;
    let failed = decide(&first.change, &f, &h);
    assert_eq!(failed.decision, Decision::Hold(Hold::Failed));
    (failed.change, f, h)
}

#[test]
fn a_producer_that_pushed_then_failed_holds_without_taking_its_head() {
    let mut c = change(State::Producing { requested: true });
    c.last_head = None;
    let mut f = facts();
    f.pull = Pull::Missing;
    let failed = decide(&c, &f, &Heard { delegate: Status::Failed, ..heard() });
    assert_eq!(failed.decision, Decision::Hold(Hold::Failed));
    assert_eq!(failed.change.state, State::Held { was: Box::new(c.state), why: Hold::Failed });
    assert_eq!(failed.change.last_head, None);
}

#[test]
fn failed_repairs_and_resolutions_do_not_take_a_pushed_head() {
    for state in
        [State::Repairing { requested: true, why: Repair::Ci }, State::Resolving { requested: true, base: BASE }]
    {
        let c = change(state);
        let mut f = facts();
        f.branch = Some(NEW);
        let failed = decide(&c, &f, &Heard { delegate: Status::Failed, ..heard() });
        assert_eq!(failed.decision, Decision::Hold(Hold::Failed));
        assert_eq!(failed.change.state, State::Held { was: Box::new(c.state), why: Hold::Failed });
        assert_eq!(failed.change.last_head, Some(HEAD));
    }
}

#[test]
fn an_untitled_pushed_head_waits_for_a_successful_producer() {
    let mut f = facts();
    f.pull = Pull::Missing;
    f.titled = false;
    let mut c = change(State::Producing { requested: false });
    c.last_head = None;
    let fresh = decide(&c, &f, &heard());
    assert_eq!(fresh.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(fresh.change.last_head, None);
    for delegate in [Status::Pending, Status::Passed] {
        let waiting = decide(&fresh.change, &f, &Heard { delegate, ..heard() });
        assert_eq!(waiting.decision, Decision::Wait { until: Some(at(31)) });
        assert_eq!(waiting.change.state, State::Producing { requested: true });
        assert_eq!(waiting.change.last_head, None);
    }
    f.titled = true;
    let opening = decide(&fresh.change, &f, &Heard { delegate: Status::Passed, ..heard() });
    assert_eq!(opening.change.state, State::Opening { requested: false });
}

fn failed_ci_repair() -> (Change, Facts, Heard, Limits) {
    let limits = Limits { repairs: 3, questions: 4, ..LIMITS };
    let mut f = facts();
    f.ci.status = Status::Failed;
    let c = change(State::Checking { head: HEAD });
    let repair = step(&c, &f, &heard(), at(1), &limits);
    assert_eq!(repair.decision, Decision::Delegate(Delegate::Repair(Repair::Ci)));
    let h = Heard { delegate: Status::Failed, ..heard() };
    let failed = step(&repair.change, &f, &h, at(2), &limits);
    assert_eq!(failed.decision, Decision::Hold(Hold::Failed));
    assert_eq!(failed.change.repairs, 1, "a permanent failure does not exhaust three repairs");
    (failed.change, f, h, limits)
}

#[test]
fn a_released_failed_ci_repair_requests_one_replacement_despite_the_stale_failure() {
    // The change step is the lowest world: no connector or model causes this transition.
    let (c, f, h, limits) = failed_ci_repair();
    let reset = step(&released(&c, at(3)), &f, &h, at(3), &limits);
    assert_eq!(reset.change.state, State::Repairing { requested: false, why: Repair::Ci });
    assert_eq!(reset.change.repairs, 0);
    let fresh = step(&reset.change, &f, &h, at(4), &limits);
    assert_eq!(fresh.decision, Decision::Delegate(Delegate::Repair(Repair::Ci)));
    assert_eq!(fresh.change.repairs, 1);
    assert_eq!(fresh.change.last_head, Some(HEAD));
    let pending = step(&fresh.change, &f, &heard(), at(5), &limits);
    assert_eq!(pending.decision, Decision::Wait { until: Some(at(34)) });
    assert_eq!(pending.change.repairs, 1);
}

fn failed_resolution(delegate: Delegate) -> (Change, Facts, Heard) {
    let mut c = change(State::Resolving { requested: true, base: BASE });
    c.resolutions = LIMITS.resolutions;
    let f = Facts { delegate: Some(delegate), ..facts() };
    let h = Heard { delegate: Status::Failed, ..heard() };
    let failed = step(&c, &f, &h, at(2), &LIMITS);
    assert_eq!(failed.decision, Decision::Hold(Hold::Failed));
    assert_eq!(failed.change.last_head, Some(HEAD));
    (failed.change, f, h)
}

#[test]
fn a_released_failed_resolution_decides_from_fresh_facts_despite_the_stale_failure() {
    use crate::Way;
    for (terminal, delegate) in [
        ("resolution failed", Delegate::Resolve { base: BASE }),
        // The connector reports a cancelled delegate as Failed too.
        ("resolution cancelled", Delegate::Resolve { base: BASE }),
        ("merge job failed", Delegate::Merge { base: BASE, from: Some(HEAD) }),
    ] {
        let (c, f, h) = failed_resolution(delegate);
        for (mergeable, live, way, state, decision, resolutions, last_head) in [
            (Status::Passed, false, Way::Queue, State::Checking { head: NEW }, Decision::None, 0, Some(NEW)),
            (
                Status::Failed,
                false,
                Way::Queue,
                State::Resolving { requested: true, base: MERGE },
                Decision::Delegate(Delegate::Resolve { base: MERGE }),
                1,
                Some(NEW),
            ),
            (
                Status::Unknown,
                false,
                Way::Queue,
                State::WaitingProducer { base: BASE },
                Decision::Wait { until: Some(at(4000)) },
                0,
                Some(NEW),
            ),
            (
                Status::Pending,
                false,
                Way::Queue,
                State::WaitingProducer { base: BASE },
                Decision::Wait { until: Some(at(4000)) },
                0,
                Some(NEW),
            ),
            (
                Status::Failed,
                true,
                Way::Queue,
                State::WaitingProducer { base: BASE },
                Decision::Wait { until: None },
                0,
                Some(HEAD),
            ),
            (
                Status::Failed,
                false,
                Way::Turn,
                State::Preparing { from: NEW, main: BASE, ready_since: at(3600) },
                Decision::Effect(Effect::Prepare { head: NEW, main: BASE }),
                0,
                Some(NEW),
            ),
        ] {
            let mut held = c.clone();
            held.way = way;
            let fresh = Facts {
                branch: Some(NEW),
                pull: Pull::Open { head: NEW, base: MERGE },
                base_tip: MERGE,
                merge_job: true,
                mergeable,
                settling: Some(at(4000)),
                ..f.clone()
            };
            let stale = Heard { delegate_live: live, ..h };
            let reset = step(&released(&held, at(3)), &fresh, &stale, at(3), &LIMITS);
            assert_eq!(reset.decision, Decision::None, "{terminal}");
            assert_eq!(reset.change.state, State::WaitingProducer { base: BASE }, "{terminal}");
            assert_eq!(reset.change.resolutions, 0, "{terminal}");
            // A live pushing task must not stall even long after the release.
            let next = step(&reset.change, &fresh, &stale, at(3600), &LIMITS);
            assert_eq!(next.decision, decision, "{terminal}, {mergeable:?}, {way:?}, live={live}");
            assert_eq!(next.change.state, state, "{terminal}");
            assert_eq!(next.change.resolutions, resolutions, "{terminal}");
            assert_eq!(next.change.last_head, last_head, "{terminal}");
        }
        let fresh = Facts { branch: Some(NEW), base_tip: MERGE, mergeable: Status::Failed, merge_job: true, ..f };
        let reset = step(&released(&c, at(3)), &fresh, &h, at(3), &LIMITS);
        let replacement = step(&reset.change, &fresh, &h, at(4), &LIMITS);
        assert_eq!(replacement.decision, Decision::Delegate(Delegate::Resolve { base: MERGE }), "{terminal}");
        let pending = step(&replacement.change, &fresh, &heard(), at(5), &LIMITS);
        assert_eq!(pending.decision, Decision::Wait { until: Some(at(34)) }, "{terminal}");
        assert_eq!(pending.change.resolutions, 1, "{terminal}");
    }
}

#[test]
fn a_released_failed_resolution_obeys_the_reset_allowance_and_current_authority() {
    use crate::{AuthorityHold, AuthorityLack, AuthorityRecovery};
    let cause = AuthorityHold {
        recovery: AuthorityRecovery { task: 7, spec: 0, hold: 2, tasks: 2 },
        lack: AuthorityLack::Tasks,
        had: 0,
        released: false,
    };
    let (c, mut f, h) = failed_resolution(Delegate::Resolve { base: BASE });
    f.branch = Some(NEW);
    f.base_tip = MERGE;
    f.mergeable = Status::Failed;
    f.merge_job = true;
    for (allowance, lacking, decision, count) in [
        (1, None, Decision::Delegate(Delegate::Resolve { base: MERGE }), 1),
        (0, None, Decision::Hold(Hold::Resolutions), 0),
        (1, Some(cause), Decision::Hold(Hold::Authority(cause)), 0),
    ] {
        let limits = Limits { resolutions: allowance, ..LIMITS };
        let stale = Heard { lacking, ..h };
        let reset = step(&released(&c, at(3)), &f, &stale, at(3), &limits);
        let next = step(&reset.change, &f, &stale, at(4), &limits);
        assert_eq!(next.decision, decision);
        assert_eq!(next.change.resolutions, count);
    }
}

#[test]
fn a_resolution_held_for_allowance_or_authority_keeps_its_release_path() {
    use crate::{AuthorityHold, AuthorityLack, AuthorityRecovery};
    let authority = AuthorityHold {
        recovery: AuthorityRecovery { task: 7, spec: 0, hold: 2, tasks: 2 },
        lack: AuthorityLack::Tasks,
        had: 0,
        released: false,
    };
    let state = State::Resolving { requested: false, base: BASE };
    for why in [Hold::Resolutions, Hold::Authority(authority)] {
        let mut c = change(State::Held { was: Box::new(state.clone()), why });
        c.resolutions = LIMITS.resolutions;
        let f = Facts { delegate: Some(Delegate::Resolve { base: BASE }), merge_job: true, ..facts() };
        let reset = step(&released(&c, at(3)), &f, &heard(), at(3), &LIMITS);
        assert_eq!(reset.change.state, state);
        assert_eq!(reset.change.resolutions, 0);
        let next = step(&reset.change, &f, &heard(), at(4), &LIMITS);
        assert_eq!(next.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
        assert_eq!(next.change.resolutions, 0, "the unrequested resolution was counted before the hold");
    }
}

#[test]
fn a_released_failed_ci_repairs_replacement_requires_current_authority() {
    use crate::{AuthorityHold, AuthorityLack, AuthorityRecovery};
    let (c, f, h, limits) = failed_ci_repair();
    let reset = step(&released(&c, at(3)), &f, &h, at(3), &limits);
    let cause = AuthorityHold {
        recovery: AuthorityRecovery { task: c.task, spec: 0, hold: 2, tasks: 2 },
        lack: AuthorityLack::Tasks,
        had: 0,
        released: false,
    };
    let blocked = step(&reset.change, &f, &Heard { lacking: Some(cause), ..h }, at(4), &limits);
    assert_eq!(blocked.decision, Decision::Hold(Hold::Authority(cause)));
    assert_eq!(blocked.change.repairs, 0);
    let bare = step(&released(&blocked.change, at(5)), &f, &Heard { lacking: Some(cause), ..h }, at(5), &limits);
    assert_eq!(bare.decision, Decision::Hold(Hold::Authority(AuthorityHold { released: true, ..cause })));
}

#[test]
fn a_released_failed_repair_keeps_its_reason_and_checks_the_current_base() {
    let (c, mut f, h, limits) = failed_ci_repair();
    for why in
        [Repair::Ci, Repair::Semantic, Repair::Gate(9), Repair::Landing { gate: 3 }, Repair::Review { head: NEW }]
    {
        let mut held = c.clone();
        held.state = State::Held { was: Box::new(State::Repairing { requested: true, why }), why: Hold::Failed };
        let reset = step(&released(&held, at(3)), &f, &h, at(3), &limits);
        let fresh = step(&reset.change, &f, &h, at(4), &limits);
        assert_eq!(fresh.decision, Decision::Delegate(Delegate::Repair(why)));
        assert_eq!(fresh.change.repairs, 1);
        f.contains_base = Status::Failed;
        let updating = step(&reset.change, &f, &h, at(4), &limits);
        assert_eq!(updating.decision, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
        assert_eq!(updating.change.repairs, 0, "taking the new base does not spend a repair");
        f.contains_base = Status::Passed;
    }
}

#[test]
fn a_released_change_makes_a_fresh_producer_after_its_first_failed_for_good() {
    let (c, mut f, h) = failed_producer();
    f.branch = Some(HEAD);
    let lifted = released(&c, at(1));
    let reset = decide(&lifted, &f, &h);
    assert_eq!(reset.change.state, State::Producing { requested: false });
    let fresh = decide(&reset.change, &f, &h);
    assert_eq!(fresh.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(fresh.change.state, State::Producing { requested: true });
    f.branch = None;
    assert_eq!(decide(&fresh.change, &f, &heard()).decision, Decision::Wait { until: Some(at(31)) });
    f.branch = Some(NEW);
    f.titled = true;
    let opened = decide(&fresh.change, &f, &Heard { delegate: Status::Passed, ..heard() });
    assert_eq!(opened.change.state, State::Opening { requested: false });
    assert_eq!(decide(&opened.change, &f, &heard()).decision, Decision::Effect(Effect::Open));
}

#[test]
fn a_released_failed_producers_replacement_needs_authority_and_a_bare_release_holds_again() {
    use crate::{AuthorityHold, AuthorityLack, AuthorityRecovery};
    for lack in [AuthorityLack::Tasks, AuthorityLack::Depth, AuthorityLack::Spend] {
        let (c, mut f, mut h) = failed_producer();
        f.branch = Some(HEAD);
        let cause = AuthorityHold {
            recovery: AuthorityRecovery { task: c.task, spec: 0, hold: 2, tasks: 2 },
            lack,
            had: 1,
            released: false,
        };
        if lack == AuthorityLack::Spend {
            h.spend = Some(crate::DelegateSpend {
                recovery: cause.recovery,
                left: cause.had,
                coding: cause.had + 1,
                review: 0,
            });
        } else {
            h.lacking = Some(cause);
        }
        let reset = decide(&released(&c, at(1)), &f, &h);
        let blocked = decide(&reset.change, &f, &h);
        assert_eq!(blocked.decision, Decision::Hold(Hold::Authority(cause)));
        let again = decide(&released(&blocked.change, at(1)), &f, &h);
        assert_eq!(again.decision, Decision::Hold(Hold::Authority(AuthorityHold { released: true, ..cause })));
        for released in [false, true] {
            let words = crate::authority_words(AuthorityHold { released, ..cause });
            assert!(words.ends_with(b"; or ask afresh; bare release holds again"));
            assert!(has_hold_words(&words, b"; cancel: task cancel 7 REASON"));
            assert_eq!(has_hold_words(&words, b"task release"), lack == AuthorityLack::Tasks);
            if lack == AuthorityLack::Tasks {
                assert!(has_hold_words(
                    &words,
                    b"task amend 7 --rev 0 --tasks 4 REASON; then release it: task release 7 --hold 2",
                ));
            }
        }
    }
}

fn has_hold_words(words: &[u8], expected: &[u8]) -> bool {
    for part in words.windows(expected.len()) {
        if part == expected {
            return true;
        }
    }
    false
}

#[test]
fn a_release_keeps_other_producing_holds_and_restarts_a_failed_producer_with_a_last_head() {
    let (mut c, f, h) = failed_producer();
    let stalled = change(State::Held { was: Box::new(State::Producing { requested: true }), why: Hold::Stalled });
    let reset = decide(&released(&stalled, at(1)), &f, &h);
    assert_eq!(reset.change.state, State::Producing { requested: true });
    c.last_head = Some(HEAD);
    let reset = decide(&released(&c, at(1)), &f, &h);
    assert_eq!(reset.change.state, State::Producing { requested: false });
}

/// A conflict resolves under rules asking no update (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_conflict_under_rules_asking_no_update_is_resolved() {
    let c = change(State::First { head: HEAD, ready_since: at(0) });
    let mut f = facts();
    f.mergeable = Status::Failed;
    f.updates = false;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.change.state, State::Resolving { requested: true, base: BASE });
    assert_eq!(next.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(next.change.resolutions, 1);
}

#[test]
fn a_conflict_under_rules_asking_up_to_date_is_resolved() {
    let c = change(State::First { head: HEAD, ready_since: at(0) });
    let mut f = facts();
    f.mergeable = Status::Failed;
    f.updates = true;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.change.state, State::Resolving { requested: true, base: BASE });
    assert_eq!(next.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(next.change.resolutions, 1);
}

#[test]
fn a_head_already_behind_takes_its_base_before_its_ci() {
    for status in [Status::Unknown, Status::Pending] {
        for eager in [false, true] {
            let mut c = change(State::Checking { head: HEAD });
            c.gates = Box::new([Gate {
                number: 9,
                kind: GateKind::Agent,
                blocking: true,
                freshness: Freshness::Clean,
                eager,
            }]);
            let mut f = facts();
            f.contains_base = Status::Failed;
            f.ci.status = status;
            let next = decide(&c, &f, &heard());
            assert_eq!(next.decision, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
            assert_eq!(next.change.state, State::Updating { from: HEAD, base: BASE, ready_since: at(1) });
            assert_eq!(next.change.updates, 1);
        }
    }
}

#[test]
fn a_green_head_already_behind_goes_to_its_gates() {
    let c = change(State::Checking { head: HEAD });
    let mut f = facts();
    f.contains_base = Status::Failed;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.updates, 0);
}

#[test]
fn a_red_head_already_behind_takes_its_base_before_repair() {
    let c = change(State::Checking { head: HEAD });
    let mut f = facts();
    f.contains_base = Status::Failed;
    f.ci.status = Status::Failed;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
    assert_eq!(next.change.state, State::Updating { from: HEAD, base: BASE, ready_since: at(1) });
    assert_eq!((next.change.updates, next.change.repairs), (1, 0));
}

#[test]
fn an_unchecked_head_waits_until_its_base_containment_is_known() {
    let c = change(State::Checking { head: HEAD });
    for contains_base in [Status::Unknown, Status::Pending] {
        let mut f = facts();
        f.contains_base = contains_base;
        f.ci.status = Status::Pending;
        let next = decide(&c, &f, &heard());
        assert_eq!(next.change.state, c.state);
        assert_eq!(next.change.updates, 0);
        assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    }
}

#[test]
fn a_head_already_behind_waits_for_ci_read_at_that_head() {
    let c = change(State::Checking { head: HEAD });
    let mut f = facts();
    f.contains_base = Status::Failed;
    f.ci = Ci { head: NEW, status: Status::Pending };
    let next = decide(&c, &f, &heard());
    assert_eq!(next.change.state, c.state);
    assert_eq!(next.change.updates, 0);
    assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
}

#[test]
fn a_clean_update_waits_for_its_ci_though_the_base_moves_again() {
    let mut c = change(State::Checking { head: HEAD });
    c.clean = Box::new([NEW]);
    c.updates = 1;
    let mut f = facts();
    f.contains_base = Status::Failed;
    f.ci.status = Status::Pending;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.change.state, c.state);
    assert_eq!(next.change.updates, 1);
    assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
}

#[test]
fn an_unchecked_head_leaves_its_last_update_for_its_turn() {
    for updates in [LIMITS.updates - 1, LIMITS.updates, u32::MAX] {
        let mut c = change(State::Checking { head: HEAD });
        c.updates = updates;
        let mut f = facts();
        f.contains_base = Status::Failed;
        f.ci.status = Status::Pending;
        let next = decide(&c, &f, &heard());
        assert_eq!(next.change.state, c.state);
        assert_eq!(next.change.updates, updates);
        assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    }
}

#[test]
fn a_clean_update_preserves_the_turn_and_carries_a_review() {
    let gate = Gate { number: 9, kind: GateKind::Agent, blocking: true, freshness: Freshness::Clean, eager: false };
    let mut c = change(State::First { head: HEAD, ready_since: at(0) });
    c.gates = Box::new([gate]);
    c.ready_since = Some(at(0));
    c.owns_turn = true;
    let mut f = facts();
    f.contains_base = Status::Failed;
    f.gates = Box::new([GateReport { number: 9, head: HEAD, status: Status::Passed }]);
    let one = decide(&c, &f, &heard());
    assert_eq!(one.decision, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
    assert_eq!(one.change.state, State::Updating { from: HEAD, base: BASE, ready_since: at(0) });
    let mut h = heard();
    h.effect = EffectResult::Made;
    f.branch = Some(NEW);
    f.pull = Pull::Open { head: NEW, base: BASE };
    f.ci = Ci { head: NEW, status: Status::Pending };
    let two = decide(&one.change, &f, &h);
    assert_eq!(two.change.state, State::Checking { head: NEW });
    assert_eq!(two.change.clean.as_ref(), &[HEAD]);
    f.ci.status = Status::Passed;
    let three = decide(&two.change, &f, &heard());
    let four = decide(&three.change, &f, &heard());
    assert_eq!(four.change.state, State::First { head: NEW, ready_since: at(0) });
    assert_eq!(four.decision, Decision::Ready);
}
#[test]
fn a_conflict_leaves_the_queue_and_reasks_gates_after_resolution() {
    let mut c = change(State::Updating { from: HEAD, base: BASE, ready_since: at(0) });
    c.owns_turn = true;
    c.ready_since = Some(at(0));
    let mut h = heard();
    h.effect = EffectResult::Conflict;
    let one = decide(&c, &facts(), &h);
    assert_eq!(one.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert_eq!(one.change.state, State::Resolving { requested: true, base: BASE });
    assert!(!one.change.owns_turn);
    h.effect = EffectResult::None;
    h.delegate = Status::Passed;
    let mut f = facts();
    f.branch = Some(NEW);
    let two = decide(&one.change, &f, &h);
    assert_eq!(two.change.state, State::Checking { head: NEW });
    assert!(two.change.clean.is_empty());
}
#[test]
fn an_update_conflict_resolves_after_release_and_keeps_its_resolution_bound() {
    let updating = State::Updating { from: HEAD, base: BASE, ready_since: at(0) };
    let held = change(State::Held { was: Box::new(updating.clone()), why: Hold::Failed });
    let lifted = released(&held, at(2));
    let resumed = step(&lifted, &facts(), &heard(), at(3), &LIMITS);
    assert_eq!(resumed.change.state, updating);
    let mut h = heard();
    h.effect = EffectResult::Conflict;
    for mut c in [change(updating), resumed.change] {
        let one = decide(&c, &facts(), &h);
        assert_eq!(one.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
        assert_eq!(one.change.resolutions, 1);
        assert!(!one.change.owns_turn);
        c.resolutions = LIMITS.resolutions;
        assert_eq!(decide(&c, &facts(), &h).decision, Decision::Hold(Hold::Resolutions));
    }
}
#[test]
fn a_persons_request_in_first_repairs_with_its_reviewed_head() {
    for (request, repairs, contains_base, expected, count) in [
        (Some(NEW), 0, Status::Passed, Decision::Delegate(Delegate::Repair(Repair::Review { head: NEW })), 1),
        (Some(NEW), LIMITS.repairs, Status::Passed, Decision::Hold(Hold::Repairs), LIMITS.repairs),
        (Some(NEW), 0, Status::Failed, Decision::Effect(Effect::Update { head: HEAD, base: BASE }), 0),
        (None, 0, Status::Passed, Decision::Effect(Effect::Merge { head: HEAD, base: BASE }), 0),
    ] {
        let mut c = change(State::First { head: HEAD, ready_since: at(0) });
        c.repairs = repairs;
        let mut f = facts();
        f.contains_base = contains_base;
        let h = Heard { review_request: request, ..heard() };
        let next = decide(&c, &f, &h);
        assert_eq!(next.decision, expected, "{request:?}, {repairs}, {contains_base:?}");
        assert_eq!(next.change.repairs, count);
    }
}

#[test]
fn a_failed_check_takes_the_base_before_a_repair_and_both_bounds_hold() {
    let mut f = facts();
    f.ci.status = Status::Failed;
    f.contains_base = Status::Failed;
    let one = decide(&change(State::Checking { head: HEAD }), &f, &heard());
    assert_eq!(one.decision, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
    f.contains_base = Status::Passed;
    let two = decide(&change(State::Checking { head: HEAD }), &f, &heard());
    assert_eq!(two.decision, Decision::Delegate(Delegate::Repair(Repair::Ci)));
    let mut capped = change(State::Checking { head: HEAD });
    capped.repairs = LIMITS.repairs;
    assert_eq!(decide(&capped, &f, &heard()).decision, Decision::Hold(Hold::Repairs));
    capped.state = State::Updating { from: HEAD, base: BASE, ready_since: at(0) };
    capped.resolutions = LIMITS.resolutions;
    let mut h = heard();
    h.effect = EffectResult::Conflict;
    assert_eq!(decide(&capped, &f, &h).decision, Decision::Hold(Hold::Resolutions));
}
#[test]
fn a_failed_clean_update_is_a_semantic_repair_and_a_broken_base_gets_one_queue_repair() {
    let mut c = change(State::Checking { head: HEAD });
    c.clean = Box::new([NEW]);
    let mut f = facts();
    f.ci.status = Status::Failed;
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Delegate(Delegate::Repair(Repair::Semantic)));
    c.state = State::First { head: HEAD, ready_since: at(0) };
    f.ci.status = Status::Passed;
    f.base_ci.status = Status::Failed;
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::QueueRepair);
    f.queue_repair_active = true;
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Wait { until: None });
}

#[test]
fn a_semantic_repair_keeps_clean_predecessors_until_its_push_is_taken() {
    let mut c = change(State::Checking { head: HEAD });
    c.clean = Box::new([NEW]);
    let mut f = facts();
    f.ci.status = Status::Failed;
    let due = decide(&c, &f, &heard());
    assert_eq!(due.decision, Decision::Delegate(Delegate::Repair(Repair::Semantic)));
    assert_eq!(due.change.clean.as_ref(), &[NEW]);
    f.branch = Some(MERGE);
    let pushed = decide(&due.change, &f, &Heard { delegate: Status::Passed, ..heard() });
    assert_eq!(pushed.change.last_head, Some(MERGE));
    assert_eq!(pushed.change.state, State::Checking { head: MERGE });
    assert!(pushed.change.clean.is_empty());
}

#[test]
fn ci_and_gate_repairs_clear_clean_predecessors_when_due() {
    for why in [Repair::Ci, Repair::Gate(9), Repair::Review { head: NEW }] {
        let mut c = change(State::Repairing { requested: false, why });
        c.clean = Box::new([NEW]);
        let mut f = facts();
        f.ci.status = Status::Failed;
        let due = decide(&c, &f, &heard());
        assert_eq!(due.decision, Decision::Delegate(Delegate::Repair(why)));
        assert!(due.change.clean.is_empty(), "{why:?}");
    }
}

#[test]
fn release_reopens_a_closed_pull_and_recreates_a_deleted_branch() {
    let mut c = change(State::Held { was: Box::new(State::Checking { head: HEAD }), why: Hold::PullClosed });
    let mut h = heard();
    h.released = true;
    let one = decide(&c, &facts(), &h);
    assert_eq!(one.decision, Decision::Effect(Effect::Reopen));
    assert_eq!(one.change.state, State::Reopening { requested: true, retarget: false });
    let mut f = facts();
    f.pull = Pull::Closed;
    assert_eq!(decide(&one.change, &f, &heard()).decision, Decision::Wait { until: Some(at(31)) });
    c.state = State::Held { was: Box::new(State::Checking { head: HEAD }), why: Hold::BranchMissing };
    let two = decide(&c, &f, &h);
    assert_eq!(two.decision, Decision::Effect(Effect::CreateBranch { head: HEAD }));
    assert_eq!(two.change.state, State::Recreating { head: HEAD });
}
#[test]
fn a_released_opening_move_opens_before_checking_the_new_head() {
    for way in [crate::Way::Queue, crate::Way::Turn] {
        let mut c = change(State::Held { was: Box::new(State::Opening { requested: true }), why: Hold::BranchMoved });
        c.way = way;
        let mut f = Facts { branch: Some(NEW), pull: Pull::Missing, ..facts() };
        let lifted = decide(&c, &f, &Heard { released: true, ..heard() });
        assert_eq!(lifted.change.last_head, Some(NEW));
        assert_eq!(lifted.change.state, State::Opening { requested: false });
        let opening = decide(&lifted.change, &f, &heard());
        assert_eq!(opening.decision, Decision::Effect(Effect::Open));
        assert_eq!(opening.change.state, State::Opening { requested: true });
        f.pull = Pull::Open { head: NEW, base: BASE };
        let opened = decide(&opening.change, &f, &heard());
        let expected = match way {
            crate::Way::Queue => State::Checking { head: NEW },
            crate::Way::Turn => State::Gating { head: NEW, asked: Box::new([]) },
        };
        assert_eq!(opened.change.state, expected);
        assert_eq!(opened.decision, Decision::None);
    }
}

#[test]
fn a_released_opening_move_finds_the_pull_already_open() {
    let c = change(State::Held { was: Box::new(State::Opening { requested: false }), why: Hold::BranchMoved });
    let f = Facts { branch: Some(NEW), pull: Pull::Open { head: NEW, base: BASE }, ..facts() };
    let next = decide(&c, &f, &Heard { released: true, ..heard() });
    assert_eq!(next.change.last_head, Some(NEW));
    assert_eq!(next.change.state, State::Checking { head: NEW });
    assert_eq!(next.decision, Decision::None);
}

#[test]
fn a_requesters_branch_move_during_opening_still_opens_the_pull() {
    for requested in [false, true] {
        let c = change(State::Opening { requested });
        let f = Facts { pull: Pull::Missing, ..moved_branch(true) };
        let next = decide(&c, &f, &heard());
        assert_eq!(next.change.last_head, Some(NEW));
        assert_eq!(next.change.state, State::Opening { requested: true });
        assert_eq!(next.decision, Decision::Effect(Effect::Open));
    }
}

#[test]
fn a_released_missing_requester_branch_without_a_head_has_no_producer() {
    let mut c = change(State::Held { was: Box::new(State::Opening { requested: false }), why: Hold::BranchMissing });
    c.last_head = None;
    let mut f = Facts { requester_branch: true, branch: None, pull: Pull::Missing, ..facts() };
    let lifted = decide(&c, &f, &Heard { released: true, ..heard() });
    assert_eq!(lifted.change.state, State::Opening { requested: false });
    let missing = decide(&lifted.change, &f, &heard());
    assert_eq!(missing.decision, Decision::Hold(Hold::BranchMissing));
    f.branch = Some(NEW);
    let lifted = decide(&missing.change, &f, &Heard { released: true, ..heard() });
    let opening = decide(&lifted.change, &f, &heard());
    assert_eq!(opening.change.last_head, Some(NEW));
    assert_eq!(opening.decision, Decision::Effect(Effect::Open));
    assert_eq!(opening.change.state, State::Opening { requested: true });

    f.requester_branch = false;
    f.branch = None;
    let produced = decide(&c, &f, &Heard { released: true, ..heard() });
    assert_eq!(produced.change.state, State::Producing { requested: false });
}

#[test]
fn a_release_never_checks_or_gates_without_an_open_pull() {
    let authority = crate::AuthorityHold {
        recovery: crate::AuthorityRecovery { task: 2, spec: 0, hold: 3, tasks: 7 },
        lack: crate::AuthorityLack::Tasks,
        had: 6,
        released: false,
    };
    for way in [crate::Way::Queue, crate::Way::Turn] {
        for (was, _) in movable_phases() {
            for why in [
                Hold::Failed,
                Hold::Repairs,
                Hold::Rejected,
                Hold::BranchMoved,
                Hold::Stalled,
                Hold::Resolutions,
                Hold::Updates,
                Hold::BranchMissing,
                Hold::PullClosed,
                Hold::Retargeted,
                Hold::Questions,
                Hold::Authority(authority),
            ] {
                for pull in [Pull::Missing, Pull::Closed] {
                    let mut c = change(State::Held { was: Box::new(was.clone()), why });
                    c.way = way;
                    let f = Facts { pull, ..facts() };
                    let next = decide(&c, &f, &Heard { released: true, ..heard() });
                    assert!(
                        !matches!(next.change.state, State::Checking { .. } | State::Gating { .. }),
                        "{way:?} {was:?} {why:?} {pull:?}: {:?}",
                        next.change.state
                    );
                }
            }
        }
    }
}

#[test]
fn a_reopened_pull_keeps_its_target_when_the_landing_tip_advances() {
    let c = change(State::Reopening { requested: true, retarget: false });
    let mut f = facts();
    f.base_tip = NEW;
    f.expected_base = NEW;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.change.state, State::Checking { head: HEAD });
    assert_eq!(next.decision, Decision::None);

    f.drift = Some(Hold::Retargeted);
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Hold(Hold::Retargeted));
}

#[test]
fn drift_writer_slots_and_stalls_do_not_write_over_a_branch() {
    let mut f = facts();
    f.writer_taken = true;
    assert_eq!(
        decide(&change(State::First { head: HEAD, ready_since: at(0) }), &f, &heard()).decision,
        Decision::Wait { until: Some(at(30)) }
    );
    f.writer_taken = false;
    f.drift = Some(Hold::BranchMoved);
    assert_eq!(
        decide(&change(State::First { head: HEAD, ready_since: at(0) }), &f, &heard()).decision,
        Decision::Hold(Hold::BranchMoved)
    );
    f.drift = None;
    let held = step(&change(State::Checking { head: HEAD }), &f, &heard(), at(31), &LIMITS);
    assert_eq!(held.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
    f.ci.status = Status::Pending;
    assert_eq!(
        step(&change(State::Checking { head: HEAD }), &f, &heard(), at(31), &LIMITS).decision,
        Decision::Hold(Hold::Stalled)
    );
}
fn movable_phases() -> [(State, Option<Wall>); 7] {
    [
        (State::Opening { requested: false }, Some(at(30))),
        (State::Opening { requested: true }, Some(at(30))),
        (State::Checking { head: HEAD }, Some(at(30))),
        (State::Gating { head: HEAD, asked: Box::new([9]) }, Some(at(30))),
        (State::Queued { head: HEAD, ready_since: at(0) }, Some(at(30))),
        (State::First { head: HEAD, ready_since: at(0) }, Some(at(30))),
        (State::Ready { head: HEAD, ready_since: at(0) }, None),
    ]
}

fn moved_branch(requester_branch: bool) -> Facts {
    Facts { requester_branch, branch: Some(NEW), asking: false, ..facts() }
}

#[test]
fn a_move_of_a_requesters_held_branch_is_its_new_head_in_each_phase() {
    for (way, expected) in [
        (crate::Way::Queue, State::Checking { head: NEW }),
        (crate::Way::Turn, State::Gating { head: NEW, asked: Box::new([]) }),
    ] {
        for (state, _) in movable_phases() {
            let mut c = change(state);
            c.way = way;
            c.clean = Box::new([BASE]);
            c.gate_failures = Box::new([GateFailure { number: 9, count: 1 }]);
            c.owns_turn = true;
            c.repairs = LIMITS.repairs;
            c.resolutions = LIMITS.resolutions;
            c.updates = LIMITS.updates;
            let next = decide(&c, &moved_branch(true), &heard());
            assert_eq!(next.change.state, expected, "{:?}", c.state);
            assert_eq!(next.change.last_head, Some(NEW));
            assert_eq!(next.change.since, at(1));
            assert!(next.change.clean.is_empty());
            assert!(next.change.gate_failures.is_empty());
            assert!(!next.change.owns_turn);
            assert_eq!(next.change.repairs, c.repairs);
            assert_eq!(next.change.resolutions, c.resolutions);
            assert_eq!(next.change.updates, c.updates);
            assert_eq!(next.decision, Decision::None);
        }
    }
}

#[test]
fn a_move_of_its_own_branch_still_holds_it() {
    for (state, _) in movable_phases() {
        let c = change(state);
        let next = decide(&c, &moved_branch(false), &heard());
        assert_eq!(next.change.state, State::Held { was: Box::new(c.state), why: Hold::BranchMoved });
        assert_eq!(next.decision, Decision::Hold(Hold::BranchMoved));
    }
}

#[test]
fn a_requesters_branch_with_drift_or_a_writer_is_not_taken() {
    for (state, _) in movable_phases() {
        let c = change(state);
        for why in [Hold::BranchMoved, Hold::Retargeted, Hold::BranchMissing] {
            let mut f = moved_branch(true);
            f.drift = Some(why);
            let next = decide(&c, &f, &heard());
            assert_eq!(next.change.state, State::Held { was: Box::new(c.state.clone()), why });
            assert_eq!(next.change.last_head, Some(HEAD));
            assert_eq!(next.decision, Decision::Hold(why));
        }
    }
    for (state, until) in movable_phases() {
        let c = change(state);
        let mut f = moved_branch(true);
        f.writer_taken = true;
        let next = decide(&c, &f, &heard());
        assert_eq!(next.change, c);
        assert_eq!(next.decision, Decision::Wait { until });
    }
}

#[test]
fn a_held_change_on_its_requesters_branch_takes_the_head_at_release() {
    let mut c = change(State::Checking { head: HEAD });
    c.repairs = LIMITS.repairs;
    let mut f = facts();
    f.requester_branch = true;
    f.ci.status = Status::Failed;
    let held = decide(&c, &f, &heard()).change;
    f.branch = Some(NEW);
    let waiting = decide(&held, &f, &heard());
    assert_eq!(waiting.change, held);
    assert_eq!(waiting.decision, Decision::None);
    assert_eq!(waiting.change.state, State::Held { was: Box::new(c.state), why: Hold::Repairs });

    let lifted = released(&waiting.change, at(1));
    let reset = decide(&lifted, &f, &heard());
    let taken = decide(&reset.change, &f, &heard());
    assert_eq!(taken.change.last_head, Some(NEW));
    assert_eq!(taken.change.state, State::Checking { head: NEW });
    assert_eq!(taken.change.repairs, 0);
    assert_eq!(taken.decision, Decision::None);
    f.ci = Ci { head: NEW, status: Status::Pending };
    assert_eq!(decide(&taken.change, &f, &heard()).decision, Decision::Wait { until: Some(at(31)) });
    f.ci.status = Status::Passed;
    assert_eq!(decide(&taken.change, &f, &heard()).change.state, State::Gating { head: NEW, asked: Box::new([]) });
}

#[test]
fn a_requesters_branch_move_during_landing_keeps_the_merge_head() {
    let c = change(State::Landing { head: HEAD });
    let next = decide(&c, &moved_branch(true), &heard());
    assert_eq!(next.change, c);
    assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
}

#[test]
fn generated_phase_cells_decide_the_same_from_the_same_snapshot() {
    let states = [
        State::Producing { requested: false },
        State::Producing { requested: true },
        State::Opening { requested: false },
        State::Opening { requested: true },
        State::Recreating { head: HEAD },
        State::Reopening { requested: true, retarget: false },
        State::Checking { head: HEAD },
        State::Gating { head: HEAD, asked: Box::new([]) },
        State::CancellingGates { head: HEAD },
        State::Queued { head: HEAD, ready_since: at(0) },
        State::First { head: HEAD, ready_since: at(0) },
        State::Updating { from: HEAD, base: BASE, ready_since: at(0) },
        State::Resolving { requested: true, base: BASE },
        State::Repairing { requested: true, why: Repair::Ci },
        State::Landing { head: HEAD },
        State::Landed { merge: MERGE, since: at(1) },
        State::Held { was: Box::new(State::Checking { head: HEAD }), why: Hold::Stalled },
    ];
    let statuses = [Status::Unknown, Status::Pending, Status::Passed, Status::Failed];
    let effects =
        [EffectResult::None, EffectResult::Pending, EffectResult::Made, EffectResult::Conflict, EffectResult::Failed];
    for state in states {
        for status in statuses {
            for effect in effects {
                let mut f = facts();
                f.ci.status = status;
                f.mergeable = status;
                let mut h = heard();
                h.delegate = status;
                h.effect = effect;
                let c = change(state.clone());
                assert_eq!(decide(&c, &f, &h), decide(&c, &f, &h));
            }
        }
    }
}
#[test]
fn queue_age_precedes_later_priority_and_priority_breaks_young_ties() {
    let ready = [
        Ready { task: 1, priority: 20, since: at(1) },
        Ready { task: 2, priority: 10, since: at(15) },
        Ready { task: 3, priority: 1, since: at(17) },
    ];
    assert_eq!(first(&ready, Duration::from_secs(10), at(20)), Some(1));
    assert_eq!(first(&ready, Duration::from_secs(10), at(18)), Some(1));
    assert_eq!(first(&ready[1..], Duration::from_secs(10), at(18)), Some(3));
}
#[test]
fn generated_queue_order_agrees_with_an_independent_rank_statement() {
    for seed in 0..200_u64 {
        let now = at(100);
        let window = Duration::from_secs(20);
        let mut ready = [Ready { task: 0, priority: 0, since: at(0) }; 4];
        for (index, item) in ready.iter_mut().enumerate() {
            let number = u64::try_from(index).expect("index fits");
            *item = Ready {
                task: number + 1,
                priority: i32::try_from((seed.wrapping_mul(17).wrapping_add(number * 13)) % 9).expect("rank fits"),
                since: at(60 + ((seed.wrapping_mul(31).wrapping_add(number * 11)) % 39)),
            };
        }
        let mut expected = ready[0];
        for item in &ready[1..] {
            let item_aged = now.as_nanos() - item.since.as_nanos() >= window.as_nanos();
            let old_aged = now.as_nanos() - expected.since.as_nanos() >= window.as_nanos();
            let item_wins = if item_aged != old_aged {
                item_aged
            } else if item_aged && item.since != expected.since {
                item.since < expected.since
            } else if item.priority != expected.priority {
                item.priority < expected.priority
            } else if item.since != expected.since {
                item.since < expected.since
            } else {
                item.task < expected.task
            };
            if item_wins {
                expected = *item;
            }
        }
        assert_eq!(first(&ready, window, now), Some(expected.task), "seed {seed}");
    }
}

#[test]
fn a_pending_wait_inside_the_settle_window_ends_with_it_and_never_stalls() {
    let mut f = facts();
    f.mergeable = Status::Pending;
    f.settling = Some(at(50));
    let decision =
        step(&change(State::First { head: HEAD, ready_since: at(0) }), &f, &heard(), at(40), &LIMITS).decision;
    assert_eq!(decision, Decision::Wait { until: Some(at(50)) });
}

#[test]
fn a_pending_wait_past_the_settle_window_stalls_as_any_wait() {
    let mut f = facts();
    f.mergeable = Status::Pending;
    f.settling = Some(at(40));
    let decision =
        step(&change(State::First { head: HEAD, ready_since: at(0) }), &f, &heard(), at(40), &LIMITS).decision;
    assert_eq!(decision, Decision::Hold(Hold::Stalled));
}

fn reviewing() -> Change {
    let mut c = change(State::Gating { head: HEAD, asked: Box::new([9]) });
    c.gates = Box::new([Gate {
        number: 9,
        kind: GateKind::Agent,
        blocking: true,
        freshness: Freshness::Exact,
        eager: false,
    }]);
    c
}

#[test]
fn a_failed_review_is_asked_again_at_the_same_head() {
    let failed = crate::review_failed(&reviewing(), 9, HEAD, at(2), &LIMITS);
    assert_eq!(failed.gate_failures.as_ref(), &[GateFailure { number: 9, count: 1 }]);
    let next = step(&failed, &facts(), &heard(), at(3), &LIMITS);
    assert_eq!(next.decision, Decision::Delegate(Delegate::Gate { number: 9, head: HEAD }));
    assert_eq!(next.change.repairs, 0);
    assert_eq!(next.change.state, State::Gating { head: HEAD, asked: Box::new([9]) });
    assert!(
        decide(&next.change, &facts(), &heard()).decision == Decision::Wait { until: Some(at(32)) },
        "a repeated snapshot does not request a third review"
    );
}

#[test]
fn a_second_failed_review_holds_the_change() {
    let failed = crate::review_failed(&reviewing(), 9, HEAD, at(2), &LIMITS);
    let retry = step(&failed, &facts(), &heard(), at(3), &LIMITS);
    let twice = crate::review_failed(&retry.change, 9, HEAD, at(4), &LIMITS);
    let next = step(&twice, &facts(), &heard(), at(5), &LIMITS);
    assert_eq!(next.decision, Decision::Hold(Hold::Failed));
    assert_eq!(next.change.repairs, 0);
    assert_eq!(next.change.gate_failures.as_ref(), &[GateFailure { number: 9, count: 2 }]);
    assert_eq!(decide(&next.change, &facts(), &heard()).decision, Decision::None);
}

#[test]
fn a_new_head_starts_its_review_failures_at_zero() {
    let mut c = reviewing();
    c.gate_failures = Box::new([GateFailure { number: 9, count: 2 }]);
    c.state = State::Checking { head: NEW };
    let mut f = facts();
    f.branch = Some(NEW);
    f.pull = Pull::Open { head: NEW, base: BASE };
    f.ci.head = NEW;
    let checked = decide(&c, &f, &heard());
    assert!(checked.change.gate_failures.is_empty());
    let review = decide(&checked.change, &f, &heard());
    let failed = crate::review_failed(&review.change, 9, NEW, at(2), &LIMITS);
    assert_eq!(failed.gate_failures.as_ref(), &[GateFailure { number: 9, count: 1 }]);
    assert_eq!(decide(&failed, &f, &heard()).decision, Decision::Delegate(Delegate::Gate { number: 9, head: NEW }));
}

#[test]
fn each_gate_has_its_own_failure_count() {
    let mut c = reviewing();
    c.gates = Box::new([c.gates[0], Gate { number: 10, ..c.gates[0] }]);
    c.state = State::Gating { head: HEAD, asked: Box::new([9, 10]) };
    let one = crate::review_failed(&c, 9, HEAD, at(2), &LIMITS);
    let two = crate::review_failed(&one, 10, HEAD, at(3), &LIMITS);
    assert_eq!(
        two.gate_failures.as_ref(),
        &[GateFailure { number: 9, count: 1 }, GateFailure { number: 10, count: 1 }]
    );
    assert_eq!(decide(&two, &facts(), &heard()).decision, Decision::Delegate(Delegate::Gate { number: 9, head: HEAD }));
}

#[test]
fn a_release_resets_failed_review_tries_before_any_forge_read() {
    let mut c =
        change(State::Held { was: Box::new(State::Gating { head: HEAD, asked: Box::new([]) }), why: Hold::Failed });
    c.gate_failures = Box::new([GateFailure { number: 1, count: 2 }]);
    c.repairs = LIMITS.repairs;
    let lifted = released(&c, at(5));
    assert_eq!(lifted.state, State::Gating { head: HEAD, asked: Box::new([]) });
    assert!(lifted.gate_failures.is_empty());
    assert_eq!(lifted.released, Some(Hold::Failed));
    assert_eq!(lifted.repairs, 0);
    assert_eq!(lifted.since, at(5));
    let decided = step(&lifted, &facts(), &heard(), at(10), &LIMITS);
    assert_eq!(decided.change.released, None);
    assert_eq!(decided.change.since, at(5), "a slow read does not restart the release's stall clock");
}

#[test]
fn a_cancel_ends_every_held_cause_unless_the_change_landed() {
    for why in [
        Hold::Failed,
        Hold::Repairs,
        Hold::Rejected,
        Hold::BranchMoved,
        Hold::Stalled,
        Hold::Resolutions,
        Hold::Updates,
        Hold::BranchMissing,
        Hold::PullClosed,
        Hold::Retargeted,
    ] {
        let c = change(State::Held { was: Box::new(State::Gating { head: HEAD, asked: Box::new([]) }), why });
        let mut h = heard();
        h.cancelled = true;
        let ended = decide(&c, &facts(), &h);
        assert_eq!(ended.decision, Decision::Cancel, "{why:?}");
        assert_eq!(ended.change.state, State::Ended, "{why:?}");
        let mut f = facts();
        f.pull = Pull::Merged { commit: MERGE };
        let landed = decide(&c, &f, &h);
        assert_eq!(landed.change.state, State::Landed { merge: MERGE, since: at(1) });
        assert_eq!(landed.decision, Decision::Finish { merge: MERGE });
    }
}

#[test]
fn every_delegate_checks_authority_before_becoming_pending() {
    use crate::{AuthorityHold, AuthorityLack, AuthorityRecovery};
    let cause = AuthorityHold {
        recovery: AuthorityRecovery { task: 2, spec: 0, hold: 3, tasks: 7 },
        lack: AuthorityLack::Tasks,
        had: 6,
        released: false,
    };
    let mut standing = heard();
    standing.lacking = Some(cause);
    let mut f = facts();
    f.ci.status = Status::Failed;
    for state in [
        State::Producing { requested: false },
        State::Repairing { requested: false, why: Repair::Ci },
        State::Resolving { requested: false, base: BASE },
    ] {
        let c = change(state);
        if let State::Producing { .. } = c.state {
            f.branch = None;
        } else {
            f.branch = Some(HEAD);
        }
        let next = decide(&c, &f, &standing);
        assert_eq!(next.decision, Decision::Hold(Hold::Authority(cause)));
        assert_eq!(next.change.state, State::Held { was: Box::new(c.state), why: Hold::Authority(cause) });
        assert_eq!((next.change.repairs, next.change.resolutions), (0, 0));
    }
}

#[test]
fn a_bare_authority_release_holds_again_and_a_funded_one_retries() {
    use crate::{AuthorityHold, AuthorityLack, AuthorityRecovery};
    let mut f = facts();
    f.branch = None;
    for lack in [AuthorityLack::Tasks, AuthorityLack::Depth, AuthorityLack::Spend] {
        let cause = AuthorityHold {
            recovery: AuthorityRecovery { task: u64::MAX, spec: u64::MAX, hold: u64::MAX, tasks: u64::MAX },
            lack,
            had: 6,
            released: false,
        };
        let c =
            change(State::Held { was: Box::new(State::Producing { requested: false }), why: Hold::Authority(cause) });
        let lifted = released(&c, at(1));
        let mut h = heard();
        h.lacking = Some(cause);
        let bare = decide(&lifted, &f, &h);
        assert_eq!(bare.decision, Decision::Hold(Hold::Authority(AuthorityHold { released: true, ..cause })));
        h.lacking = None;
        let funded = decide(&lifted, &f, &h);
        let retry = decide(&funded.change, &f, &h);
        assert_eq!(retry.decision, Decision::Delegate(Delegate::Produce));
    }
}
mod turn;

#[test]
fn ci_failing_while_a_review_runs_cancels_it_then_repairs() {
    let c = change(State::Gating { head: HEAD, asked: Box::new([9]) });
    let mut f = facts();
    f.ci.status = Status::Failed;
    f.live_gate = Some(HEAD);
    let cancelled = decide(&c, &f, &heard());
    assert_eq!(cancelled.decision, Decision::CancelGates { head: HEAD });
    assert_eq!(cancelled.change.state, State::CancellingGates { head: HEAD });
    assert_eq!(cancelled.change.repairs, 0);
    let waiting = decide(&cancelled.change, &f, &heard());
    assert_eq!(waiting.decision, Decision::Wait { until: Some(at(30)) });
    assert_eq!(waiting.change.repairs, 0);
    f.live_gate = None;
    let repair = decide(&waiting.change, &f, &heard());
    assert_eq!(repair.decision, Decision::Delegate(Delegate::Repair(Repair::Ci)));
    assert_eq!(repair.change.repairs, 1);
    let again = decide(&repair.change, &f, &heard());
    assert_eq!(again.decision, Decision::Wait { until: Some(at(31)) });
    assert_eq!(again.change.repairs, 1);
}

#[test]
fn ci_failing_with_no_live_gate_repairs_at_once() {
    let c = change(State::Gating { head: HEAD, asked: Box::new([]) });
    let mut f = facts();
    f.ci.status = Status::Failed;
    let repair = decide(&c, &f, &heard());
    assert_eq!(repair.decision, Decision::Delegate(Delegate::Repair(Repair::Ci)));
    assert_eq!(repair.change.repairs, 1);
}

#[test]
fn a_live_gate_keeps_its_delegate_slot_until_it_ends() {
    let mut c = change(State::Gating { head: HEAD, asked: Box::new([9]) });
    let gate = Gate { number: 9, kind: GateKind::Agent, blocking: true, freshness: Freshness::Exact, eager: false };
    c.gates = Box::new([gate, Gate { number: 10, ..gate }]);
    let mut f = facts();
    f.live_gate = Some(HEAD);
    let waiting = decide(&c, &f, &heard());
    assert_eq!(waiting.decision, Decision::Wait { until: Some(at(30)) });
    assert_eq!(waiting.change.state, c.state);
    f.live_gate = None;
    f.gates = Box::new([GateReport { number: 9, head: HEAD, status: Status::Passed }]);
    let second = decide(&waiting.change, &f, &heard());
    assert_eq!(second.decision, Decision::Delegate(Delegate::Gate { number: 10, head: HEAD }));
}

#[test]
fn cancelled_gates_preserve_the_semantic_repair_and_its_bound() {
    let mut c = change(State::Gating { head: HEAD, asked: Box::new([9]) });
    c.clean = Box::new([BASE]);
    let mut f = facts();
    f.ci.status = Status::Failed;
    f.live_gate = Some(HEAD);
    let cancelled = decide(&c, &f, &heard());
    f.live_gate = None;
    let repair = decide(&cancelled.change, &f, &heard());
    assert_eq!(repair.decision, Decision::Delegate(Delegate::Repair(Repair::Semantic)));
    c.repairs = LIMITS.repairs;
    let cancelled = decide(&c, &Facts { live_gate: Some(HEAD), asking: false, ..f.clone() }, &heard());
    let held = decide(&cancelled.change, &f, &heard());
    assert_eq!(held.decision, Decision::Hold(Hold::Repairs));
    assert_eq!(held.change.repairs, LIMITS.repairs);
}

#[test]
fn a_live_advisory_gate_does_not_hold_a_green_head() {
    let mut c = change(State::Gating { head: HEAD, asked: Box::new([9]) });
    c.gates = Box::new([Gate {
        number: 9,
        kind: GateKind::Agent,
        blocking: false,
        freshness: Freshness::Exact,
        eager: false,
    }]);
    let mut f = facts();
    f.live_gate = Some(HEAD);
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Ready);
    f.ci.status = Status::Failed;
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::CancelGates { head: HEAD });
}

mod green;
mod merge;
mod relay;
mod verifying;

#[test]
fn a_changes_questions_default_to_none() {
    let questions = Questions::default();
    assert_eq!(questions.route, Route::None);
    assert_eq!(questions.taken, 0);
    assert!(questions.open.is_empty());
}

#[test]
fn the_open_bound_leaves_room_for_the_one_past_the_bound() {
    // The reference's bound (draft/reference-code-production.md, section 8).
    const QUESTIONS: usize = 4;
    const { assert!(QUESTIONS_OPEN > QUESTIONS) };
}

fn pending_facts(pending: Box<[Pending]>) -> Facts {
    Facts { pending, branch: None, pull: Pull::Missing, kept: true, ..facts() }
}

#[test]
fn a_change_with_a_pending_change_without_a_head_waits_with_no_stall() {
    let c = change(State::Producing { requested: false });
    let mut f = pending_facts(Box::new([Pending { task: 11, kind: PendingKind::Change { head: None, kept: false } }]));
    let waiting = decide(&c, &f, &heard());
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    assert_eq!(waiting.change.state, State::Producing { requested: false });
    assert_eq!(waiting.change.early, Early::None);
    let waiting = step(&waiting.change, &f, &heard(), at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    assert_eq!(waiting.change.state, State::Producing { requested: false });
    f.pending = Box::new([Pending { task: 11, kind: PendingKind::Change { head: Some(HEAD), kept: false } }]);
    let started = step(&waiting.change, &f, &heard(), at(61), &LIMITS);
    assert_eq!(started.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(started.change.since, at(61));
    assert_eq!(step(&started.change, &f, &heard(), at(62), &LIMITS).decision, Decision::Wait { until: Some(at(91)) });
}

#[test]
fn dependencies_finishing_before_an_early_start_leave_a_fresh_producer_stall() {
    let c = change(State::Producing { requested: false });
    let mut f = pending_facts(Box::new([Pending { task: 11, kind: PendingKind::Other }]));
    let waiting = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    f.pending = Box::new([]);
    let started = step(&waiting.change, &f, &heard(), at(90), &LIMITS);
    assert_eq!(started.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(started.change.early, Early::None);
    assert_eq!(started.change.since, at(90));
    assert_eq!(step(&started.change, &f, &heard(), at(91), &LIMITS).decision, Decision::Wait { until: Some(at(120)) });
}

#[test]
fn a_change_whose_pending_changes_have_heads_starts_early_from_them() {
    let c = change(State::Producing { requested: false });
    let f = pending_facts(Box::new([
        Pending { task: 19, kind: PendingKind::Change { head: Some(NEW), kept: false } },
        Pending { task: 12, kind: PendingKind::Elsewhere },
        Pending { task: 11, kind: PendingKind::Change { head: Some(HEAD), kept: false } },
    ]));
    let started = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(started.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(started.change.state, State::Producing { requested: true });
    assert_eq!(
        started.change.early,
        Early::Started(Box::new([StartedFrom { task: 19, head: NEW }, StartedFrom { task: 11, head: HEAD },]))
    );
    assert_eq!(started.change.since, at(60));
    let repeated = step(&started.change, &f, &heard(), at(61), &LIMITS);
    assert_eq!(repeated.decision, Decision::Wait { until: Some(at(90)) });
    let mut pushed = f;
    pushed.branch = Some(NEW);
    let opening = step(&repeated.change, &pushed, &heard(), at(62), &LIMITS);
    assert_eq!(opening.change.state, State::Opening { requested: false });
    assert_eq!(opening.change.early, started.change.early);
}

#[test]
fn a_pending_dependency_kept_by_its_own_dependency_blocks_the_early_start() {
    let c = change(State::Producing { requested: false });
    let f = pending_facts(Box::new([
        Pending { task: 11, kind: PendingKind::Change { head: Some(HEAD), kept: false } },
        Pending { task: 12, kind: PendingKind::Change { head: Some(NEW), kept: true } },
    ]));
    let waiting = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    assert_eq!(waiting.change.state, State::Producing { requested: false });
    assert_eq!(waiting.change.early, Early::None);
}

#[test]
fn a_pending_other_dependency_blocks_the_early_start() {
    let c = change(State::Producing { requested: false });
    let f = pending_facts(Box::new([
        Pending { task: 11, kind: PendingKind::Change { head: Some(HEAD), kept: false } },
        Pending { task: 12, kind: PendingKind::Other },
    ]));
    let waiting = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    assert_eq!(waiting.change.state, State::Producing { requested: false });
    assert_eq!(waiting.change.early, Early::None);
}

#[test]
fn only_elsewhere_dependencies_start_the_producer_with_no_heads() {
    let c = change(State::Producing { requested: false });
    let f = pending_facts(Box::new([Pending { task: 12, kind: PendingKind::Elsewhere }]));
    let started = decide(&c, &f, &heard());
    assert_eq!(started.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(started.change.state, State::Producing { requested: true });
    assert_eq!(started.change.early, Early::Started(Box::new([])));
}

#[test]
fn a_conflicting_early_start_cancels_its_producer_and_waits() {
    let c = change(State::Producing { requested: false });
    let mut f =
        pending_facts(Box::new([Pending { task: 11, kind: PendingKind::Change { head: Some(HEAD), kept: false } }]));
    let started = decide(&c, &f, &heard());
    let mut h = Heard { start_conflict: Some(HEAD), ..heard() };
    f.writer_taken = true;
    let cancelled = decide(&started.change, &f, &h);
    assert_eq!(cancelled.decision, Decision::CancelProducer);
    assert_eq!(cancelled.change.early, Early::Spent(Box::new([StartedFrom { task: 11, head: HEAD }])));
    assert_eq!(cancelled.change.state, State::Producing { requested: false });
    let waiting = step(&cancelled.change, &f, &h, at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    f.pending = Box::new([]);
    assert_eq!(step(&waiting.change, &f, &h, at(61), &LIMITS).decision, Decision::Wait { until: None });
    f.pending = Box::new([Pending { task: 11, kind: PendingKind::Change { head: Some(NEW), kept: false } }]);
    f.writer_taken = false;
    h.start_conflict = None;
    h.delegate = Status::Failed;
    let ended = step(&waiting.change, &f, &h, at(62), &LIMITS);
    assert_eq!(ended.decision, Decision::Wait { until: None });
    assert_eq!(ended.change.state, State::Producing { requested: false });
    assert_eq!(ended.change.early, cancelled.change.early);
    f.pending = Box::new([]);
    let replacement = step(&ended.change, &f, &h, at(63), &LIMITS);
    assert_eq!(replacement.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(replacement.change.since, at(63));
}

#[test]
fn a_failed_early_producer_holds_nothing_and_the_next_waits_for_the_dependencies() {
    let c = change(State::Producing { requested: false });
    let mut f =
        pending_facts(Box::new([Pending { task: 11, kind: PendingKind::Change { head: Some(HEAD), kept: false } }]));
    let started = decide(&c, &f, &heard());
    f.branch = Some(NEW);
    let h = Heard { delegate: Status::Failed, ..heard() };
    let failed = decide(&started.change, &f, &h);
    assert_eq!(failed.decision, Decision::Wait { until: None });
    assert_eq!(failed.change.early, Early::Spent(Box::new([StartedFrom { task: 11, head: HEAD }])));
    assert_eq!(failed.change.state, State::Producing { requested: false });
    let waiting = step(&failed.change, &f, &h, at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    f.pending = Box::new([]);
    let replacement = step(&waiting.change, &f, &h, at(61), &LIMITS);
    assert_eq!(replacement.decision, Decision::Delegate(Delegate::Produce));
    assert_eq!(replacement.change.since, at(61));
    assert_eq!(replacement.change.early, failed.change.early);
    assert_eq!(
        step(&replacement.change, &f, &heard(), at(62), &LIMITS).decision,
        Decision::Wait { until: Some(at(91)) }
    );
    let failed = step(&replacement.change, &f, &h, at(63), &LIMITS);
    assert_eq!(failed.decision, Decision::Hold(Hold::Failed));
    assert_eq!(
        failed.change.state,
        State::Held { was: Box::new(State::Producing { requested: true }), why: Hold::Failed }
    );
}

#[test]
fn a_change_never_starts_early_twice() {
    let mut c = change(State::Producing { requested: false });
    c.early = Early::Spent(Box::new([StartedFrom { task: 11, head: HEAD }]));
    let f = pending_facts(Box::new([Pending { task: 11, kind: PendingKind::Change { head: Some(NEW), kept: false } }]));
    let waiting = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    assert_eq!(waiting.change.state, State::Producing { requested: false });
    assert_eq!(waiting.change.early, c.early);
}

fn updating_change() -> Change {
    let mut c = change(State::Updating { from: HEAD, base: BASE, ready_since: at(0) });
    c.updates = 1;
    c.clean = Box::new([MERGE]);
    c
}

#[test]
fn an_update_let_go_updates_again_where_its_head_is_behind() {
    let c = updating_change();
    let mut f = facts();
    f.contains_base = Status::Failed;
    f.base_tip = NEW;
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::Effect(Effect::Update { head: HEAD, base: NEW }));
    assert_eq!(next.change.updates, 2);
    assert_eq!(next.change.state, State::Updating { from: HEAD, base: NEW, ready_since: at(0) });
    assert_eq!(next.change.since, at(1));
}

#[test]
fn an_update_let_go_holds_updates_at_its_bound() {
    let mut c = updating_change();
    c.updates = LIMITS.updates;
    let mut f = facts();
    f.contains_base = Status::Failed;
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Hold(Hold::Updates));
}

#[test]
fn an_update_let_go_goes_on_where_its_head_holds_the_base() {
    let c = updating_change();
    let next = decide(&c, &facts(), &heard());
    assert_eq!(next.change.state, State::Checking { head: HEAD });
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.clean, c.clean);
    assert_eq!(next.change.last_head, Some(HEAD));
    assert_eq!(next.change.since, at(1));
    assert_eq!(next.change.updates, 1);
}

#[test]
fn an_update_let_go_takes_a_moved_head_holding_the_base() {
    let mut c = updating_change();
    let mut f = facts();
    f.branch = Some(NEW);
    let next = decide(&c, &f, &heard());
    assert_eq!(next, decide(&c, &f, &Heard { effect: EffectResult::Made, ..heard() }));
    assert_eq!(next.change.state, State::Checking { head: NEW });
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.change.clean.as_ref(), &[MERGE, HEAD]);
    assert_eq!(next.change.last_head, Some(NEW));
    assert_eq!(next.change.since, at(1));
    c.clean = Box::new([MERGE; 4]);
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Hold(Hold::Updates));
}

#[test]
fn an_update_let_go_holds_a_moved_head_not_holding_the_base() {
    let mut f = facts();
    f.branch = Some(NEW);
    f.contains_base = Status::Failed;
    assert_eq!(decide(&updating_change(), &f, &heard()).decision, Decision::Hold(Hold::BranchMoved));
}

#[test]
fn an_update_let_go_waits_on_what_it_cannot_read() {
    let c = updating_change();
    for (branch, contains_base) in [
        (None, Status::Passed),
        (None, Status::Failed),
        (Some(HEAD), Status::Unknown),
        (Some(HEAD), Status::Pending),
        (Some(NEW), Status::Unknown),
        (Some(NEW), Status::Pending),
    ] {
        let f = Facts { branch, contains_base, ..facts() };
        let next = decide(&c, &f, &heard());
        assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
        assert_eq!(next.change, c);
        assert_eq!(step(&c, &f, &heard(), at(31), &LIMITS).decision, Decision::Hold(Hold::Stalled));
    }
}

#[test]
fn a_released_failed_update_decides_again() {
    let mut c = updating_change();
    c.updates = LIMITS.updates;
    c.state = State::Held { was: Box::new(c.state), why: Hold::Failed };
    let f = Facts { contains_base: Status::Failed, ..facts() };
    let lifted = released(&c, at(1));
    let resumed = decide(&lifted, &f, &heard());
    assert_eq!(resumed.decision, Decision::None);
    let next = decide(&resumed.change, &f, &heard());
    assert_eq!(next.decision, Decision::Effect(Effect::Update { head: HEAD, base: BASE }));
    assert_eq!(next.change.updates, 1);
}

#[test]
fn an_update_outstanding_still_waits() {
    let c = updating_change();
    let f = Facts { contains_base: Status::Failed, ..facts() };
    let next = decide(&c, &f, &Heard { effect: EffectResult::Pending, ..heard() });
    assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    assert_eq!(next.change, c);
}

mod close;

#[test]
fn runner_faults_run_again_within_the_head_bound() {
    use crate::Fault;
    for (fault, bound, rerun) in [
        (Fault::Runner { reruns: 0 }, 1, true),
        (Fault::Runner { reruns: 1 }, 1, false),
        (Fault::Runner { reruns: 0 }, 0, false),
        (Fault::Change, 1, false),
    ] {
        for state in [
            State::Checking { head: HEAD },
            State::Gating { head: HEAD, asked: Box::new([11]) },
            State::CancellingGates { head: HEAD },
        ] {
            for clean in [false, true] {
                let mut c = change(state.clone());
                c.owns_turn = true;
                if clean {
                    c.clean = Box::new([BASE]);
                }
                let mut f = facts();
                f.ci.status = Status::Failed;
                f.fault = fault;
                f.gates = Box::new([GateReport { number: 11, head: HEAD, status: Status::Passed }]);
                let pending = step(&c, &f, &Heard { effect: EffectResult::Pending, ..heard() }, at(5), &LIMITS);
                assert_eq!(pending.change, c);
                assert_eq!(pending.decision, Decision::Wait { until: Some(at(30)) });
                let next = step(&c, &f, &heard(), at(5), &Limits { reruns: bound, ..LIMITS });
                if rerun {
                    let mut expected = c.clone();
                    expected.since = at(5);
                    assert_eq!(next.change, expected);
                    assert_eq!(next.decision, Decision::Effect(Effect::Rerun { head: HEAD }));
                    let mut running = f.clone();
                    running.ci.status = Status::Pending;
                    running.fault = Fault::Change;
                    let waiting =
                        step(&next.change, &running, &Heard { effect: EffectResult::Made, ..heard() }, at(6), &LIMITS);
                    assert_eq!(waiting.change, next.change);
                    assert_eq!(waiting.decision, Decision::Wait { until: Some(at(35)) });
                } else {
                    let why = if clean { Repair::Semantic } else { Repair::Ci };
                    assert_eq!(next.decision, Decision::Delegate(Delegate::Repair(why)));
                }
            }
        }
        let mut c = change(State::Gating { head: HEAD, asked: Box::new([11]) });
        c.owns_turn = true;
        let mut f = facts();
        f.ci.status = Status::Failed;
        f.fault = fault;
        f.live_gate = Some(HEAD);
        let next = step(&c, &f, &heard(), at(5), &Limits { reruns: bound, ..LIMITS });
        if rerun {
            assert_eq!(next.decision, Decision::Effect(Effect::Rerun { head: HEAD }));
            assert_eq!(next.change.state, c.state);
            assert!(next.change.owns_turn);
        } else {
            assert_eq!(next.decision, Decision::CancelGates { head: HEAD });
        }
    }
}

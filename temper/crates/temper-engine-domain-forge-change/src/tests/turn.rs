//! A turn change judges heads and carries out its landing's requests (domain/landing.md, 3).
use super::*;
use crate::{Ask, Way};

fn turn(state: State) -> Change {
    let mut c = change(state);
    c.way = Way::Turn;
    c.landing = Some(8);
    c
}
fn ready() -> Change {
    let mut c = turn(State::Ready { head: HEAD, ready_since: at(0) });
    c.ready_since = Some(at(0));
    c
}
fn gate(number: u64, freshness: Freshness) -> Gate {
    Gate { number, kind: GateKind::Agent, blocking: true, freshness, eager: false }
}
fn prepare_facts() -> Facts {
    let mut f = facts();
    f.ask = Some(Ask::Prepare { head: HEAD, main: BASE });
    f
}

#[test]
fn a_turn_change_skips_checking_and_gates_at_once() {
    for status in [Status::Unknown, Status::Pending, Status::Failed, Status::Passed] {
        let mut c = turn(State::Opening { requested: true });
        c.gates = Box::new([gate(9, Freshness::Exact)]);
        let mut f = facts();
        f.ci = Ci { head: NEW, status };
        let opened = decide(&c, &f, &heard());
        assert_eq!(opened.change.state, State::Gating { head: HEAD, asked: Box::new([]) });
        let gating = decide(&opened.change, &f, &heard());
        assert_eq!(gating.decision, Decision::Delegate(Delegate::Gate { number: 9, head: HEAD }));
        assert_eq!(gating.change.repairs, 0);
    }
}

#[test]
fn a_turn_change_ready_tells_the_top_once_its_gates_hold() {
    let mut c = turn(State::Gating { head: HEAD, asked: Box::new([9]) });
    c.gates = Box::new([gate(9, Freshness::Exact)]);
    let mut f = facts();
    f.ci.status = Status::Failed;
    f.gates = Box::new([GateReport { revision: 0, number: 9, head: HEAD, status: Status::Passed }]);
    f.kept = true;
    let kept = decide(&c, &f, &heard());
    assert_eq!(kept.decision, Decision::Wait { until: None });
    assert_eq!(kept.change.ready_since, None);
    f.kept = false;
    let ready = decide(&kept.change, &f, &heard());
    assert_eq!(ready.decision, Decision::Ready);
    assert_eq!(ready.change.state, State::Ready { head: HEAD, ready_since: at(1) });
    assert_eq!(decide(&ready.change, &f, &heard()).decision, Decision::Wait { until: None });
}

#[test]
fn a_ready_change_waits_without_an_ask() {
    let c = ready();
    let mut f = facts();
    f.ci.status = Status::Failed;
    f.first = true;
    f.kept = true;
    f.contains_base = Status::Failed;
    let next = step(&c, &f, &heard(), at(60), &LIMITS);
    assert_eq!(next.change, c);
    assert_eq!(next.decision, Decision::Wait { until: None });
}

#[test]
fn a_prepare_ask_makes_a_rebase_update() {
    let next = decide(&ready(), &prepare_facts(), &heard());
    assert_eq!(next.decision, Decision::Effect(Effect::Prepare { head: HEAD, main: BASE }));
    assert_eq!(next.change.state, State::Preparing { from: HEAD, main: BASE, ready_since: at(0) });
    assert_eq!(next.change.updates, 1);
    assert_eq!(
        decide(&next.change, &prepare_facts(), &Heard { effect: EffectResult::Pending, ..heard() }).decision,
        Decision::Wait { until: Some(at(31)) }
    );
}

#[test]
fn a_prepare_ask_requires_its_head_and_the_writer_slot() {
    let mut f = prepare_facts();
    f.writer_taken = true;
    let blocked = step(&ready(), &f, &heard(), at(60), &LIMITS);
    assert_eq!(blocked.decision, Decision::Hold(Hold::Stalled));
    f.writer_taken = false;
    f.ask = Some(Ask::Prepare { head: NEW, main: BASE });
    assert_eq!(decide(&ready(), &f, &heard()).decision, Decision::Wait { until: Some(at(30)) });
}

#[test]
fn a_rebased_head_containing_main_is_ready_with_its_old_head_clean() {
    let prepared = decide(&ready(), &prepare_facts(), &heard());
    let mut f = facts();
    f.branch = Some(NEW);
    f.pull = Pull::Open { head: NEW, base: BASE };
    f.ci.status = Status::Failed;
    let mut h = heard();
    h.effect = EffectResult::Made;
    for contains in [Status::Unknown, Status::Pending, Status::Failed] {
        f.contains_base = contains;
        assert_eq!(decide(&prepared.change, &f, &h).decision, Decision::Wait { until: Some(at(31)) });
    }
    f.contains_base = Status::Passed;
    let next = decide(&prepared.change, &f, &h);
    assert_eq!(next.decision, Decision::Ready);
    assert_eq!(next.change.state, State::Ready { head: NEW, ready_since: at(0) });
    assert_eq!(&*next.change.clean, &[HEAD]);
    assert_eq!(next.change.last_head, Some(NEW));
}

#[test]
fn a_conflicting_rebase_resolves_and_asks_every_gate_again() {
    let mut c = ready();
    c.gates = Box::new([gate(9, Freshness::Clean), gate(10, Freshness::Exact)]);
    c.clean = Box::new([MERGE]);
    let prepared = decide(&c, &prepare_facts(), &heard());
    let mut h = heard();
    h.effect = EffectResult::Conflict;
    let resolving = decide(&prepared.change, &facts(), &h);
    assert_eq!(resolving.decision, Decision::Delegate(Delegate::Resolve { base: BASE }));
    assert!(resolving.change.clean.is_empty());
    h.delegate = Status::Passed;
    let mut f = facts();
    f.branch = Some(NEW);
    f.gates = Box::new([
        GateReport { revision: 0, number: 9, head: HEAD, status: Status::Passed },
        GateReport { revision: 0, number: 10, head: HEAD, status: Status::Passed },
    ]);
    let resolved = decide(&resolving.change, &f, &h);
    assert_eq!(resolved.change.state, State::Gating { head: NEW, asked: Box::new([]) });
    let one = decide(&resolved.change, &f, &heard());
    assert_eq!(one.decision, Decision::Delegate(Delegate::Gate { number: 9, head: NEW }));
    let two = decide(&one.change, &f, &heard());
    assert_eq!(two.decision, Decision::Delegate(Delegate::Gate { number: 10, head: NEW }));
}

#[test]
fn a_ready_change_lands_only_when_told_its_prefix() {
    let mut f = facts();
    f.pull = Pull::Merged { commit: NEW };
    assert_eq!(decide(&ready(), &f, &heard()).change.state, ready().state);
    f.ask = Some(Ask::Landed { prefix: NEW });
    let next = decide(&ready(), &f, &heard());
    assert_eq!(next.change.state, State::Landed { merge: NEW, since: at(1) });
    assert_eq!(next.decision, Decision::Finish { merge: NEW });
}

#[test]
fn a_repair_ask_delegates_a_landing_repair_counted_against_repairs() {
    let mut f = facts();
    f.contains_base = Status::Failed;
    f.ask = Some(Ask::Repair { gate: 3, candidate: NEW, below: None });
    let repairing = decide(&ready(), &f, &heard());
    assert_eq!(
        repairing.decision,
        Decision::Delegate(Delegate::Repair(Repair::LandingAt { gate: 3, candidate: NEW, below: None }))
    );
    assert_eq!(repairing.change.repairs, 1);
    assert_eq!(
        repairing.change.state,
        State::Repairing { requested: true, why: Repair::LandingAt { gate: 3, candidate: NEW, below: None } }
    );
    let mut exhausted = ready();
    exhausted.repairs = LIMITS.repairs;
    assert_eq!(decide(&exhausted, &f, &heard()).decision, Decision::Hold(Hold::Repairs));
    let mut h = heard();
    h.delegate = Status::Passed;
    f.branch = Some(NEW);
    let repaired = decide(&repairing.change, &f, &h);
    assert_eq!(repaired.change.state, State::Gating { head: NEW, asked: Box::new([]) });
    assert!(repaired.change.clean.is_empty());
    let again = decide(&repaired.change, &f, &heard());
    assert_eq!(again.change.state, State::Ready { head: NEW, ready_since: at(0) });
    assert_eq!(again.decision, Decision::Ready);
}

#[test]
fn ready_and_judging_changes_keep_the_blamed_repair_context() {
    use crate::{Below, Judgement, Landed};
    let below = Some(Below {
        by_test: true,
        top: true,
        green: BASE,
        changes: [Some(Landed { change: 2341, pull: 7 }), None, None, None],
    });
    let why = Repair::LandingAt { gate: 3, candidate: NEW, below };
    let f = Facts { ask: Some(Ask::Repair { gate: 3, candidate: NEW, below }), ..facts() };
    let mut judging = ready();
    judging.state = State::Judging {
        head: HEAD,
        ready_since: at(0),
        asked: Box::new([]),
        target: Judgement {
            landing: 81,
            round: 1,
            pull: 9,
            branch: b"prefix".as_slice().into(),
            base: b"main".as_slice().into(),
            prefix: NEW,
        },
    };
    for c in [ready(), judging] {
        let next = decide(&c, &f, &heard());
        assert_eq!(next.decision, Decision::Delegate(Delegate::Repair(why)));
        assert_eq!(next.change.state, State::Repairing { requested: true, why });
        let waiting = decide(&next.change, &facts(), &heard());
        assert_eq!(waiting.change.state, next.change.state, "the ask need not be read again");
    }
}

#[test]
fn a_clean_gate_counts_at_a_clean_prepares_candidate_and_an_exact_one_is_asked_again() {
    let mut c = ready();
    c.gates = Box::new([gate(9, Freshness::Clean), gate(10, Freshness::Exact)]);
    let prepared = decide(&c, &prepare_facts(), &heard());
    let mut f = facts();
    f.branch = Some(NEW);
    f.gates = Box::new([
        GateReport { revision: 0, number: 9, head: HEAD, status: Status::Passed },
        GateReport { revision: 0, number: 10, head: HEAD, status: Status::Passed },
    ]);
    let mut h = heard();
    h.effect = EffectResult::Made;
    let exact = decide(&prepared.change, &f, &h);
    assert_eq!(exact.decision, Decision::Delegate(Delegate::Gate { number: 10, head: NEW }));
    assert_eq!(&*exact.change.clean, &[HEAD]);
    f.gates = Box::new([
        GateReport { revision: 0, number: 9, head: HEAD, status: Status::Passed },
        GateReport { revision: 0, number: 10, head: NEW, status: Status::Passed },
    ]);
    let next = decide(&exact.change, &f, &heard());
    assert_eq!(next.decision, Decision::Ready);
    assert_eq!(next.change.state, State::Ready { head: NEW, ready_since: at(0) });
}

#[test]
fn a_queue_change_ignores_an_ask() {
    for ask in [Ask::Prepare { head: HEAD, main: BASE }, Ask::Repair { gate: 3, candidate: NEW, below: None }] {
        let mut f = facts();
        f.ask = Some(ask.clone());
        for c in [
            change(State::Opening { requested: true }),
            change(State::Gating { head: HEAD, asked: Box::new([]) }),
            change(State::Queued { head: HEAD, ready_since: at(0) }),
            change(State::First { head: HEAD, ready_since: at(0) }),
        ] {
            let with_ask = decide(&c, &f, &heard());
            f.ask = None;
            assert_eq!(with_ask, decide(&c, &f, &heard()));
            f.ask = Some(ask.clone());
        }
    }
}

#[test]
fn turn_gate_failures_still_repair_or_hold_and_keep_their_bounds() {
    for kind in [GateKind::Agent, GateKind::Check, GateKind::Person] {
        let mut c = turn(State::Gating { head: HEAD, asked: Box::new([9]) });
        let mut g = gate(9, Freshness::Exact);
        g.kind = kind;
        c.gates = Box::new([g]);
        let mut f = facts();
        f.ci.status = Status::Failed;
        f.contains_base = Status::Unknown;
        f.gates = Box::new([GateReport { revision: 0, number: 9, head: HEAD, status: Status::Failed }]);
        let next = decide(&c, &f, &heard());
        let expected = match kind {
            GateKind::Agent | GateKind::Check | GateKind::Follow => {
                Decision::Delegate(Delegate::Repair(Repair::Gate(9)))
            }
            GateKind::Person => Decision::Hold(Hold::Rejected),
        };
        assert_eq!(next.decision, expected);
    }
}

#[test]
fn clean_prepare_history_and_updates_are_bounded() {
    let mut c = ready();
    c.updates = LIMITS.updates;
    assert_eq!(decide(&c, &prepare_facts(), &heard()).decision, Decision::Hold(Hold::Updates));
    c.updates = 0;
    c.clean = Box::new([MERGE; 4]);
    let prepared = decide(&c, &prepare_facts(), &heard());
    let mut f = facts();
    f.branch = Some(NEW);
    let mut h = heard();
    h.effect = EffectResult::Made;
    assert_eq!(decide(&prepared.change, &f, &h).decision, Decision::Hold(Hold::Updates));
}

fn preparing_change() -> Change {
    let mut c = turn(State::Preparing { from: HEAD, main: BASE, ready_since: at(0) });
    c.updates = 1;
    c.clean = Box::new([MERGE]);
    c.gates = Box::new([gate(9, Freshness::Exact)]);
    c
}

#[test]
fn a_prepare_let_go_updates_again_where_its_head_is_behind() {
    let c = preparing_change();
    let f = Facts { contains_base: Status::Failed, ..facts() };
    let next = decide(&c, &f, &heard());
    assert_eq!(next.decision, Decision::Effect(Effect::Prepare { head: HEAD, main: BASE }));
    assert_eq!(next.change.updates, 2);
    assert_eq!(next.change.state, State::Preparing { from: HEAD, main: BASE, ready_since: at(0) });
    assert_eq!(next.change.since, at(1));
}

#[test]
fn a_prepare_let_go_holds_updates_at_its_bound() {
    let mut c = preparing_change();
    c.updates = LIMITS.updates;
    let f = Facts { contains_base: Status::Failed, ..facts() };
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Hold(Hold::Updates));
}

#[test]
fn a_prepare_let_go_goes_on_where_its_head_holds_the_base() {
    let c = preparing_change();
    let next = decide(&c, &facts(), &heard());
    assert_eq!(next, decide(&c, &facts(), &Heard { effect: EffectResult::Made, ..heard() }));
    assert_eq!(next.change.state, State::Gating { head: HEAD, asked: Box::new([9]) });
    assert_eq!(next.decision, Decision::Delegate(Delegate::Gate { number: 9, head: HEAD }));
    assert_eq!(next.change.clean, c.clean);
    assert_eq!(next.change.last_head, Some(HEAD));
    assert_eq!(next.change.ready_since, Some(at(0)));
    assert_eq!(next.change.since, at(1));
}

#[test]
fn a_prepare_let_go_takes_a_moved_head_holding_the_base() {
    let mut c = preparing_change();
    let f = Facts { branch: Some(NEW), ..facts() };
    let next = decide(&c, &f, &heard());
    assert_eq!(next, decide(&c, &f, &Heard { effect: EffectResult::Made, ..heard() }));
    assert_eq!(next.change.state, State::Gating { head: NEW, asked: Box::new([9]) });
    assert_eq!(next.decision, Decision::Delegate(Delegate::Gate { number: 9, head: NEW }));
    assert_eq!(next.change.clean.as_ref(), &[MERGE, HEAD]);
    assert_eq!(next.change.last_head, Some(NEW));
    assert_eq!(next.change.ready_since, Some(at(0)));
    assert_eq!(next.change.since, at(1));
    c.clean = Box::new([MERGE; 4]);
    assert_eq!(decide(&c, &f, &heard()).decision, Decision::Hold(Hold::Updates));
}

#[test]
fn a_prepare_let_go_holds_a_moved_head_not_holding_the_base() {
    let f = Facts { branch: Some(NEW), contains_base: Status::Failed, ..facts() };
    assert_eq!(decide(&preparing_change(), &f, &heard()).decision, Decision::Hold(Hold::BranchMoved));
}

#[test]
fn a_prepare_let_go_waits_on_what_it_cannot_read() {
    let c = preparing_change();
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
fn a_released_failed_prepare_decides_again() {
    let mut c = preparing_change();
    c.updates = LIMITS.updates;
    c.state = State::Held { was: Box::new(c.state), why: Hold::Failed };
    let f = Facts { contains_base: Status::Failed, ..facts() };
    let lifted = released(&c, at(1));
    let resumed = decide(&lifted, &f, &heard());
    assert_eq!(resumed.decision, Decision::None);
    let next = decide(&resumed.change, &f, &heard());
    assert_eq!(next.decision, Decision::Effect(Effect::Prepare { head: HEAD, main: BASE }));
    assert_eq!(next.change.updates, 1);
}

#[test]
fn a_prepare_let_go_at_a_main_that_moved_asks_its_own_main_again() {
    let f = Facts { base_tip: NEW, contains_base: Status::Failed, ..facts() };
    let next = decide(&preparing_change(), &f, &heard());
    assert_eq!(next.decision, Decision::Effect(Effect::Prepare { head: HEAD, main: BASE }));
    assert_eq!(next.change.state, State::Preparing { from: HEAD, main: BASE, ready_since: at(0) });
}

#[test]
fn a_prepare_outstanding_still_waits() {
    let c = preparing_change();
    let f = Facts { contains_base: Status::Failed, ..facts() };
    let next = decide(&c, &f, &Heard { effect: EffectResult::Pending, ..heard() });
    assert_eq!(next.decision, Decision::Wait { until: Some(at(30)) });
    assert_eq!(next.change, c);
}

// Both update styles use the refusal table in domain/forge/updates.md, 1.
fn stale_change(preparing: bool) -> Change {
    if preparing {
        preparing_change()
    } else {
        let mut c = change(State::Updating { from: HEAD, base: BASE, ready_since: at(0) });
        c.updates = 1;
        c
    }
}

#[test]
fn a_stale_update_replans_or_takes_its_observed_head_without_spending_updates() {
    let h = Heard { effect: EffectResult::Stale, ..heard() };
    for preparing in [false, true] {
        let mut c = stale_change(preparing);
        c.updates = LIMITS.updates;
        for (head, contains_base) in [
            (HEAD, Status::Passed),
            (NEW, Status::Passed),
            (NEW, Status::Failed),
            (HEAD, Status::Unknown),
            (NEW, Status::Pending),
        ] {
            let f = Facts { branch: Some(head), contains_base, ..facts() };
            let next = decide(&c, &f, &h);
            if contains_base == Status::Passed {
                assert_eq!(next, decide(&c, &f, &heard()));
                assert_eq!(next.change.stale, None);
            } else if contains_base == Status::Failed {
                assert_eq!(next.decision, Decision::Hold(Hold::BranchMoved));
                assert_eq!(next.change.stale, None);
            } else {
                assert_eq!(next.decision, Decision::Wait { until: Some(at(31)) });
                assert_eq!(next.change.stale, Some(at(1)));
            }
            assert_eq!(next.change.updates, c.updates);
        }
        let f = Facts {
            base_read: NEW,
            pull: Pull::Open { head: HEAD, base: NEW },
            contains_base: Status::Failed,
            ..facts()
        };
        let next = decide(&c, &f, &h);
        let effect = if preparing {
            Effect::Prepare { head: HEAD, main: NEW }
        } else {
            Effect::Update { head: HEAD, base: NEW }
        };
        assert_eq!(next.decision, Decision::Effect(effect));
        let state = if preparing {
            State::Preparing { from: HEAD, main: NEW, ready_since: at(0) }
        } else {
            State::Updating { from: HEAD, base: NEW, ready_since: at(0) }
        };
        assert_eq!(next.change.state, state);
        assert_eq!(next.change.since, at(1));
        assert_eq!(next.change.stale, Some(at(1)));
        assert_eq!(next.change.updates, c.updates);
    }
}

#[test]
fn stale_reads_back_off_across_a_restart_until_stall_not_updates() {
    let h = Heard { effect: EffectResult::Stale, ..heard() };
    for preparing in [false, true] {
        for base_read in [BASE, NEW] {
            let mut c = stale_change(preparing);
            c.updates = LIMITS.updates;
            let f = Facts { base_read, contains_base: Status::Failed, ..facts() };
            let first = decide(&c, &f, &h);
            assert_eq!(first.decision, Decision::Wait { until: Some(at(2)) });
            assert_eq!(first.change.stale, Some(at(1)));
            // A restored row keeps the first refusal and the last decision.
            c.stale = Some(at(1));
            c.since = at(5);
            let early = step(&c, &f, &h, at(6), &LIMITS);
            assert_eq!(early.decision, Decision::Wait { until: Some(at(9)) });
            assert_eq!(early.change, c);
            let retry = step(&c, &f, &h, at(9), &LIMITS);
            let effect = if preparing {
                Effect::Prepare { head: HEAD, main: BASE }
            } else {
                Effect::Update { head: HEAD, base: BASE }
            };
            assert_eq!(retry.decision, Decision::Effect(effect));
            assert_eq!(retry.change.stale, c.stale);
            assert_eq!(retry.change.updates, c.updates);
            assert_eq!(retry.change.since, at(9));
            assert_eq!(step(&retry.change, &f, &h, at(10), &LIMITS).decision, Decision::Wait { until: Some(at(17)) });
            let held = step(&retry.change, &f, &h, at(31), &LIMITS);
            assert_eq!(held.decision, Decision::Hold(Hold::Stalled));
            assert_eq!(held.change.stale, None);
            assert_eq!(held.change.updates, c.updates);
        }
        let mut c = stale_change(preparing);
        c.stale = Some(at(1));
        c.since = at(100);
        let limits = Limits { stall: Duration::from_secs(1000), ..LIMITS };
        let f = Facts { contains_base: Status::Failed, ..facts() };
        assert_eq!(step(&c, &f, &h, at(159), &limits).decision, Decision::Wait { until: Some(at(160)) });
        assert!(matches!(step(&c, &f, &h, at(160), &limits).decision, Decision::Effect(_)));
        c.since = at(0);
        assert_eq!(decide(&c, &f, &h).decision, Decision::Wait { until: Some(at(2)) });
    }
}

#[test]
fn releasing_a_stale_update_counts_afresh_then_replans_without_another_count() {
    for preparing in [false, true] {
        let mut c = stale_change(preparing);
        c.stale = Some(at(1));
        c.updates = LIMITS.updates;
        c.state = State::Held { was: Box::new(c.state), why: Hold::Stalled };
        let lifted = released(&c, at(40));
        assert_eq!(lifted.stale, None);
        assert_eq!(lifted.updates, 0);
        let f = Facts { contains_base: Status::Failed, ..facts() };
        let resumed = step(&lifted, &f, &heard(), at(40), &LIMITS);
        let resend = step(&resumed.change, &f, &heard(), at(41), &LIMITS);
        assert_eq!(resend.change.updates, 1);
        let effect = if preparing {
            Effect::Prepare { head: HEAD, main: BASE }
        } else {
            Effect::Update { head: HEAD, base: BASE }
        };
        assert_eq!(resend.decision, Decision::Effect(effect));
        let moved = Facts { base_read: NEW, pull: Pull::Open { head: HEAD, base: NEW }, ..f };
        let replan = step(&resend.change, &moved, &Heard { effect: EffectResult::Stale, ..heard() }, at(42), &LIMITS);
        assert_eq!(replan.change.updates, 1);
        let effect = if preparing {
            Effect::Prepare { head: HEAD, main: NEW }
        } else {
            Effect::Update { head: HEAD, base: NEW }
        };
        assert_eq!(replan.decision, Decision::Effect(effect));
    }
}

#[test]
fn a_preparations_conflict_uses_the_merge_job_before_a_resolution() {
    for merge_job in [false, true] {
        let f = Facts { merge_job, contains_base: Status::Failed, ..facts() };
        let next = decide(&preparing_change(), &f, &Heard { effect: EffectResult::Conflict, ..heard() });
        let delegate =
            if merge_job { Delegate::Merge { base: BASE, from: Some(HEAD) } } else { Delegate::Resolve { base: BASE } };
        assert_eq!(next.decision, Decision::Delegate(delegate));
        assert_eq!(next.change.resolutions, 1);
    }
}

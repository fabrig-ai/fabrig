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
    f.gates = Box::new([GateReport { number: 9, head: HEAD, status: Status::Passed }]);
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
    assert_eq!(decide(&ready(), &f, &heard()).decision, Decision::Hold(Hold::BranchMoved));
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
        GateReport { number: 9, head: HEAD, status: Status::Passed },
        GateReport { number: 10, head: HEAD, status: Status::Passed },
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
    f.ask = Some(Ask::Repair { gate: 3, candidate: NEW });
    let repairing = decide(&ready(), &f, &heard());
    assert_eq!(repairing.decision, Decision::Delegate(Delegate::Repair(Repair::Landing { gate: 3 })));
    assert_eq!(repairing.change.repairs, 1);
    assert_eq!(repairing.change.state, State::Repairing { requested: true, why: Repair::Landing { gate: 3 } });
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
fn a_clean_gate_counts_at_a_clean_prepares_candidate_and_an_exact_one_is_asked_again() {
    let mut c = ready();
    c.gates = Box::new([gate(9, Freshness::Clean), gate(10, Freshness::Exact)]);
    let prepared = decide(&c, &prepare_facts(), &heard());
    let mut f = facts();
    f.branch = Some(NEW);
    f.gates = Box::new([
        GateReport { number: 9, head: HEAD, status: Status::Passed },
        GateReport { number: 10, head: HEAD, status: Status::Passed },
    ]);
    let mut h = heard();
    h.effect = EffectResult::Made;
    let exact = decide(&prepared.change, &f, &h);
    assert_eq!(exact.decision, Decision::Delegate(Delegate::Gate { number: 10, head: NEW }));
    assert_eq!(&*exact.change.clean, &[HEAD]);
    f.gates = Box::new([
        GateReport { number: 9, head: HEAD, status: Status::Passed },
        GateReport { number: 10, head: NEW, status: Status::Passed },
    ]);
    let next = decide(&exact.change, &f, &heard());
    assert_eq!(next.decision, Decision::Ready);
    assert_eq!(next.change.state, State::Ready { head: NEW, ready_since: at(0) });
}

#[test]
fn a_queue_change_ignores_an_ask() {
    for ask in [Ask::Prepare { head: HEAD, main: BASE }, Ask::Repair { gate: 3, candidate: NEW }] {
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
        f.gates = Box::new([GateReport { number: 9, head: HEAD, status: Status::Failed }]);
        let next = decide(&c, &f, &heard());
        let expected = match kind {
            GateKind::Agent | GateKind::Check => Decision::Delegate(Delegate::Repair(Repair::Gate(9))),
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

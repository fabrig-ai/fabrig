use crate::{
    Decision, Facts, Fault, Flight, GateFacts, GateProof, Hold, Landing, Limits, Member, MemberFacts, Phase, Posted,
    Posting, Project, Proof, Repository, Status, Stepped, Verdict, step,
};
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

const HEAD: [u8; 32] = [1; 32];
const MAIN: [u8; 32] = [2; 32];
const NEW: [u8; 32] = [3; 32];
const KEY: [u8; 32] = [4; 32];
const LIMITS: Limits = Limits { members: 6, gates: 4, poll: Duration::from_secs(5) };
const PROJECT: Project = Project::DEFAULT;
fn at(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.saturating_mul(1_000_000_000))
}
fn landing_of_one() -> Landing {
    Landing {
        round: crate::Round::new(Box::from(b"temper/1/l61".as_slice())),

        task: 7,
        project: 9,
        phase: Phase::Waiting,
        preparation: 0,
        stale_proofs: 0,
        branch: Box::from(b"main".as_slice()),
        members: Box::new([Member {
            unit: 0,
            branch: Box::from(b"temper/1/l61".as_slice()),
            combined: None,
            recorded: false,
            told: false,
            judging: false,
            leave_told: false,
            left: None,
            change: 8,
            repository: Repository { forge: 1, repository: 2 },
            pull: 10,
            head: None,
            base: None,
            candidate: None,
            gates: Box::new([GateProof { proof: Proof::Unkeyed, posted: Posted::Nothing, faults: 0 }]),
            asked: None,
            landed: None,
        }]),
    }
}
fn facts() -> Facts {
    Facts {
        opened: Some(10),
        opening: Status::Passed,
        closed: Status::Unknown,
        preparation: crate::Preparation::Idle,
        members: Box::new([MemberFacts {
            deleted: Status::Unknown,
            outcome: None,
            dependencies: Box::new([]),
            held: false,
            judgement: Status::Passed,
            refused: None,
            contained: Status::Failed,
            candidate: Some((HEAD, crate::Combine::Merge)),
            preparation: crate::Preparation::Idle,
            record: Status::Unknown,
            ready: true,
            cancelled: false,
            head: HEAD,
            main: MAIN,
            contains_main: Status::Passed,
            asked: false,
            merge: Status::Unknown,
            merged: None,
            gates: Box::new([GateFacts {
                expired: None,
                run: 0,
                blocking: true,
                key: None,
                receipt: Status::Unknown,
                flight: Flight::None,
                posting: Status::Unknown,
            }]),
        }]),
        turn_free: false,
        released: false,
    }
}
pub(crate) fn decide(landing: &Landing, facts: &Facts) -> Stepped {
    step(landing, facts, &PROJECT, &LIMITS, at(10))
}
fn preparing() -> Landing {
    Landing { phase: Phase::Preparing, ..landing_of_one() }
}
fn proving(proof: Proof) -> Landing {
    let mut landing = decide(&preparing(), &facts()).landing;
    landing.members[0].gates[0].proof = proof;
    landing
}
fn keyed() -> Landing {
    proving(Proof::Keyed { key: KEY })
}
fn dispatched() -> Landing {
    let mut landing = proving(Proof::Dispatched { key: KEY, try_: 1 });
    landing.members[0].gates[0].posted = Posted::Pending;
    landing
}
fn turn() -> Landing {
    Landing { phase: Phase::Turn { at: at(4) }, ..proving(Proof::Passed { key: KEY }) }
}

fn close_round(landing: &Landing, f: &Facts) -> Stepped {
    let close = decide(landing, f);
    assert_eq!(close.decision, Decision::Close);
    let mut f = f.clone();
    f.closed = Status::Passed;
    let delete = decide(&close.landing, &f);
    assert_eq!(delete.decision, Decision::Delete { member: 0 });
    f.members[0].deleted = Status::Passed;
    decide(&delete.landing, &f)
}

#[test]
fn a_member_not_ready_waits() {
    let mut f = facts();
    f.members[0].ready = false;
    let next = decide(&landing_of_one(), &f);
    assert_eq!(next.decision, Decision::Wait { until: None });
    assert_eq!(next.landing.phase, Phase::Waiting);
}
#[test]
fn a_candidate_found_on_the_landing_branch_is_proved() {
    let next = decide(&preparing(), &facts());
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.landing.phase, Phase::Proving);
    assert_eq!(next.landing.preparation, 1);
    let member = &next.landing.members[0];
    assert_eq!((member.head, member.base, member.candidate), (Some(HEAD), Some(MAIN), Some(HEAD)));
    assert_eq!(member.gates[0].proof, Proof::Unkeyed);
}
#[test]
fn a_prepare_job_waits_and_its_branch_supplies_the_candidate() {
    let mut f = facts();
    f.members[0].candidate = None;
    let one = decide(&preparing(), &f);
    assert_eq!(one.decision, Decision::Prepare { member: 0, head: HEAD, main: MAIN });
    assert_eq!(one.landing.preparation, 1);
    f.preparation = crate::Preparation::Running;
    assert_eq!(decide(&one.landing, &f).decision, Decision::Wait { until: None });
    f.members[0].candidate = Some((NEW, crate::Combine::Merge));
    let two = decide(&one.landing, &f);
    assert_eq!(two.landing.members[0].candidate, Some(NEW));
    assert_eq!(two.landing.members[0].head, Some(HEAD));
    assert_eq!(two.landing.phase, Phase::Proving);
}
#[test]
fn an_unkeyed_gate_asks_for_its_key() {
    let next = decide(&proving(Proof::Unkeyed), &facts());
    assert_eq!(next.decision, Decision::Key { member: 0, gate: 0, candidate: HEAD });
    assert_eq!(next.landing.members[0].gates[0].proof, Proof::Unkeyed);
}
#[test]
fn a_passed_receipt_holds_with_no_dispatch_and_posts_received() {
    let mut f = facts();
    f.members[0].gates[0].key = Some(KEY);
    f.members[0].gates[0].receipt = Status::Passed;
    let next = decide(&proving(Proof::Unkeyed), &f);
    assert_eq!(next.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Received });
    assert_eq!(next.landing.members[0].gates[0].proof, Proof::Passed { key: KEY });
    assert_eq!(next.landing.members[0].gates[0].posted, Posted::Verdict);
    assert_eq!(decide(&next.landing, &f).decision, Decision::Proved);
}
#[test]
fn a_key_with_no_receipt_is_dispatched_then_posted_pending() {
    let one = decide(&keyed(), &facts());
    assert_eq!(one.decision, Decision::Dispatch { member: 0, gate: 0, key: KEY, try_: 1 });
    assert_eq!(one.landing.members[0].gates[0].proof, Proof::Dispatched { key: KEY, try_: 1 });
    let two = decide(&one.landing, &facts());
    assert_eq!(two.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Pending });
    assert_eq!(two.landing.members[0].gates[0].posted, Posted::Pending);
    assert_eq!(decide(&two.landing, &facts()).decision, Decision::Wait { until: None });
}
#[test]
fn a_key_in_flight_for_another_landing_waits() {
    let mut f = facts();
    f.members[0].gates[0].flight = Flight::Other;
    let next = decide(&keyed(), &f);
    assert_eq!(next.decision, Decision::Wait { until: None });
    assert_eq!(next.landing.members[0].gates[0].proof, Proof::Keyed { key: KEY });
}
#[test]
fn a_runners_fault_retries_up_to_gate_retries_then_holds() {
    let mut f = facts();
    f.members[0].gates[0].flight = Flight::Mine { run: Some(Verdict::Fault(Fault::SetUp)) };
    let one = decide(&dispatched(), &f);
    assert_eq!(one.decision, Decision::Retry { member: 0, gate: 0, key: KEY, try_: 2 });
    assert_eq!(one.landing.members[0].gates[0].faults, 1);
    f.members[0].gates[0].flight = Flight::Mine { run: None };
    assert_eq!(decide(&one.landing, &f).decision, Decision::Wait { until: Some(at(15)) });
    f.members[0].gates[0].flight = Flight::Mine { run: Some(Verdict::Fault(Fault::Completion)) };
    let two = decide(&one.landing, &f);
    assert_eq!(two.decision, Decision::Retry { member: 0, gate: 0, key: KEY, try_: 3 });
    assert_eq!(two.landing.members[0].gates[0].faults, 2);
    f.members[0].gates[0].flight = Flight::Mine { run: Some(Verdict::Fault(Fault::Lost)) };
    let three = decide(&two.landing, &f);
    assert_eq!(three.decision, Decision::Hold(Hold::Faults { member: 0, gate: 0 }));
    assert_eq!(
        three.landing.phase,
        Phase::Held { was: Box::new(Phase::Proving), why: Hold::Faults { member: 0, gate: 0 } }
    );
}
#[test]
fn a_failed_receipt_repairs_its_member() {
    let mut f = facts();
    f.members[0].gates[0].receipt = Status::Failed;
    let one = decide(&dispatched(), &f);
    assert_eq!(one.decision, Decision::Leave { member: 0, why: crate::Leave::Failed { gate: 0, run: 0 } });
    assert_eq!(one.landing.members[0].gates[0].proof, Proof::Failed { key: KEY });
    let two = decide(&one.landing, &f);
    assert_eq!(two.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Ran { passed: false } });
    assert_eq!(decide(&two.landing, &f).decision, Decision::Close);
}

#[test]
fn a_failed_gate_posts_its_verdict_even_while_its_member_repairs() {
    let mut f = facts();
    f.members[0].gates[0].receipt = Status::Failed;
    let repair = decide(&dispatched(), &f);
    assert_eq!(repair.decision, Decision::Leave { member: 0, why: crate::Leave::Failed { gate: 0, run: 0 } });
    f.members[0].ready = false;
    f.members[0].gates[0].posting = Status::Pending;
    let waiting = decide(&repair.landing, &f);
    assert_eq!(waiting.landing.phase, Phase::Proving);
    assert_eq!(waiting.decision, Decision::Wait { until: None });
    f.members[0].gates[0].posting = Status::Passed;
    let posted = decide(&waiting.landing, &f);
    assert_eq!(posted.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Ran { passed: false } });
    assert_eq!(posted.landing.phase, Phase::Proving);
    assert_eq!(posted.landing.members[0].candidate, Some(HEAD));
    assert_eq!(decide(&posted.landing, &f).decision, Decision::Close);
}

#[test]
fn a_main_that_moves_during_proof_is_checked_in_the_turn() {
    let mut f = facts();
    f.members[0].main = NEW;
    let proved = decide(&proving(Proof::Passed { key: KEY }), &f);
    assert_eq!(proved.decision, Decision::Proved);
    assert_eq!(proved.landing.members[0].base, Some(MAIN));
    f.turn_free = true;
    let owned = decide(&proved.landing, &f);
    assert_eq!(owned.landing.phase, Phase::Turn { at: at(10) });
    let stale = decide(&owned.landing, &f);
    let stale = close_round(&stale.landing, &f);
    assert_eq!(stale.decision, Decision::Stale);
    assert_eq!(stale.landing.stale_proofs, 1);
}

#[test]
fn a_repaired_head_does_not_erase_the_old_candidates_unposted_failure() {
    let mut f = facts();
    f.members[0].gates[0].receipt = Status::Failed;
    let repair = decide(&dispatched(), &f);
    f.members[0].head = NEW;
    f.members[0].gates[0].posting = Status::Pending;
    let pending = decide(&repair.landing, &f);
    assert_eq!(pending.decision, Decision::Wait { until: None });
    assert_eq!(pending.landing.members[0].candidate, Some(HEAD));
    f.members[0].gates[0].posting = Status::Passed;
    let posted = decide(&pending.landing, &f);
    assert_eq!(posted.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Ran { passed: false } });
    assert_eq!(posted.landing.members[0].candidate, Some(HEAD));
    assert_eq!(decide(&posted.landing, &f).landing.round.ending, Some(crate::RoundEnd::Leave));
}
#[test]
fn a_member_back_to_repairing_takes_the_landing_back_to_waiting() {
    for phase in [Phase::Preparing, Phase::Proving] {
        let mut landing = keyed();
        landing.phase = phase;
        let mut f = facts();
        f.members[0].ready = false;
        let next = decide(&landing, &f);
        assert_eq!(next.landing.phase, Phase::Waiting);
        assert_eq!(next.landing.members, landing.members);
        f.members[0].ready = true;
        f.members[0].head = NEW;
        let ready = decide(&next.landing, &f);
        assert_eq!(ready.decision, Decision::Leave { member: 0, why: crate::Leave::HeadMoved { head: NEW } });
        assert_eq!(ready.landing.round.ending, Some(crate::RoundEnd::Leave));
        assert_eq!(ready.landing.members[0].candidate, Some(HEAD));
    }
}
#[test]
fn an_advisory_gate_holds_nothing() {
    let mut f = facts();
    f.members[0].gates[0].blocking = false;
    let one = decide(&keyed(), &f);
    assert_eq!(one.decision, Decision::Dispatch { member: 0, gate: 0, key: KEY, try_: 1 });
    let two = decide(&one.landing, &f);
    assert_eq!(two.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Pending });
    f.members[0].gates[0].flight = Flight::Mine { run: Some(Verdict::Running) };
    assert_eq!(decide(&two.landing, &f).decision, Decision::Proved);
    f.members[0].gates[0].receipt = Status::Failed;
    let failed = decide(&dispatched(), &f);
    assert_eq!(failed.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Ran { passed: false } });
    assert_eq!(decide(&failed.landing, &f).decision, Decision::Proved);
    f.members[0].gates[0].receipt = Status::Unknown;
    f.members[0].gates[0].flight = Flight::Mine { run: Some(Verdict::Fault(Fault::Lost)) };
    let mut exhausted = dispatched();
    exhausted.members[0].gates[0].faults = PROJECT.gate_retries;
    assert_eq!(decide(&exhausted, &f).decision, Decision::Proved);
}
#[test]
fn every_blocking_gate_passed_proves_the_landing() {
    let next = decide(&proving(Proof::Passed { key: KEY }), &facts());
    assert_eq!(next.decision, Decision::Proved);
    assert_eq!(next.landing.phase, Phase::Proved { at: at(10) });
}
#[test]
fn a_proved_landing_takes_its_own_trains_turn() {
    let landing = Landing { phase: Phase::Proved { at: at(4) }, ..keyed() };
    for free in [false, true] {
        let mut f = facts();
        f.turn_free = free;
        let next = decide(&landing, &f);
        if free {
            assert_eq!(next.landing.phase, Phase::Turn { at: at(4) });
            assert_eq!(next.decision, Decision::None);
        } else {
            assert_eq!(next.landing.phase, landing.phase);
            assert_eq!(next.decision, Decision::Wait { until: None });
        }
    }
}
#[test]
fn the_turn_merges_at_the_candidate_then_tells_its_change_after_readback() {
    let landing = turn();
    let mut f = facts();
    let one = decide(&landing, &f);
    assert_eq!(one.decision, Decision::Merge { member: 0, candidate: HEAD });
    f.members[0].merge = Status::Passed;
    assert_eq!(decide(&one.landing, &f).decision, Decision::Wait { until: None });
    f.members[0].main = HEAD;
    let two = decide(&one.landing, &f);
    assert_eq!(two.decision, Decision::Landed { member: 0, prefix: HEAD });
    assert_eq!(two.landing.members[0].landed, Some(HEAD));
    assert_eq!(decide(&two.landing, &f).decision, Decision::Finish);
}
#[test]
fn a_main_moved_before_the_turns_merge_closes_before_preparing_again() {
    let landing = turn();
    let mut f = facts();
    f.members[0].main = NEW;
    let next = decide(&landing, &f);
    let next = close_round(&next.landing, &f);
    assert_eq!(next.decision, Decision::Stale);
    assert_eq!(next.landing.stale_proofs, 1);
    assert_eq!(next.landing.members[0].candidate, None);
}
#[test]
fn a_merged_member_lands_the_landing() {
    let mut f = facts();
    f.members[0].merged = Some(HEAD);
    f.members[0].main = HEAD;
    let told = decide(&turn(), &f);
    assert_eq!(told.decision, Decision::Landed { member: 0, prefix: HEAD });
    let next = decide(&told.landing, &f);
    assert_eq!(next.decision, Decision::Finish);
    assert_eq!(next.landing.phase, Phase::Landed);
    assert_eq!(next.landing.members[0].landed, Some(HEAD));
    assert_eq!(decide(&next.landing, &f).decision, Decision::Finish);
}
#[test]
fn a_cancelled_member_cancels_a_landing_of_one() {
    let mut f = facts();
    f.members[0].cancelled = true;
    let next = decide(&landing_of_one(), &f);
    assert_eq!(next.decision, Decision::Leave { member: 0, why: crate::Leave::Cancelled });
    assert_eq!(next.landing.members[0].change, 8, "the result retains the member's identity");
    f.members[0].deleted = Status::Passed;
    let ended = decide(&next.landing, &f);
    assert_eq!(ended.decision, Decision::Finish);
    assert_eq!(ended.landing.phase, Phase::Landed);
    assert!(ended.landing.members[0].landed.is_none());
}

#[test]
fn a_held_landing_waits_until_released() {
    let mut landing = dispatched();
    landing.members[0].gates[0].faults = 2;
    landing.phase = Phase::Held { was: Box::new(Phase::Proving), why: Hold::Faults { member: 0, gate: 0 } };
    assert_eq!(decide(&landing, &facts()).decision, Decision::Wait { until: None });
    let mut f = facts();
    f.released = true;
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.landing.phase, Phase::Preparing);
    assert_eq!(next.landing.members[0].gates[0].faults, 0);
}
#[test]
fn current_runs_poll_and_wait_for_a_receipt_before_proving() {
    for run in [None, Some(Verdict::Running), Some(Verdict::Passed), Some(Verdict::Failed)] {
        let mut f = facts();
        f.members[0].gates[0].flight = Flight::Mine { run };
        let next = decide(&dispatched(), &f);
        assert_eq!(next.decision, Decision::Wait { until: Some(at(15)) });
        assert_eq!(next.landing.phase, Phase::Proving);
    }
}
#[test]
fn a_passed_run_posts_ran_after_the_pending_post_completes() {
    let mut f = facts();
    f.members[0].gates[0].receipt = Status::Passed;
    f.members[0].gates[0].posting = Status::Pending;
    let one = decide(&dispatched(), &f);
    assert_eq!(one.decision, Decision::Wait { until: None });
    f.members[0].gates[0].posting = Status::Passed;
    let two = decide(&one.landing, &f);
    assert_eq!(two.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Ran { passed: true } });
    assert_eq!(decide(&two.landing, &f).decision, Decision::Proved);
}
#[test]
fn a_cancelled_member_keeps_its_candidate_and_proofs_until_cleanup() {
    let landing = keyed();
    let mut f = facts();
    f.members[0].cancelled = true;
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::Leave { member: 0, why: crate::Leave::Cancelled });
    assert_eq!(next.landing.members[0].candidate, landing.members[0].candidate);
    assert_eq!(next.landing.members[0].gates, landing.members[0].gates);
    assert_eq!(next.landing.members[0].left, Some(crate::Leave::Cancelled));
}

#[test]
fn a_refused_merge_holds_without_discarding_its_candidate() {
    let mut f = facts();
    f.members[0].merge = Status::Failed;
    let next = decide(&turn(), &f);
    assert_eq!(next.decision, Decision::Hold(Hold::Turn));
    assert_eq!(next.landing.members[0].candidate, Some(HEAD));
    assert_eq!(next.landing.stale_proofs, 0);
}

#[test]
fn an_unchanged_candidate_keeps_its_proofs_and_preparation() {
    let mut landing = keyed();
    landing.phase = Phase::Preparing;
    let next = decide(&landing, &facts());
    assert_eq!(next.landing.members, landing.members);
    assert_eq!(next.landing.preparation, landing.preparation);
    assert_eq!(next.landing.phase, Phase::Proving);
}
#[test]
fn the_same_snapshot_always_gives_the_same_decision() {
    for landing in [landing_of_one(), preparing(), keyed(), dispatched(), turn()] {
        assert_eq!(decide(&landing, &facts()), decide(&landing, &facts()));
    }
}

#[test]
fn a_single_landing_keeps_its_candidate_and_merges_without_a_round_pull() {
    let mut landing = turn();
    landing.round = crate::Round::new(Box::new([]));
    let f = facts();
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::Merge { member: 0, candidate: HEAD });
    assert!(next.landing.round.branch.is_empty());
    let mut readback = f;
    readback.members[0].merged = Some(HEAD);
    let next = decide(&next.landing, &readback);
    assert_eq!(next.decision, Decision::Finish);
    assert_eq!(next.landing.members[0].landed, Some(HEAD));
}

#[test]
fn a_single_landing_prepares_its_member_without_opening_a_round() {
    let mut landing = preparing();
    landing.round = crate::Round::new(Box::new([]));
    let mut f = facts();
    f.members[0].contains_main = Status::Failed;
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::Prepare { member: 0, head: HEAD, main: MAIN });
    assert!(next.landing.round.branch.is_empty());
    f.members[0].contains_main = Status::Passed;
    let next = decide(&next.landing, &f);
    assert_eq!(next.landing.phase, Phase::Proving);
    assert!(next.landing.round.pull.is_none());
}

pub(crate) fn stack(count: usize) -> Landing {
    let mut landing = preparing();
    let template = landing.members[0].clone();
    let mut members = skein_lib::List::with_capacity(u32::try_from(count).unwrap());
    for index in 0..count {
        let change = u64::try_from(index.checked_add(1).unwrap()).unwrap();
        members.push(Member {
            change,
            unit: u16::try_from(index).unwrap(),
            branch: skein_lib::bytes::concat(&[b"temper/1/l61-", skein_lib::Decimal::of(change).as_bytes()]),
            ..template.clone()
        }).unwrap();
    }
    landing.members = members.into_boxed();
    landing
}
pub(crate) fn stack_facts(count: usize) -> Facts {
    let mut f = facts();
    let template = f.members[0].clone();
    let mut members = skein_lib::List::with_capacity(u32::try_from(count).unwrap());
    for index in 0..count {
        members.push(MemberFacts { head: head(index), candidate: None, ..template.clone() }).unwrap();
    }
    f.members = members.into_boxed();
    f.opened = None;
    f.opening = Status::Unknown;
    f
}
pub(crate) fn head(index: usize) -> [u8; 32] { [u8::try_from(index.checked_add(10).unwrap()).unwrap(); 32] }
pub(crate) fn prefix(index: usize) -> [u8; 32] { [u8::try_from(index.checked_add(20).unwrap()).unwrap(); 32] }
fn found(f: &mut Facts) {
    for (index, fresh) in f.members.iter_mut().enumerate() {
        fresh.candidate = Some((prefix(index), crate::Combine::Merge));
    }
}
fn stacked(count: usize) -> (Landing, Facts) {
    let mut f = stack_facts(count);
    found(&mut f);
    f.opened = Some(99);
    f.opening = Status::Passed;
    let next = decide(&stack(count), &f);
    assert_eq!(next.landing.phase, Phase::Proving);
    (next.landing, f)
}

#[test]
fn a_stack_of_three_prepares_once_and_proves_only_its_top() {
    let mut f = stack_facts(3);
    let prepare = decide(&stack(3), &f);
    assert_eq!(prepare.decision, Decision::Prepare { member: 0, head: head(0), main: MAIN });
    assert_eq!(prepare.landing.preparation, 1);
    for member in &prepare.landing.members { assert_eq!(member.base, Some(MAIN)); }
    f.preparation = crate::Preparation::Running;
    assert_eq!(decide(&prepare.landing, &f).decision, Decision::Wait { until: None });
    found(&mut f);
    let open = decide(&prepare.landing, &f);
    assert_eq!(open.decision, Decision::Open);
    assert_eq!(open.landing.preparation, 1);
    f.opened = Some(99);
    let proving = decide(&open.landing, &f);
    let key = decide(&proving.landing, &f);
    assert_eq!(key.decision, Decision::Key { member: 2, gate: 0, candidate: prefix(2) });
    f.members[2].gates[0].key = Some(KEY);
    let dispatch = decide(&key.landing, &f);
    assert_eq!(dispatch.decision, Decision::Dispatch { member: 2, gate: 0, key: KEY, try_: 1 });
    let post = decide(&dispatch.landing, &f);
    assert_eq!(post.decision, Decision::Post { member: 2, gate: 0, posting: Posting::Pending });
    f.members[2].gates[0].receipt = Status::Passed;
    let passed = decide(&post.landing, &f);
    assert_eq!(passed.decision, Decision::Post { member: 2, gate: 0, posting: Posting::Ran { passed: true } });
    assert_eq!(decide(&passed.landing, &f).decision, Decision::Proved);
    for member in &passed.landing.members[..2] { assert_eq!(member.gates[0].proof, Proof::Unkeyed); }
}

#[test]
fn prefixes_found_are_taken_and_a_failed_job_without_progress_holds() {
    let mut f = stack_facts(3);
    f.members[0].candidate = Some((prefix(0), crate::Combine::Merge));
    f.members[1].candidate = Some((prefix(1), crate::Combine::Squash));
    f.preparation = crate::Preparation::Failed;
    let taken = decide(&stack(3), &f);
    assert_eq!(taken.decision, Decision::Prepare { member: 2, head: head(2), main: prefix(1) });
    assert_eq!(taken.landing.members[1].combined, Some(crate::Combine::Squash));
    assert_eq!(decide(&taken.landing, &f).decision, Decision::Hold(Hold::Prepare));
    f.members[2].candidate = Some((prefix(2), crate::Combine::Merge));
    assert_eq!(decide(&taken.landing, &f).decision, Decision::Open);
    let empty = stack_facts(3);
    let asked = decide(&stack(3), &empty);
    let mut failed = empty;
    failed.preparation = crate::Preparation::Failed;
    assert_eq!(decide(&asked.landing, &failed).decision, Decision::Hold(Hold::Prepare));
}

#[test]
fn conflicts_at_the_job_leave_without_ending_the_stack() {
    for (outcome, why) in [(crate::Outcome::Conflict, crate::Leave::Conflict),
        (crate::Outcome::EarlierConflict, crate::Leave::EarlierConflict)] {
        let mut f = stack_facts(3);
        let asked = decide(&stack(3), &f);
        f.members[0].candidate = Some((prefix(0), crate::Combine::Merge));
        f.members[1].outcome = Some(outcome);
        f.members[2].candidate = Some((prefix(2), crate::Combine::Merge));
        let left = decide(&asked.landing, &f);
        assert_eq!(left.decision, Decision::Leave { member: 1, why });
        assert_eq!(left.landing.round.ending, None);
        let open = decide(&left.landing, &f);
        assert_eq!(open.decision, Decision::Open);
        f.opened = Some(99);
        let proving = decide(&open.landing, &f);
        f.members[2].gates[0].key = Some(KEY);
        f.members[2].gates[0].receipt = Status::Passed;
        let post = decide(&proving.landing, &f);
        assert_eq!(decide(&post.landing, &f).decision, Decision::Proved);
        assert_eq!(post.landing.members[0].candidate, Some(prefix(0)));
        assert_eq!(post.landing.members[1].candidate, None);
    }
}

#[test]
fn leaving_mid_proof_deletes_only_the_changed_suffix_then_restacks() {
    let (mut landing, mut f) = stacked(3);
    landing.members[0].gates[0] = GateProof { proof: Proof::Passed { key: KEY }, posted: Posted::Verdict, faults: 0 };
    f.members[1].head = NEW;
    let left = decide(&landing, &f);
    assert_eq!(left.decision, Decision::Leave { member: 1, why: crate::Leave::HeadMoved { head: NEW } });
    assert_eq!(left.landing.round.changed, Some(1));
    assert_eq!(decide(&left.landing, &f).decision, Decision::Close);
    f.closed = Status::Passed;
    let delete = decide(&left.landing, &f);
    assert_eq!(delete.decision, Decision::Delete { member: 1 });
    f.members[1].deleted = Status::Passed;
    let delete = decide(&delete.landing, &f);
    assert_eq!(delete.decision, Decision::Delete { member: 2 });
    f.members[2].deleted = Status::Passed;
    let clean = decide(&delete.landing, &f);
    assert_eq!(clean.landing.members[0], landing.members[0]);
    for member in &clean.landing.members[1..] { assert!(member.candidate.is_none()); }
    f.members[1].candidate = None;
    f.members[2].candidate = None;
    f.opened = None;
    let prepare = decide(&clean.landing, &f);
    assert_eq!(prepare.decision, Decision::Prepare { member: 2, head: head(2), main: prefix(0) });
    f.members[2].candidate = Some(([40; 32], crate::Combine::Merge));
    f.opened = Some(100);
    let proving = decide(&prepare.landing, &f);
    f.members[2].gates[0].key = Some(KEY);
    f.members[2].gates[0].receipt = Status::Passed;
    let receipt = decide(&proving.landing, &f);
    assert_eq!(receipt.decision, Decision::Post { member: 2, gate: 0, posting: Posting::Received });
    assert_eq!(decide(&receipt.landing, &f).decision, Decision::Proved);
}

#[test]
fn a_top_that_leaves_opens_the_kept_prefix_without_a_job() {
    let (landing, mut f) = stacked(3);
    f.members[2].head = NEW;
    let left = decide(&landing, &f);
    f.closed = Status::Passed;
    let delete = decide(&left.landing, &f);
    assert_eq!(delete.decision, Decision::Delete { member: 2 });
    f.members[2].deleted = Status::Passed;
    let clean = decide(&delete.landing, &f);
    f.opened = None;
    assert_eq!(decide(&clean.landing, &f).decision, Decision::Open);
    f.opened = Some(100);
    let proving = decide(&clean.landing, &f);
    assert_eq!(decide(&proving.landing, &f).decision, Decision::Key { member: 1, gate: 0, candidate: prefix(1) });
}

#[test]
fn leaving_cascades_over_units_and_dependencies_before_notifications() {
    let (mut landing, mut f) = stacked(4);
    landing.members[0].unit = 1;
    landing.members[1].unit = 1;
    f.members[2].dependencies = Box::new([1]);
    f.members[3].dependencies = Box::new([2]);
    f.members[1].cancelled = true;
    let left = decide(&landing, &f);
    assert_eq!(left.decision, Decision::Leave { member: 0, why: crate::Leave::Unit { change: 2 } });
    assert_eq!(left.landing.round.changed, Some(0));
    assert_eq!(left.landing.members[1].left, Some(crate::Leave::Cancelled));
    assert_eq!(left.landing.members[2].left, Some(crate::Leave::Dependency { change: 2 }));
    assert_eq!(left.landing.members[3].left, Some(crate::Leave::Dependency { change: 3 }));
    let mut landing = left.landing;
    for member in 1..4 {
        let next = decide(&landing, &f);
        assert_eq!(next.decision, Decision::Leave { member, why: landing.members[usize::from(member)].left.unwrap() });
        landing = next.landing;
    }
    assert_eq!(decide(&landing, &f).decision, Decision::Close);
}

#[test]
fn a_red_repairs_one_unit_but_falls_back_for_several() {
    for units in [1, 2, 3] {
        let (mut landing, mut f) = stacked(3);
        for (index, member) in landing.members.iter_mut().enumerate() { member.unit = u16::try_from(index % units).unwrap(); }
        f.members[2].gates[0].key = Some(KEY);
        f.members[2].gates[0].receipt = Status::Failed;
        f.members[2].gates[0].run = 71;
        let why = if units == 1 { crate::Leave::Failed { gate: 0, run: 71 } }
            else { crate::Leave::Fallback { units: u16::try_from(units).unwrap(), top: prefix(2) } };
        for member in 0..3 {
            let next = decide(&landing, &f);
            assert_eq!(next.decision, Decision::Leave { member, why });
            for member in &next.landing.members { assert_eq!(member.left, Some(why)); assert!(member.landed.is_none()); }
            landing = next.landing;
        }
        let post = decide(&landing, &f);
        assert_eq!(post.decision, Decision::Post { member: 2, gate: 0, posting: Posting::Ran { passed: false } });
        f.closed = Status::Passed;
        for member in 0..3 {
            assert_eq!(decide(&post.landing, &f).decision, Decision::Delete { member });
            f.members[usize::from(member)].deleted = Status::Passed;
        }
        assert_eq!(decide(&post.landing, &f).decision, Decision::Finish);
    }
}

#[test]
fn judgement_is_asked_once_per_member_and_all_must_hold() {
    let (mut landing, mut f) = stacked(3);
    for fresh in &mut f.members { fresh.judgement = Status::Pending; }
    for member in 0..3 {
        let next = decide(&landing, &f);
        assert_eq!(next.decision, Decision::Judge { member });
        assert_eq!(next.landing.members[usize::from(member)].candidate, Some(prefix(usize::from(member))));
        landing = next.landing;
    }
    f.members[2].gates[0].key = Some(KEY);
    f.members[2].gates[0].receipt = Status::Passed;
    let post = decide(&landing, &f);
    assert_eq!(decide(&post.landing, &f).decision, Decision::Wait { until: None });
    for fresh in &mut f.members { fresh.judgement = Status::Passed; }
    assert_eq!(decide(&post.landing, &f).decision, Decision::Proved);
    f.members[1].refused = Some(75);
    assert_eq!(decide(&post.landing, &f).decision, Decision::Leave { member: 1, why: crate::Leave::Refused { gate: 75 } });
}

#[test]
fn the_turn_rebuilds_stale_stacks_and_lands_each_member_at_its_prefix() {
    let (mut landing, mut f) = stacked(3);
    landing.phase = Phase::Turn { at: at(4) };
    landing.members[0].combined = Some(crate::Combine::Squash);
    landing.members[2].combined = Some(crate::Combine::Squash);
    assert_eq!(decide(&landing, &f).decision, Decision::Merge { member: 2, candidate: prefix(2) });
    f.members[2].merge = Status::Passed;
    assert_eq!(decide(&landing, &f).decision, Decision::Wait { until: None });
    for fresh in &mut f.members { fresh.main = prefix(2); }
    let record = decide(&landing, &f);
    assert_eq!(record.decision, Decision::Record { member: 0, prefix: prefix(0) });
    for (i, member) in record.landing.members.iter().enumerate() { assert_eq!(member.landed, Some(prefix(i))); }
    f.members[0].record = Status::Passed;
    let told = decide(&record.landing, &f);
    assert_eq!(told.decision, Decision::Landed { member: 0, prefix: prefix(0) });
    let told = decide(&told.landing, &f);
    assert_eq!(told.decision, Decision::Landed { member: 1, prefix: prefix(1) });
    let record = decide(&told.landing, &f);
    assert_eq!(record.decision, Decision::Record { member: 2, prefix: prefix(2) });
    f.members[2].record = Status::Passed;
    let told = decide(&record.landing, &f);
    assert_eq!(told.decision, Decision::Landed { member: 2, prefix: prefix(2) });
    assert_eq!(decide(&told.landing, &f).decision, Decision::Finish);

    let (mut landing, mut f) = stacked(3);
    landing.phase = Phase::Turn { at: at(4) };
    for fresh in &mut f.members { fresh.main = NEW; }
    let stale = decide(&landing, &f);
    assert_eq!(stale.landing.round.ending, Some(crate::RoundEnd::Stale));
    f.closed = Status::Passed;
    for member in 0..3 {
        assert_eq!(decide(&stale.landing, &f).decision, Decision::Delete { member });
        f.members[usize::from(member)].deleted = Status::Passed;
    }
    let clean = decide(&stale.landing, &f);
    assert_eq!(clean.decision, Decision::Stale);
    for fresh in &mut f.members { fresh.candidate = None; }
    let rebuild = decide(&clean.landing, &f);
    assert_eq!(rebuild.decision, Decision::Prepare { member: 0, head: head(0), main: NEW });
    for member in &rebuild.landing.members { assert_eq!(member.base, Some(NEW)); assert!(member.candidate.is_none()); }
}

#[test]
fn a_member_the_base_contains_is_told_and_the_stack_goes_on() {
    let mut f = stack_facts(3);
    f.members[0].contained = Status::Passed;
    let landed = decide(&stack(3), &f);
    assert_eq!(landed.decision, Decision::Landed { member: 0, prefix: MAIN });
    assert_eq!(landed.landing.phase, Phase::Preparing);
    assert_eq!(decide(&landed.landing, &f).decision, Decision::Prepare { member: 1, head: head(1), main: MAIN });
}

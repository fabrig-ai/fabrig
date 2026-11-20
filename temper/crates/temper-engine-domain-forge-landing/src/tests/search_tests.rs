//! A selected probe proves only its cases (domain/landing-search-by-test.md, 3–7).
use super::step::{prefix, stacked};
use crate::{
    Awaited, Decision, Facts, Flight, GateProof, Landing, Leave, Limits, Phase, Posted, Posting, Probed, Project,
    Proof, RoundEnd, Search, SearchTests, Searched, Status, Stepped, TestId, Verdict, step,
};
use skein_lib::{Duration, Wall};

const KEY: [u8; 32] = [4; 32];
const LIMITS: Limits = Limits { members: 6, gates: 4, poll: Duration::from_secs(5), tests_bytes: 4, write_attempts: 2 };
pub(crate) fn ids(names: &[&[u8]]) -> Box<[TestId]> {
    let mut ids = skein_lib::List::with_capacity(u32::try_from(names.len()).unwrap());
    for id in names {
        ids.push(Box::from(*id)).unwrap();
    }
    ids.into_boxed()
}
pub(crate) fn decide(landing: &Landing, facts: &Facts) -> Stepped {
    step(landing, facts, &Project::DEFAULT, &LIMITS, Wall::from_nanos(10))
}
pub(crate) fn top_failed(count: usize) -> (Landing, Facts) {
    let (mut landing, mut facts) = stacked(count);
    for (member, fresh) in landing.members.iter_mut().zip(&mut facts.members) {
        let proof = GateProof { proof: Proof::Keyed { key: KEY }, posted: Posted::Nothing, faults: 0 };
        member.gates = Box::new([proof, proof]);
        let mut gate = fresh.gates[0];
        gate.key = Some(KEY);
        fresh.gates = Box::new([gate, gate]);
        fresh.failed = Box::new([None, None]);
    }
    let top = facts.members.last_mut().unwrap();
    top.gates[0].receipt = Status::Failed;
    top.gates[0].run = 81;
    top.failed[0] = Some(ids(&[b"a", b"b"]));
    top.gates[1].receipt = Status::Passed;
    (landing, facts)
}
pub(crate) fn searching(count: usize) -> (Landing, Facts) {
    let (mut landing, facts) = top_failed(count);
    landing.round.search = Some(Search {
        green: 0,
        red: u16::try_from(count).unwrap(),
        tests: SearchTests::ByTest { gates: Box::new([Some(ids(&[b"a", b"b"])), None]), probed: Box::new([]) },
    });
    (landing, facts)
}
fn outcomes(landing: &mut Landing, passed: bool) {
    let SearchTests::ByTest { probed, .. } = &mut landing.round.search.as_mut().unwrap().tests else {
        panic!("by test")
    };
    *probed = Box::new([Probed { member: 1, gate: 0, run: 82, passed }]);
}
fn bounds(landing: &Landing, green: u16, red: u16) {
    let search = landing.round.search.as_ref().unwrap();
    assert_eq!((search.green, search.red), (green, red));
}
fn selected(landing: &Landing) -> &[Option<Box<[TestId]>>] {
    let SearchTests::ByTest { gates, .. } = &landing.round.search.as_ref().unwrap().tests else { panic!("by test") };
    gates
}
#[test]
fn start_selects_every_failed_blocking_gate_and_no_other_gate() {
    let (landing, f) = top_failed(3);
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    bounds(&next.landing, 0, 3);
    assert_eq!(selected(&next.landing), [Some(ids(&[b"a", b"b"])), None]);
    let mut f = f;
    f.members[2].gates[1].receipt = Status::Failed;
    f.members[2].failed[1] = Some(ids(&[b"c"]));
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    assert_eq!(selected(&next.landing), [Some(ids(&[b"a", b"b"])), Some(ids(&[b"c"]))]);
}
#[test]
fn missing_ids_or_one_line_feed_past_room_starts_whole() {
    for (first, second) in
        [(None, Some(ids(&[b"b"]))), (Some(ids(&[b"a"])), None), (Some(ids(&[b"a", b"bb"])), Some(ids(&[b"c"])))]
    {
        let (landing, mut f) = top_failed(3);
        f.members[2].gates[1].receipt = Status::Failed;
        f.members[2].failed = Box::new([first, second]);
        let next = decide(&landing, &f);
        assert_eq!(next.decision, Decision::None);
        bounds(&next.landing, 0, 3);
        assert_eq!(next.landing.round.search.unwrap().tests, SearchTests::Whole);
    }
}
#[test]
fn only_selected_gates_decide_and_a_passed_receipt_holds() {
    let (mut landing, mut f) = searching(3);
    outcomes(&mut landing, false);
    f.members[0].gates[0].receipt = Status::Passed;
    f.members[0].gates[1].receipt = Status::Failed;
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    bounds(&next.landing, 1, 3);
    assert_eq!(next.landing.members[0].gates[0].posted, Posted::ByTest);
    assert_eq!(next.landing.members[0].gates[1], landing.members[0].gates[1]);
}
#[test]
fn a_selected_failure_is_red_and_each_red_gate_narrows() {
    let (mut landing, mut f) = searching(3);
    outcomes(&mut landing, false);
    let SearchTests::ByTest { gates, .. } = &mut landing.round.search.as_mut().unwrap().tests else {
        panic!("by test")
    };
    gates[1] = Some(ids(&[b"c", b"d"]));
    f.members[0].gates[0].receipt = Status::Failed;
    f.members[0].failed[0] = Some(ids(&[b"b", b"c"]));
    f.members[0].gates[1].receipt = Status::Failed;
    f.members[0].failed[1] = Some(ids(&[b"d", b"e"]));
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    bounds(&next.landing, 0, 1);
    assert_eq!(selected(&next.landing), [Some(ids(&[b"b"])), Some(ids(&[b"d"]))]);
    assert_eq!(next.landing.members[0].gates[0].proof, Proof::Failed { key: KEY });
}
#[test]
fn unrelated_or_missing_failed_ids_dispatch_as_absent() {
    for failed in [None, Some(ids(&[b"c"]))] {
        let (landing, mut f) = searching(3);
        f.members[0].gates[0].receipt = Status::Failed;
        f.members[0].failed[0] = failed;
        let next = decide(&landing, &f);
        assert_eq!(next.decision, Decision::Dispatch { member: 0, gate: 0, key: KEY, try_: 1 });
        bounds(&next.landing, 0, 3);
        assert_eq!(next.landing.members[0].gates[0].posted, Posted::ByTest);
    }
}
#[test]
fn a_kept_pass_by_test_is_green_even_beside_an_unrelated_receipt() {
    let (mut landing, mut f) = searching(3);
    outcomes(&mut landing, true);
    f.members[0].gates[0].receipt = Status::Failed;
    f.members[0].failed[0] = Some(ids(&[b"c"]));
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    bounds(&next.landing, 1, 3);
    assert_eq!(next.landing.phase, Phase::Proving);
}
#[test]
fn no_verdict_resets_both_bounds_and_the_next_probe_is_whole() {
    let (mut landing, mut f) = searching(3);
    landing.round.search.as_mut().unwrap().red = 2;
    outcomes(&mut landing, false);
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    bounds(&next.landing, 0, 3);
    assert_eq!(next.landing.round.search.as_ref().unwrap().tests, SearchTests::WholeAgain);
    // With the formerly selected gate green, the formerly unselected gate is now dispatched.
    let mut whole = next.landing;
    whole.members[0].gates[0] = GateProof { proof: Proof::Passed { key: KEY }, posted: Posted::Verdict, faults: 0 };
    f.members[0].gates[0].receipt = Status::Passed;
    let next = decide(&whole, &f);
    assert_eq!(next.decision, Decision::Dispatch { member: 0, gate: 1, key: KEY, try_: 1 });
    bounds(&next.landing, 0, 3);
}
#[test]
fn flights_wait_and_probes_post_no_pending_or_success_now_or_later() {
    let (mut landing, mut f) = searching(5);
    // Probe member 2, already dispatched by test, while member 1's late run is followed.
    landing.members[0].gates[0] =
        GateProof { proof: Proof::Dispatched { key: KEY, try_: 1 }, posted: Posted::ByTest, faults: 0 };
    landing.members[1].gates[0] = landing.members[0].gates[0];
    f.members[1].gates[0].flight = Flight::Mine { run: Some(Verdict::Running) };
    let next = decide(&landing, &f);
    assert_eq!(
        next.decision,
        Decision::Wait { until: Some(Wall::from_nanos(5_000_000_010)), on: Awaited::Run { member: 1, gate: 0 } }
    );
    bounds(&next.landing, 0, 5);
    f.members[0].gates[0].receipt = Status::Passed;
    f.members[1].gates[0].receipt = Status::Passed;
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    bounds(&next.landing, 2, 5);
    assert_eq!(next.landing.members[0].gates[0].posted, Posted::ByTest);
    assert_eq!(next.landing.members[1].gates[0].posted, Posted::ByTest);
}
#[test]
fn a_red_probe_posts_its_failed_receipt() {
    let (mut landing, mut f) = searching(3);
    landing.round.search.as_mut().unwrap().red = 1;
    landing.members[0].gates[0] = GateProof { proof: Proof::Failed { key: KEY }, posted: Posted::ByTest, faults: 0 };
    f.members[0].gates[0].receipt = Status::Failed;
    f.members[0].failed[0] = Some(ids(&[b"b"]));
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::Post { member: 0, gate: 0, posting: Posting::Ran { passed: false } });
    assert_eq!(next.landing.members[0].gates[0].posted, Posted::Verdict);
    bounds(&next.landing, 0, 1);
}
fn at_boundary(count: usize, green: u16, red: u16) -> (Landing, Facts) {
    let (mut landing, mut f) = searching(count);
    let search = landing.round.search.as_mut().unwrap();
    search.green = green;
    search.red = red;
    let fresh = &mut f.members[usize::from(red.checked_sub(1).unwrap())];
    fresh.gates[0].receipt = Status::Failed;
    fresh.gates[0].run = 81;
    fresh.failed[0] = Some(ids(&[b"b"]));
    (landing, f)
}
#[test]
fn boundary_blames_its_unit_and_keeps_the_search_line() {
    for (green, red) in [(0_u16, 1_u16), (1, 2), (1, 3)] {
        let (mut landing, f) = at_boundary(3, green, red);
        if red == 3 {
            landing.members[2].unit = landing.members[1].unit;
        }
        let red_commit = prefix(usize::from(red.checked_sub(1).unwrap()));
        let changes = if red == 3 { Box::from([2, 3].as_slice()) } else { Box::from([u64::from(red)].as_slice()) };
        let next = decide(&landing, &f);
        let green_commit = if green == 0 { [2; 32] } else { prefix(0) };
        assert_eq!(
            next.decision,
            Decision::Leave {
                member: green,
                why: Leave::Blamed { gate: 0, run: 81, red: red_commit, green: green_commit }
            }
        );
        assert_eq!(next.landing.round.ending, Some(RoundEnd::Cut));
        assert_eq!(next.landing.round.changed, Some(green));
        assert_eq!(
            next.landing.searched,
            Some(Searched { red: red_commit, green: if green == 0 { None } else { Some(prefix(0)) }, changes })
        );
        for member in &next.landing.members {
            assert!(member.landed.is_none());
        }
    }
}
fn cut(count: usize, green: u16, red: u16) -> (Landing, Facts) {
    let (mut landing, f) = at_boundary(count, green, red);
    landing.round.ending = Some(RoundEnd::Cut);
    landing.round.changed = Some(green);
    let member = &mut landing.members[usize::from(green)];
    member.left = Some(Leave::Blamed { gate: 0, run: 81, red: prefix(usize::from(green)), green: prefix(0) });
    member.leave_told = true;
    landing.searched =
        Some(Searched { red: prefix(usize::from(green)), green: Some(prefix(0)), changes: Box::new([u64::from(red)]) });
    (landing, f)
}
#[test]
fn cut_closes_and_deletes_only_the_blamed_member_and_above() {
    let (landing, f) = cut(3, 1, 2);
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::Close);
    assert_eq!(next.landing.round.search, landing.round.search);
    let mut f = f;
    f.closed = Status::Passed;
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::Delete { member: 1 });
    f.members[1].deleted = Status::Passed;
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::Delete { member: 2 });
    assert_eq!(next.landing.members[0].candidate, Some(prefix(0)));
}
#[test]
fn cut_keeps_green_prefixes_but_resets_their_proofs_and_ends_unlanded() {
    for (count, green, red) in [(3, 1, 2), (3, 0, 1), (2, 1, 2)] {
        let (mut landing, mut f) = cut(count, green, red);
        landing.members[0].gates[0] =
            GateProof { proof: Proof::Passed { key: KEY }, posted: Posted::ByTest, faults: 1 };
        f.closed = Status::Passed;
        for fresh in &mut f.members {
            fresh.deleted = Status::Passed;
        }
        let next = decide(&landing, &f);
        assert_eq!(next.decision, Decision::None);
        assert_eq!(next.landing.phase, Phase::Preparing);
        assert_eq!(next.landing.round.search, None);
        assert_eq!(next.landing.searched, landing.searched);
        for (index, member) in next.landing.members.iter().enumerate() {
            assert!(member.landed.is_none());
            assert_eq!(member.candidate, if index < usize::from(green) { Some(prefix(index)) } else { None });
            for gate in &member.gates {
                assert_eq!(*gate, GateProof { proof: Proof::Unkeyed, posted: Posted::Nothing, faults: 0 });
            }
        }
        assert_eq!(next.landing.preparation, if count == 2 { 2 } else { 1 });
    }
}
#[test]
fn every_probe_green_blames_top_and_opens_kept_top_with_no_job() {
    let (mut landing, f) = searching(2);
    outcomes(&mut landing, true);
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    bounds(&next.landing, 1, 2);
    assert_eq!(next.landing.phase, Phase::Proving);
    let blamed = decide(&next.landing, &f);
    assert_eq!(
        blamed.decision,
        Decision::Leave { member: 1, why: Leave::Blamed { gate: 0, run: 81, red: prefix(1), green: prefix(0) } }
    );
    assert_eq!(blamed.landing.searched.as_ref().unwrap().changes.as_ref(), [2]);
    let (mut kept, mut f) = cut(2, 1, 2);
    kept.phase = Phase::Preparing;
    kept.round.ending = None;
    kept.round.search = None;
    kept.round.pull = None;
    kept.preparation = 2;
    kept.members[1].candidate = None;
    kept.members[1].base = None;
    f.opened = None;
    f.opening = Status::Unknown;
    let next = decide(&kept, &f);
    assert_eq!(next.decision, Decision::Open);
    assert_eq!(next.landing.members[0].candidate, Some(prefix(0)));
    assert_eq!(next.landing.searched, kept.searched);
}
#[test]
fn an_earlier_conflict_and_a_by_test_cut_keep_two_members_for_the_next_turn() {
    let (mut landing, mut f) = top_failed(4);
    landing.phase = Phase::Preparing;
    landing.members[2].candidate = None;
    f.members[2].candidate = None;
    f.members[2].outcome = Some(crate::Outcome::EarlierConflict);
    f.members[3].gates[0].receipt = Status::Passed;
    f.members[3].failed[0] = None;
    f.members[3].gates[1].receipt = Status::Failed;
    f.members[3].gates[1].run = 81;
    f.members[3].failed[1] = Some(ids(&[b"a", b"b"]));
    for fresh in &mut f.members[..2] {
        for gate in &mut fresh.gates {
            gate.receipt = Status::Passed;
        }
    }
    f.turn_free = true;
    let advance = |landing: &mut Landing, facts: &Facts, expected| {
        let next = decide(landing, facts);
        assert_eq!(next.decision, expected);
        *landing = next.landing;
    };
    advance(&mut landing, &f, Decision::Leave { member: 2, why: Leave::EarlierConflict });
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::Post { member: 3, gate: 0, posting: Posting::Received });
    assert_eq!(landing.members[3].gates[0].proof, Proof::Passed { key: KEY });
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::Post { member: 3, gate: 1, posting: Posting::Ran { passed: false } });
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::None);
    bounds(&landing, 2, 4);
    advance(
        &mut landing,
        &f,
        Decision::Leave { member: 3, why: Leave::Blamed { gate: 1, run: 81, red: prefix(3), green: prefix(1) } },
    );
    advance(&mut landing, &f, Decision::Close);
    f.closed = Status::Passed;
    advance(&mut landing, &f, Decision::Delete { member: 3 });
    f.members[3].deleted = Status::Passed;
    advance(&mut landing, &f, Decision::None);
    assert_eq!(landing.preparation, 2);
    assert_eq!(landing.members[0].candidate, Some(prefix(0)));
    assert_eq!(landing.members[1].candidate, Some(prefix(1)));
    f.opened = None;
    f.opening = Status::Unknown;
    advance(&mut landing, &f, Decision::Open);
    f.opened = Some(100);
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::Post { member: 1, gate: 0, posting: Posting::Received });
    advance(&mut landing, &f, Decision::Post { member: 1, gate: 1, posting: Posting::Received });
    advance(&mut landing, &f, Decision::Proved);
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::Merge { member: 1, candidate: prefix(1) });
    for (index, fresh) in f.members.iter_mut().enumerate().take(2) {
        fresh.main = prefix(1);
        fresh.merged = Some(prefix(index));
    }
    advance(&mut landing, &f, Decision::Landed { member: 0, prefix: prefix(0) });
    advance(&mut landing, &f, Decision::Landed { member: 1, prefix: prefix(1) });
    advance(&mut landing, &f, Decision::Finish);
}
#[test]
fn a_cut_with_one_kept_member_prepares_again_when_main_moved() {
    let (mut landing, mut f) = top_failed(3);
    // The middle member left on an earlier conflict; the search blames the top.
    landing.phase = Phase::Preparing;
    landing.members[1].candidate = None;
    f.members[1].candidate = None;
    f.members[1].outcome = Some(crate::Outcome::EarlierConflict);
    f.members[2].gates[0].receipt = Status::Passed;
    f.members[2].failed[0] = None;
    f.members[2].gates[1].receipt = Status::Failed;
    f.members[2].gates[1].run = 81;
    f.members[2].failed[1] = Some(ids(&[b"a", b"b"]));
    for gate in &mut f.members[0].gates {
        gate.receipt = Status::Passed;
    }
    f.turn_free = true;
    let advance = |landing: &mut Landing, facts: &Facts, expected| {
        let next = decide(landing, facts);
        assert_eq!(next.decision, expected);
        *landing = next.landing;
    };
    advance(&mut landing, &f, Decision::Leave { member: 1, why: Leave::EarlierConflict });
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::Post { member: 2, gate: 0, posting: Posting::Received });
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::Post { member: 2, gate: 1, posting: Posting::Ran { passed: false } });
    advance(&mut landing, &f, Decision::None);
    bounds(&landing, 1, 3);
    advance(
        &mut landing,
        &f,
        Decision::Leave { member: 2, why: Leave::Blamed { gate: 1, run: 81, red: prefix(2), green: prefix(0) } },
    );
    assert_eq!(landing.round.ending, Some(RoundEnd::Cut));
    for fresh in &mut f.members {
        fresh.main = [19; 32];
    }
    advance(&mut landing, &f, Decision::Close);
    f.closed = Status::Passed;
    advance(&mut landing, &f, Decision::Delete { member: 2 });
    f.members[2].deleted = Status::Passed;
    advance(&mut landing, &f, Decision::None);
    f.opened = None;
    f.opening = Status::Unknown;
    advance(&mut landing, &f, Decision::None);
    assert_eq!(landing.round.ending, Some(RoundEnd::Stale));
    advance(&mut landing, &f, Decision::Delete { member: 0 });
    f.members[0].deleted = Status::Passed;
    f.members[0].candidate = None;
    advance(&mut landing, &f, Decision::Stale);
    advance(&mut landing, &f, Decision::Prepare { member: 0, head: super::step::head(0), main: [19; 32] });
    assert_eq!(landing.members[0].base, Some([19; 32]));
    let candidate = [30; 32];
    f.members[0].candidate = Some((candidate, crate::Combine::Merge));
    advance(&mut landing, &f, Decision::Open);
    f.opened = Some(100);
    advance(&mut landing, &f, Decision::None);
    for gate in 0..2 {
        advance(&mut landing, &f, Decision::Post { member: 0, gate, posting: Posting::Received });
    }
    advance(&mut landing, &f, Decision::Proved);
    advance(&mut landing, &f, Decision::None);
    advance(&mut landing, &f, Decision::Merge { member: 0, candidate });
    f.members[0].main = candidate;
    f.members[0].merged = Some(candidate);
    advance(&mut landing, &f, Decision::Landed { member: 0, prefix: candidate });
    advance(&mut landing, &f, Decision::Finish);
}
#[test]
fn opening_the_next_round_takes_the_search_line() {
    let (mut landing, mut f) = cut(2, 1, 2);
    landing.phase = Phase::Preparing;
    landing.round.ending = None;
    landing.round.search = None;
    landing.round.pull = None;
    landing.members[1].candidate = None;
    landing.members[1].base = None;
    f.opened = Some(100);
    let next = decide(&landing, &f);
    assert_eq!(next.decision, Decision::None);
    assert_eq!(next.landing.phase, Phase::Proving);
    assert_eq!(next.landing.round.pull, Some(100));
    assert_eq!(next.landing.searched, None);
}

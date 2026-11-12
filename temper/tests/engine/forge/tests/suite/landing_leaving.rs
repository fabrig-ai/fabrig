//! Leaving and bounded proof in the forge's world (domain/landing.md, 4.2–4.3, 7).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{World, REPO, changes, landings::{self, CHANGE, LANDING, PROJECT}, translate};
const MEMBER: &[u8] = b"temper/51";

fn configured(latency: u64, failure: bool, advisory: bool) -> World {
    let mut world = World::public_turn(31, raw::Workflow {
        file: b"gate.yml".as_slice().into(), jobs: Box::new([b"test".as_slice().into(), b"lint".as_slice().into()]),
        fails: Box::new([]), flaky: Box::new([]), report: None, select: None,
        latency_min: Duration::from_secs(latency), latency_max: Duration::from_secs(latency), silent: 0, passes: 1000,
        cue: failure.then(|| raw::Cue::File { path: b"file".as_slice().into(), green: b"fixed".as_slice().into() }),
    });
    world.adopt();
    let mut gates = Vec::new();
    if advisory { gates.push(landing::Gate { name: b"advisory".as_slice().into(), workflow: b"gate.yml".as_slice().into(), ignored: Box::new([b"file".as_slice().into()]), blocking: false, base: false }); }
    gates.push(landing::Gate { name: b"gate".as_slice().into(), workflow: b"gate.yml".as_slice().into(), ignored: Box::new([b"docs".as_slice().into()]), blocking: true, base: false });
    assert!(world.project_landing(PROJECT, change::Way::Turn, Box::new([top::RepositoryGates { repository: REPO, gates: gates.into_boxed_slice(), tiers: Box::new([]) }])));
    world.produce(MEMBER);
    world.push_file(b"main", b"base", b"base moved");
    landings::ready_change(&mut world, CHANGE, MEMBER);
    let mut row = landings::landing_row(world.change(CHANGE));
    if advisory { row.landing.members[0].gates = vec![landing::GateProof { proof: landing::Proof::Unkeyed, posted: landing::Posted::Nothing, faults: 0 }; 2].into_boxed_slice(); }
    world.event(top::Event::Landing { row });
    world
}
fn proving(world: &mut World) {
    for entry in 100..120 {
        landings::step_landing(world, LANDING, entry);
        if world.landing(LANDING).landing.phase == landing::Phase::Proving { return; }
    }
    panic!("not proving: {:?}", world.landing(LANDING));
}
fn left(world: &mut World, why: landing::Leave) {
    let pull = world.landing(LANDING).landing.round.pull;
    let branch = world.landing(LANDING).landing.round.branch.clone();
    landings::drive_landing(world, LANDING);
    let row = &world.landing(LANDING).landing;
    assert_eq!(row.phase, landing::Phase::Landed);
    assert_eq!(row.members[0].left, Some(why));
    assert!(row.members[0].landed.is_none());
    assert!(!world.has_branch(&branch));
    if let Some(number) = pull {
        let raw::Answer::Pull(pull) = world.inspect(&raw::Read::Pull { number }) else { panic!("round pull") };
        assert_eq!(pull.state, raw::State::Closed);
        assert_eq!(pull.merged, None);
    }
}
fn step_change(world: &mut World, entry: u64) -> change::Decision {
    changes::change_step_task(world, CHANGE, entry, change::Status::Passed, Box::new([]), false, false)
}
#[test]
fn a_held_member_leaves_and_keeps_its_place_ahead_of_a_later_waiter() {
    let mut world = configured(100, false, false);
    proving(&mut world);
    let ready = world.change(CHANGE).change.ready_since;
    world.produce(b"temper/52");
    landings::ready_change(&mut world, 52, b"temper/52");
    world.event(top::Event::StepLanding { task: LANDING, entry: 200, heard: top::LandingHeard {
        held: Box::new([CHANGE]), failure: Box::new([]), preparation: landing::Preparation::Idle,
        kept: Box::new([]), released: false, cancelled: false,
    } });
    world.run_for(1);
    assert_eq!(world.landing_leave_words(LANDING).as_deref(), Some(b"held".as_slice()));
    left(&mut world, landing::Leave::Held);
    assert_eq!(world.change(CHANGE).change.ready_since, ready);
    assert_eq!(world.first_waiting_change(), Some(CHANGE), "released earlier member overtakes the later waiter");
}
#[test]
fn a_cancelled_member_leaves_and_closes_its_round() {
    let mut world = configured(100, false, false);
    proving(&mut world);
    let ready = world.change(CHANGE).change.ready_since;
    world.event(top::Event::SettleEffects { task: CHANGE, root: CHANGE, ending: top::ReleaseEnding::Cancelled });
    left(&mut world, landing::Leave::Cancelled);
    assert_eq!(world.change(CHANGE).change.ready_since, ready);
    assert_eq!(world.landing_leave_words(LANDING).as_deref(), Some(b"cancelled".as_slice()));
}
#[test]
fn a_head_pushed_by_another_hand_leaves_and_holds_its_change_as_drift() {
    let mut world = configured(100, false, false);
    proving(&mut world);
    let ready = world.change(CHANGE).change.ready_since;
    world.push_as(MEMBER, b"new", b"another hand", 2);
    let head = translate::commit(world.branch(MEMBER));
    assert_eq!(step_change(&mut world, 200), change::Decision::Hold(change::Hold::BranchMoved));
    left(&mut world, landing::Leave::HeadMoved { head });
    assert_eq!(world.change(CHANGE).change.ready_since, ready);
    let hex: String = head[..20].iter().map(|byte| format!("{byte:02x}")).collect();
    assert_eq!(world.landing_leave_words(LANDING).as_deref(), Some(format!("head moved to {hex}").as_bytes()));
}
#[test]
fn a_real_failure_snapshots_each_failed_job_and_only_changed_keys_run_after_repair() {
    let mut world = configured(2, true, true);
    proving(&mut world);
    let ready = world.change(CHANGE).change.ready_since;
    let mut failed = None;
    for entry in 200..260 {
        if let landing::Decision::Leave { why: landing::Leave::Failed { gate, run }, .. } = landings::step_landing(&mut world, LANDING, entry) { failed = Some((gate, run)); break; }
    }
    let (gate, run) = failed.expect("real workflow failure leaves");
    assert_eq!(gate, 1);
    let candidate = world.landing(LANDING).landing.members[0].candidate.expect("candidate");
    let base = world.landing(LANDING).landing.members[0].base.expect("base");
    assert_eq!(step_change(&mut world, 300), change::Decision::Delegate(change::Delegate::Repair(change::Repair::Landing { gate })));
    let head = world.change(CHANGE).change.last_head.expect("own head");
    let brief = world.ci_brief(CHANGE, head);
    let brief = String::from_utf8(brief.into_vec()).expect("brief");
    for words in ["Gate: gate", "Run: ", "Candidate: ", "Base: ", "Failed job: test", "Failed job: lint", "Failed step: ", "End of its log:"] { assert!(brief.contains(words), "{words}: {brief}"); }
    for commit in [candidate, base] { let hex: String = commit[..20].iter().map(|byte| format!("{byte:02x}")).collect(); assert!(brief.contains(&hex)); }
    left(&mut world, landing::Leave::Failed { gate, run });
    world.push(MEMBER, b"fixed");
    for entry in 400..410 { step_change(&mut world, entry); if matches!(world.change(CHANGE).change.state, change::State::Ready { .. }) { break; } }
    assert_eq!(world.change(CHANGE).change.ready_since, ready);
    assert!(matches!(world.change(CHANGE).change.state, change::State::Ready { .. }));
    let mut next = landings::landing_row(world.change(CHANGE));
    next.landing.task = 62; next.landing.round = landing::Round::new(b"temper/1/l62".as_slice().into());
    next.landing.members[0].gates = vec![landing::GateProof { proof: landing::Proof::Unkeyed, posted: landing::Posted::Nothing, faults: 0 }; 2].into_boxed_slice();
    world.take_calls();
    world.event(top::Event::Landing { row: next });
    landings::drive_landing(&mut world, 62);
    assert!(world.landing(62).landing.members[0].landed.is_some());
    assert_eq!(world.take_calls().iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 1, "the unchanged advisory key is received, not run again");
}
#[test]
fn runner_faults_past_the_projects_retries_hold_without_repair() {
    let mut world = configured(2, false, false);
    world.gate_policy(landing::Project { gate_retries: 1, ..landing::Project::DEFAULT });
    world.runner_setup_faults(1000);
    proving(&mut world);
    for entry in 200..260 {
        if matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Hold(_)) { break; }
    }
    assert!(matches!(world.landing(LANDING).landing.phase, landing::Phase::Held { why: landing::Hold::Faults { member: 0, gate: 0 }, .. }));
    assert_eq!(world.landing_hold_words(LANDING).as_deref(), Some(b"gate gate of org/repo failed on its runner past its retries".as_slice()));
    assert_eq!(world.change(CHANGE).change.repairs, 0);
    assert_eq!(world.take_calls().iter().filter(|op| matches!(op, api::Op::Write(api::Write::Rerun { .. }))).count(), 2, "one retry of each of two jobs");
}
#[test]
fn a_run_past_the_projects_timeout_holds_naming_its_run() {
    let mut world = configured(100, false, false);
    world.gate_policy(landing::Project { gate_timeout: Duration::from_secs(5), ..landing::Project::DEFAULT });
    proving(&mut world);
    for entry in 200..240 { if matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Hold(_)) { break; } }
    let landing::Phase::Held { why: landing::Hold::Timeout { run, .. }, .. } = world.landing(LANDING).landing.phase else { panic!("timeout hold: {:?}", world.landing(LANDING)) };
    assert_ne!(run, 0);
    assert_eq!(world.landing_hold_words(LANDING).as_deref(), Some(format!("gate gate of org/repo: run {run} running past its bound").as_bytes()));
    assert_eq!(world.change(CHANGE).change.repairs, 0);
}
#[test]
fn a_nonstale_round_pull_or_merge_refusal_holds_then_release_prepares_with_receipts() {
    for at_merge in [false, true] {
        let mut world = configured(2, false, false);
        if at_merge {
            proving(&mut world);
            for entry in 200..230 { landings::step_landing(&mut world, LANDING, entry); if matches!(world.landing(LANDING).landing.phase, landing::Phase::Turn { .. }) { break; } }
        } else {
            landings::step_landing(&mut world, LANDING, 100);
            assert!(matches!(landings::step_landing(&mut world, LANDING, 101), landing::Decision::Prepare { .. }));
        }
        world.permission(raw::Permission::Read);
        for entry in 300..320 { if matches!(landings::step_landing(&mut world, LANDING, entry), landing::Decision::Hold(_)) { break; } }
        assert!(matches!(world.landing(LANDING).landing.phase, landing::Phase::Held { why: landing::Hold::Turn, .. }), "{:?}", world.landing(LANDING));
        assert!(world.landing_hold_words(LANDING).expect("hold").starts_with(b"turn: "));
        world.permission(raw::Permission::Admin);
        world.event(top::Event::StepLanding { task: LANDING, entry: 400, heard: top::LandingHeard { held: Box::new([]), failure: Box::new([]), preparation: landing::Preparation::Idle, kept: Box::new([]), released: true, cancelled: false } });
        world.run_for(1);
        landings::drive_landing(&mut world, LANDING);
        assert_eq!(world.landing(LANDING).landing.preparation, 2);
        assert!(world.landing(LANDING).landing.members[0].landed.is_some());
        assert_eq!(world.take_calls().iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 1, "release keeps a passed receipt");
    }
}
#[test]
fn cancelling_the_landing_returns_its_member_ready_and_frees_the_train() {
    let mut world = configured(100, false, false);
    proving(&mut world);
    let ready = world.change(CHANGE).change.ready_since;
    world.event(top::Event::SettleEffects { task: LANDING, root: LANDING, ending: top::ReleaseEnding::Cancelled });
    assert_eq!(world.change(CHANGE).change.landing, None);
    assert!(matches!(world.change(CHANGE).change.state, change::State::Ready { .. }));
    assert_eq!(world.change(CHANGE).change.ready_since, ready);
    assert_eq!(world.first_waiting_change(), Some(CHANGE));
    let mut row = landings::landing_row(world.change(CHANGE));
    row.landing.task = 62; row.landing.round = landing::Round::new(b"temper/1/l62".as_slice().into());
    world.event(top::Event::Landing { row });
    assert_eq!(world.change(CHANGE).change.landing, Some(62));
}

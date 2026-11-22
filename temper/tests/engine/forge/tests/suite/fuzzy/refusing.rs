//! Other users and a finite policy refusal keep the batch checks (testing/fakes.md, 2).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use skein_world::{
    faults::Kind,
    streams::{Stream, Streams},
    sweep::{Exclusion, Sweep},
};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{REPO, World, changes, fake_config, translate};

const CLOSE_REFUSED: Kind = Kind { family: "refusing", name: "close_refused" };
const CLOSED_BY_ANOTHER: Kind = Kind { family: "refusing", name: "closed_by_another" };
const MERGED_BY_ANOTHER: Kind = Kind { family: "refusing", name: "merged_by_another" };
const OUTSIDE: u64 = 54;
const OUTSIDE_BRANCH: &[u8] = b"temper/outside";

// This change is open but never queued with the batch. Its release follows
// changes.rs's other-user merge: cancelled, with no engine landing record.
fn merge_unqueued(world: &mut World) {
    world.external(raw::Op::Git(raw::Git::Create { branch: OUTSIDE_BRANCH.into(), commit: world.branch(b"main") }), 2);
    world.push_file(OUTSIDE_BRANCH, b"outside", b"not a batch member");
    let pull = world.open_pull(OUTSIDE_BRANCH, b"main");
    let mut row = changes::change_row();
    row.task = OUTSIDE;
    row.change.task = OUTSIDE;
    row.branch = OUTSIDE_BRANCH.into();
    row.pull = Some(pull);
    let head = translate::commit(world.branch(OUTSIDE_BRANCH));
    row.change.last_head = Some(head);
    row.change.state = change::State::Checking { head };
    world.event(top::Event::Change { row });
    let branch = top::Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: top::What::Branch(Box::new([b"temper".as_slice().into(), b"outside".as_slice().into()])),
    };
    let name = top::Name { forge: REPO.forge, repository: REPO.repository, what: top::What::Pull(pull) };
    world.merge_open_change_after_read(OUTSIDE, MERGED_BY_ANOTHER);
    world.event(top::Event::Hold { entry: 0, task: OUTSIDE, resource: branch.clone(), from: None });
    world.event(top::Event::Names { task: OUTSIDE, resources: Box::new([name.clone()]) });
    world.take_calls();
    world.run_for(60);
    assert!(
        matches!(world.stored().get(&top::Key::PullState(name)),
        Some(top::Stored::PullState(state)) if state.state == api::State::Closed),
        "connector read back the other user's merge"
    );
    let reads = world
        .take_calls()
        .iter()
        .filter(|op| {
            matches!(op,
        api::Op::Read(api::Read::Pull { number, .. }) if *number == pull)
        })
        .count();
    assert!(reads >= 2, "connector reads before and after the other user's merge");
    assert!(!matches!(world.change(OUTSIDE).change.state, change::State::Held { .. }), "an external merge is not held");
    release_unqueued(world, &branch);
}

fn release_unqueued(world: &mut World, branch: &top::Name) {
    world.event(top::Event::SettleEffects { task: OUTSIDE, root: OUTSIDE, ending: top::ReleaseEnding::Cancelled });
    world.event(top::Event::Release {
        task: OUTSIDE,
        root: OUTSIDE,
        ending: top::ReleaseEnding::Cancelled,
        reason: Some(b"cancelled".as_slice().into()),
        entry: 100_000,
    });
    let mut released = false;
    for entry in 100_001..100_010 {
        world.run_for(1);
        for request in world.take_seen() {
            match request {
                top::Request::Released { task: OUTSIDE } => released = true,
                top::Request::ContinueRelease { task: OUTSIDE } => {
                    world.event(top::Event::ContinueRelease { task: OUTSIDE, entry });
                }
                top::Request::ReleaseFailed { task: OUTSIDE, .. } => panic!("external merge release held: {request:?}"),
                _ => {}
            }
        }
        if released {
            break;
        }
    }
    assert!(released, "the open, unqueued change ends");
    assert!(world.hold(branch).is_none());
    assert_eq!(world.change(OUTSIDE).ending, Some(top::Ending::Cancelled));
    assert!(!world.stored().iter().any(|(_, record)| matches!(record, top::Stored::Landed { task: OUTSIDE, .. })));
    let calls = world.take_calls();
    assert!(!calls.iter().any(|op| matches!(op, api::Op::Write(api::Write::Post { .. } | api::Write::Close { .. }))));
    assert!(calls.iter().any(
        |op| matches!(op, api::Op::Write(api::Write::DeleteBranch { branch }) if branch.as_ref() == OUTSIDE_BRANCH)
    ));
}

#[test]
fn refusals_keep_the_batches_invariants() {
    let mut sweep = Sweep::new("refusals_keep_the_batches_invariants");
    for kind in [CLOSE_REFUSED, CLOSED_BY_ANOTHER, MERGED_BY_ANOTHER] {
        sweep.configure(kind);
    }
    let seed = 1;
    eprintln!("seed {seed}");
    let streams = Streams::new(seed);
    let workload = streams.seed_of(Stream::Workload, "landing_batches");
    let mut world = World::batch_turn_in(&streams, Some(super::super::landing_batches::workflow(true)), fake_config());
    let holding = &["a change ready or queued for landing, or a batch member, until its record"];
    sweep
        .exclude(Exclusion::new(
            "a_member_merged_by_another_lands_like_any_landing",
            "landing train",
            MERGED_BY_ANOTHER,
            holding,
        ))
        .expect("one unrecorded-batch exclusion");
    world.exclude_batch_member_merges(sweep.leaves_out(MERGED_BY_ANOTHER, holding).expect("batch exclusion"));
    world.refuse_next_close_for(Duration::from_secs(2));
    world.other_users_after_member_reads(
        vec![(51, CLOSED_BY_ANOTHER, false), (52, MERGED_BY_ANOTHER, true), (53, MERGED_BY_ANOTHER, true)],
        Duration::from_secs(4),
    );
    let (mut world, _) = super::landing_batches::run_repairing_in(world, workload, 360);
    merge_unqueued(&mut world);
    eprintln!("seed {seed}: {} restored-fault holds released", world.restored_fault_releases());
    sweep.ran(world.ledger());
    let excluded = sweep.left_out("a_member_merged_by_another_lands_like_any_landing");
    assert!(excluded > 0, "the queued/batch exclusion was reached");
    eprintln!("seed {seed}: {excluded} queued/batch merge draws left out");
    sweep.check_fell().expect("configured refusals fell");
}

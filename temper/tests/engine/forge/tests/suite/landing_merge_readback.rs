//! A turn settles uncertain writes and reads containing tips (domain/landing-turn.md, 1).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{
    World,
    landings::{self, LANDING},
    translate,
};

fn at_turn(count: u64) -> World {
    let mut world = super::landing_batches::batch(count);
    for entry in 1000..1100 {
        landings::step_landing(&mut world, LANDING, entry);
        if matches!(world.landing(LANDING).landing.phase, landing::Phase::Turn { .. }) {
            return world;
        }
    }
    panic!("turn not reached: {:?}", world.landing(LANDING));
}

fn merge(world: &mut World) -> u64 {
    let pull = world.landing(LANDING).landing.round.pull.expect("round pull");
    assert!(matches!(landings::step_landing_raw(world, LANDING, 2000), landing::Decision::Merge { .. }));
    pull
}

fn assert_landed(world: &World) {
    let row = world.landing(LANDING);
    assert_eq!(row.landing.phase, landing::Phase::Landed);
    assert_eq!(row.landing.stale_proofs, 0);
    assert_eq!(row.landing.preparation, 1);
    for member in &row.landing.members {
        let prefix = member.candidate.expect("prefix");
        assert_eq!(member.landed, Some(prefix));
        assert!(
            matches!(world.change(member.change).change.state, change::State::Landed { merge, .. } if merge == prefix)
        );
    }
    assert!(world.landing_hold_words(LANDING).is_none());
}

fn merges(calls: &[api::Op], pull: u64) -> usize {
    calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Merge { number, .. }) if *number == pull)).count()
}

#[test]
fn a_timed_out_merge_found_merged_lands_every_member_at_its_prefix() {
    let mut world = at_turn(3);
    world.timeout_merges(1);
    let pull = merge(&mut world);
    world.run_for(20);
    assert!(world.take_seen().iter().any(|request| matches!(
        request,
        top::Request::Outcome { outcome: skein_forge_client::Outcome::Made { found: true, .. }, .. }
    )));
    landings::drive_landing(&mut world, LANDING);
    assert_landed(&world);
    assert_eq!(merges(&world.take_calls(), pull), 1, "the client's find, not a second merge, settled it");
}

#[test]
fn a_merge_lost_before_the_forge_is_made_again_while_its_pull_stays_open() {
    let mut world = at_turn(3);
    world.lose_writes();
    let pull = merge(&mut world);
    let raw::Answer::Pull(read) = world.inspect(&raw::Read::Pull { number: pull }) else { panic!("pull") };
    assert_eq!(read.state, raw::State::Open);
    assert_eq!(read.merged, None);
    world.answer_writes();
    landings::drive_landing(&mut world, LANDING);
    assert_landed(&world);
    assert_eq!(merges(&world.take_calls(), pull), 2, "one lost call, one client retry");
}

#[test]
fn a_closed_unmerged_round_prepares_again_without_dispatching_its_gate_again() {
    let mut world = at_turn(1);
    let base = world.branch(b"main");
    world.lose_writes();
    let pull = merge(&mut world);
    world.close(pull);
    world.answer_writes();
    world.run_for(20);
    assert_eq!(world.branch(b"main"), base);
    let raw::Answer::Pull(read) = world.inspect(&raw::Read::Pull { number: pull }) else { panic!("pull") };
    assert_eq!((read.state, read.merged), (raw::State::Closed, None));
    landings::drive_landing(&mut world, LANDING);
    let row = world.landing(LANDING);
    assert_eq!(row.landing.phase, landing::Phase::Landed);
    assert_eq!(row.landing.preparation, 2);
    assert!(row.landing.members[0].landed.is_some());
    assert_ne!(row.landing.round.pull, Some(pull));
    assert!(world.landing_hold_words(LANDING).is_none());
    let calls = world.take_calls();
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::Dispatch { .. }))).count(), 1);
    assert_eq!(calls.iter().filter(|op| matches!(op, api::Op::Write(api::Write::OpenPull { .. }))).count(), 2);
}

#[test]
fn a_tip_pushed_above_the_merged_candidate_lands_every_prefix_without_a_stale_round() {
    let mut world = at_turn(3);
    world.timeout_merges(1);
    merge(&mut world);
    let candidate = world.landing(LANDING).landing.members[2].candidate.expect("candidate");
    assert_eq!(translate::commit(world.branch(b"main")), candidate);
    world.push_file(b"main", b"after", b"pushed atop the candidate before readback");
    let tip = world.branch(b"main");
    assert_ne!(translate::commit(tip), candidate);
    landings::drive_landing(&mut world, LANDING);
    assert_landed(&world);
    assert_eq!(world.branch(b"main"), tip);
    assert!(world.take_calls().iter().any(|op| matches!(op,
        api::Op::Read(api::Read::Commits { before, after, page: 1 })
            if *before == translate::commit(tip) && *after == candidate)));
}

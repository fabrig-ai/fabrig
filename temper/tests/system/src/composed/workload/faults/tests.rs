use super::*;
use skein_lib::StateHasher;
use skein_world::streams::{Stream, Streams};
use std::hash::Hasher;
use temper_faults::people;
use temper_workload::{
    draw::{COMPOSED, Workload},
    parties::{self, When},
};

const KINDS: [Kind; 13] = [
    forge_faults::TIMED_OUT,
    forge_faults::HEAD_BEHIND,
    forge_faults::JOB_LOG,
    forge_faults::CLOSED_BY_ANOTHER,
    temper_blame::LOST,
    stale::MOVED_BASE,
    skein_llm_kit::faults::RATE_WINDOW,
    restarts::ENGINE_CRASH_MID_TURN,
    DISK_FULL,
    people::HOLDS_A_MEMBER,
    people::RELEASES_A_MEMBER,
    parks::DELIVERED_THEN_PARKED,
    bounds::FORGE_READ_ADMISSIONS,
];

fn digest(planned: &[(Kind, &str)], acts: &[Action]) -> u64 {
    let mut digest = StateHasher::new();
    digest.write(format!("{planned:?}\n{acts:?}").as_bytes());
    digest.finish()
}

#[test]
fn composed_draw_keeps_seed_six() {
    let draw = FaultDraw::new(6, &KINDS);
    assert_eq!(digest(&draw.planned(), draw.acts()), 15_246_353_655_105_236_273);
}

#[test]
fn composed_draw_classifies_every_kind() {
    for seed in (0..128).chain([u64::MAX]) {
        let workload = Workload::draw(&Streams::new(seed), &COMPOSED);
        let draw = FaultDraw::new(seed, &KINDS);
        let planned = draw.planned();
        let ineligible = draw.ineligible();
        for kind in KINDS {
            assert_ne!(
                planned.iter().any(|(planned, _)| *planned == kind),
                ineligible.contains(&kind),
                "seed {seed}: {kind:?}"
            );
        }
        for (kind, goal) in planned {
            assert!(KINDS.contains(&kind));
            if kind == bounds::FORGE_READ_ADMISSIONS {
                assert_eq!(goal, "");
                continue;
            }
            let goal = workload.goal(goal).expect("planned goal exists");
            match kind {
                stale::MOVED_BASE | people::HOLDS_A_MEMBER | people::RELEASES_A_MEMBER => assert!(goal.train),
                forge_faults::HEAD_BEHIND | forge_faults::JOB_LOG => {
                    assert!(goal.changes.iter().any(|change| change.red_once))
                }
                parks::DELIVERED_THEN_PARKED => assert!(goal.changes.iter().any(|change| !change.asks)),
                _ => {}
            }
        }
        for (index, hold) in draw.acts().iter().enumerate().filter(|(_, act)| act.act == Act::HoldMember) {
            assert!(
                draw.acts()[index + 1..].iter().any(|release| release.act == Act::ReleaseMember
                    && release.party == hold.party
                    && release.target == hold.target),
                "seed {seed}: {hold:?} has no release"
            );
        }
    }
}

#[test]
fn composed_draw_omits_kinds_without_a_red_change() {
    let mut absent = vec![forge_faults::HEAD_BEHIND, forge_faults::JOB_LOG];
    absent.sort_unstable();
    let seed = 2;
    let workload = Workload::draw(&Streams::new(seed), &COMPOSED);
    assert!(workload.goals.iter().flat_map(|goal| &goal.changes).all(|change| !change.red_once));
    let draw = FaultDraw::new(seed, &KINDS);
    assert_eq!(draw.ineligible(), absent);
    assert_eq!(draw.planned().len(), KINDS.len() - absent.len());
    let mut rng = Streams::new(seed).rng(Stream::Faults, "forge");
    // Time out, head lag and job log each spend a target and a step.
    for _ in 0..6 {
        rng.next_u64();
    }
    let goal = usize::try_from(rng.below(3)).expect("three goals");
    let after = rng.below(16);
    let close =
        draw.plans.iter().find(|plan| plan.kind == forge_faults::CLOSED_BY_ANOTHER).expect("close has a target");
    assert_eq!((close.goal.as_str(), close.after), (workload.goals[goal].name.as_str(), after));
    let only = FaultDraw::new(seed, &[forge_faults::TIMED_OUT]);
    assert_eq!(only.planned(), draw.planned()[..1]);
    assert_eq!(only.ineligible(), []);
    assert_eq!(only.acts(), []);
}

#[test]
fn composed_draw_adds_the_missing_release() {
    let streams = Streams::new(1);
    let workload = Workload::draw(&streams, &COMPOSED);
    let original = parties::draw(&streams, &workload, &[Act::HoldMember, Act::ReleaseMember]);
    let [hold] = &original[..] else { panic!("seed 1 draws a lone hold: {original:?}") };
    assert_eq!(hold.act, Act::HoldMember);
    let after = u32::try_from(streams.rng(Stream::Parties, "release").between(0, u64::from(parties::AFTER_MOST)))
        .expect("bounded delay");
    let release = Action { act: Act::ReleaseMember, when: When::Held, after, ..hold.clone() };
    let draw = FaultDraw::new(1, &KINDS);
    assert_eq!(draw.acts(), &[hold.clone(), release]);
}

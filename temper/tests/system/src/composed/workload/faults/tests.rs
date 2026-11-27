use super::*;
use skein_lib::StateHasher;
use skein_world::streams::{Stream, Streams};
use std::hash::Hasher;
use temper_workload::{
    draw::{COMPOSED, Goal, Workload},
    parties::{self, When},
};

fn digest(planned: &[(Kind, &str)], acts: &[Action]) -> u64 {
    let mut digest = StateHasher::new();
    digest.write(format!("{planned:?}\n{acts:?}").as_bytes());
    digest.finish()
}

#[test]
fn composed_draw_keeps_seed_six() {
    let draw = FaultDraw::new(6, GATE_KINDS);
    assert_eq!(digest(&draw.planned(), draw.acts()), 15_246_353_655_105_236_273);
}

#[test]
fn composed_draw_classifies_every_kind() {
    for seed in (0..128).chain([u64::MAX]) {
        let workload = Workload::draw(&Streams::new(seed), &COMPOSED);
        let draw = FaultDraw::new(seed, GATE_KINDS);
        let planned = draw.planned();
        let ineligible = draw.ineligible();
        for &kind in GATE_KINDS {
            assert_ne!(
                planned.iter().any(|(planned, _)| *planned == kind),
                ineligible.contains(&kind),
                "seed {seed}: {kind:?}"
            );
        }
        for (kind, goal) in planned {
            assert!(GATE_KINDS.contains(&kind));
            if kind == bounds::FORGE_READ_ADMISSIONS {
                assert_eq!(goal, "");
                continue;
            }
            let goal = workload.goal(goal).expect("planned goal exists");
            match kind {
                stale::MOVED_BASE | people::HOLDS_A_MEMBER | people::RELEASES_A_MEMBER => assert!(goal.train),
                forge_faults::HEAD_BEHIND | forge_faults::JOB_LOG => assert!(red_goal(goal)),
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
    let draw = FaultDraw::new(seed, GATE_KINDS);
    assert_eq!(draw.ineligible(), absent);
    assert_eq!(draw.planned().len(), GATE_KINDS.len() - absent.len());
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
    let draw = FaultDraw::new(1, GATE_KINDS);
    assert_eq!(draw.acts(), &[hold.clone(), release]);
}

fn red_goal(goal: &Goal) -> bool {
    goal.changes.iter().any(|change| change.red_once)
}

#[test]
fn catalogue_kind_draws_its_own_coin_target_and_moment() {
    const EXTRA: Kind = Kind { family: "ci", name: "test_red" };
    let kinds = [GATE_KINDS, &[EXTRA]].concat();
    let (mut planned, mut skipped, mut ineligible) = (false, false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::draw_with_catalogue(seed, &workload, &kinds, &[(EXTRA, red_goal)]);
        let gate = FaultDraw::new(seed, GATE_KINDS);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != EXTRA).collect::<Vec<_>>(),
            gate.plans.iter().collect::<Vec<_>>()
        );
        assert_eq!(draw.acts(), gate.acts());

        let plan = draw.plans.iter().find(|plan| plan.kind == EXTRA);
        let mut rng = streams.rng(Stream::Faults, "ci/test_red");
        let goals: Vec<_> = workload.goals.iter().filter(|goal| red_goal(goal)).collect();
        if rng.below(2) == 0 {
            assert!(plan.is_none(), "seed {seed}: the coin skips the kind");
            assert!(!draw.ineligible.contains(&EXTRA));
            skipped = true;
        } else if goals.is_empty() {
            assert!(plan.is_none());
            assert!(draw.ineligible.contains(&EXTRA), "seed {seed}: the coin fell but no goal was eligible");
            ineligible = true;
        } else {
            let plan = plan.expect("the coin fell on an eligible workload");
            let index =
                usize::try_from(rng.below(u64::try_from(goals.len()).expect("bounded goals"))).expect("goal index");
            assert_eq!(plan.goal, goals[index].name, "seed {seed}: the target comes after the coin");
            assert_eq!(plan.after, rng.below(16), "seed {seed}: the moment comes after the target");
            assert!(!draw.ineligible.contains(&EXTRA));
            planned = true;
        }
    }
    assert!(planned && skipped && ineligible);
}

#[test]
fn spent_credits_draws_its_own_coin_target_and_moment() {
    use skein_llm_kit::faults::SPENT_CREDITS;

    assert!(!GATE_KINDS.contains(&SPENT_CREDITS));
    assert!(CAMPAIGN_KINDS.contains(&SPENT_CREDITS));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without_spent: Vec<_> = kinds.iter().copied().filter(|&kind| kind != SPENT_CREDITS).collect();
    let (mut lands, mut skips) = (false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let draw = FaultDraw::new(seed, &kinds);
        let before = FaultDraw::new(seed, &without_spent);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != SPENT_CREDITS).collect::<Vec<_>>(),
            before.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: other injection plans"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|&(kind, _)| kind != SPENT_CREDITS).collect::<Vec<_>>(),
            before.planned(),
            "seed {seed}: other plans"
        );
        assert_eq!(draw.ineligible(), before.ineligible(), "seed {seed}: ineligible kinds");
        assert_eq!(draw.acts(), before.acts(), "seed {seed}: acts");

        let streams = Streams::new(seed);
        let mut rng = streams.rng(Stream::Faults, "provider/spent credits");
        let spent: Vec<_> = draw.plans.iter().filter(|plan| plan.kind == SPENT_CREDITS).collect();
        if rng.below(2) == 0 {
            assert!(spent.is_empty(), "seed {seed}: the coin skips spent credits");
            skips = true;
        } else {
            let workload = Workload::draw(&streams, &COMPOSED);
            let index = usize::try_from(rng.below(u64::try_from(workload.goals.len()).expect("bounded goals")))
                .expect("goal index");
            let [plan] = spent.as_slice() else { panic!("seed {seed}: the coin plans spent credits once: {spent:?}") };
            assert_eq!(plan.goal, workload.goals[index].name, "seed {seed}: the target comes after the coin");
            assert_eq!(plan.after, rng.below(16), "seed {seed}: the moment comes after the target");
            lands = true;
        }
    }
    assert!(lands && skips);
}

#[test]
fn settled_mid_turn_draws_its_own_coin_goal_and_held_query() {
    use restarts::ENGINE_RESTART_MID_TURN as SETTLED;

    assert!(!GATE_KINDS.contains(&SETTLED));
    assert!(CAMPAIGN_KINDS.contains(&SETTLED));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != SETTLED).collect();
    let mut goal_indices = BTreeSet::new();
    let mut queries = BTreeSet::new();
    let mut skipped = false;
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let baseline = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|p| p.kind != SETTLED).collect::<Vec<_>>(),
            baseline.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: another kind's plan moved"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|(kind, _)| *kind != SETTLED).collect::<Vec<_>>(),
            baseline.planned(),
            "seed {seed}: another kind's target moved"
        );
        assert_eq!(draw.ineligible(), baseline.ineligible(), "seed {seed}");
        assert_eq!(draw.acts(), baseline.acts(), "seed {seed}");

        let plan = draw.plans.iter().find(|plan| plan.kind == SETTLED);
        let mut rng = streams.rng(Stream::Faults, "restarts/engine restarted mid-turn");
        if rng.below(2) == 0 {
            assert!(plan.is_none(), "seed {seed}: the coin skips the kind");
            skipped = true;
        } else {
            let index =
                usize::try_from(rng.below(workload.goals.len().try_into().expect("goal count"))).expect("goal index");
            let after = rng.below(16);
            let query = usize::try_from(rng.between(1, 2)).expect("held query");
            let plan = plan.unwrap_or_else(|| panic!("seed {seed}: the coin plans a restart"));
            assert_eq!(
                (plan.goal.as_str(), plan.after, plan.query),
                (workload.goals[index].name.as_str(), after, query),
                "seed {seed}"
            );
            goal_indices.insert(index);
            queries.insert(query);
        }
    }
    assert!(skipped);
    assert_eq!(goal_indices, BTreeSet::from([0, 1, 2]));
    assert_eq!(queries, BTreeSet::from([1, 2]));
}

#[test]
fn busy_catalogue_kinds_draw_their_own_coins_targets_and_moments() {
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    for (kind, part) in [(forge_faults::UNAVAILABLE, "busy/unavailable"), (forge_faults::LIMITED, "busy/limited")] {
        assert!(CAMPAIGN_KINDS.contains(&kind), "{part} is configured outside the gate");
        assert!(!GATE_KINDS.contains(&kind), "{part} is not configured at the gate");
        let without: Vec<_> = kinds.iter().copied().filter(|&other| other != kind).collect();
        let mut targets = BTreeSet::new();
        let mut skipped = false;
        for seed in (0..64).chain([u64::MAX]) {
            let streams = Streams::new(seed);
            let workload = Workload::draw(&streams, &COMPOSED);
            let draw = FaultDraw::new(seed, &kinds);
            let baseline = FaultDraw::new(seed, &without);
            assert_eq!(
                draw.plans.iter().filter(|plan| plan.kind != kind).collect::<Vec<_>>(),
                baseline.plans.iter().collect::<Vec<_>>(),
                "seed {seed}: adding {part} keeps the other injection plans"
            );
            assert_eq!(
                draw.planned().into_iter().filter(|(other, _)| *other != kind).collect::<Vec<_>>(),
                baseline.planned(),
                "seed {seed}: adding {part} keeps the other planned goals"
            );
            assert_eq!(draw.ineligible(), baseline.ineligible(), "seed {seed}: {part}");
            assert_eq!(draw.acts(), baseline.acts(), "seed {seed}: {part}");

            let mut rng = streams.rng(Stream::Faults, part);
            let expected = (rng.below(2) != 0).then(|| {
                let index = usize::try_from(rng.below(u64::try_from(workload.goals.len()).expect("bounded goals")))
                    .expect("goal index");
                targets.insert(index);
                (workload.goals[index].name.as_str(), rng.below(16))
            });
            skipped |= expected.is_none();
            let plans: Vec<_> = draw
                .plans
                .iter()
                .filter(|plan| plan.kind == kind)
                .map(|plan| (plan.goal.as_str(), plan.after))
                .collect();
            assert_eq!(plans, expected.as_slice(), "seed {seed}: {part} coin, target and moment");
        }
        assert!(skipped, "{part} is skipped on a losing coin");
        assert_eq!(targets.len(), usize::from(COMPOSED.goals.most), "{part} can strike any goal");
    }
}

#[test]
fn set_up_draws_its_own_coin_goal_and_moment() {
    use temper_blame::SET_UP;
    assert!(CAMPAIGN_KINDS.contains(&SET_UP));
    assert!(!GATE_KINDS.contains(&SET_UP));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without_setup: Vec<_> = kinds.iter().copied().filter(|kind| *kind != SET_UP).collect();
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let without = FaultDraw::new(seed, &without_setup);
        assert_eq!(
            draw.planned().into_iter().filter(|(kind, _)| *kind != SET_UP).collect::<Vec<_>>(),
            without.planned(),
            "seed {seed}"
        );
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != SET_UP).collect::<Vec<_>>(),
            without.plans.iter().collect::<Vec<_>>(),
            "seed {seed}"
        );
        assert_eq!(draw.acts(), without.acts(), "seed {seed}");
        assert_eq!(draw.ineligible(), without.ineligible(), "seed {seed}");

        let mut rng = streams.rng(Stream::Faults, "ci/set_up");
        let expected = (rng.below(2) != 0).then(|| {
            let index = usize::try_from(rng.below(u64::try_from(workload.goals.len()).expect("bounded goals")))
                .expect("goal index");
            (workload.goals[index].name.as_str(), rng.below(16))
        });
        let planned: Vec<_> =
            draw.plans.iter().filter(|plan| plan.kind == SET_UP).map(|plan| (plan.goal.as_str(), plan.after)).collect();
        assert_eq!(planned, expected.as_slice(), "seed {seed}: ci/set_up coin, goal and moment");
    }
}

#[test]
fn wrong_tree_is_observed_without_a_coin_when_a_change_can_go_red() {
    use temper_blame::WRONG_TREE;
    let (mut planned, mut ineligible) = (false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let workload = Workload::draw(&Streams::new(seed), &COMPOSED);
        let draw = FaultDraw::new(seed, &[WRONG_TREE]);
        assert!(draw.plans.is_empty(), "observed kinds have no injection plan");
        if workload.goals.iter().any(red_goal) {
            assert_eq!(draw.planned(), [(WRONG_TREE, "")], "seed {seed}");
            assert_eq!(draw.ineligible(), []);
            planned = true;
        } else {
            assert_eq!(draw.planned(), []);
            assert_eq!(draw.ineligible(), [WRONG_TREE], "seed {seed}");
            ineligible = true;
        }
    }
    assert!(planned && ineligible);
}

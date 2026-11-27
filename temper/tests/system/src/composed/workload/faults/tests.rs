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
fn branch_holds_draws_its_own_coin_without_a_goal() {
    use bounds::BRANCH_HOLDS;
    assert!(!GATE_KINDS.contains(&BRANCH_HOLDS));
    assert!(CAMPAIGN_KINDS.contains(&BRANCH_HOLDS));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|kind| *kind != BRANCH_HOLDS).collect();
    let (mut planned, mut skipped) = (false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let gate = FaultDraw::new(seed, GATE_KINDS);
        assert_eq!(gate.bounds(), &[bounds::FORGE_READ_ADMISSIONS], "seed {seed}");
        let draw = FaultDraw::new(seed, &kinds);
        let unchanged = FaultDraw::new(seed, &without);
        let mut observed = draw.planned();
        let coin = Streams::new(seed).rng(Stream::Faults, "the engine's own bounds/branch holds").below(2);
        if coin == 0 {
            assert!(!observed.iter().any(|(kind, _)| *kind == BRANCH_HOLDS), "seed {seed}");
            skipped = true;
        } else {
            let index = observed.iter().position(|(kind, _)| *kind == BRANCH_HOLDS).expect("coin plans branch holds");
            assert_eq!(observed.remove(index), (BRANCH_HOLDS, ""), "seed {seed}");
            planned = true;
        }
        assert_eq!(observed, unchanged.planned(), "seed {seed}");
        assert_eq!(draw.plans, unchanged.plans, "seed {seed}: injection moments");
        assert_eq!(draw.acts(), unchanged.acts(), "seed {seed}");
        assert_eq!(draw.ineligible(), unchanged.ineligible(), "seed {seed}");
    }
    assert!(planned && skipped);
}

#[test]
fn landing_rows_draws_its_own_coin_without_a_goal() {
    use bounds::LANDING_ROWS;
    assert!(!GATE_KINDS.contains(&LANDING_ROWS));
    assert!(CAMPAIGN_KINDS.contains(&LANDING_ROWS));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != LANDING_ROWS).collect();
    let (mut planned, mut skipped) = (false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let draw = FaultDraw::new(seed, &kinds);
        let unchanged = FaultDraw::new(seed, &without);
        let coin = Streams::new(seed).rng(Stream::Faults, "the engine's own bounds/landing rows").below(2);
        let observed: Vec<_> = draw.planned().into_iter().filter(|&(kind, _)| kind == LANDING_ROWS).collect();
        if coin == 0 {
            assert!(observed.is_empty(), "seed {seed}: the coin skips landing rows");
            skipped = true;
        } else {
            assert_eq!(observed, [(LANDING_ROWS, "")], "seed {seed}: the coin plans landing rows without a goal");
            planned = true;
        }
        assert_eq!(draw.bounds().contains(&LANDING_ROWS), coin != 0, "seed {seed}: observed bound");
        assert_eq!(
            draw.planned().into_iter().filter(|&(kind, _)| kind != LANDING_ROWS).collect::<Vec<_>>(),
            unchanged.planned(),
            "seed {seed}: other kinds"
        );
        assert_eq!(draw.plans, unchanged.plans, "seed {seed}: injection plans");
        assert_eq!(draw.acts(), unchanged.acts(), "seed {seed}: acts");
        assert_eq!(draw.ineligible(), unchanged.ineligible(), "seed {seed}: ineligible kinds");
    }
    assert!(planned && skipped);
}

#[test]
fn fetch_cut_draws_its_own_coin_goal_and_moment() {
    assert!(CAMPAIGN_KINDS.contains(&FETCH_CUT));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without_cut: Vec<_> = kinds.iter().copied().filter(|&kind| kind != FETCH_CUT).collect();
    let mut struck_goals = BTreeSet::new();
    let mut skipped = false;
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let without = FaultDraw::new(seed, &without_cut);
        let planned = draw.planned();
        assert_eq!(
            planned.iter().copied().filter(|&(kind, _)| kind != FETCH_CUT).collect::<Vec<_>>(),
            without.planned(),
            "seed {seed}: other targets"
        );
        assert!(draw.plans.iter().filter(|p| p.kind != FETCH_CUT).eq(&without.plans), "seed {seed}: other plans");
        assert_eq!(draw.acts(), without.acts(), "seed {seed}: acts");
        assert_eq!(draw.ineligible(), without.ineligible(), "seed {seed}: ineligible kinds");

        let mut rng = streams.rng(Stream::Faults, "worker/a workspace fetch timing out partway");
        let expected = if rng.below(2) == 0 {
            skipped = true;
            None
        } else {
            let goals = u64::try_from(workload.goals.len()).expect("bounded goals");
            let index = usize::try_from(rng.below(goals)).expect("goal index");
            struck_goals.insert(index);
            Some((workload.goals[index].name.as_str(), rng.below(16)))
        };
        assert_eq!(
            draw.plans.iter().filter(|p| p.kind == FETCH_CUT).map(|p| (p.goal.as_str(), p.after)).collect::<Vec<_>>(),
            expected.into_iter().collect::<Vec<_>>(),
            "seed {seed}: coin, then any goal, then moment"
        );
    }
    assert!(skipped);
    assert_eq!(struck_goals, BTreeSet::from([0, 1, 2]));
}

#[test]
fn missing_report_draws_its_own_coin_target_and_moment() {
    use temper_blame::MISSING_REPORT;
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != MISSING_REPORT).collect();
    let (mut planned, mut skipped, mut ineligible) = (false, false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let with = FaultDraw::new(seed, &kinds);
        let base = FaultDraw::new(seed, &without);
        assert_eq!(with.acts(), base.acts());
        assert_eq!(
            with.plans.iter().filter(|plan| plan.kind != MISSING_REPORT).collect::<Vec<_>>(),
            base.plans.iter().collect::<Vec<_>>()
        );
        assert_eq!(
            with.planned().into_iter().filter(|&(kind, _)| kind != MISSING_REPORT).collect::<Vec<_>>(),
            base.planned()
        );
        assert_eq!(
            with.ineligible().into_iter().filter(|&kind| kind != MISSING_REPORT).collect::<Vec<_>>(),
            base.ineligible()
        );
        let plan = with.plans.iter().find(|plan| plan.kind == MISSING_REPORT);
        let mut rng = streams.rng(Stream::Faults, "ci/missing_report");
        let goals: Vec<_> = workload.goals.iter().filter(|goal| red_goal(goal)).collect();
        if rng.below(2) == 0 {
            assert!(plan.is_none(), "seed {seed}: the coin skips the kind");
            assert!(!with.ineligible().contains(&MISSING_REPORT));
            skipped = true;
        } else if goals.is_empty() {
            assert!(plan.is_none());
            assert!(with.ineligible().contains(&MISSING_REPORT), "seed {seed}: no red goal");
            ineligible = true;
        } else {
            let index =
                usize::try_from(rng.below(u64::try_from(goals.len()).expect("bounded goals"))).expect("goal index");
            let after = rng.below(16);
            let plan = plan.unwrap_or_else(|| panic!("seed {seed}: the coin fell on a red goal"));
            assert_eq!((plan.goal.as_str(), plan.after), (goals[index].name.as_str(), after), "seed {seed}");
            assert!(!with.ineligible().contains(&MISSING_REPORT));
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
fn expired_sign_in_draws_its_own_coin_target_and_moment() {
    use skein_llm_kit::faults::EXPIRED_SIGN_IN;

    assert!(CAMPAIGN_KINDS.contains(&EXPIRED_SIGN_IN));
    assert!(!GATE_KINDS.contains(&EXPIRED_SIGN_IN));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != EXPIRED_SIGN_IN).collect();
    let mut targets = BTreeSet::new();
    let mut skipped = false;
    for seed in (0..64).chain([u64::MAX]) {
        let draw = FaultDraw::new(seed, &kinds);
        let baseline = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != EXPIRED_SIGN_IN).collect::<Vec<_>>(),
            baseline.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: another kind's plan moved"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|&(kind, _)| kind != EXPIRED_SIGN_IN).collect::<Vec<_>>(),
            baseline.planned(),
            "seed {seed}: another kind's target moved"
        );
        assert_eq!(draw.ineligible(), baseline.ineligible(), "seed {seed}");
        assert_eq!(draw.acts(), baseline.acts(), "seed {seed}");

        let streams = Streams::new(seed);
        let mut rng = streams.rng(Stream::Faults, "provider/expired sign-in");
        let expired: Vec<_> = draw.plans.iter().filter(|plan| plan.kind == EXPIRED_SIGN_IN).collect();
        if rng.below(2) == 0 {
            assert!(expired.is_empty(), "seed {seed}: the coin skips expired sign-in");
            skipped = true;
        } else {
            let workload = Workload::draw(&streams, &COMPOSED);
            let index =
                usize::try_from(rng.below(workload.goals.len().try_into().expect("goal count"))).expect("goal index");
            let [plan] = expired.as_slice() else { panic!("seed {seed}: the coin plans expired sign-in once") };
            assert_eq!(plan.goal, workload.goals[index].name, "seed {seed}: target drawn after the coin");
            assert_eq!(plan.after, rng.below(16), "seed {seed}: moment drawn after the target");
            targets.insert(index);
        }
    }
    assert!(skipped);
    assert_eq!(targets, BTreeSet::from([0, 1, 2]));
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
fn base_behind_draws_its_own_coin_target_and_moment() {
    use forge_faults::BASE_BEHIND;
    assert!(CAMPAIGN_KINDS.contains(&BASE_BEHIND));
    assert!(!GATE_KINDS.contains(&BASE_BEHIND));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != BASE_BEHIND).collect();
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let other = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != BASE_BEHIND).collect::<Vec<_>>(),
            other.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: other injection plans"
        );
        let planned = draw.planned();
        assert_eq!(
            planned.iter().copied().filter(|(kind, _)| *kind != BASE_BEHIND).collect::<Vec<_>>(),
            other.planned(),
            "seed {seed}: other planned kinds"
        );
        assert_eq!(draw.ineligible(), other.ineligible(), "seed {seed}: ineligible kinds");
        assert_eq!(draw.acts(), other.acts(), "seed {seed}: party acts");

        let mut rng = streams.rng(Stream::Faults, "lagging/base_behind");
        let expected = if rng.below(2) == 0 {
            None
        } else {
            let count = u64::try_from(workload.goals.len()).expect("bounded goals");
            let index = usize::try_from(rng.below(count)).expect("goal index");
            Some((workload.goals[index].name.as_str(), rng.below(16)))
        };
        let actual: Vec<_> = draw
            .plans
            .iter()
            .filter(|plan| plan.kind == BASE_BEHIND)
            .map(|plan| (plan.goal.as_str(), plan.after))
            .collect();
        assert_eq!(actual, expected.as_slice(), "seed {seed}: BASE_BEHIND's coin, goal and moment");
    }
}

#[test]
fn merge_late_draws_its_own_coin_train_and_moment() {
    use forge_faults::MERGE_LATE;
    assert!(CAMPAIGN_KINDS.contains(&MERGE_LATE));
    assert!(!GATE_KINDS.contains(&MERGE_LATE));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != MERGE_LATE).collect();
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let other = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != MERGE_LATE).collect::<Vec<_>>(),
            other.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: other injection plans"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|&(kind, _)| kind != MERGE_LATE).collect::<Vec<_>>(),
            other.planned(),
            "seed {seed}: other planned kinds"
        );
        assert_eq!(draw.ineligible(), other.ineligible(), "seed {seed}: ineligible kinds");
        assert_eq!(draw.acts(), other.acts(), "seed {seed}: party acts");

        let mut rng = streams.rng(Stream::Faults, "lagging/merge_late");
        let expected = (rng.below(2) != 0).then(|| {
            let train = workload.goals.iter().find(|goal| goal.train).expect("composed train");
            let _ = rng.below(1);
            (train.name.as_str(), rng.below(16))
        });
        let actual: Vec<_> = draw
            .plans
            .iter()
            .filter(|plan| plan.kind == MERGE_LATE)
            .map(|plan| (plan.goal.as_str(), plan.after))
            .collect();
        assert_eq!(actual, expected.as_slice(), "seed {seed}: MERGE_LATE's coin, train and moment");
    }
}

#[test]
fn silent_draws_its_own_coin_goal_and_moment() {
    use temper_blame::SILENT;
    assert!(!GATE_KINDS.contains(&SILENT));
    assert!(CAMPAIGN_KINDS.contains(&SILENT));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != SILENT).collect();
    let mut targets = BTreeSet::new();
    let mut skipped = false;
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let baseline = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != SILENT).collect::<Vec<_>>(),
            baseline.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: other injection plans"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|(kind, _)| *kind != SILENT).collect::<Vec<_>>(),
            baseline.planned(),
            "seed {seed}: other planned goals"
        );
        assert_eq!(draw.acts(), baseline.acts(), "seed {seed}");
        assert_eq!(draw.ineligible(), baseline.ineligible(), "seed {seed}");

        let mut rng = streams.rng(Stream::Faults, "ci/silent");
        let expected = (rng.below(2) != 0).then(|| {
            let index = usize::try_from(rng.below(u64::try_from(workload.goals.len()).expect("bounded goals")))
                .expect("goal index");
            targets.insert(index);
            (workload.goals[index].name.as_str(), rng.below(16))
        });
        skipped |= expected.is_none();
        let planned: Vec<_> =
            draw.plans.iter().filter(|plan| plan.kind == SILENT).map(|plan| (plan.goal.as_str(), plan.after)).collect();
        assert_eq!(planned, expected.as_slice(), "seed {seed}: ci/silent coin, goal and moment");
    }
    assert!(skipped, "silence is skipped on a losing coin");
    assert_eq!(targets.len(), usize::from(COMPOSED.goals.most), "silence can strike any goal");
}

#[test]
fn close_refused_draws_its_own_coin_target_and_moment() {
    use forge_faults::CLOSE_REFUSED;

    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != CLOSE_REFUSED).collect();
    let (mut planned, mut skipped) = (false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let baseline = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != CLOSE_REFUSED).collect::<Vec<_>>(),
            baseline.plans.iter().collect::<Vec<_>>(),
            "seed {seed}"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|&(kind, _)| kind != CLOSE_REFUSED).collect::<Vec<_>>(),
            baseline.planned(),
            "seed {seed}"
        );
        assert_eq!(draw.ineligible(), baseline.ineligible(), "seed {seed}");
        assert_eq!(draw.acts(), baseline.acts(), "seed {seed}");

        let mut rng = streams.rng(Stream::Faults, "refusing/close_refused");
        let expected = if rng.below(2) == 0 {
            skipped = true;
            None
        } else {
            planned = true;
            let index = usize::try_from(rng.below(u64::try_from(workload.goals.len()).expect("bounded goals")))
                .expect("goal index");
            Some((workload.goals[index].name.as_str(), rng.below(16)))
        };
        let plan = draw.plans.iter().find(|plan| plan.kind == CLOSE_REFUSED);
        assert_eq!(plan.map(|plan| (plan.goal.as_str(), plan.after)), expected, "seed {seed}");
    }
    assert!(planned && skipped);
}

#[test]
fn merged_by_another_draws_its_own_coin_non_train_goal_and_moment() {
    use forge_faults::{CLOSE_REFUSED, MERGED_BY_ANOTHER};

    assert!(CAMPAIGN_KINDS.windows(2).any(|pair| pair == [CLOSE_REFUSED, MERGED_BY_ANOTHER]));
    assert!(!GATE_KINDS.contains(&MERGED_BY_ANOTHER));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != MERGED_BY_ANOTHER).collect();
    let (mut planned, mut skipped) = (false, false);
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let baseline = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != MERGED_BY_ANOTHER).collect::<Vec<_>>(),
            baseline.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: other injection plans"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|&(kind, _)| kind != MERGED_BY_ANOTHER).collect::<Vec<_>>(),
            baseline.planned(),
            "seed {seed}: other planned kinds"
        );
        assert_eq!(draw.ineligible(), baseline.ineligible(), "seed {seed}: ineligible kinds");
        assert_eq!(draw.acts(), baseline.acts(), "seed {seed}: party acts");

        let mut rng = streams.rng(Stream::Faults, "refusing/merged_by_another");
        let expected = if rng.below(2) == 0 {
            skipped = true;
            None
        } else {
            planned = true;
            let goals: Vec<_> = workload.goals.iter().filter(|goal| !goal.train).collect();
            let index =
                usize::try_from(rng.below(u64::try_from(goals.len()).expect("bounded goals"))).expect("goal index");
            Some((goals[index].name.as_str(), rng.below(16)))
        };
        let actual: Vec<_> = draw
            .plans
            .iter()
            .filter(|plan| plan.kind == MERGED_BY_ANOTHER)
            .map(|plan| (plan.goal.as_str(), plan.after))
            .collect();
        assert_eq!(actual, expected.as_slice(), "seed {seed}: outside merge's coin, non-train goal and moment");
    }
    assert!(planned && skipped);
}

#[test]
fn reopened_draws_its_own_coin_goal_and_moment() {
    use stale::REOPENED;

    assert!(CAMPAIGN_KINDS.contains(&REOPENED));
    assert!(!GATE_KINDS.contains(&REOPENED));
    let kinds = [GATE_KINDS, CAMPAIGN_KINDS].concat();
    let without: Vec<_> = kinds.iter().copied().filter(|&kind| kind != REOPENED).collect();
    let mut targets = BTreeSet::new();
    let mut skipped = false;
    for seed in (0..64).chain([u64::MAX]) {
        let streams = Streams::new(seed);
        let workload = Workload::draw(&streams, &COMPOSED);
        let draw = FaultDraw::new(seed, &kinds);
        let baseline = FaultDraw::new(seed, &without);
        assert_eq!(
            draw.plans.iter().filter(|plan| plan.kind != REOPENED).collect::<Vec<_>>(),
            baseline.plans.iter().collect::<Vec<_>>(),
            "seed {seed}: other injection plans"
        );
        assert_eq!(
            draw.planned().into_iter().filter(|&(kind, _)| kind != REOPENED).collect::<Vec<_>>(),
            baseline.planned(),
            "seed {seed}: other planned kinds"
        );
        assert_eq!(draw.ineligible(), baseline.ineligible(), "seed {seed}: ineligible kinds");
        assert_eq!(draw.acts(), baseline.acts(), "seed {seed}: party acts");

        let mut rng = streams.rng(Stream::Faults, "stale/reopened_by_another");
        let expected = if rng.below(2) == 0 {
            skipped = true;
            None
        } else {
            let goals = u64::try_from(workload.goals.len()).expect("bounded goals");
            let index = usize::try_from(rng.below(goals)).expect("goal index");
            let goal = workload.goals[index].name.as_str();
            targets.insert(index);
            Some((goal, rng.below(16)))
        };
        let actual: Vec<_> = draw
            .plans
            .iter()
            .filter(|plan| plan.kind == REOPENED)
            .map(|plan| (plan.goal.as_str(), plan.after))
            .collect();
        assert_eq!(actual, expected.as_slice(), "seed {seed}: reopen's coin, goal and moment");
    }
    assert!(skipped);
    assert_eq!(targets, BTreeSet::from([0, 1, 2]));
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

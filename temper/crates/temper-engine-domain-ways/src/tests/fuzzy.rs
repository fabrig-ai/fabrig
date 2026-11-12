//! Random paths keep the derivation's invariants (temper/docs/design/domain/forge/change/branch-path.md).
#![expect(clippy::disallowed_methods, reason = "ordinary random tests build and inspect bounded fixture paths")]
use crate::invariants::bounds;
use skein_lib::Rng;
use temper_engine_domain_forge::{
    Follows, GateLast, GatePattern, JudgeFreshness,
    settings::{Held, LandingRule, RuleApproval, RuleGate},
};
use temper_engine_domain_forge_change::Way;
use temper_engine_domain_ways::{Actor, Inputs, Repository, Role, Start, Step, Tier, derive};

#[test]
#[expect(clippy::too_many_lines, reason = "one seeded sweep builds inputs and checks their path invariants")]
fn paths_end_at_a_bar_and_only_code_lands() {
    let mut conflicts = 0_u32;
    for seed in 0..64 {
        let mut rng = Rng::new(seed);
        let by = if rng.chance(500) { Way::Queue } else { Way::Turn };
        let mut input = Inputs {
            branch: b"main".as_slice().into(),
            repository: Some(Repository {
                host: b"forge".as_slice().into(),
                owner: b"owner".as_slice().into(),
                name: b"name".as_slice().into(),
                ci: rng.chance(500),
                checks: Box::default(),
                contexts: Box::default(),
                workflows: Box::default(),
                tiers: Box::default(),
                gates: if rng.chance(500) { Box::new([b"unit".as_slice().into()]) } else { Box::default() },
            }),
            rules: Box::default(),
            start: Start::Chat,
            producer: 1,
            reviewer: 2,
            by,
            merge_job: rng.chance(500),
            roles: Box::new([Role { number: 0, name: b"owner".as_slice().into(), yours: rng.chance(500) }]),
            agents: Box::default(),
        };
        input.start = if rng.chance(500) {
            Start::Chat
        } else {
            Start::Goal { charter: 5, design: rng.chance(500), close_out: rng.chance(500) }
        };
        input.branch = if rng.chance(500) { b"main".as_slice().into() } else { b"release/v1".as_slice().into() };
        if rng.chance(100) {
            input.repository = None;
        } else {
            let repository = input.repository.as_mut().unwrap();
            repository.contexts = (0..rng.below(4))
                .map(|_| match rng.below(3) {
                    0 => b"build".as_slice().into(),
                    1 => b"lint".as_slice().into(),
                    _ => b"ci / test".as_slice().into(),
                })
                .collect();
            repository.workflows = (0..rng.below(4))
                .map(|_| match rng.below(3) {
                    0 => b"ci.yml".as_slice().into(),
                    1 => b"lint.yml".as_slice().into(),
                    _ => b"release.yaml".as_slice().into(),
                })
                .collect();
            repository.checks = (0..rng.below(3)).map(|_| u32::try_from(rng.below(4)).unwrap()).collect();
            repository.tiers = (0..rng.below(4))
                .map(|index| Tier {
                    name: if index == 0 { b"slow".as_slice().into() } else { b"goal".as_slice().into() },
                    follows: match rng.below(4) {
                        0 => Follows::Landing,
                        1 => Follows::Goals,
                        2 => Follows::Branch(b"release/v1".as_slice().into()),
                        _ => Follows::Branch(b"temper/0/g0".as_slice().into()),
                    },
                    description: Box::default(),
                })
                .collect();
        }
        input.rules = (0..rng.below(9))
            .map(|number| {
                let segments = if rng.chance(500) {
                    Box::from([b"branch".as_slice().into()])
                } else {
                    [b"forge".as_slice(), b"forge", b"owner", b"name", b"branch"].map(Box::from).into()
                };
                let last = match rng.below(5) {
                    0 => GateLast::Exact(b"main".as_slice().into()),
                    1 => GateLast::Open(b"temper".as_slice().into()),
                    2 => GateLast::Exact(b"not-main".as_slice().into()),
                    _ => GateLast::Open(Box::default()),
                };
                Held {
                    scope: if rng.chance(500) { 0 } else { 9 },
                    number: u32::try_from(number).unwrap(),
                    rule: LandingRule {
                        sets: Box::new([]),
                        pattern: GatePattern { segments, last },
                        enforce: rng.chance(500),
                        ci: rng.chance(500),
                        up_to_date: rng.chance(500),
                        gates: (0..rng.below(4))
                            .map(|_| RuleGate {
                                number: u32::try_from(rng.below(4)).unwrap(),
                                blocking: rng.chance(500),
                                freshness: if rng.chance(500) { JudgeFreshness::Clean } else { JudgeFreshness::Exact },
                            })
                            .collect(),
                        approvals: (0..rng.below(3))
                            .map(|_| RuleApproval {
                                role: u32::try_from(rng.below(3)).unwrap(),
                                people: u32::try_from(rng.below(4)).unwrap(),
                                freshness: JudgeFreshness::Clean,
                            })
                            .collect(),
                    },
                }
            })
            .collect();
        let path = derive(&input);
        assert_eq!(path, derive(&input), "seed {seed}: replay");
        bounds(&path);
        assert!(path.stops.iter().filter(|stop| stop.step == Step::Repair).count() <= 1, "seed {seed}: one repair");
        assert_eq!(path.stops.last().unwrap().step, Step::Branch, "seed {seed}: landing bar");
        let (goal, design, close_out) = match input.start {
            Start::Chat => (false, false, false),
            Start::Goal { design, close_out, .. } => (true, design, close_out),
        };
        assert_eq!(
            path.stops.iter().filter(|stop| stop.step == Step::Design).count(),
            usize::from(goal && design),
            "seed {seed}: design iff the goal designs first"
        );
        assert_eq!(
            path.stops.iter().filter(|stop| stop.step == Step::Plan).count(),
            usize::from(goal && !design),
            "seed {seed}: plan iff the goal does not design first"
        );
        assert_eq!(
            path.stops.iter().filter(|stop| stop.step == Step::CloseOut).count(),
            usize::from(goal && close_out),
            "seed {seed}: close-out once iff the goal makes one"
        );
        if close_out {
            let close = path.stops.iter().position(|stop| stop.step == Step::CloseOut).unwrap();
            let landing =
                path.stops.iter().rposition(|stop| stop.step == Step::Merge || stop.step == Step::Turn).unwrap();
            let bar = path.stops.iter().rposition(|stop| stop.step == Step::Branch).unwrap();
            assert!(landing < close && close < bar, "seed {seed}: close-out after landing and before its bar");
            assert_eq!(path.stops.get(close).unwrap().actor, Actor::Agent, "seed {seed}: close-out is an agent's");
        }
        let mut goal_segment = goal;
        for stop in &path.stops {
            if stop.step == Step::Branch {
                goal_segment = false;
            }
            assert_eq!(
                stop.needed,
                [Step::Repair, Step::Conflict, Step::Resolve, Step::CatchUp].contains(&stop.step),
                "seed {seed}: needed {:?}",
                stop.step
            );
            assert!(stop.over.len() <= temper_engine_domain_ways::OVER, "seed {seed}: over bound");
            assert!(
                stop.over.is_empty() || (stop.step == Step::Repair || (stop.step == Step::Merge && goal_segment)),
                "seed {seed}: over only on repair or goal merge"
            );
            if stop.step == Step::Merge && !stop.over.is_empty() {
                assert_eq!(stop.over.as_ref(), b"into goal", "seed {seed}: goal merge");
                assert!(goal, "seed {seed}: goal only");
            }
            match stop.step {
                Step::Conflict | Step::CatchUp | Step::Merge | Step::Prepare | Step::Prove | Step::Turn => {
                    assert_eq!(stop.actor, Actor::Code, "seed {seed}: {:?} belongs to code", stop.step);
                    if stop.step == Step::Conflict {
                        conflicts += 1;
                    }
                }
                Step::Ask
                | Step::Draft
                | Step::Plan
                | Step::Produce
                | Step::ProduceMany
                | Step::Check
                | Step::Review
                | Step::Repair
                | Step::Approve
                | Step::Resolve
                | Step::Verify
                | Step::Branch
                | Step::Design
                | Step::CloseOut => {}
            }
        }
    }
    assert!(conflicts > 0, "the sweep exercises conflicts");
}

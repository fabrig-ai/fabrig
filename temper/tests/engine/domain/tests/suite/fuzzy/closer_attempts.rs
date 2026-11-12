//! A closer's attempt bound over seeded endings (temper/docs/design/domain/forge/branches.md, 2).
use skein_lib::Rng;
use temper_engine_domain_forge_closer::{Child, Closer, Decision, Facts, Running, Step, step};

#[test]
fn seeded_step_endings_hold_only_for_the_branch_or_the_bound() {
    let mut covered = std::collections::BTreeSet::new();
    for seed in 0..8 {
        let mut rng = Rng::new(seed);
        let bound = u8::try_from(rng.below(4) + 1).expect("bound 1 through 4");
        let kind = if rng.below(2) == 0 { Step::Close } else { Step::Land };
        let mut closer = Closer::new();
        closer.closed = Some(Box::new([]));
        let mut f = Facts {
            goal_branch: true,
            close_job: true,
            pending: false,
            landed: Box::new([]),
            base_holds: Some(false),
            child: Child::Pending,
            landing_child: Child::Pending,
            released: false,
            close_ended: 0,
            land_ended: 0,
            bound,
        };
        let mut since_release = 1;
        for count in 1..=16 {
            let change = u64::from(count);
            closer.running = Some(Running { step: kind, change });
            closer.landing = (kind == Step::Land).then_some(change);
            // Cancelled and failed endings, with no hold, another hold, or a branch hold.
            let failed = rng.below(2) == 0;
            let reason = rng.below(3);
            let branch = reason == 2;
            covered.insert((bound, failed, reason));
            f.child = Child::Ended { branch };
            f.landing_child = f.child;
            f.close_ended = if kind == Step::Close { count } else { 99 };
            f.land_ended = if kind == Step::Land { count } else { 99 };
            f.pending = rng.below(2) == 0;
            if kind == Step::Land {
                f.base_holds = None;
                let checked = step(&closer, &f);
                assert_eq!(checked.decision, Decision::Check, "seed {seed}, ending failed {failed}");
                closer = checked.closer;
                f.base_holds = Some(false);
            }
            let mut s = step(&closer, &f);
            let must_hold = branch || count >= u32::from(bound);
            assert_eq!(
                matches!(s.decision, Decision::Hold(_)),
                must_hold,
                "seed {seed}, bound {bound}, ending {count}, failed {failed}, reason {reason}"
            );
            if must_hold {
                assert!(since_release <= u32::from(bound), "seed {seed}: too many changes between releases");
                assert!(matches!(step(&s.closer, &f).decision, Decision::Hold(_)));
                f.released = true;
                since_release = 0;
            } else if f.pending {
                assert_eq!(s.decision, Decision::Wait);
                assert_eq!(s.closer.retry, Some(kind));
            } else {
                assert_eq!(s.decision, Decision::Make(kind));
            }
            if must_hold || f.pending {
                f.pending = false;
                s = step(&s.closer, &f);
            }
            assert_eq!(s.decision, Decision::Make(kind), "seed {seed}");
            assert_eq!(s.closer.hold, None);
            closer = s.closer;
            f.released = false;
            since_release += 1;
            assert!(since_release <= u32::from(bound), "seed {seed}: made past its bound without a release");
        }
    }
    for bound in 1..=4 {
        assert!(covered.iter().any(|(seen, _, _)| *seen == bound), "seed set covers bound {bound}");
    }
    for failed in [false, true] {
        for reason in 0..3 {
            assert!(covered.iter().any(|(_, seen, hold)| *seen == failed && *hold == reason));
        }
    }
}

//! Verdict writes belong to the judged task, and refusal never holds it (after-landing.md, 9).
use super::*;

const STATUS: u64 = 301;
const COMMENT: u64 = 302;
const PULL: u64 = 41;
fn ready(logged: bool) -> Domain {
    let mut d = dispatched();
    let tier = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    tier.record.run.as_mut().unwrap().landings = Box::new([
        p::Landing { commit: NEXT_TIP, key: NEXT_TIP },
        p::Landing { commit: LAST_TIP, key: LAST_TIP }, // A push temper did not make.
    ]);
    if logged {
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::LandingLog(LandingLog {
                    repository: REPO,
                    landings: Box::new([LoggedLanding {
                        commit: NEXT_TIP,
                        branch: b"main".as_slice().into(),
                        at: env().wall,
                        changes: Box::new([
                            LoggedChange { task: CHANGE, pull: Some(PULL), title: Box::new([]), signed: Box::new([]) },
                            LoggedChange { task: CHANGE + 1, pull: None, title: Box::new([]), signed: Box::new([]) },
                        ]),
                    }]),
                }),
            },
        );
    } else {
        let mut change = fixtures::opening_row();
        change.pull = Some(PULL);
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change)) });
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Verification(Verification {
                    repository: REPO,
                    commit: NEXT_TIP,
                    changes: Box::new([CHANGE]),
                    tiers: Box::new([TierVerification { tier: TIER.into(), verdict: None, repaired: None }]),
                }),
            },
        );
    }
    completed(&mut d, api::Conclusion::Success, None);
    tier_step(&mut d, RELEASE, false);
    made(&mut d, RELEASE, skein_forge_client::Made::Set);
    tier_step(&mut d, RELEASE + 1, false); // Keeps the verdict before the first write.
    assert!(row(&d).record.judged.is_some());
    assert_eq!(row(&d).task, Some(TIER_TASK));
    d
}

#[test]
fn verdict_posts_use_logged_or_live_members_and_skip_refusals() {
    for logged in [false, true] {
        for refuse_status in [false, true] {
            let mut d = ready(logged);
            tier_step(&mut d, STATUS, false);
            let (_, what, effect) = d.describe_tier_effect(TIER_TASK, STATUS).expect("status effect");
            assert_eq!((what, effect.kind), (What::Repository, 6));
            assert_eq!(
                effect.effect.write,
                api::Write::Status {
                    commit: BASE,
                    context: b"temper/verdict/browser".as_slice().into(),
                    check: api::Check::Passed,
                    description: b"run 1 passed".as_slice().into(),
                    link: b"https://forge.example/run/71".as_slice().into()
                }
            );
            assert_eq!(
                row(&d).purpose,
                Some(purpose::EffectPurpose::Tier { repository: REPO, tier: 0, run: 1, operation: 6, part: 0 })
            );
            outcome(
                &mut d,
                STATUS,
                if refuse_status {
                    skein_forge_client::Outcome::Failed(api::Error::Forbidden)
                } else {
                    skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Set, found: false }
                },
            );
            tier_step(&mut d, COMMENT, false);
            let (_, what, effect) = d.describe_tier_effect(TIER_TASK, COMMENT).expect("comment effect");
            assert_eq!((what, effect.kind), (What::Pull(PULL), 7));
            let api::Write::Post { number, key, body } = effect.effect.write else { panic!("keyed comment") };
            assert_eq!(number, PULL);
            assert_eq!(body.as_ref(), b"Cleared by browser run 1.\n\nRun: https://forge.example/run/71");
            let purpose = purpose::EffectPurpose::Tier { repository: REPO, tier: 0, run: 1, operation: 7, part: 0 };
            assert_eq!(key, purpose.key(&d.namespace, LIMITS.client.op_bytes).unwrap());
            outcome(
                &mut d,
                COMMENT,
                if refuse_status {
                    skein_forge_client::Outcome::Made { made: skein_forge_client::Made::Commented(2), found: false }
                } else {
                    skein_forge_client::Outcome::Failed(api::Error::Missing)
                },
            );
            let out = tier_step(&mut d, COMMENT + 1, false);
            assert!(matches!(choice(&out), Some(TierDecision::Done { .. })));
            assert!(row(&d).task.is_none(), "one member, no comment for the outside push or the member with no PR");
            assert!(row(&d).held.is_none());
            assert!(d.entries.is_empty());
        }
    }
}

#[test]
fn goal_and_dropped_runs_post_nothing() {
    for goal in [false, true] {
        let mut d = dispatched();
        let out = if goal {
            let row = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
            row.goal = Some(7);
            row.goals = Box::new([TierGoal {
                goal: 7,
                branch: b"temper/1/g7".as_slice().into(),
                tip: Some(BASE),
                ran: None,
                failing: p::Failures::Tests(Box::new([])),
                started: None,
            }]);
            completed(&mut d, api::Conclusion::Success, None);
            tier_step(&mut d, RELEASE, false);
            made(&mut d, RELEASE, skein_forge_client::Made::Set);
            tier_step(&mut d, RELEASE + 1, false)
        } else {
            close_tier(&mut d, TIER_TASK, ReleaseEnding::Cancelled)
        };
        assert!(row(&d).posting.is_none());
        assert!(out.iter().all(|request| !matches!(request,
            Request::Save { record: Stored::Entry { entry, .. }, .. } if matches!(entry.effect.write, api::Write::Status { .. } | api::Write::Post { .. }))));
        assert!(row(&d).task.is_none());
    }
}

#[test]
fn fault_hold_parts_survive_releases_and_saturate_without_reposting() {
    for previous in [0, 1, u16::MAX - 1, u16::MAX] {
        let mut d = dispatched();
        let kept = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
        kept.record.run.as_mut().unwrap().faults = RETRIES;
        if previous > 0 {
            kept.posting = Some(Box::new(TierPosting {
                number: 1,
                tip: BASE,
                link: Box::new([]),
                check: api::Check::Pending,
                description: Box::new([]),
                landings: Box::new([]),
                comments: Box::new([]),
                next: 1,
                entry: None,
                part: previous,
                op_bytes: LIMITS.client.op_bytes,
            }));
        }
        completed(&mut d, api::Conclusion::Cancelled, None);
        let mut out = tier_step(&mut d, STATUS, false);
        if previous == u16::MAX {
            assert!(!d.entries.contains_key(&STATUS), "an exhausted part is skipped, never reused");
        } else {
            assert_eq!(
                row(&d).purpose,
                Some(purpose::EffectPurpose::Tier {
                    repository: REPO,
                    tier: 0,
                    run: 1,
                    operation: 6,
                    part: previous + 1
                })
            );
            made(&mut d, STATUS, skein_forge_client::Made::Set);
            out = tier_step(&mut d, STATUS + 1, false);
        }
        assert!(matches!(choice(&out), Some(TierDecision::Held { .. })));
        assert_eq!(row(&d).posting.as_ref().unwrap().part, previous.saturating_add(1));
    }
}

#[test]
fn fault_status_precedes_the_hold_and_the_verdict_replaces_it() {
    let mut d = dispatched();
    for fault in 0..=RETRIES {
        completed(&mut d, api::Conclusion::Cancelled, None);
        if fault < RETRIES {
            let entry = DISPATCH + u64::from(fault) + 1;
            tier_step(&mut d, entry, false);
            made(&mut d, entry, skein_forge_client::Made::Set);
        }
    }
    let out = tier_step(&mut d, STATUS, false);
    assert_eq!(choice(&out), Some(&TierDecision::Wait { until: None }));
    let (_, what, effect) = d.describe_tier_effect(TIER_TASK, STATUS).expect("pending before hold");
    assert_eq!((what, effect.kind), (What::Repository, 6));
    let api::Write::Status { context, check, description, link, .. } = effect.effect.write else {
        panic!("pending status")
    };
    assert_eq!(context.as_ref(), b"temper/verdict/browser");
    assert_eq!(check, api::Check::Pending);
    assert_eq!(description.as_ref(), b"held after 3 runner faults, last: cancelled");
    assert_eq!(link.as_ref(), b"https://forge.example/run/71");
    made(&mut d, STATUS, skein_forge_client::Made::Set);
    assert!(matches!(choice(&tier_step(&mut d, COMMENT, false)), Some(TierDecision::Held { .. })));
    tier_step(&mut d, COMMENT + 1, true);
    made(&mut d, COMMENT + 1, skein_forge_client::Made::Set);
    completed(&mut d, api::Conclusion::Success, None);
    tier_step(&mut d, RELEASE, false);
    made(&mut d, RELEASE, skein_forge_client::Made::Set);
    tier_step(&mut d, RELEASE + 1, false);
    tier_step(&mut d, STATUS + 3, false);
    let (_, _, effect) = d.describe_tier_effect(TIER_TASK, STATUS + 3).unwrap();
    let api::Write::Status { context: later, check, .. } = effect.effect.write else { panic!("verdict status") };
    assert_eq!(later, context);
    assert_eq!(check, api::Check::Passed);
    made(&mut d, STATUS + 3, skein_forge_client::Made::Set);
    assert!(matches!(choice(&tier_step(&mut d, STATUS + 4, false)), Some(TierDecision::Done { .. })));
}

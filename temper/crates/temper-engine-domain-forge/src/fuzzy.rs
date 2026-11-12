//! Seeded approval selection invariants (temper/docs/design/domain/forge/change/gates.md, 2).
use crate::{ChangeEvidence, Criterion, JudgeFreshness as Freshness, Reviewer, SIGN_OFFS, judge, tests};
use alloc::collections::BTreeSet;
use skein_forge::api::Verdict;
use skein_lib::{List, Rng, Wall};

#[test]
fn branch_head_records_respect_capacity_and_refuse_only_new_names() {
    use crate::{BranchHead, Name, Request, What, domain::heads};
    use skein_lib::Queue;
    for seed in 0..256 {
        let mut rng = Rng::new(seed);
        let mut env = tests::env();
        env.limits.client.resources = u32::try_from(rng.between(1, 8)).unwrap();
        let mut d = tests::domain_with_limits(&env.limits);
        for _ in 0..rng.between(32, 128) {
            let number = rng.below(12);
            let name = Name {
                forge: 1, repository: 2,
                what: What::Branch(Box::new([skein_lib::Decimal::of(number).as_bytes().into()])),
            };
            let head = BranchHead { name: name.clone(), commit: [u8::try_from(rng.below(256)).unwrap(); 32], owned: rng.chance(500) };
            let fits = d.heads.contains_key(&name) || d.heads.len() < d.heads.capacity();
            let needed = u64::from(d.heads.len()).saturating_add(1);
            let mut out = Queue::uncapped();
            assert_eq!(heads::record(&mut d, &env, head.clone(), Some(7), &mut out), fits, "seed {seed}");
            assert!(d.heads.len() <= env.limits.client.resources, "seed {seed}");
            if fits {
                assert_eq!(d.heads.get(&name), Some(&head), "seed {seed}");
                assert!(out.is_empty(), "seed {seed}");
            } else {
                assert_eq!(out.len(), 1, "seed {seed}");
                assert_eq!(out.pop(), Some(Request::HeadRefused {
                    task: Some(7), resource: name.clone(),
                    configured: u64::from(env.limits.client.resources), needed,
                }), "seed {seed}");
                assert!(!d.heads.contains_key(&name), "seed {seed}");
            }
        }
    }
}

#[test]
fn sign_offs_are_a_bounded_ordered_union_and_times_do_not_change_verdicts() {
    let repository = tests::adopted();
    let head = [1; 32];
    let clean = [[2; 32], [3; 32]];
    let evidence = ChangeEvidence {
        touched: crate::Touched::UNKNOWN,
        prior: tests::fixtures::opening_row().change,
        head: Some(head),
        base_tip: [4; 32],
        contains_base: temper_engine_domain_forge_change::Status::Passed,
        ci: temper_engine_domain_forge_change::Status::Passed,
        gates: Box::new([]),
        size: None,
        reviews: Box::new([]),
        reviews_complete: true,
    };
    for seed in 0..256 {
        let mut rng = Rng::new(seed);
        let mut approvals = List::with_capacity(8);
        for _ in 0..rng.below(8) {
            approvals
                .push((
                    u32::try_from(rng.between(1, 4)).unwrap(),
                    if rng.chance(500) { Freshness::Exact } else { Freshness::Clean },
                ))
                .unwrap();
        }
        let mut reviewers = List::with_capacity(80);
        let mut retimed = List::with_capacity(80);
        for _ in 0..rng.between(8, 80) {
            let review = Reviewer {
                person: rng.between(1, 24),
                role: u32::try_from(rng.between(1, 4)).unwrap(),
                head: [u8::try_from(rng.between(1, 5)).unwrap(); 32],
                verdict: match rng.below(5) {
                    0 => Verdict::Comment,
                    1 => Verdict::RequestChanges,
                    2..=4 => Verdict::Approve,
                    _ => unreachable!("bounded random verdict"),
                },
                at: Wall::from_nanos(rng.below(16)),
            };
            reviewers.push(review).unwrap();
            retimed.push(Reviewer { at: Wall::from_nanos(rng.next_u64()), ..review }).unwrap();
        }
        let reviewers = reviewers.into_boxed();
        let retimed = retimed.into_boxed();
        let approvals = approvals.into_boxed();
        let signed = judge::sign_offs(&approvals, head, &clean, &reviewers);
        let mut counted = BTreeSet::new();
        for (role, freshness) in &approvals {
            let (people, _) = judge::approval_count(*role, *freshness, head, &clean, &reviewers);
            for review in people {
                counted.insert(review.person);
            }
            for people in [0, 1, 2, 8] {
                let criterion = Criterion::Approval { role: *role, people, freshness: *freshness };
                assert_eq!(
                    judge::judge(criterion, &repository, head, &clean, &evidence, &reviewers),
                    judge::judge(criterion, &repository, head, &clean, &evidence, &retimed),
                    "seed {seed}: timestamps cannot change approval verdicts"
                );
            }
        }
        assert_eq!(signed.len(), counted.len().min(SIGN_OFFS), "seed {seed}");
        let mut seen = BTreeSet::new();
        let mut previous = None;
        for sign in signed {
            assert!(counted.contains(&sign.person), "seed {seed}: counted subset");
            assert!(seen.insert(sign.person), "seed {seed}: distinct people");
            let key = (sign.at, sign.person);
            if let Some(previous) = previous {
                assert!(previous < key, "seed {seed}: ordered");
            }
            previous = Some(key);
            let mut found = false;
            for review in &reviewers {
                found |= review.person == sign.person
                    && review.head == sign.head
                    && review.at == sign.at
                    && review.verdict == Verdict::Approve;
            }
            assert!(found, "seed {seed}: retained review");
        }
    }
}

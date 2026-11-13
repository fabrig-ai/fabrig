//! Seeded approval selection invariants (temper/docs/design/domain/forge/change/gates.md, 2).
use crate::{ChangeEvidence, Criterion, JudgeFreshness as Freshness, Reviewer, SIGN_OFFS, judge, tests};
use alloc::collections::BTreeSet;
use skein_forge::api::Verdict;
use skein_lib::{List, Rng, Wall};

#[test]
fn branch_head_records_respect_capacity_and_refuse_only_new_names() {
    use crate::{BranchHead, Hold, Name, Request, What, domain::heads};
    use skein_lib::Queue;
    for seed in 0..256 {
        let mut rng = Rng::new(seed);
        let mut env = tests::env();
        env.limits.client.resources = u32::try_from(rng.between(1, 8)).unwrap();
        env.limits.holds = env.limits.client.resources;
        let mut d = tests::domain_with_limits(&env.limits);
        d.repositories.insert(tests::adopted().provider, Box::new(tests::adopted())).unwrap();
        for _ in 0..rng.between(32, 128) {
            let number = rng.below(12);
            let name = Name {
                forge: 1,
                repository: 2,
                what: What::Branch(Box::new([skein_lib::Decimal::of(number).as_bytes().into()])),
            };
            let head = BranchHead {
                name: name.clone(),
                commit: [u8::try_from(rng.below(256)).unwrap(); 32],
                owned: rng.chance(500),
            };
            let fits = d.heads.contains_key(&name) || d.heads.len() < d.heads.capacity();
            let needed = u64::from(d.heads.len()).saturating_add(1);
            let mut out = Queue::uncapped();
            assert_eq!(heads::record(&mut d, &env, head.clone(), Some(7), &mut out), fits, "seed {seed}");
            assert!(d.heads.len() <= env.limits.client.resources, "seed {seed}");
            if fits {
                d.holds
                    .insert(
                        name.clone(),
                        Hold { name: name.clone(), task: 7, writer: None, drift: None, design_writers: Box::new([]) },
                    )
                    .unwrap();
                assert_eq!(d.heads.get(&name), Some(&head), "seed {seed}");
                assert!(out.is_empty(), "seed {seed}");
            } else {
                assert_eq!(out.len(), 1, "seed {seed}");
                assert_eq!(
                    out.pop(),
                    Some(Request::HeadRefused {
                        task: Some(7),
                        resource: name.clone(),
                        configured: u64::from(env.limits.client.resources),
                        needed,
                    }),
                    "seed {seed}"
                );
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

#[test]
fn branch_heads_free_exactly_the_unneeded_subset() {
    use crate::{BranchHead, Hold, Key, Request, Subscriber, Topic, domain::heads};
    use skein_lib::Queue;
    for seed in 0..256 {
        let mut rng = Rng::new(seed);
        let env = tests::env();
        let mut d = tests::domain();
        d.repositories.insert(tests::adopted().provider, Box::new(tests::adopted())).unwrap();
        let mut needed = BTreeSet::new();
        let mut freed = BTreeSet::new();
        for number in 0..rng.between(1, u64::from(env.limits.client.resources)) {
            let branch: Box<[u8]> = format!("branch-{number}").as_bytes().into();
            let name = crate::tiers::name(tests::adopted().provider, &branch);
            let tip = [u8::try_from(rng.below(256)).unwrap(); 32];
            d.heads
                .insert(name.clone(), BranchHead { name: name.clone(), commit: tip, owned: rng.chance(500) })
                .unwrap();
            let reason = rng.below(6);
            if reason == 0 {
                freed.insert(name.clone());
                // Ended changes must not keep either branch alive.
                let mut row = tests::fixtures::opening_row();
                row.task = number;
                row.change.task = number;
                row.branch = branch.clone();
                row.base = branch;
                row.change.state = temper_engine_domain_forge_change::State::Ended;
                d.changes.insert(number, row).unwrap();
            } else {
                needed.insert(name.clone());
                match reason {
                    1 => {
                        d.holds
                            .insert(
                                name.clone(),
                                Hold { name, task: number, writer: None, drift: None, design_writers: Box::new([]) },
                            )
                            .unwrap();
                    }
                    2 => {
                        d.names.insert(number, Box::new([name])).unwrap();
                    }
                    3 | 4 => {
                        let mut row = tests::fixtures::opening_row();
                        row.task = number;
                        row.change.task = number;
                        if reason == 3 {
                            row.branch = branch;
                        } else {
                            row.base = branch;
                        }
                        d.changes.insert(number, row).unwrap();
                    }
                    5 => {
                        let row = Subscriber {
                            task: number,
                            number,
                            topic: Topic::Landings { repository: tests::adopted().provider, branch },
                            own_change: None,
                            goal_tasks: None,
                            paths: Box::new([]),
                        };
                        d.subscriptions.insert((number, row.topic.clone()), row).unwrap();
                    }
                    _ => unreachable!("six reasons"),
                }
            }
        }
        let mut out = Queue::uncapped();
        heads::free(&mut d, &env, &mut out);
        let mut retained = BTreeSet::new();
        for (name, _) in &d.heads {
            retained.insert(name.clone());
        }
        assert_eq!(retained, needed, "seed {seed}");
        assert_eq!(usize::try_from(out.len()).unwrap(), freed.len(), "one erase per freed head, seed {seed}");
        for request in &out {
            let Request::Erase { key: Key::BranchHead(name) } = request else {
                panic!("only head erasures, seed {seed}")
            };
            assert!(freed.remove(name), "erase once, seed {seed}");
        }
        assert!(freed.is_empty(), "all erasures, seed {seed}");
    }
}

/// A small independent live-record oracle (domain/forge/reds.md, 8).
#[test]
#[expect(clippy::too_many_lines, reason = "one seeded oracle compares writes, ends, releases and every live goal read")]
fn goal_reds_remain_bounded_unique_ordered_and_live_under_random_writes_and_releases() {
    use crate::{Event, Limits, Red, RedRecorded, RedRefusal, ReleaseEnding, Request, Stored};
    use skein_lib::{Map, Queue, Set};
    for seed in 0..128 {
        let mut rng = Rng::new(seed);
        let mut env = tests::env();
        env.limits = Limits { branch_reds: 2, ..env.limits };
        let mut d = tests::domain_with_limits(&env.limits);
        tests::outputs(&mut d, Event::Restore { record: Stored::Repository(tests::adopted()) });
        let mut live = Set::with_capacity(3);
        let mut model: Map<(u64, Box<[u8]>), Red> = Map::with_capacity(6);
        let mut last = 0_u64;
        for goal in 7..10 {
            tests::reds::take(&mut d, &env.limits, goal);
            live.insert(goal).unwrap();
        }
        for _ in 0_u32..128 {
            let goal = rng.between(7, 9);
            let key = skein_lib::Decimal::of(rng.below(4)).as_bytes().into();
            let address = (goal, key);
            let mut out = Queue::uncapped();
            match rng.below(4) {
                0 | 1 => {
                    let mut held = 0_u32;
                    for ((owner, _), _) in &model {
                        if *owner == goal {
                            held += 1;
                        }
                    }
                    let replaced = model.get(&address).cloned();
                    let mut call = tests::reds::call(goal, &address.1);
                    call.at = rng.next_u64();
                    call.head = Box::new([u8::try_from(rng.below(256)).unwrap(); 32]);
                    let result = d.record_red(&env.limits, call.clone(), &mut out);
                    if !live.contains(&goal) {
                        assert_eq!(result, Err(RedRefusal::NoGoal), "seed {seed}");
                        assert!(out.is_empty());
                    } else if replaced.is_none() && held == 2 {
                        assert_eq!(result, Err(RedRefusal::Full { bound: 2 }), "seed {seed}");
                        assert!(out.is_empty());
                    } else {
                        last += 1;
                        assert_eq!(result, Ok(RedRecorded { generation: last, replaced }), "seed {seed}");
                        let row = Red {
                            goal,
                            key: call.key,
                            head: call.head,
                            why: call.why,
                            by: call.by,
                            at: call.at,
                            generation: last,
                        };
                        model.insert(address, row.clone()).unwrap();
                        assert_eq!(
                            out.pop(),
                            Some(Request::Save { record: Stored::Red(row), release: None, read_afresh: None })
                        );
                        assert_eq!(
                            out.pop(),
                            Some(Request::Save {
                                record: Stored::RedNumber { last },
                                release: None,
                                read_afresh: None
                            })
                        );
                        assert!(out.is_empty());
                    }
                }
                2 => {
                    let expected = if live.contains(&goal) {
                        model.remove(&address).ok_or(RedRefusal::NoSuch)
                    } else {
                        Err(RedRefusal::NoGoal)
                    };
                    assert_eq!(d.end_red(goal, &address.1, &mut out), expected, "seed {seed}");
                }
                3 => {
                    if live.remove(&goal) {
                        tests::outputs(
                            &mut d,
                            Event::Release {
                                task: goal,
                                root: goal,
                                ending: ReleaseEnding::Cancelled,
                                reason: None,
                                entry: 22,
                            },
                        );
                        for key in 0..4 {
                            model.remove(&(goal, skein_lib::Decimal::of(key).as_bytes().into()));
                        }
                    } else {
                        tests::reds::take(&mut d, &env.limits, goal);
                        live.insert(goal).unwrap();
                    }
                }
                _ => unreachable!("four acts"),
            }
            for goal in 7..10 {
                let mut previous: Option<&[u8]> = None;
                let mut count = 0_u32;
                for (_, row) in d.reds(goal) {
                    assert!(live.contains(&goal), "seed {seed}: released goal");
                    assert_eq!(row.goal, goal, "seed {seed}: goal-local read");
                    if let Some(previous) = previous {
                        assert!(previous < row.key.as_ref(), "seed {seed}: unique ordered keys");
                    }
                    previous = Some(&row.key);
                    assert_eq!(Some(row), model.get(&(goal, row.key.clone())), "seed {seed}");
                    assert!(row.generation <= last);
                    count += 1;
                }
                let mut expected = 0_u32;
                for ((owner, _), _) in &model {
                    if *owner == goal {
                        expected += 1;
                    }
                }
                assert_eq!(count, expected, "seed {seed}: complete read");
                assert!(count <= 2, "seed {seed}: per-goal bound");
            }
        }
    }
}

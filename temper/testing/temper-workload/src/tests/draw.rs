use std::collections::BTreeSet;

use skein_world::streams::{Stream, Streams};

use crate::draw::{COMPOSED, ENGINE, Party, Shape, Span, Workload};

#[test]
fn workload_replays_its_own_stream_and_draw_order() {
    let streams = Streams::new(7);
    let workload = Workload::draw(&streams, &ENGINE);
    assert_eq!(workload, Workload::draw(&Streams::new(7), &ENGINE));
    let _ = streams.rng(Stream::Parties, "first").next_u64();
    assert_eq!(workload, Workload::draw(&streams, &ENGINE));
    assert_ne!(workload, Workload::draw(&Streams::new(8), &ENGINE));

    // The prefix pins the stream and draw order, independently of the draw's helpers.
    let mut rng = streams.rng(Stream::Workload, "goals");
    let count = rng.between(2, 4);
    assert_eq!(workload.goals.len() as u64, count);
    let mut parties: Vec<_> =
        (0..count).map(|_| if rng.below(2) == 0 { Party::First } else { Party::Second }).collect();
    if parties.iter().all(|party| *party == parties[0]) {
        *parties.last_mut().unwrap() = if parties[0] == Party::First { Party::Second } else { Party::First };
    }
    assert_eq!(workload.goals.iter().map(|goal| goal.party).collect::<Vec<_>>(), parties);
    let milestone = rng.between(0, 2);
    assert_eq!(workload.goals.iter().filter(|goal| goal.after.is_some()).count() as u64, milestone.saturating_sub(1));
    assert_eq!(workload.goals.last().unwrap().changes.len() as u64, rng.between(2, 3));
}

#[test]
fn shapes_names_files_and_dependencies_hold_over_seeds() {
    for shape in [ENGINE, COMPOSED] {
        for seed in 0..64 {
            let workload = Workload::draw(&Streams::new(seed), &shape);
            assert!((usize::from(shape.goals.least)..=usize::from(shape.goals.most)).contains(&workload.goals.len()));
            assert_eq!(workload.tier, shape.tier);
            assert_eq!(workload.goals.iter().map(|goal| goal.party).collect::<BTreeSet<_>>().len(), 2);
            assert_eq!(workload.goals.iter().filter(|goal| goal.train).count(), 1);
            let mut names = BTreeSet::new();
            let mut paths = BTreeSet::new();
            let mut milestone = 0;
            for (i, goal) in workload.goals.iter().enumerate() {
                assert_eq!(goal.name, format!("g{i}"));
                assert!(names.insert(goal.name.clone()));
                assert_eq!(workload.goal(&goal.name), Some(goal));
                let changes = if goal.train { shape.train } else { shape.changes };
                assert!((usize::from(changes.least)..=usize::from(changes.most)).contains(&goal.changes.len()));
                if goal.train {
                    assert_eq!(i + 1, workload.goals.len());
                }
                if let Some(after) = &goal.after {
                    milestone += 1;
                    assert_eq!(after, &format!("g{}", i - 1));
                    assert_eq!(i, milestone);
                }
                for (j, change) in goal.changes.iter().enumerate() {
                    assert_eq!(change.name, format!("g{i}.c{j}"));
                    assert!(names.insert(change.name.clone()));
                    assert_eq!(workload.change(&change.name), Some((goal, change)));
                    assert!(change.after.len() <= 1);
                    for after in &change.after {
                        let (dependency_goal, dependency) = workload.change(after).expect("named dependency exists");
                        if j > 0 {
                            assert_eq!(dependency_goal, goal);
                            assert_eq!(dependency, &goal.changes[j - 1]);
                        } else {
                            assert_eq!(goal.after.as_deref(), Some(dependency_goal.name.as_str()));
                            assert_eq!(dependency_goal.changes.last(), Some(dependency));
                        }
                    }
                    assert!((1..=2).contains(&change.files.len()));
                    for (k, file) in change.files.iter().enumerate() {
                        assert_eq!(file.path, format!("g{i}/c{j}/f{k}.txt"));
                        assert!(paths.insert(file.path.clone()));
                        let prefix = format!("{} f{k} ", change.name);
                        let hex = file.text.strip_prefix(&prefix).unwrap().strip_suffix('\n').unwrap();
                        assert!(!hex.contains('\n'));
                        u64::from_str_radix(hex, 16).expect("a drawn u64 in hex");
                    }
                }
            }
            let milestone_goals = if milestone == 0 { 0 } else { milestone + 1 };
            assert!(
                (usize::from(shape.milestone.least)..=usize::from(shape.milestone.most)).contains(&milestone_goals)
            );
            if shape == COMPOSED {
                assert_eq!(workload.goals.len(), 3);
                assert_eq!(milestone_goals, 2);
                assert!(workload.goals[2].train);
                assert!(workload.goals[2].changes.len() >= 2);
                assert!(workload.tier);
            }
            assert!(workload.goal("missing").is_none());
            assert!(workload.change("missing").is_none());
        }
    }
}

#[test]
fn shapes_without_trains_and_oversized_milestones_use_existing_goals() {
    let mut shape = Shape {
        goals: Span { least: 1, most: 1 },
        changes: Span { least: 1, most: 1 },
        milestone: Span { least: 3, most: 3 },
        train: Span { least: 0, most: 1 },
        tier: false,
    };
    for seed in 0..64 {
        let workload = Workload::draw(&Streams::new(seed), &shape);
        assert_eq!(workload.goals.len(), 1);
        assert!(!workload.tier);
        assert!(!workload.goals[0].train);
        assert!(workload.goals[0].after.is_none());
        assert_eq!(workload.goals[0].changes.len(), 1);
    }
    shape.goals = Span { least: 0, most: 0 };
    assert_eq!(Workload::draw(&Streams::new(7), &shape).goals, []);
}

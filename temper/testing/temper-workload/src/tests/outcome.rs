use std::collections::BTreeMap;

use crate::outcome::{End, GoalOutcome, Left, Outcome};

fn example() -> Outcome {
    Outcome {
        goals: ["g1", "g2", "g10"]
            .into_iter()
            .map(|name| {
                (
                    name.to_owned(),
                    GoalOutcome { end: End::Landed, landed: vec![format!("{name}.c0")], holds: vec![], tries: 1 },
                )
            })
            .collect(),
        main: BTreeMap::from([
            ("g1/c0/f0.txt".to_owned(), "first\n".to_owned()),
            ("g2/c0/f0.txt".to_owned(), "second\n".to_owned()),
            ("g10/c0/f0.txt".to_owned(), "tenth\n".to_owned()),
        ]),
    }
}

#[test]
fn comparison_finds_each_field_missing_goal_and_missing_path() {
    let before = example();
    assert_eq!(before.compare(&before, &[]), Ok(()));
    let mut after = before.clone();
    after.goals.get_mut("g1").unwrap().end = End::Held("deadline".to_owned());
    assert_eq!(
        before.compare(&after, &[]).unwrap_err(),
        "compare: g1: end Landed without restarts, Held(\"deadline\") with them"
    );
    after = before.clone();
    after.goals.get_mut("g1").unwrap().landed.clear();
    assert_eq!(
        before.compare(&after, &[]).unwrap_err(),
        "compare: g1: landed [\"g1.c0\"] without restarts, [] with them"
    );
    after = before.clone();
    after.goals.get_mut("g1").unwrap().holds.push("deadline".to_owned());
    assert_eq!(
        before.compare(&after, &[]).unwrap_err(),
        "compare: g1: holds [] without restarts, [\"deadline\"] with them"
    );
    after = before.clone();
    after.goals.get_mut("g1").unwrap().tries = 2;
    assert_eq!(before.compare(&after, &[]).unwrap_err(), "compare: g1: tries 1 without restarts, 2 with them");
    after = before.clone();
    after.main.insert("g1/c0/f0.txt".to_owned(), "changed\n".to_owned());
    assert_eq!(
        before.compare(&after, &[]).unwrap_err(),
        "compare: g1: main g1/c0/f0.txt: Some(\"first\\n\") without restarts, Some(\"changed\\n\") with them"
    );
    after.main.remove("g1/c0/f0.txt");
    assert!(before.compare(&after, &[]).unwrap_err().contains("Some(\"first\\n\") without restarts, None with them"));
    assert!(after.compare(&before, &[]).unwrap_err().contains("None without restarts, Some(\"first\\n\") with them"));
    after = before.clone();
    after.goals.remove("g1");
    assert!(before.compare(&after, &[]).unwrap_err().starts_with("compare: g1: goal Some("));
    assert!(after.compare(&before, &[]).unwrap_err().starts_with("compare: g1: goal None without restarts, Some("));
}

#[test]
fn comparison_reports_first_difference_per_goal_in_name_order() {
    let before = example();
    let mut after = before.clone();
    for goal in after.goals.values_mut() {
        goal.end = End::Failed;
        goal.landed.clear();
        goal.tries = 2;
    }
    after.main.clear();
    assert_eq!(
        before.compare(&after, &[]).unwrap_err(),
        "compare: g1: end Landed without restarts, Failed with them; compare: g10: end Landed without restarts, Failed with them; compare: g2: end Landed without restarts, Failed with them"
    );
}

#[test]
fn comparison_skips_only_named_goals_and_their_directory() {
    let before = example();
    let mut after = before.clone();
    after.goals.remove("g1");
    after.main.remove("g1/c0/f0.txt");
    after.main.insert("g1/extra".to_owned(), "new".to_owned());
    let left = [Left { goal: "g1".to_owned(), why: "a changed outcome is expected" }];
    assert_eq!(before.compare(&after, &left), Ok(()));
    assert_eq!(after.compare(&before, &left), Ok(()));
    after.main.remove("g10/c0/f0.txt");
    let finding = before.compare(&after, &left).unwrap_err();
    assert!(finding.starts_with("compare: g10: main g10/c0/f0.txt:"), "{finding}");
    let unknown = [Left { goal: "absent".to_owned(), why: "not a goal" }];
    assert_eq!(before.compare(&before, &unknown).unwrap_err(), "compare: absent: left names no goal in either outcome");
}

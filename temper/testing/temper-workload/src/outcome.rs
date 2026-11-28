//! Outcomes in logical names (`temper/docs/design/testing/workloads-kit.md`, 1.4).

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum End {
    Landed,
    Done,
    Failed,
    Cancelled,
    Held(String),
    Waiting,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GoalOutcome {
    pub end: End,
    /// Changes whose files main holds, by name, sorted.
    pub landed: Vec<String>,
    /// Hold kinds in jig's words, sorted.
    pub holds: Vec<String>,
    /// Normalized task tries.
    pub tries: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Outcome {
    pub goals: BTreeMap<String, GoalOutcome>,
    /// Main's files under the workload's paths, path to text.
    pub main: BTreeMap<String, String>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Left {
    pub goal: String,
    pub why: &'static str,
}

impl Outcome {
    /// Compares each goal and its paths, apart from goals explicitly left out.
    pub fn compare(&self, restarted: &Outcome, left: &[Left]) -> Result<(), String> {
        let names: BTreeSet<_> =
            self.goals.keys().chain(restarted.goals.keys()).chain(left.iter().map(|item| &item.goal)).collect();
        let mut findings = Vec::new();
        for name in names {
            let before = self.goals.get(name);
            let after = restarted.goals.get(name);
            if before.is_none() && after.is_none() {
                findings.push(format!("compare: {name}: left names no goal in either outcome"));
                continue;
            }
            if left.iter().any(|item| &item.goal == name) {
                continue;
            }
            let difference = match (before, after) {
                (Some(before), Some(after)) => {
                    goal_difference(before, after).or_else(|| self.path_difference(restarted, name))
                }
                _ => Some(format!("goal {before:?} without restarts, {after:?} with them")),
            };
            if let Some(difference) = difference {
                findings.push(format!("compare: {name}: {difference}"));
            }
        }
        if findings.is_empty() { Ok(()) } else { Err(findings.join("; ")) }
    }

    fn path_difference(&self, restarted: &Outcome, goal: &str) -> Option<String> {
        let prefix = format!("{goal}/");
        let paths: BTreeSet<_> =
            self.main.keys().chain(restarted.main.keys()).filter(|path| path.starts_with(&prefix)).collect();
        for path in paths {
            let before = self.main.get(path);
            let after = restarted.main.get(path);
            if before != after {
                return Some(format!("main {path}: {before:?} without restarts, {after:?} with them"));
            }
        }
        None
    }
}

fn goal_difference(before: &GoalOutcome, after: &GoalOutcome) -> Option<String> {
    if before.end != after.end {
        Some(format!("end {:?} without restarts, {:?} with them", before.end, after.end))
    } else if before.landed != after.landed {
        Some(format!("landed {:?} without restarts, {:?} with them", before.landed, after.landed))
    } else if before.holds != after.holds {
        Some(format!("holds {:?} without restarts, {:?} with them", before.holds, after.holds))
    } else if before.tries != after.tries {
        Some(format!("tries {} without restarts, {} with them", before.tries, after.tries))
    } else {
        None
    }
}

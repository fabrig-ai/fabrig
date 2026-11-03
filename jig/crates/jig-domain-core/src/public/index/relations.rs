//! Lists and counts are derived only after public membership (jig/docs/design/domain/people.md, 13.7).
use super::{Location, Project, Relations};
use crate::{PublicCounts, PublicIndexLimits, PublicLine, PublicPhase};
use skein_lib::{List, Map};

pub(super) const ZERO: PublicCounts = PublicCounts { working: 0, waiting: 0, held: 0, closing: 0, ended: 0 };

pub(super) fn capacity(limits: PublicIndexLimits) -> u32 {
    u32::from(limits.goals)
        .saturating_add(u32::from(limits.ended))
        .saturating_mul(u32::from(limits.lines).saturating_add(1))
}

impl Project {
    pub(super) fn relink(&mut self, limits: PublicIndexLimits) {
        let previous = core::mem::replace(&mut self.relations, Map::with_capacity(capacity(limits)));
        for &goal in &self.live {
            let tree = self.trees.get_mut(&goal).expect("selected live goal");
            let facts = facts(&tree.line, &tree.lines);
            add_goal(&mut self.relations, goal, &facts, &tree.omitted, limits, &tree.lines, None);
            tree.line.delegates = counts(&facts, goal);
            for line in &mut tree.lines {
                line.delegates = counts(&facts, line.task);
            }
        }
        for index in 0..self.ended.len() {
            let frozen = self.ended.get(index).expect("frozen goal index");
            let facts = facts(&frozen.line, &frozen.tree);
            let mut omitted = Map::with_capacity(u32::from(limits.lines).saturating_add(1));
            for (&task, _) in &facts {
                let folded = match previous.get(&task) {
                    Some(row) => row.folded,
                    None => 0,
                };
                omitted.insert(task, folded).expect("one frozen folded count per line");
            }
            add_goal(&mut self.relations, frozen.line.task, &facts, &omitted, limits, &frozen.tree, Some(index));
            // Frozen lines, including their counts, are never changed by relinking.
        }
    }
}

fn facts(line: &PublicLine, tree: &[PublicLine]) -> Map<u64, (Option<u64>, PublicPhase)> {
    let capacity = u32::try_from(tree.len()).expect("bounded tree").checked_add(1).expect("goal and tree");
    let mut facts = Map::with_capacity(capacity);
    facts.insert(line.task, (None, line.phase)).expect("goal slot");
    for line in tree {
        facts.insert(line.task, (line.parent, line.phase)).expect("one fact per public line");
    }
    facts
}

fn add_goal(
    all: &mut Map<u64, Relations>,
    goal: u64,
    facts: &Map<u64, (Option<u64>, PublicPhase)>,
    omitted: &Map<u64, u32>,
    limits: PublicIndexLimits,
    lines: &[PublicLine],
    ended: Option<u32>,
) {
    for (&task, &(parent, _)) in facts {
        let mut lineage = List::with_capacity(u32::from(limits.depth));
        let mut next = parent;
        for _ in 0..limits.depth {
            let Some(ancestor) = next else { break };
            lineage.push(ancestor).expect("public depth bound");
            next = facts.get(&ancestor).expect("public prefix includes every ancestor").0;
        }
        assert!(next.is_none(), "public lineage ends at its goal");
        let mut delegates = List::with_capacity(u32::from(limits.lines));
        for (&number, &(parent, _)) in facts {
            if parent == Some(task) {
                delegates.push(number).expect("public delegates within tree lines");
            }
        }
        let folded = omitted.get(&task).copied().unwrap_or(0);
        let mut position = 0;
        if task != goal {
            for (index, line) in lines.iter().enumerate() {
                if line.task == task {
                    position = u32::try_from(index)
                        .expect("bounded public line")
                        .checked_add(1)
                        .expect("public line position");
                    break;
                }
            }
            assert!(position != 0, "public membership has a cached line");
        }
        let location = match ended {
            Some(goal) => Location::Ended { goal, line: position },
            None => Location::Live { line: position },
        };
        all.insert(
            task,
            Relations { goal, location, lineage: lineage.into_boxed(), delegates: delegates.into_boxed(), folded },
        )
        .expect("bounded public membership");
    }
}

fn counts(facts: &Map<u64, (Option<u64>, PublicPhase)>, task: u64) -> PublicCounts {
    let mut counts = ZERO;
    for (_, &(parent, phase)) in facts {
        if parent != Some(task) {
            continue;
        }
        let counter = match phase {
            PublicPhase::Waiting => &mut counts.waiting,
            PublicPhase::Held => &mut counts.held,
            PublicPhase::Closing => &mut counts.closing,
            PublicPhase::Ended | PublicPhase::Done | PublicPhase::Failed | PublicPhase::Cancelled => &mut counts.ended,
            PublicPhase::Idle
            | PublicPhase::Due
            | PublicPhase::Preparing
            | PublicPhase::Claimed
            | PublicPhase::Running
            | PublicPhase::BackingOff
            | PublicPhase::Active => &mut counts.working,
        };
        *counter = counter.checked_add(1).expect("bounded public delegates");
    }
    counts
}

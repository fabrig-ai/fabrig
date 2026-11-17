//! A milestone's task facts (temper/docs/design/domain/milestones.md, 3 and 7).
use alloc::boxed::Box;
use jig_client::{contract, shapes};

/// The stock milestone template (client/pages/milestone.md, 1).
pub const TEMPLATE: u32 = 8;

#[must_use]
pub fn is(chip: &contract::Chip) -> bool {
    chip.details.details.executor
        == (contract::Executor::Procedure { connector: shapes::RECURRING, code: shapes::GATHERING })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lineage<'a> {
    pub milestone: &'a contract::Chip,
    pub goal: &'a contract::Chip,
}

/// Jig serves ancestors root first (`jig-domain-core`, `documents/task_tree.rs`).
#[must_use]
pub fn lineage(task: &contract::Task) -> Option<Lineage<'_>> {
    let mut ancestors = task.lineage_chips.iter();
    for chip in ancestors.by_ref() {
        if is(chip) {
            let goal = match ancestors.next() {
                Some(goal) => goal,
                None if task.requester == contract::Requester::Task(chip.task) => &task.chip,
                None => return None,
            };
            return Some(Lineage { milestone: chip, goal });
        }
    }
    None
}
use skein_lib::{Wall, bytes};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub name: Box<[u8]>,
    pub words: Option<Box<[u8]>>,
    pub maker: contract::Requester,
    pub whole: Option<Wall>,
    pub made: Wall,
}

#[must_use]
pub fn facts(task: &contract::Task) -> Facts {
    let spec = task.spec.specification.words.bytes();
    let end = bytes::find(spec, b"\n").unwrap_or(spec.len());
    let name = spec.get(..end).expect("first line within spec");
    let words = match bytes::find(spec, b"\n\n") {
        Some(blank) => {
            let start = blank.checked_add(2).expect("blank line within spec");
            let words = spec.get(start..).expect("words after blank line");
            if words.is_empty() { None } else { Some(bytes::copy_of(words)) }
        }
        None => None,
    };
    Facts {
        name: bytes::copy_of(name),
        words,
        maker: task.requester,
        whole: match task.concluded {
            Some(concluded) => Some(concluded.at),
            None => None,
        },
        made: task.made,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Of {
    Milestone(u64),
    Read(u64),
    None,
}

#[must_use]
pub fn of(line: &contract::TaskLine, milestones: &[u64]) -> Of {
    match line.requester {
        contract::Requester::Task(task) => {
            if milestones.contains(&task) {
                Of::Milestone(task)
            } else {
                Of::Read(task)
            }
        }
        contract::Requester::Person(_) | contract::Requester::Deployment { .. } => Of::None,
    }
}

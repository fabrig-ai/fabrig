//! A milestone's task facts (temper/docs/design/domain/milestones.md, 3 and 7).
use alloc::boxed::Box;
use jig_client::contract;
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

//! Goal-filter membership carried with the inbox's original row cursor
//! (`client/domain/pages.md`, section 3). Live membership follows the bounded
//! requester walk; retained membership uses the supplied original ancestry.

pub use crate::below::documents::inbox_selection::*;

use crate::{
    Core, Limits,
    documents::{Cursor, Missing},
};
use jig_domain_tasks as tasks;
use skein_lib::List;

pub(super) fn live(core: &Core, limits: &Limits, task: u64, cursor: Cursor) -> Result<InboxSelection, Missing> {
    let capacity = limits.tasks.depth.checked_add(1).ok_or(Missing::TooLarge)?;
    let mut goals = List::with_capacity(capacity);
    if task == tasks::PROPOSAL_WITHOUT_TASK {
        return Ok(InboxSelection { cursor, goals });
    }
    let mut number = task;
    for _ in 0..=limits.tasks.depth {
        goals.push(number).expect("one root for each bounded requester step");
        let Some(row) = core.tasks.task(number) else {
            break;
        };
        match row.requester {
            tasks::Party::Task(parent) => number = parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        }
    }
    Ok(InboxSelection { cursor, goals })
}

pub(crate) fn retained(
    limits: &Limits,
    task: u64,
    cursor: Cursor,
    ancestors: &[u64],
) -> Result<InboxSelection, Missing> {
    let capacity = limits.tasks.depth.checked_add(1).ok_or(Missing::TooLarge)?;
    if ancestors.len() > skein_lib::length::usize_of(limits.tasks.depth) {
        return Err(Missing::TooLarge);
    }
    let mut goals = List::with_capacity(capacity);
    if task != tasks::PROPOSAL_WITHOUT_TASK {
        goals.push(task).expect("self always has a place");
    }
    for root in ancestors {
        goals.push(*root).expect("whole admitted original ancestry fits");
    }
    Ok(InboxSelection { cursor, goals })
}

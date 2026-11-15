//! Ended erasure pages find logged tasks by their summaries (protocol/store/logs.md, 4 and 9).
use super::{Failure, Spent, erasure, point};
use crate::{Store, intervals, keys, pages};
use jig_domain_core_vocabulary::{self as core, reads};
use jig_domain_tasks as tasks;
use skein_kv::Range;
use skein_lib::{List, Map};

pub(super) fn read(store: &mut Store, range: &reads::Range, spent: &mut Spent) -> Result<reads::Loaded, Failure> {
    if spent.rows == 0 {
        return Ok(super::empty());
    }
    let extents = pages::load(store, Range::prefix(&crate::logs::EXTENTS)).or(Err(Failure::Corrupt))?;
    if extents.rows.is_empty() {
        return erasure::read(store, range, None, spent);
    }
    let after = match &range.after {
        Some(reads::Position::Owner(core::Key::Tasks(tasks::Key::Ended(task)))) => Some(*task),
        Some(reads::Position::Owner(_) | reads::Position::Inbox { .. }) => return Err(Failure::Corrupt),
        None => None,
    };
    let (candidates, mut more) = candidates(store, after, spent.rows.saturating_add(1))?;
    let mut found = List::with_capacity(spent.rows.min(store.limits.store.page.bytes));
    let mut last = range.after.clone();
    for (number, value) in &candidates {
        if spent.rows == 0 {
            more = true;
            break;
        }
        let position = reads::Position::Owner(core::Key::Tasks(tasks::Key::Ended(*number)));
        if let Some(value) = value {
            let row = erasure::plain(value.clone());
            let bytes = row.owned_bytes().ok_or(Failure::Corrupt)?;
            let Some(remaining) = spent.bytes.checked_sub(bytes) else {
                more = true;
                break;
            };
            spent.bytes = remaining;
            found.push(row).expect("candidate count and page rows bound the result");
        }
        spent.rows = spent.rows.checked_sub(1).expect("candidate row room checked");
        last = Some(position);
    }
    Ok(reads::Loaded::Page { rows: found.into_boxed(), next: if more { last } else { None } })
}

fn candidates(
    store: &mut Store,
    after: Option<u64>,
    rows: u32,
) -> Result<(Map<u64, Option<core::Record>>, bool), Failure> {
    let mut found = Map::uncapped();
    let mut more = false;
    // Stores whose history is in the map have ended rows there. Once moved, summaries
    // name their point reads; no walk spans the ended records in task logs.
    for tag in [6, 30] {
        let mut range = Range::prefix(&[keys::JIG, 1, tag]);
        if let Some(after) = after {
            let key = if tag == 6 { tasks::Key::Ended(after) } else { tasks::Key::Summary(after) };
            range.start = intervals::point(&core::Key::Tasks(key)).end.ok_or(Failure::Corrupt)?;
        }
        let walked = pages::reader::walk(store, &[range], None, rows).or(Err(Failure::Corrupt))?;
        more |= walked.next.is_some();
        for item in walked.rows {
            let value = super::paged::decode(store, item.row)?;
            match value {
                core::Record::Tasks(tasks::Stored::Ended(row)) => {
                    found
                        .insert(row.number, Some(core::Record::Tasks(tasks::Stored::Ended(row))))
                        .expect("page rows bound candidates");
                }
                core::Record::Tasks(tasks::Stored::Summary(row)) => {
                    if !found.contains_key(&row.number) {
                        let value = point::record(store, &core::Key::Tasks(tasks::Key::Ended(row.number)))?;
                        found.insert(row.number, value).expect("page rows bound candidates");
                    }
                }
                core::Record::Core(_)
                | core::Record::Tasks(_)
                | core::Record::People(_)
                | core::Record::Notes(_)
                | core::Record::Accounts(_) => return Err(Failure::Corrupt),
            }
        }
    }
    Ok((found, more))
}

//! Stored rows without owner or transcript joins for whole-task erasure
//! (jig/docs/design/protocol/store.md, 11.1).
use super::{Failure, Spent, joins, paged};
use crate::{Store, intervals, pages};
use ::core::mem::size_of;
use alloc::boxed::Box;
use jig_domain_core::{self as core, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

pub(super) fn read(
    store: &mut Store,
    range: &reads::Range,
    task: Option<u64>,
    spent: &mut Spent,
) -> Result<reads::Loaded, Failure> {
    let intervals = intervals::scope(range.party, &range.scope).ok_or(Failure::Corrupt)?;
    let after = paged::cursor(range, &intervals)?;
    let walked = pages::walk(store, &intervals, after.as_ref(), spent.rows).ok_or(Failure::Corrupt)?;
    let mut found = List::with_capacity(u32::try_from(walked.rows.len()).or(Err(Failure::Corrupt))?);
    let mut last = range.after.clone();
    for item in walked.rows {
        let value = paged::decode(store, item.row)?;
        let position = reads::Position::Owner(value.key());
        let selected = match task {
            Some(task) => match belongs(store, &value, task, spent) {
                Ok(selected) => selected,
                Err(Failure::Cut | Failure::Joins) if last.is_some() && last != range.after => {
                    return Ok(reads::Loaded::Page { rows: found.into_boxed(), next: last });
                }
                Err(error @ (Failure::Cut | Failure::Joins | Failure::Corrupt)) => return Err(error),
            },
            None => true,
        };
        if !selected {
            spent.rows = spent.rows.checked_sub(1).ok_or(Failure::Cut)?;
            last = Some(position);
            continue;
        }
        let row = reads::Row {
            value,
            owner: None,
            ancestors: Box::new([]),
            related: None,
            unjoined: None,
            ended_read: None,
            linked: Box::new([]),
            inbox: None,
        };
        let bytes = match &row.value {
            // An ended read owns no heap bytes and is not a party-read value.
            core::Record::People(people::Stored::EndedRead { .. }) => u64::try_from(size_of::<reads::Row>())
                .or(Err(Failure::Corrupt))?
                .checked_add(u64::try_from(size_of::<core::Record>()).or(Err(Failure::Corrupt))?)
                .ok_or(Failure::Corrupt)?,
            core::Record::Core(_)
            | core::Record::Tasks(_)
            | core::Record::People(_)
            | core::Record::Notes(_)
            | core::Record::Accounts(_) => row.owned_bytes().ok_or(Failure::Corrupt)?,
        };
        let Some(remaining) = spent.bytes.checked_sub(bytes) else {
            return Ok(reads::Loaded::Page { rows: found.into_boxed(), next: last });
        };
        spent.bytes = remaining;
        spent.rows = spent.rows.checked_sub(1).ok_or(Failure::Cut)?;
        last = Some(position);
        found.push(row).expect("bounded by walked rows");
    }
    Ok(reads::Loaded::Page { rows: found.into_boxed(), next: if walked.next.is_some() { last } else { None } })
}

fn belongs(store: &mut Store, value: &core::Record, task: u64, spent: &mut Spent) -> Result<bool, Failure> {
    Ok(match value {
        core::Record::Tasks(tasks::Stored::Update(row)) => row.task == task,
        core::Record::Tasks(tasks::Stored::DecidedProposal(row)) => row.first() == task,
        core::Record::Core(core::CoreRecord::ProposalDecision(row)) => match row.proposer {
            people::Proposer::Task(proposer) => proposer == task,
            people::Proposer::Party(_) => {
                let key = core::Key::Tasks(tasks::Key::DecidedProposal(row.proposal));
                match joins::get(store, &key, spent)? {
                    Some(core::Record::Tasks(tasks::Stored::DecidedProposal(proposal))) => proposal.first() == task,
                    Some(
                        core::Record::Tasks(_)
                        | core::Record::Core(_)
                        | core::Record::People(_)
                        | core::Record::Notes(_)
                        | core::Record::Accounts(_),
                    )
                    | None => false,
                }
            }
        },
        core::Record::Tasks(_)
        | core::Record::Core(_)
        | core::Record::People(_)
        | core::Record::Notes(_)
        | core::Record::Accounts(_) => true,
    })
}

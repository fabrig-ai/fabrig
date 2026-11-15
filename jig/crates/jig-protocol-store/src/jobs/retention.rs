//! Backfill immutable summaries and next stages (protocol/store/retention.md, 8).
use super::{JobAnswer, JobPage, JobRefusal, JobWrite};
use crate::{codec, core_v6, keys, tasks_v13};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{CoreKey, Key};
use jig_domain_people::Proposer;
use jig_domain_tasks::{RetentionRow, RetentionStage, Stored, TaskSummary};
use skein_lib::{List, Reader, Wall};

pub(super) fn rows(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    for row in &page.rows {
        let Some(Key::Tasks(jig_domain_tasks::Key::Ended(number))) = keys::decode(&row.key) else {
            return Err(refusal(&row.key, "not an ended task key"));
        };
        let Ok(Stored::Ended(record)) = codec::tasks::decode(&row.value, &tasks_v13::CEILINGS, &[]) else {
            return Err(refusal(&row.key, "not an ended task record"));
        };
        if record.number != number {
            return Err(refusal(&row.key, "ended task key disagrees with record"));
        }
        let added = [
            put(Stored::Summary(Box::new(TaskSummary::of(&record))), &row.key)?,
            put(Stored::Retention(RetentionRow::of(&record)), &row.key)?,
        ];
        let Some(total) = room(page, &writes, bytes, &added) else {
            if writes.is_empty() {
                return Err(refusal(&row.key, "room cannot hold one task's retention rows"));
            }
            return Ok(JobAnswer { writes: writes.into_boxed(), through });
        };
        bytes = total;
        through = Some(row.key.clone());
        for write in added {
            writes.push(write).expect("uncapped job writes");
        }
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None })
}
pub(super) fn proposals(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    for row in &page.rows {
        let Some(Key::Core(CoreKey::ProposalDecision { proposer: Proposer::Party(party), proposal })) =
            keys::decode(&row.key)
        else {
            return Err(refusal(&row.key, "not a party proposal decision key"));
        };
        let Ok(record) = core_v6::Row::decode(&core_v6::CEILINGS, &mut Reader::new(&row.value)) else {
            return Err(refusal(&row.key, "invalid proposal decision record"));
        };
        let core_v6::Value::ProposalDecision(record) = record.into_parts().value else {
            return Err(refusal(&row.key, "not a proposal decision record"));
        };
        let record = record.into_parts();
        if codec::core::decode_proposer(record.proposer) != Proposer::Party(party) || record.proposal != proposal {
            return Err(refusal(&row.key, "proposal decision key disagrees with record"));
        }
        let write = put(
            Stored::Retention(RetentionRow {
                stage: RetentionStage::Proposal,
                since: Wall::from_nanos(record.at),
                number: proposal,
            }),
            &row.key,
        )?;
        let Some(total) = room(page, &writes, bytes, core::slice::from_ref(&write)) else {
            if writes.is_empty() {
                return Err(refusal(&row.key, "room cannot hold one proposal's retention row"));
            }
            return Ok(JobAnswer { writes: writes.into_boxed(), through });
        };
        bytes = total;
        through = Some(row.key.clone());
        writes.push(write).expect("uncapped job writes");
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None })
}
fn room(page: &JobPage, writes: &List<JobWrite>, bytes: u64, added: &[JobWrite]) -> Option<u64> {
    let count = writes.len().checked_add(u32::try_from(added.len()).ok()?)?;
    let mut total = bytes;
    for write in added {
        total = total.checked_add(write.bytes())?;
    }
    if count > page.room.writes || total > page.room.bytes { None } else { Some(total) }
}
fn put(record: Stored, source: &[u8]) -> Result<JobWrite, JobRefusal> {
    let Some(key) = keys::encode(&Key::Tasks(record.key()), u32::MAX) else {
        return Err(refusal(source, "retention key does not encode"));
    };
    let Ok(value) = codec::tasks::encode(&record, &tasks_v13::CEILINGS, &[]) else {
        return Err(refusal(source, "retention value does not encode"));
    };
    Ok(JobWrite::Put { key, value })
}
fn refusal(key: &[u8], why: &str) -> JobRefusal {
    JobRefusal { key: Some(Box::from(key)), why: Box::from(why) }
}

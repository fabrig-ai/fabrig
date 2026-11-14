//! Split whole live rows once, within each page's commit edge (protocol/store/parts.md, 4).
use super::{JobAnswer, JobPage, JobRefusal, JobWrite};
use crate::{codec::tasks, keys, tasks_v13 as wire};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{Key, settings::ModelName};
use jig_domain_tasks::{Key as TaskKey, Stored};
use skein_lib::{List, Reader};

pub(super) fn page(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut bytes = 0_u32;
    let mut through = None;
    for row in &page.rows {
        let converted = split(row).ok_or(refusal(Some(row.key.clone()), "invalid whole live task"))?;
        let pair = match converted {
            Converted::Passed => {
                through = Some(row.key.clone());
                continue;
            }
            Converted::Pair(pair) => pair,
        };
        {
            let mut size = 0_u32;
            for write in &pair {
                let JobWrite::Put { key, value } = write else { unreachable!("parts are puts") };
                let put = skein_kv::put_bytes(
                    u32::try_from(key.len()).expect("bounded job key"),
                    u32::try_from(value.len()).expect("bounded job value"),
                )
                .ok_or(refusal(Some(row.key.clone()), "live parts exceed row bounds"))?;
                size = size.checked_add(put).ok_or(refusal(Some(row.key.clone()), "live parts size overflow"))?;
            }
            let total = bytes.checked_add(size).ok_or(refusal(Some(row.key.clone()), "live parts size overflow"))?;
            if writes.len().checked_add(2).expect("bounded job writes") > page.edge.writes || total > page.edge.bytes {
                if through.is_none() {
                    return Err(refusal(Some(row.key.clone()), "live parts exceed commit edge"));
                }
                return Ok(JobAnswer { writes: writes.into_boxed(), through });
            }
            bytes = total;
            for write in pair {
                writes.push(write).expect("uncapped job writes");
            }
        }
        through = Some(row.key.clone());
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None })
}

fn refusal(key: Option<Box<[u8]>>, why: &str) -> JobRefusal {
    JobRefusal { key, why: Box::from(why) }
}

enum Converted {
    Passed,
    Pair([JobWrite; 2]),
}

fn split(row: &super::JobRow) -> Option<Converted> {
    let Key::Tasks(TaskKey::Live(number)) = keys::decode(&row.key)? else { return None };
    let wire::RowParts { value, models } =
        wire::Row::decode(&wire::CEILINGS, &mut Reader::new(&row.value)).ok()?.into_parts();
    let mut names = List::with_capacity(models.len());
    for name in models.into_boxed() {
        names.push(ModelName { name: name.into_parts().value, efforts: Box::default() }).expect("same model names");
    }
    let record = match value {
        wire::Value::LiveHot(hot) => return (hot.run().number() == number).then_some(Converted::Passed),
        wire::Value::Live(record) => tasks::record::read_before_talk(record, &wire::CEILINGS).ok()?,
        wire::Value::LiveTalk(record) => tasks::record::read_task_record(record, &wire::CEILINGS).ok()?,
        wire::Value::Ledger(_)
        | wire::Value::ClosedLedger(_)
        | wire::Value::Writer(_)
        | wire::Value::Pool(_)
        | wire::Value::History(_)
        | wire::Value::Milestone(_)
        | wire::Value::PersonProposal(_)
        | wire::Value::DecidedProposal(_)
        | wire::Value::Activity(_)
        | wire::Value::Ended(_)
        | wire::Value::Stub(_)
        | wire::Value::Update(_)
        | wire::Value::Result(_)
        | wire::Value::EndedDelegate(_)
        | wire::Value::Delegate(_)
        | wire::Value::EndedByParty(_)
        | wire::Value::EndedByProject(_)
        | wire::Value::PlacedHold(_)
        | wire::Value::EndedTalk(_)
        | wire::Value::LiveCold(_) => return None,
    };
    if record.number != number {
        return None;
    }
    let cold = Stored::LiveCold {
        number,
        spec: Box::new(record.spec.clone()),
        contract: record.contract.clone(),
        authority: record.authority.clone(),
        instructions: record.instructions.clone(),
    };
    let hot = tasks::encode(&Stored::Live(Box::new(record)), &wire::CEILINGS, names.as_slice()).ok()?;
    let cold = tasks::encode(&cold, &wire::CEILINGS, names.as_slice()).ok()?;
    Some(Converted::Pair([
        JobWrite::Put { key: row.key.clone(), value: hot },
        JobWrite::Put { key: keys::encode(&Key::Tasks(TaskKey::LiveCold(number)), u32::MAX)?, value: cold },
    ]))
}

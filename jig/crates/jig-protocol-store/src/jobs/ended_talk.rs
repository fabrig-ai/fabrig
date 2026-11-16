//! Put ended tasks before talk again, counting their forms (protocol/store/jobs.md, 5.4).
use super::{JobAnswer, JobPage, JobRefusal, JobRow, JobWrite};
use crate::{codec::tasks, keys, tasks_v13 as wire};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{Key, settings::ModelName};
use jig_domain_tasks::{Key as TaskKey, Stored};
use skein_lib::{List, Reader};

pub(super) fn page(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    let mut ended = 0_u64;
    let mut talk = 0_u64;
    for row in &page.rows {
        let value = convert(row).ok_or(refusal(row, "invalid ended task"))?;
        match value {
            Converted::Ended(value) => {
                let write = JobWrite::Put { key: row.key.clone(), value };
                let total = bytes.checked_add(write.bytes()).ok_or(refusal(row, "ended task size overflow"))?;
                if writes.len() == page.room.writes || total > page.room.bytes {
                    if through.is_none() {
                        return Err(refusal(row, "ended task exceeds commit edge"));
                    }
                    return Ok(JobAnswer { writes: writes.into_boxed(), through, counts: Box::new([ended, talk]) });
                }
                bytes = total;
                writes.push(write).expect("uncapped job writes");
                ended = ended.checked_add(1).expect("page bounded ended tasks");
            }
            Converted::Talk => {
                talk = talk.checked_add(1).expect("page bounded ended tasks");
            }
        }
        through = Some(row.key.clone());
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through: None, counts: Box::new([ended, talk]) })
}
fn refusal(row: &JobRow, why: &str) -> JobRefusal {
    JobRefusal { key: Some(row.key.clone()), why: Box::from(why) }
}
enum Converted {
    Ended(Box<[u8]>),
    Talk,
}
fn convert(row: &JobRow) -> Option<Converted> {
    let Key::Tasks(TaskKey::Ended(number)) = keys::decode(&row.key)? else { return None };
    let wire::RowParts { value, models } =
        wire::Row::decode(&wire::CEILINGS, &mut Reader::new(&row.value)).ok()?.into_parts();
    let record = match value {
        wire::Value::Ended(record) => tasks::record::read_before_talk(record, &wire::CEILINGS).ok()?,
        wire::Value::EndedTalk(record) => return (record.record().run().number() == number).then_some(Converted::Talk),
        wire::Value::Summary(_)
        | wire::Value::Retention(_)
        | wire::Value::Ledger(_)
        | wire::Value::ClosedLedger(_)
        | wire::Value::Writer(_)
        | wire::Value::Pool(_)
        | wire::Value::History(_)
        | wire::Value::PersonProposal(_)
        | wire::Value::DecidedProposal(_)
        | wire::Value::Activity(_)
        | wire::Value::Stub(_)
        | wire::Value::Update(_)
        | wire::Value::Result(_)
        | wire::Value::EndedDelegate(_)
        | wire::Value::Delegate(_)
        | wire::Value::EndedByParty(_)
        | wire::Value::EndedByProject(_)
        | wire::Value::Attempt(_)
        | wire::Value::Hold(_)
        | wire::Value::PlacedHold(_)
        | wire::Value::LiveHot(_)
        | wire::Value::LiveCold(_) => return None,
    };
    if record.number != number {
        return None;
    }
    let mut names = List::with_capacity(models.len());
    for name in models.into_boxed() {
        names.push(ModelName { name: name.into_parts().value, efforts: Box::default() }).expect("same model names");
    }
    Some(Converted::Ended(tasks::encode(&Stored::Ended(Box::new(record)), &wire::CEILINGS, names.as_slice()).ok()?))
}

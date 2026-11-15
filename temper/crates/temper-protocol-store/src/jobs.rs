//! Temper's jobs at open (jig/docs/design/protocol/store/jobs.md, 3).
use alloc::boxed::Box;
use jig_protocol_store::{JobAnswer, JobPage, JobRefusal, JobWrite};
use skein_lib::List;
use temper_engine_domain_forge_vocabulary::{ChangeRow, Key, Stored};

mod deleted;
pub use deleted::{Present, read_present};

/// Dispatch an application's job at open.
pub fn page(job: &str, page: &JobPage, present: &Present, bounds: &crate::Bounds) -> Result<JobAnswer, JobRefusal> {
    match job {
        "change-parts" => change_parts(page),
        "deleted-repository-forge-rows" => deleted::page(page, present, bounds),
        _ => Err(refusal(None, "no such temper job")),
    }
}

fn refusal(key: Option<Box<[u8]>>, why: &str) -> JobRefusal {
    JobRefusal { key, why: Box::from(why) }
}

fn change_parts(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut bytes = 0_u64;
    let mut through = None;
    for row in &page.rows {
        let Some(change) = legacy(&row.key, &row.value) else {
            return Err(refusal(Some(row.key.clone()), "unreadable change row"));
        };
        let (cold, hot) = change.split();
        let cold_key = crate::keys::encode(&Key::ChangeCold(cold.task), u32::MAX).expect("change key");
        let hot_key = crate::keys::encode(&Key::ChangeHot(hot.task), u32::MAX).expect("change key");
        let cold = crate::records::encode(&Stored::ChangeCold(Box::new(cold)));
        let hot = crate::records::encode(&Stored::ChangeHot(Box::new(hot)));
        let row_writes = [
            JobWrite::Put { key: cold_key, value: cold },
            JobWrite::Put { key: hot_key, value: hot },
            JobWrite::Erase { key: row.key.clone() },
        ];
        let mut cost = Some(bytes);
        for write in &row_writes {
            cost = match cost {
                Some(cost) => cost.checked_add(write.bytes()),
                None => None,
            };
        }
        let count = writes.len().checked_add(3);
        let fits = match (cost, count) {
            (Some(cost), Some(count)) => cost <= page.room.bytes && count <= page.room.writes,
            (None, _) | (_, None) => false,
        };
        if !fits {
            if through.is_none() {
                return Err(refusal(Some(row.key.clone()), "change parts exceed the job edge"));
            }
            return Ok(JobAnswer { counts: Box::new([]), writes: writes.into_boxed(), through });
        }
        bytes = cost.expect("cost checked");
        for write in row_writes {
            writes.push(write).expect("uncapped job writes");
        }
        through = Some(row.key.clone());
    }
    Ok(JobAnswer { counts: Box::new([]), writes: writes.into_boxed(), through: None })
}

fn legacy(key: &[u8], bytes: &[u8]) -> Option<ChangeRow> {
    use crate::v25 as w;
    let mut reader = skein_lib::Reader::new(bytes);
    let row = w::Row::decode(&w::CEILINGS, &mut reader).ok()?;
    if reader.remaining() != 0 {
        return None;
    }
    let w::Value::Change(row) = row.into_parts().value else { return None };
    let row = crate::records::decode_legacy_change(*row).ok()?;
    let encoded = crate::keys::encode(&Key::Change(row.task), u32::MAX)?;
    if encoded.as_ref() != key {
        return None;
    }
    Some(row)
}

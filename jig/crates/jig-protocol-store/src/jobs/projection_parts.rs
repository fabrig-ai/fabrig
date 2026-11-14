//! Split whole projection rows once at open (protocol/store/parts.md, 3–4).
use super::{JobAnswer, JobPage, JobRefusal, JobWrite};
use crate::{codec::core::projections, core_v6 as wire, keys};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{CoreKey, Key, ProjectionKey};
use skein_lib::{List, Reader, Writer, length::usize_of};

pub(super) fn page(page: &JobPage) -> Result<JobAnswer, JobRefusal> {
    let mut writes = List::uncapped();
    let mut through = None;
    let mut bytes = 0_u32;
    for row in &page.rows {
        let Some(parts) = convert(&row.key, &row.value) else {
            return Err(refusal(&row.key, "invalid projection row"));
        };
        if let Converted::Parts(hot, words) = parts {
            let next = parts_size(&hot, &words, bytes);
            let fits = within(writes.len().checked_add(2), page.edge.writes) && within(next, page.edge.bytes);
            if !fits {
                if through.is_none() {
                    return Err(refusal(&row.key, "projection parts exceed job edge"));
                }
                break;
            }
            bytes = next.expect("parts fit job edge");
            writes.push(hot).expect("uncapped job writes");
            writes.push(words).expect("uncapped job writes");
        }
        through = Some(row.key.clone());
    }
    Ok(JobAnswer { writes: writes.into_boxed(), through })
}

fn refusal(key: &[u8], why: &str) -> JobRefusal {
    JobRefusal { key: Some(Box::from(key)), why: Box::from(why) }
}

enum Converted {
    Kept,
    Parts(JobWrite, JobWrite),
}

fn within(value: Option<u32>, limit: u32) -> bool {
    match value {
        Some(value) => value <= limit,
        None => false,
    }
}

fn parts_size(hot: &JobWrite, words: &JobWrite, bytes: u32) -> Option<u32> {
    bytes.checked_add(put_size(hot)?)?.checked_add(put_size(words)?)
}

fn convert(key: &[u8], bytes: &[u8]) -> Option<Converted> {
    let wire::RowParts { value } = wire::Row::decode(&wire::CEILINGS, &mut Reader::new(bytes)).ok()?.into_parts();
    let wire::Value::Projection(value) = value else { return None };
    let Some((hot, words)) = projections::split(value.into_parts().value).ok()? else { return Some(Converted::Kept) };
    let hot = put(hot)?;
    let JobWrite::Put { key: hot_key, .. } = &hot else { return None };
    if hot_key.as_ref() != key {
        return None;
    }
    Some(Converted::Parts(hot, put(words)?))
}

fn put(value: wire::ProjectionRecord) -> Option<JobWrite> {
    let key = match &value {
        wire::ProjectionRecord::GoalHot(value) => ProjectionKey::Goal(value.number()),
        wire::ProjectionRecord::TaskHot(value) => ProjectionKey::Task { goal: value.goal(), task: value.number() },
        wire::ProjectionRecord::GoalWords(value) => {
            if value.goal() != value.task() {
                return None;
            }
            ProjectionKey::GoalWords(value.goal())
        }
        wire::ProjectionRecord::TaskWords(value) => ProjectionKey::TaskWords { goal: value.goal(), task: value.task() },
        wire::ProjectionRecord::Goal(_) | wire::ProjectionRecord::Task(_) | wire::ProjectionRecord::Milestone(_) => {
            return None;
        }
    };
    let key = keys::encode(&Key::Core(CoreKey::Projection(key)), 32)?;
    let value = wire::ProjectionValue::new(&wire::CEILINGS, wire::ProjectionValueParts { value }).ok()?;
    let row = wire::Row::new(&wire::CEILINGS, wire::RowParts { value: wire::Value::Projection(value) }).ok()?;
    let mut writer = Writer::new(usize_of(row.measure()));
    row.encode(&mut writer).ok()?;
    Some(JobWrite::Put { key, value: writer.finish() })
}

fn put_size(write: &JobWrite) -> Option<u32> {
    match write {
        JobWrite::Put { key, value } => {
            skein_kv::put_bytes(u32::try_from(key.len()).ok()?, u32::try_from(value.len()).ok()?)
        }
        JobWrite::Erase { .. } => None,
    }
}

//! Data directories in name order (hearth/docs/design/domain/render/spec.md, 1.3).
use crate::{
    entries::Expansion,
    errors::{Error, Schema},
    revision::declaration::Map as Kind,
};
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::parts::data::{Data, DataChange, DataClass, DataEntry, DataKeyed, DataValue};
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{Joined, List, Map, bytes};

pub(crate) const MAX_ENTRIES: u32 = 64;
pub(crate) fn build(expansion: &Expansion) -> Result<Data, Box<[Error]>> {
    let entries = expansion.of(Kind::Data);
    let capacity = u32::try_from(entries.len()).expect("expansion bound");
    let mut rows = Map::with_capacity(capacity);
    let mut faults = List::with_capacity(capacity);
    for located in &entries {
        match crate::kinds::data::entry(located) {
            Ok(entry) => {
                rows.insert((entry.value.name.clone(), entry.source.task.clone()), (entry, located))
                    .expect("entry bound");
            }
            Err(error) => {
                faults.push(*error).expect("entry bound");
            }
        }
    }
    let mut entries = List::with_capacity(rows.len());
    for (index, ((_name, _task), (entry, located))) in rows.iter().enumerate() {
        if index == usize::try_from(MAX_ENTRIES).expect("entry limit") {
            faults
                .push(crate::kinds::secrets::fault(located, None, Schema::Above { max: i64::from(MAX_ENTRIES) }))
                .expect("entry bound");
            break;
        }
        entries.push(entry.value.clone()).expect("entry bound");
    }
    if faults.is_empty() { Ok(Data { entries: entries.into_boxed() }) } else { Err(faults.into_boxed()) }
}
#[must_use]
pub(crate) fn digest(data: &Data) -> Digest {
    let mut hash = H::new(b"part:data", 1);
    let record = hash.values();
    record.map(1);
    record.key(b"entries");
    record.map(u64::try_from(data.entries.len()).expect("entry bound"));
    for entry in &data.entries {
        record.key(&entry.name);
        record.map(2);
        record.key(b"class");
        record.bytes(entry.value.class.name());
        record.key(b"path");
        record.bytes(&entry.value.path);
    }
    hash.finish()
}
#[must_use]
pub(crate) fn change(before: Option<&Data>, after: Option<&Data>) -> DataChange {
    let before = match before {
        Some(data) => data.entries.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(data) => data.entries.as_ref(),
        None => &[],
    };
    let mut old: Map<&[u8], &DataEntry> = Map::uncapped();
    let mut new: Map<&[u8], &DataEntry> = Map::uncapped();
    for row in before {
        old.insert(row.name.as_ref(), row).expect("an uncapped map");
    }
    for row in after {
        new.insert(row.name.as_ref(), row).expect("an uncapped map");
    }
    let mut changes = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => DataKeyed::Removed { key: bytes::copy_of(key), value: row.value.clone() },
            Joined::Second(row) => DataKeyed::Added { key: bytes::copy_of(key), value: row.value.clone() },
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => {
                DataKeyed::Changed { key: bytes::copy_of(key), before: was.value.clone(), after: now.value.clone() }
            }
        };
        changes.push(change).expect("an uncapped list");
    }
    DataChange { entries: changes.into_boxed() }
}

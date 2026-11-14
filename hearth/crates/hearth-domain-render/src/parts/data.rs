//! Data directories in name order (hearth/docs/design/domain/render/spec.md, 1.3).
use crate::{
    entries::Expansion,
    errors::{Error, Schema},
    revision::declaration::Map as Kind,
};
use alloc::boxed::Box;
use hearth_lib::{encode::Digest, hash::H};
use skein_lib::{List, Map};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Data {
    pub entries: Box<[DataEntry]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataEntry {
    pub name: Box<[u8]>,
    pub value: DataValue,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataValue {
    pub class: DataClass,
    pub path: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DataClass {
    Core,
    Data,
    Cache,
}
impl DataClass {
    #[must_use]
    pub(crate) const fn name(self) -> &'static [u8] {
        match self {
            Self::Core => b"core",
            Self::Data => b"data",
            Self::Cache => b"cache",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DataChange {
    pub entries: Box<[DataKeyed]>,
}
pub type DataKeyed = crate::diff::Keyed<DataValue>;
pub(crate) const MAX_ENTRIES: u32 = 64;
pub(crate) fn build(expansion: &Expansion) -> Result<Data, Box<[Error]>> {
    let entries = expansion.of(Kind::Data);
    let capacity = u32::try_from(entries.len()).expect("expansion bound");
    let mut rows = Map::with_capacity(capacity);
    let mut faults = List::with_capacity(capacity);
    for located in &entries {
        match crate::kinds::data::entry(located) {
            Ok(entry) => {
                rows.insert((entry.name.clone(), entry.task.clone()), (entry, located)).expect("entry bound");
            }
            Err(error) => {
                faults.push(*error).expect("entry bound");
            }
        }
    }
    let mut entries = List::with_capacity(rows.len());
    for (index, ((name, _task), (entry, located))) in rows.iter().enumerate() {
        if index == usize::try_from(MAX_ENTRIES).expect("entry limit") {
            faults
                .push(crate::kinds::secrets::fault(located, None, Schema::Above { max: i64::from(MAX_ENTRIES) }))
                .expect("entry bound");
            break;
        }
        entries
            .push(DataEntry { name: name.clone(), value: DataValue { class: entry.class, path: entry.path.clone() } })
            .expect("entry bound");
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
    DataChange {
        entries: crate::diff::keyed(before, after, |row| &row.name, |change| change.map(|row| row.value.clone())),
    }
}

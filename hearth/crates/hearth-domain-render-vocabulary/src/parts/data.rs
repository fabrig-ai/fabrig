//! Data directories in name order (hearth/docs/design/domain/render/spec.md, 1.3).
use alloc::boxed::Box;
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
    pub const fn name(self) -> &'static [u8] {
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
/// Keyed changes (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DataKeyed {
    Added { key: Box<[u8]>, value: DataValue },
    Removed { key: Box<[u8]>, value: DataValue },
    Changed { key: Box<[u8]>, before: DataValue, after: DataValue },
}

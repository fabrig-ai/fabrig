//! Typed differences and the lowest changed layer (hearth/docs/design/domain/render/README.md, 4.1, 4.2).
use crate::{
    parts::{boot, config, data, exports, live, packages, secrets, software},
    revision::Oid,
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
mod resolution;
pub use resolution::*;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub target: Box<[u8]>,
    pub change: TargetChange,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TargetChange {
    Added { spec_digest: Digest },
    Removed,
    Same,
    Changed { parts: Box<[PartChange]>, rebuild: Rebuild },
    Failed { side: Side },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Side {
    From,
    To,
    Both,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PartChange {
    Secrets(secrets::Change),
    Software(software::Change),
    Live(live::Change),

    Meta(crate::parts::meta::Change),
    Config(config::Change),
    Data(data::DataChange),
    Packages(packages::Change),
    Boot(boot::Change),
    Exports(exports::Change),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Rebuild {
    FromPackages { resolution: Needs },
    FromSoftware,
    FromConfig,
    SlotOnly,
    Nothing,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Needs {
    Known,
    Needed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tally {
    pub from: Oid,
    pub to: Oid,
    pub targets: u32,
    pub changed: u32,
    pub unchanged: u32,
}
/// Git's retained unified lines (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Hunks {
    pub lines: Box<[Line]>,
    pub left_out: u32,
    pub binary: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line {
    pub kind: LineKind,
    pub text: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LineKind {
    Hunk,
    Context,
    Removed,
    Added,
    NoNewline,
}

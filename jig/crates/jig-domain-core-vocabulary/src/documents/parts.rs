//! Application parts of task rows (jig/docs/design/domain/engine/views.md, 2).

pub use jig_domain_contract::parts::{MadePart, PartAsk, PartFailure, PartsAsk, RowPart};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent declared inputs of a part kind, domain/engine/views.md, 2"
)]
pub struct PartInputs {
    pub line: bool,
    pub notes: bool,
    pub delegates: bool,
    pub delegate_notes: bool,
    pub records: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PartKind {
    pub owner: u16,
    pub kind: u32,
    pub connector: u16,
    pub bytes: u32,
    pub inputs: PartInputs,
}

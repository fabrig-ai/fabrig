//! Evaluation values and faults (hearth/docs/design/domain/render/README.md, 5.1).
pub mod filter;
pub use filter::Filter;
pub mod value;
pub use value::{Entry, Value};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FaultKind {
    Unknown,
    NoKey,
    NoIndex,
    Undefined,
    Type,
    Overflow,
    Empty,
    Path,
    Output,
    Limit,
}

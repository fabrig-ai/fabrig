//! Connector news classes (jig/docs/design/domain/connectors.md, 2).

/// Connector-classified news can be lowered by a task policy.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NewsClass {
    Wakes,
    Kept,
    Dropped,
}

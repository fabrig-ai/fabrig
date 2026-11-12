//! Public reading's documents and messages (jig/docs/design/domain/people/public-reading.md).

pub use crate::below::public::*;

pub(crate) mod index;
mod line;
pub(crate) mod reads;

pub(crate) use index::{Index, follow, publish, restored};

#[cfg(test)]
#[path = "tests/public.rs"]
mod tests;

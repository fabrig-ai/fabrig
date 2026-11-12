//! Public reading's documents and messages (jig/docs/design/domain/people/public-reading.md).

pub use jig_domain_core_vocabulary::public::*;

pub(crate) mod index;
mod line;
pub(crate) mod reads;

pub(crate) use index::{Index, follow, publish, restored};

#[cfg(test)]
#[path = "tests/public.rs"]
mod tests;

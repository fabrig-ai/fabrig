//! Pure GitHub call plans and their exchange events (forge/github.md, section 2).
use crate::github::{decode, document, header};
pub use crate::plan::Failure;
use alloc::boxed::Box;

pub mod failure;
pub mod plan;
mod reading;
pub mod reads;
pub mod start;
pub mod tables;
pub mod writes;

#[cfg(test)]
mod tests;

/// What a finished answer body was; listing rows arrive before its document.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Body {
    Document(document::Document),
    Raw { bytes: Box<[u8]>, more: bool },
    Empty,
    Malformed(decode::Malformed),
}

/// One exchange, in order: a head, streamed rows, an end; or a transport failure.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Heard {
    /// GitHub's head has no status field, so the status travels beside it.
    Head {
        status: u16,
        head: header::Head,
    },
    Row(document::Row),
    Done(Body),
    Failed(Failure),
}

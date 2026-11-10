//! Shared decoded stream events (skein/docs/design/llm/README.md, 2).
use crate::{Failure, Stop, Usage};
use alloc::boxed::Box;

/// One decoded stream event over a dialect's own `Part`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Output<P> {
    Part(P),
    TextDelta { index: u32, content_index: u32, text: Box<[u8]> },
    ToolStart { index: u32, id: Box<[u8]>, name: Box<[u8]> },
    ArgumentsDelta { index: u32, delta: Box<[u8]> },
    ReasoningDelta { index: u32, summary_index: u32, text: Box<[u8]> },
    Completed { stop: Stop, usage: Usage },
    Failed { failure: Failure, detail: Box<[u8]> },
    Progress,
}

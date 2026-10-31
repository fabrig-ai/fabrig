//! Turns told by the agent and waiting to be saved and acknowledged
//! (domain/host.md, sections 2 and 6). The chat holds its answer until this
//! queue drains, so store backpressure keeps the answer from being shown early.

use skein_lib::Queue;
/// Bounded unsaved activation numbers and conversation positions.
#[derive(Debug)]
pub(crate) struct Turns {
    pub(crate) unsaved: Queue<(u32, u32)>,
}

impl Turns {
    pub(crate) fn new(capacity: u32) -> Turns {
        Turns { unsaved: Queue::with_capacity(capacity) }
    }
}

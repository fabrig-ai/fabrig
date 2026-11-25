//! Reading and following a log (skein/docs/design/ui/web/patches.md, 1).

use skein_ui::NodeId;

use super::{Event, Op};

pub(super) trait Scroll {
    fn top(&self) -> i32;
    fn height(&self) -> i32;
    fn viewport(&self) -> i32;
    fn set_top(&self, top: i32);
}

pub(super) fn measure(host: &impl Scroll, node: NodeId) -> Event {
    Event::Scrolled { node, top: host.top(), height: host.height(), viewport: host.viewport() }
}

pub(super) fn to_end(host: &impl Scroll) {
    // CSSOM clamps scrollTop to the current range. Don't read scrollHeight
    // to compute an out-of-range position the browser will clamp anyway.
    host.set_top(i32::MAX);
}

pub(super) enum Turn<'a> {
    Now(&'a Op),
    FinishWrites,
    Follow(NodeId),
    Restore(NodeId),
    Measure(NodeId),
}

// Follow and restore see the completed DOM. Capture and input movement keep
// their places in the turn; measurements still follow all movements.
pub(super) fn replay(ops: &[Op], mut run: impl FnMut(Turn<'_>)) {
    let mut first_move = ops.len();
    let mut first_measure = ops.len();
    for (at, op) in ops.iter().enumerate() {
        if let Op::ScrollEnd { .. } | Op::PutView { .. } = op {
            first_move = first_move.min(at);
        } else if let Op::Measure { .. } = op {
            first_measure = first_measure.min(at);
        } else {
            run(Turn::Now(op));
        }
    }
    run(Turn::FinishWrites);
    for op in &ops[first_move..] {
        if let Op::ScrollEnd { node } = op {
            run(Turn::Follow(*node));
        } else if let Op::PutView { node } = op {
            run(Turn::Restore(*node));
        }
    }
    for op in &ops[first_measure..] {
        if let Op::Measure { node } = op {
            run(Turn::Measure(*node));
        }
    }
}

#[cfg(test)]
#[path = "../tests/page/scroll.rs"]
mod tests;

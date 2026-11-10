use super::{Complete, Op, Submit};
use skein_lib::{Map, Queue, Token};
/// A process's hand-off to the kernel: each owner's tokens renamed to one space (runtime/kernel.md, 5).
#[derive(Debug)]
pub struct Routes {
    active: Map<Token, (Token, Option<Token>)>,
    retired: Map<Token, Token>,
    next: u64,
}
impl Routes {
    /// Routes for at most `room` operations.
    #[must_use]
    pub fn new(room: u32) -> Self {
        Self { active: Map::with_capacity(room), retired: Map::with_capacity(room), next: 1 }
    }
    /// No operation is in flight and no cancel's target waits.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.active.is_empty() && self.retired.is_empty()
    }
    /// An operation is in flight.
    #[must_use]
    pub fn has_operations(&self) -> bool {
        !self.active.is_empty()
    }
    /// Moves records from `source` to `target` under process-wide tokens, while both have room.
    pub fn publish(&mut self, source: &mut Queue<Submit>, target: &mut Queue<Submit>) {
        for _ in 0..source.len() {
            if target.room() == 0 || self.active.len() == self.active.capacity() {
                break;
            }
            let mut submit = source.pop().expect("kernel submissions");
            let mut cancel = None;
            if let Op::Cancel { target } = &mut submit.kind {
                cancel = Some(*target);
                let mut found = None;
                for (external, (internal, _)) in &self.active {
                    if *internal == *target {
                        found = Some(*external);
                        break;
                    }
                }
                *target = match found {
                    Some(external) => external,
                    None => self.retired.remove(target).expect("cancel names submitted operation"),
                };
            }
            let external = Token::new(self.next);
            self.next = self.next.checked_add(1).expect("kernel operation ids");
            self.active.insert(external, (submit.op, cancel)).expect("kernel route room");
            submit.op = external;
            target.push(submit);
        }
    }
    /// The owner's completion for a process-wide one. `sockets` is the owner's queue, in which a
    /// cancel of this target may wait.
    pub fn complete(&mut self, mut complete: Complete, sockets: &Queue<Submit>) -> Complete {
        let (internal, cancel) = self.active.remove(&complete.op).expect("submitted operation route");
        for queued in sockets {
            if let Op::Cancel { target } = queued.kind
                && target == internal
            {
                self.retired.insert(internal, complete.op).expect("pending cancellation route");
                break;
            }
        }
        complete.op = internal;
        if let Some(internal) = cancel
            && let Op::Cancel { target } = &mut complete.kind
        {
            *target = internal;
        }
        complete
    }
}

//! Bounds as data, evaluated without recursion (data/codec/README.md, 4).
//! Every edge points to an earlier node. A bounded stack follows only reached
//! nodes, memoizing each by its index; family bounds reuse the same memo.

use skein_lib::{List, Stack};

/// One node in a postordered shape table. Sizes count heap slots, not pointees.
#[derive(Clone, Copy, Debug)]
pub enum Node {
    Scalar(u64),
    Bytes { limit: u32 },
    List { limit: u32, item: u32, size: usize },
    Option { item: u32 },
    Boxed { item: u32, size: usize },
    Record { prefix: u64, fields: &'static [u32] },
    Enum { variants: &'static [u32] },
}

/// The two independent maxima for a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub encoded: u64,
    pub decoded: u64,
}

impl Node {
    fn child(self, next: u32) -> Option<u32> {
        match self {
            Self::Scalar(_) | Self::Bytes { .. } => None,
            Self::List { item, .. } | Self::Option { item } | Self::Boxed { item, .. } => {
                if next == 0 {
                    Some(item)
                } else {
                    None
                }
            }
            Self::Record { fields, .. } => fields.get(usize::try_from(next).ok()?).copied(),
            Self::Enum { variants } => variants.get(usize::try_from(next).ok()?).copied(),
        }
    }

    fn finish(self, sum: Bounds, limits: &[u32]) -> Option<Bounds> {
        let mut result = sum;
        match self {
            Self::Scalar(size) => result.encoded = size,
            Self::Bytes { limit } => {
                result.decoded = u64::from(*limits.get(usize::try_from(limit).ok()?)?);
                result.encoded = 4_u64.checked_add(result.decoded)?;
            }
            Self::List { limit, size, .. } => {
                let count = u64::from(*limits.get(usize::try_from(limit).ok()?)?);
                result.encoded = 4_u64.checked_add(count.checked_mul(sum.encoded)?)?;
                result.decoded = count.checked_mul(sum.decoded.checked_add(u64::try_from(size).ok()?)?)?;
            }
            Self::Option { .. } | Self::Enum { .. } => result.encoded = 1_u64.checked_add(sum.encoded)?,
            Self::Boxed { size, .. } => result.decoded = sum.decoded.checked_add(u64::try_from(size).ok()?)?,
            Self::Record { prefix, .. } => result.encoded = prefix.checked_add(sum.encoded)?,
        }
        Some(result)
    }
}

#[derive(Debug)]
struct Frame {
    at: u32,
    next: u32,
    sum: Bounds,
}

impl Frame {
    const fn new(at: u32) -> Self {
        Self { at, next: 0, sum: Bounds { encoded: 0, decoded: 0 } }
    }

    fn add(&mut self, parent: Node, value: Bounds) -> Option<()> {
        match parent {
            Node::Enum { .. } => {
                self.sum.encoded = self.sum.encoded.max(value.encoded);
                self.sum.decoded = self.sum.decoded.max(value.decoded);
            }
            Node::Scalar(_)
            | Node::Bytes { .. }
            | Node::List { .. }
            | Node::Option { .. }
            | Node::Boxed { .. }
            | Node::Record { .. } => {
                self.sum.encoded = self.sum.encoded.checked_add(value.encoded)?;
                self.sum.decoded = self.sum.decoded.checked_add(value.decoded)?;
            }
        }
        Some(())
    }
}

/// What a walk has finished: for each node up to the declaration, one more than
/// the index of its bounds in `finished`, zero until it finishes. Four bytes a
/// node, allocated zeroed at once, and sixteen a reached one, so a walk holds
/// little more than the nodes it reaches.
#[derive(Debug)]
struct Memo {
    slots: List<u32>,
    finished: List<Bounds>,
}

impl Memo {
    /// Edges descend, so a walk from `at` reaches no later node: a slot for each
    /// node up to it, never more than the table has.
    #[expect(clippy::disallowed_macros, reason = "the slots are allocated once, zeroed, at their final length")]
    fn new(nodes: &[Node], at: u32) -> Option<Memo> {
        let count = u32::try_from(nodes.len()).ok()?.min(at.checked_add(1)?);
        let slots = List::from_boxed(alloc::vec![0; usize::try_from(count).ok()?].into_boxed_slice());
        Some(Memo { slots, finished: List::with_capacity(count) })
    }

    fn get(&self, at: u32) -> Option<Bounds> {
        self.finished.get(self.slots.get(at)?.checked_sub(1)?).copied()
    }

    fn insert(&mut self, at: u32, value: Bounds) -> Option<()> {
        self.finished.push(value).ok()?;
        *self.slots.get_mut(at)? = self.finished.len();
        Some(())
    }
}

fn evaluate(nodes: &[Node], limits: &[u32], at: u32, memo: &mut Memo, stack: &mut Stack<Frame>) -> Option<Bounds> {
    if let Some(value) = memo.get(at) {
        return Some(value);
    }
    stack.push(Frame::new(at)).ok()?;
    // Keep unfinished parents in place. Only an edge to an uncached child
    // pushes a frame; strictly descending edges bound the depth by the table.
    while let Some(frame) = stack.top_mut() {
        let node = *nodes.get(usize::try_from(frame.at).ok()?)?;
        if let Some(child) = node.child(frame.next) {
            if child >= frame.at {
                return None;
            }
            frame.next = frame.next.checked_add(1)?;
            if let Some(value) = memo.get(child) {
                frame.add(node, value)?;
            } else {
                stack.push(Frame::new(child)).ok()?;
            }
        } else {
            let value = node.finish(frame.sum, limits)?;
            memo.insert(frame.at, value)?;
            stack.pop()?;
            // Finish the parent's edge with the result just computed, instead
            // of visiting that edge again to retrieve it from the memo.
            let Some(parent) = stack.top_mut() else { return Some(value) };
            let node = *nodes.get(usize::try_from(parent.at).ok()?)?;
            parent.add(node, value)?;
        }
    }
    None
}

fn valid(limits: &[u32], ceilings: &[u32]) -> Option<()> {
    if limits.len() != ceilings.len() {
        return None;
    }
    for (value, ceiling) in limits.iter().zip(ceilings) {
        if value > ceiling {
            return None;
        }
    }
    Some(())
}

/// A declaration's maxima; refuses limits past a ceiling and arithmetic overflow.
#[must_use]
pub fn declaration(nodes: &[Node], limits: &[u32], ceilings: &[u32], at: u32) -> Option<Bounds> {
    valid(limits, ceilings)?;
    let mut memo = Memo::new(nodes, at)?;
    let mut stack = Stack::with_capacity(u32::try_from(nodes.len()).ok()?);
    evaluate(nodes, limits, at, &mut memo, &mut stack)
}

/// The maxima among a family's declarations, checking its limits once.
#[must_use]
pub fn family(nodes: &[Node], limits: &[u32], ceilings: &[u32], declarations: &[u32]) -> Option<Bounds> {
    valid(limits, ceilings)?;
    let mut memo = Memo::new(nodes, declarations.iter().copied().max().unwrap_or(0))?;
    let mut stack = Stack::with_capacity(u32::try_from(nodes.len()).ok()?);
    let mut result = Bounds { encoded: 0, decoded: 0 };
    for at in declarations {
        let value = evaluate(nodes, limits, *at, &mut memo, &mut stack)?;
        result.encoded = result.encoded.max(value.encoded);
        result.decoded = result.decoded.max(value.decoded);
    }
    Some(result)
}

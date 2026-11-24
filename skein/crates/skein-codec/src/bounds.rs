//! Bounds as data, evaluated without recursion (data/codec/README.md, 4).
//! Local edges point to earlier nodes. A bounded stack follows named edges,
//! computing each reached declaration once per walk.

use core::{cmp::Ordering, mem::size_of};
use skein_lib::{List, Stack};

/// A named declaration and its postordered nodes, with the root last.
#[derive(Clone, Copy, Debug)]
pub struct Declaration {
    pub name: &'static str,
    pub nodes: &'static [Node],
}

/// One node of a declaration. Local edges count within its nodes, `limit`
/// offsets are in bytes, and sizes count heap slots, not pointees.
#[derive(Clone, Copy, Debug)]
pub enum Node {
    Scalar(u64),
    Bytes { limit: usize },
    List { limit: usize, item: u32, size: usize },
    Option { item: u32 },
    Boxed { item: u32, size: usize },
    Record { prefix: u64, fields: &'static [u32] },
    Enum { variants: &'static [u32] },
    Named(&'static str),
}

/// The two independent maxima for a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub encoded: u64,
    pub decoded: u64,
}

impl Bounds {
    const ZERO: Self = Self { encoded: 0, decoded: 0 };

    fn most(self, other: Self) -> Self {
        Self { encoded: self.encoded.max(other.encoded), decoded: self.decoded.max(other.decoded) }
    }
}

fn limit_value(limits: &[u32], offset: usize) -> Option<u64> {
    let width = size_of::<u32>();
    if !offset.is_multiple_of(width) {
        return None;
    }
    Some(u64::from(*limits.get(offset.checked_div(width)?)?))
}

impl Node {
    fn bound(self, local: &List<Bounds>, limits: &[u32]) -> Option<Bounds> {
        let value = match self {
            Self::Scalar(encoded) => Bounds { encoded, decoded: 0 },
            Self::Bytes { limit } => {
                let count = limit_value(limits, limit)?;
                Bounds { encoded: 4_u64.checked_add(count)?, decoded: count }
            }
            Self::List { limit, item, size } => {
                let count = limit_value(limits, limit)?;
                let item = local.get(item)?;
                Bounds {
                    encoded: 4_u64.checked_add(count.checked_mul(item.encoded)?)?,
                    decoded: count.checked_mul(item.decoded.checked_add(u64::try_from(size).ok()?)?)?,
                }
            }
            Self::Option { item } => {
                let item = local.get(item)?;
                Bounds { encoded: 1_u64.checked_add(item.encoded)?, decoded: item.decoded }
            }
            Self::Boxed { item, size } => {
                let item = local.get(item)?;
                Bounds { encoded: item.encoded, decoded: item.decoded.checked_add(u64::try_from(size).ok()?)? }
            }
            Self::Record { prefix, fields } => {
                let mut sum = Bounds { encoded: prefix, decoded: 0 };
                for field in fields {
                    let field = local.get(*field)?;
                    sum.encoded = sum.encoded.checked_add(field.encoded)?;
                    sum.decoded = sum.decoded.checked_add(field.decoded)?;
                }
                sum
            }
            Self::Enum { variants } => {
                let mut most = Bounds::ZERO;
                for variant in variants {
                    most = most.most(*local.get(*variant)?);
                }
                most.encoded = 1_u64.checked_add(most.encoded)?;
                most
            }
            Self::Named(_) => return None, // The walk resolves named edges before bounding a local node.
        };
        Some(value)
    }
}

fn find(declarations: &[Declaration], name: &str) -> Option<u32> {
    let mut low = 0_usize;
    let mut high = declarations.len();
    // Each comparison at least halves the remaining range.
    while low < high {
        let middle = low.checked_add(high.checked_sub(low)?.checked_div(2)?)?;
        match declarations.get(middle)?.name.cmp(name) {
            Ordering::Equal => return u32::try_from(middle).ok(),
            Ordering::Greater => high = middle,
            Ordering::Less => low = middle.checked_add(1)?,
        }
    }
    None
}

#[derive(Debug)]
struct Frame<'a> {
    index: u32,
    declaration: &'a Declaration,
    local: List<Bounds>,
}

#[derive(Clone, Copy, Debug)]
enum Visit {
    Unseen,
    Pending,
    Done(Bounds),
}

struct Walk<'a> {
    declarations: &'a [Declaration],
    limits: &'a [u32],
    // A slot per declaration, at the position the name search returns.
    memo: List<Visit>,
    stack: Stack<Frame<'a>>,
    #[cfg(test)]
    starts: u32,
}

impl<'a> Walk<'a> {
    #[expect(clippy::disallowed_macros, reason = "the memo slots are allocated and filled once at their final length")]
    fn new(declarations: &'a [Declaration], limits: &'a [u32]) -> Option<Self> {
        let count = u32::try_from(declarations.len()).ok()?;
        Some(Self {
            declarations,
            limits,
            memo: List::from_boxed(alloc::vec![Visit::Unseen; declarations.len()].into_boxed_slice()),
            stack: Stack::with_capacity(count),
            #[cfg(test)]
            starts: 0,
        })
    }

    fn start(&mut self, index: u32) -> Option<()> {
        let declaration = self.declarations.get(usize::try_from(index).ok()?)?;
        let count = u32::try_from(declaration.nodes.len()).ok()?;
        *self.memo.get_mut(index)? = Visit::Pending;
        self.stack.push(Frame { index, declaration, local: List::allocated(count) }).ok()?;
        #[cfg(test)]
        {
            self.starts = self.starts.checked_add(1)?;
        }
        Some(())
    }

    fn evaluate(&mut self, name: &str) -> Option<Bounds> {
        let index = find(self.declarations, name)?;
        match self.memo.get(index)? {
            Visit::Done(bound) => return Some(*bound),
            Visit::Pending => return None,
            Visit::Unseen => self.start(index)?,
        }
        while let Some(frame) = self.stack.top_mut() {
            let at = usize::try_from(frame.local.len()).ok()?;
            if let Some(node) = frame.declaration.nodes.get(at) {
                let bound = match *node {
                    Node::Named(child) => {
                        let index = find(self.declarations, child)?;
                        match self.memo.get(index)? {
                            Visit::Done(bound) => *bound,
                            Visit::Pending => return None,
                            Visit::Unseen => {
                                self.start(index)?;
                                continue;
                            }
                        }
                    }
                    Node::Scalar(_)
                    | Node::Bytes { .. }
                    | Node::List { .. }
                    | Node::Option { .. }
                    | Node::Boxed { .. }
                    | Node::Record { .. }
                    | Node::Enum { .. } => node.bound(&frame.local, self.limits)?,
                };
                frame.local.push(bound).ok()?;
            } else {
                let bound = *frame.local.last()?;
                *self.memo.get_mut(frame.index)? = Visit::Done(bound);
                self.stack.pop()?;
                // The parent is waiting on this named edge. Finish it without
                // looking up the freshly computed bound in the memo again.
                let Some(parent) = self.stack.top_mut() else { return Some(bound) };
                parent.local.push(bound).ok()?;
            }
        }
        None
    }
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

/// The named declaration's maxima. Shapes must be sorted by name. Returns
/// `None` for invalid limits, missing or cyclic shapes, or arithmetic overflow.
#[must_use]
pub fn declaration(declarations: &[Declaration], limits: &[u32], ceilings: &[u32], name: &str) -> Option<Bounds> {
    valid(limits, ceilings)?;
    Walk::new(declarations, limits)?.evaluate(name)
}

/// Bounds for several names in one sorted family, in request order. Checks
/// limits once and shares the memo for all the requested declarations.
/// Refuses on the same conditions as [`declaration`].
#[must_use]
pub fn batch<const N: usize>(
    declarations: &[Declaration],
    limits: &[u32],
    ceilings: &[u32],
    names: [&str; N],
) -> Option<[Bounds; N]> {
    valid(limits, ceilings)?;
    let mut walk = Walk::new(declarations, limits)?;
    let mut bounds = [Bounds::ZERO; N];
    for (name, bound) in names.into_iter().zip(&mut bounds) {
        *bound = walk.evaluate(name)?;
    }
    Some(bounds)
}

/// The maxima among the declarations (sorted by name), checking limits once.
#[must_use]
pub fn family(declarations: &[Declaration], limits: &[u32], ceilings: &[u32]) -> Option<Bounds> {
    valid(limits, ceilings)?;
    let mut walk = Walk::new(declarations, limits)?;
    let mut result = Bounds::ZERO;
    for declaration in declarations {
        result = result.most(walk.evaluate(declaration.name)?);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::{Bounds, Declaration, Node, Walk, batch, declaration, family, size_of};

    #[test]
    fn a_repeated_named_child_is_only_computed_once() {
        const SHAPES: &[Declaration] = &[
            Declaration { name: "Child", nodes: &[Node::Scalar(2)] },
            Declaration {
                name: "Parent",
                nodes: &[Node::Named("Child"), Node::Named("Child"), Node::Record { prefix: 0, fields: &[0, 1] }],
            },
        ];
        const REACHED: u32 = 2; // Parent and Child, despite both edges to Child.
        const CHAIN: &[Declaration] = &[
            Declaration {
                name: "A",
                nodes: &[Node::Named("B"), Node::Named("B"), Node::Record { prefix: 2, fields: &[0, 1] }],
            },
            Declaration { name: "B", nodes: &[Node::Named("C"), Node::List { limit: 0, item: 0, size: 8 }] },
            Declaration { name: "C", nodes: &[Node::Bytes { limit: size_of::<u32>() }] },
        ];
        const LIMITS: &[u32] = &[2, 3]; // Two C items per B, three bytes per C.
        // C is (4+3, 3), B is (4+2*7, 2*(8+3)), and A has two B fields and a two-byte prefix.
        const A: Bounds = Bounds { encoded: 38, decoded: 44 };
        const B: Bounds = Bounds { encoded: 18, decoded: 22 };
        const C: Bounds = Bounds { encoded: 7, decoded: 3 };

        let mut walk = Walk::new(SHAPES, &[]).expect("two declarations");
        let bound = walk.evaluate("Parent");
        assert_eq!((bound, walk.starts), (Some(Bounds { encoded: 4, decoded: 0 }), REACHED));
        assert_eq!(declaration(CHAIN, LIMITS, LIMITS, "A"), Some(A));
        assert_eq!(family(CHAIN, LIMITS, LIMITS), Some(A));
        assert_eq!(batch(CHAIN, LIMITS, LIMITS, ["B", "A", "C"]), Some([B, A, C]));
    }
}

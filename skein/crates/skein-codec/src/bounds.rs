//! Bounds as data, evaluated without recursion (data/codec/README.md, 4).
//! Local edges point to earlier nodes. A bounded stack follows named edges,
//! computing each reached declaration once and sharing its bound across callers.

use core::{cmp::Ordering, mem::size_of};
use skein_lib::{List, Map, Stack};

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

fn find<'a>(mut declarations: &'a [Declaration], name: &str) -> Option<&'a Declaration> {
    // Each comparison at least halves the remaining slice.
    while !declarations.is_empty() {
        let middle = declarations.len().checked_div(2)?;
        let (before, rest) = declarations.split_at_checked(middle)?;
        let (declaration, after) = rest.split_first()?;
        match declaration.name.cmp(name) {
            Ordering::Equal => return Some(declaration),
            Ordering::Greater => declarations = before,
            Ordering::Less => declarations = after,
        }
    }
    None
}

#[derive(Debug)]
struct Frame<'a> {
    declaration: &'a Declaration,
    local: List<Bounds>,
}

#[derive(Clone, Copy, Debug)]
enum Visit {
    Pending,
    Done(Bounds),
}

struct Walk<'a> {
    declarations: &'a [Declaration],
    limits: &'a [u32],
    memo: Map<&'static str, Visit>,
    stack: Stack<Frame<'a>>,
    #[cfg(test)]
    starts: u32,
}

impl<'a> Walk<'a> {
    fn new(declarations: &'a [Declaration], limits: &'a [u32]) -> Option<Self> {
        let count = u32::try_from(declarations.len()).ok()?;
        Some(Self {
            declarations,
            limits,
            memo: Map::with_capacity(count),
            stack: Stack::with_capacity(count),
            #[cfg(test)]
            starts: 0,
        })
    }

    fn start(&mut self, name: &str) -> Option<()> {
        let declaration = find(self.declarations, name)?;
        let count = u32::try_from(declaration.nodes.len()).ok()?;
        let _previous = self.memo.insert(declaration.name, Visit::Pending).ok()?;
        self.stack.push(Frame { declaration, local: List::allocated(count) }).ok()?;
        #[cfg(test)]
        {
            self.starts = self.starts.checked_add(1)?;
        }
        Some(())
    }

    fn evaluate(&mut self, name: &str) -> Option<Bounds> {
        match self.memo.get(name) {
            Some(Visit::Done(bound)) => return Some(*bound),
            Some(Visit::Pending) => return None,
            None => self.start(name)?,
        }
        while let Some(frame) = self.stack.top_mut() {
            let at = usize::try_from(frame.local.len()).ok()?;
            if let Some(node) = frame.declaration.nodes.get(at) {
                let bound = match *node {
                    Node::Named(child) => match self.memo.get(child) {
                        Some(Visit::Done(bound)) => *bound,
                        Some(Visit::Pending) => return None,
                        None => {
                            self.start(child)?;
                            continue;
                        }
                    },
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
                *self.memo.get_mut(frame.declaration.name)? = Visit::Done(bound);
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
    use super::{Bounds, Declaration, Node, Walk};

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
        let mut walk = Walk::new(SHAPES, &[]).expect("two declarations");
        let bound = walk.evaluate("Parent");
        assert_eq!((bound, walk.starts), (Some(Bounds { encoded: 4, decoded: 0 }), REACHED));
    }
}

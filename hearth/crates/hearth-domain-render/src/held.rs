//! Held value bytes, alike for composed and validated settings (hearth/docs/design/domain/render/README.md, 2.3).
use crate::{
    compose::{Composed, Composite, Pair},
    evaluate::{ELEMENT, Entry, Value},
};
use core::slice::Iter;
use skein_lib::Stack;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Tree<'a> {
    Composed(&'a Composed),
    Validated(&'a Value),
}
#[derive(Clone, Debug)]
enum Children<'a> {
    ComposedList(Iter<'a, Composed>),
    ComposedMap(Iter<'a, Pair>),
    ValidatedList(Iter<'a, Value>),
    ValidatedMap(Iter<'a, Entry>),
}
impl<'a> Children<'a> {
    // An absent validated entry still occupies its map slot.
    fn next(&mut self) -> Option<(u64, Option<Tree<'a>>)> {
        let (key, tree) = match self {
            Self::ComposedList(values) => (None, Some(Tree::Composed(values.next()?))),
            Self::ValidatedList(values) => (None, Some(Tree::Validated(values.next()?))),
            Self::ComposedMap(values) => {
                let pair = values.next()?;
                (Some(pair.key.as_ref()), Some(Tree::Composed(&pair.value)))
            }
            Self::ValidatedMap(values) => {
                let entry = values.next()?;
                let mut tree = None;
                if let Some(value) = &entry.value {
                    tree = Some(Tree::Validated(value));
                }
                (Some(entry.key.as_ref()), tree)
            }
        };
        let key = match key {
            Some(key) => u64::try_from(key.len()).expect("key bytes"),
            None => 0,
        };
        Some((ELEMENT.saturating_add(key), tree))
    }
    fn map_size(&self, length: usize) -> u64 {
        let mut entries = self.clone();
        let mut size = 0_u64;
        for _ in 0..length {
            let (bytes, _) = entries.next().expect("map entry count");
            size = size.saturating_add(bytes);
        }
        size
    }
}

pub(crate) fn charge(value: Tree<'_>, left: &mut u64, depth: u32) -> bool {
    let mut stack = Stack::with_capacity(depth);
    let mut next = Some(value);
    for _ in 0..left.saturating_mul(2).saturating_add(u64::from(depth)).saturating_add(1) {
        if let Some(value) = next.take() {
            let (size, children) = match value {
                Tree::Composed(Composed { kind: Composite::Bool(_) | Composite::Int(_), .. })
                | Tree::Validated(Value::Bool(_) | Value::Int(_)) => (0, None),
                Tree::Composed(Composed { kind: Composite::String(bytes), .. })
                | Tree::Validated(Value::String(bytes)) => (u64::try_from(bytes.len()).expect("string bytes"), None),
                Tree::Composed(Composed { kind: Composite::List(values), .. }) => (
                    u64::try_from(values.len()).expect("list count").saturating_mul(ELEMENT),
                    Some(Children::ComposedList(values.iter())),
                ),
                Tree::Validated(Value::List(values)) => (
                    u64::try_from(values.len()).expect("list count").saturating_mul(ELEMENT),
                    Some(Children::ValidatedList(values.iter())),
                ),
                Tree::Composed(Composed { kind: Composite::Map(values), .. }) => {
                    let children = Children::ComposedMap(values.iter());
                    (children.map_size(values.len()), Some(children))
                }
                Tree::Validated(Value::Map(values)) => {
                    let children = Children::ValidatedMap(values.iter());
                    (children.map_size(values.len()), Some(children))
                }
            };
            if let Some(children) = children {
                stack.push(children).expect("settings depth");
            }
            let Some(rest) = left.checked_sub(size) else { return false };
            *left = rest;
        }
        let Some(mut children) = stack.pop() else { return true };
        if let Some((_, tree)) = children.next() {
            next = tree;
            stack.push(children).expect("popped frame fits");
        }
    }
    false
}
